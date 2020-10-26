//! Configuration file support.

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
    #[serde(default)]
    pub errors: ErrorsConfig,
}

#[serde(default)]
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RulesConfig {
    #[serde(deserialize_with = "from_rule_objects")]
    pub errors: Vec<Box<dyn CstRule>>,

    #[serde(deserialize_with = "from_rule_objects")]
    pub warnings: Vec<Box<dyn CstRule>>,

    pub groups: Vec<String>,
    pub allowed: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorsConfig {
    pub formatter: String,
}

impl Default for ErrorsConfig {
    fn default() -> Self {
        Self {
            formatter: "long".to_string(),
        }
    }
}

impl Config {
    /// Search for a config file in the current directory,
    ///
    /// # Returns
    ///
    /// The config as a `SimpleFile` and a `Diagnostic` if it fails
    /// If the returned `SimpleFile` is `None`, the `Diagnostic` is a warning and can be
    /// emitted without any files and the default config should be used.
    ///
    /// If the `SimpleFile` is `Some`, the `Diagnostic` is an error and should be reported to the
    /// user.
    pub fn new_threaded() -> JoinHandle<Result<Self, (Option<SimpleFile>, Diagnostic)>> {
        thread::spawn(|| {
            let (source, path) = match current_dir()
                .map(|path| path.join(CONFIG_NAME))
                .and_then(|path| Ok((read_to_string(&path)?, path)))
            {
                Ok(val) => val,
                Err(err) => {
                    let d = Diagnostic::warning(
                        0,
                        "config",
                        format!("failed to read config, using default config: {}", err),
                    );
                    return Err((None, d));
                }
            };

            match from_str(&source) {
                Ok(config) => Ok(config),
                Err(err) => {
                    let config_file = SimpleFile::new(path.to_string_lossy().into(), source);
                    let d = if let Some(idx) = err
                        .line_col()
                        .and_then(|(line, col)| Some(config_file.line_range(0, line)?.start + col))
                    {
                        let pos_regex = regex::Regex::new(" at line \\d+ column \\d+$").unwrap();
                        let msg = err.to_string();
                        let msg = pos_regex.replace(&msg, "");
                        Diagnostic::error(1, "config", msg).primary(idx..idx, "")
                    } else {
                        Diagnostic::error(1, "config", err.to_string())
                    };
                    Err((Some(config_file), d))
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
    ///
    /// # Returns
    ///
    /// An `IntoIterator` implementation that will return the
    /// rule and an optional `Diagnostic` which is a warning and should be emitted if present.
    pub fn intersect_allowed<'a, T>(
        &'a self,
        rules: T,
        issue_warnings: bool,
    ) -> impl Iterator<Item = (T::Item, Option<Diagnostic>)> + 'a
    where
        T: IntoIterator + 'a,
        T::Item: Borrow<Box<dyn CstRule>>,
    {
        rules.into_iter().filter_map(move |rule| {
            let res = self
                .allowed
                .iter()
                .any(|allowed| allowed == rule.borrow().name());

            let d = if res && issue_warnings {
                Some(Diagnostic::warning(
                    1,
                    "config",
                    format!(
                        "ignoring configuration for '{}' because it is explicitly allowed",
                        rule.borrow().name()
                    ),
                ))
            } else {
                None
            };

            if !res {
                Some((rule, d))
            } else {
                None
            }
        })
    }

    /// Collects all rules and reutrns a new `CstRuleStore`.
    ///
    /// # Returns
    ///
    /// The actual rule store and a list of warnings that should be emitted.
    pub fn store(&self) -> (CstRuleStore, Vec<Diagnostic>) {
        let mut store = CstRuleStore::new();
        let mut all_warns = vec![];

        let (mut rules, warns): (_, Vec<_>) = self
            .intersect_allowed(
                Self::unique_rules(self.errors.clone(), self.warnings.clone()),
                true,
            )
            .unzip();
        all_warns.extend(warns.into_iter().flatten());

        for group in &self.groups {
            if let Some(group_rules) = get_group_rules_by_name(&group) {
                let (list, warns): (_, Vec<_>) = self
                    .intersect_allowed(group_rules.into_iter(), false)
                    .into_iter()
                    .unzip();
                all_warns.extend(warns.into_iter().flatten());

                rules = Self::unique_rules(rules, list).collect();
            } else {
                let d = Diagnostic::warning(1, "config", format!("unknown rule group '{}'", group));
                all_warns.push(d);
            }
        }

        store.load_rules(rules);
        (store, all_warns)
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
