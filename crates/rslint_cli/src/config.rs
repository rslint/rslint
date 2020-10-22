//! Configuration file support.

use crate::lint_warn;
use heck::{CamelCase, KebabCase};
use rslint_core::{
    get_group_rules_by_name, get_rule_by_name, get_rule_suggestion, CstRule, CstRuleStore,
    RuleLevel,
};
use rslint_errors::{
    file::{Files, SimpleFile},
    Diagnostic,
};
use serde::de::{
    value::MapAccessDeserializer, DeserializeSeed, Error, IntoDeserializer, MapAccess, Visitor,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Borrow;
use std::env::current_dir;
use std::fmt;
use std::fs::read_to_string;
use std::marker::PhantomData;
use std::thread::{self, JoinHandle};
use toml::from_str;

/// The name of the config file to search for.
pub const CONFIG_NAME: &str = "rslintrc.toml";

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub rules: Option<RulesConfig>,
}

#[serde(default)]
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RulesConfig {
    #[serde(deserialize_with = "from_rule_objects")]
    errors: Vec<Box<dyn CstRule>>,

    #[serde(deserialize_with = "from_rule_objects")]
    warnings: Vec<Box<dyn CstRule>>,

    groups: Vec<String>,
    allowed: Vec<String>,
}

impl Config {
    /// Search for a config file in the current directory,
    /// return None if there is no config or if its unreadable.
    /// This returns a thread handle which was spawned for multithreaded IO.
    pub fn new_threaded() -> JoinHandle<Option<Self>> {
        thread::spawn(|| {
            let (source, path) = match current_dir()
                .map(|path| path.join(CONFIG_NAME))
                .and_then(|path| Ok((read_to_string(&path)?, path)))
            {
                Ok(val) => val,
                Err(err) => {
                    crate::lint_warn!("failed to read config, using default config: {}", err);
                    return None;
                }
            };

            match from_str(&source) {
                Ok(config) => Some(config),
                Err(err) => {
                    let files = SimpleFile::new(path.to_string_lossy().into(), source);
                    let d = if let Some(idx) = err
                        .line_col()
                        .and_then(|(line, col)| Some(files.line_range(0, line)?.start + col))
                    {
                        let pos_regex = regex::Regex::new(" at line \\d+ column \\d+$").unwrap();
                        let msg = err.to_string();
                        let msg = pos_regex.replace(&msg, "");
                        Diagnostic::error(0, "config", msg).primary(idx..idx, "")
                    } else {
                        Diagnostic::error(0, "config", err.to_string())
                    };
                    crate::emit_diagnostic(&d, &files);
                    None
                }
            }
        })
    }
}

impl RulesConfig {
    pub fn error_rule_names(&self) -> impl Iterator<Item = &str> {
        // grouped rules are errors by default
        self.errors
            .iter()
            .map(|rule| rule.name())
            .chain(self.grouped_rules().map(|rule| rule.name()))
    }

    pub fn warning_rule_names(&self) -> impl Iterator<Item = &str> {
        self.warnings.iter().map(|rule| rule.name())
    }

    /// The rules declared in the config using the `groups` field.
    pub fn grouped_rules<'a>(&'a self) -> impl Iterator<Item = Box<dyn CstRule>> + 'a {
        self.groups
            .iter()
            .filter_map(|group| get_group_rules_by_name(group))
            .map(|rules| rules.into_iter())
            .flatten()
    }

    pub fn rule_level_by_name(&self, rule_name: &str) -> RuleLevel {
        if self.warning_rule_names().any(|name| name == rule_name) {
            RuleLevel::Warning
        } else {
            RuleLevel::Error
        }
    }

    /// Remove any rules which are explicitly allowed by the `allowed` field
    ///
    /// if `issue_warnings` is true, linter warnings will be emitted stating a rule's configuration
    /// was ignore because its explicitly allowed.
    pub fn intersect_allowed<'a, T>(
        &'a self,
        rules: T,
        issue_warnings: bool,
    ) -> impl IntoIterator<Item = T::Item> + 'a
    where
        T: IntoIterator + 'a,
        T::Item: Borrow<Box<dyn CstRule>>,
    {
        rules.into_iter().filter(move |rule| {
            let res = self
                .allowed
                .iter()
                .any(|allowed| allowed == rule.borrow().name());

            if res && issue_warnings {
                lint_warn!(
                    "ignoring configuration for '{}' because it is explicitly allowed",
                    rule.borrow().name()
                );
            }
            !res
        })
    }

    pub fn store(&self) -> CstRuleStore {
        let mut store = CstRuleStore::new();
        let mut rules: Vec<_> = self
            .intersect_allowed(
                Self::unique_rules(self.errors.clone(), self.warnings.clone()),
                true,
            )
            .into_iter()
            .collect();

        for group in &self.groups {
            if let Some(group_rules) = get_group_rules_by_name(&group) {
                rules = Self::unique_rules(
                    rules,
                    self.intersect_allowed(group_rules.into_iter(), false)
                        .into_iter()
                        .collect(),
                )
                .collect();
            } else {
                lint_warn!("Unknown rule group '{}'", group);
            }
        }

        store.load_rules(rules);
        store
    }

    #[allow(clippy::needless_collect)]
    fn unique_rules(
        first: Vec<Box<dyn CstRule>>,
        second: Vec<Box<dyn CstRule>>,
    ) -> impl Iterator<Item = Box<dyn CstRule>> {
        // collecting is necessary because otherwise, filter's closure might outlive the current function
        let filtered = second
            .into_iter()
            .filter(|rule| !first.iter().any(|prev| prev.name() == rule.name()))
            .collect::<Vec<_>>();
        first.into_iter().chain(filtered.into_iter())
    }
}

fn from_rule_objects<'de, D>(deserializer: D) -> Result<Vec<Box<dyn CstRule>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TypetagObjects<T> {
        _type: PhantomData<T>,
    }

    impl<'de> Visitor<'de> for TypetagObjects<Box<dyn CstRule>> {
        type Value = Vec<Box<dyn CstRule>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("zero or more rule-to-config pairs")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(key) = map.next_key::<String>()? {
                let de = MapAccessDeserializer::new(Entry {
                    key: Some(key.to_camel_case().into_deserializer()),
                    value: &mut map,
                });
                if get_rule_by_name(&key.to_kebab_case()).is_none() {
                    if let Some(suggestion) = get_rule_suggestion(&key.to_kebab_case()) {
                        return Err(M::Error::custom(format!(
                            "Unknown rule '{}'. did you mean '{}'?",
                            key, suggestion
                        )));
                    } else {
                        return Err(M::Error::custom(format!("Unknown rule '{}'", key)));
                    }
                } else {
                    vec.push(Box::<dyn CstRule>::deserialize(de)?);
                }
            }
            Ok(vec)
        }
    }

    struct Entry<K, V> {
        key: Option<K>,
        value: V,
    }

    impl<'de, K, V> MapAccess<'de> for Entry<K, V>
    where
        K: Deserializer<'de, Error = V::Error>,
        V: MapAccess<'de>,
    {
        type Error = V::Error;

        fn next_key_seed<S>(&mut self, seed: S) -> Result<Option<S::Value>, Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            self.key.take().map(|key| seed.deserialize(key)).transpose()
        }

        fn next_value_seed<S>(&mut self, seed: S) -> Result<S::Value, Self::Error>
        where
            S: DeserializeSeed<'de>,
        {
            self.value.next_value_seed(seed)
        }
    }

    deserializer.deserialize_map(TypetagObjects { _type: PhantomData })
}
