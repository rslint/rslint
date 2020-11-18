//! Configuration file support.

mod de;
use async_fs::read_to_string;
use dirs_next::config_dir;
use rslint_core::{get_group_rules_by_name, CstRule, CstRuleStore, Diagnostic, RuleLevel};
use rslint_errors::file::{Files, SimpleFile};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, env, path::PathBuf};

/// The name of the config file to search for.
pub const CONFIG_NAME: &str = "rslintrc.toml";

/// A list of boxed rule implementations.
pub type RuleList = Vec<Box<dyn CstRule>>;

#[derive(Debug, Deserialize, Serialize)]
struct ConfigRepr {
    rules: Option<RulesConfigRepr>,
    #[serde(default)]
    errors: ErrorsConfigRepr,
}

impl Default for ConfigRepr {
    fn default() -> Self {
        Self {
            rules: None,
            errors: Default::default(),
        }
    }
}

#[serde(default)]
#[derive(Debug, Deserialize, Serialize, Default)]
struct RulesConfigRepr {
    #[serde(deserialize_with = "de::from_rule_objects")]
    errors: RuleList,

    #[serde(deserialize_with = "de::from_rule_objects")]
    warnings: RuleList,

    groups: Vec<String>,
    allowed: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ErrorsConfigRepr {
    formatter: String,
}

impl Default for ErrorsConfigRepr {
    fn default() -> Self {
        Self {
            formatter: "long".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Config {
    repr: ConfigRepr,
    warnings: RefCell<Vec<Diagnostic>>,
}

impl Config {
    /// Creates a new config by first searching for a config in the current
    /// dir and all of it ancestors, and if `no_global_config` is `false`,
    /// look in the systems config directory.
    ///
    /// # Returns
    ///
    /// The config or an `Err` if the toml inside the config is invalid.
    /// The `Diagnostic` can be emitted by using the `SimpleFile` as a file database.
    pub async fn new(no_global_config: bool, emit_diagnostic: fn(SimpleFile, Diagnostic)) -> Self {
        let path = Self::find_config(no_global_config);

        let (source, path) = if let Some(path) = path.as_ref() {
            match read_to_string(path).await {
                Ok(c) => (c, path),
                Err(_) => return Self::default(),
            }
        } else {
            return Self::default();
        };

        match toml::from_str::<ConfigRepr>(&source) {
            Ok(repr) => Self {
                repr,
                warnings: Default::default(),
            },

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
                emit_diagnostic(config_file, d);
                Default::default()
            }
        }
    }

    fn find_config(global_config: bool) -> Option<PathBuf> {
        let path = env::current_dir().ok()?;
        for path in path.ancestors() {
            let path = path.join(CONFIG_NAME);
            if path.exists() {
                return Some(path);
            }
        }

        let path = config_dir()?.join(CONFIG_NAME);
        if global_config && path.exists() {
            return Some(path);
        }

        None
    }

    /// Take all warnings out of this `Config`.
    pub fn warnings(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut *self.warnings.borrow_mut())
    }

    /// Returns the formatter that should be used.
    pub fn formatter(&self) -> String {
        self.repr.errors.formatter.clone()
    }

    pub fn warning_rule_names(&self) -> impl Iterator<Item = &str> {
        self.repr
            .rules
            .iter()
            .flat_map(|rules| &rules.warnings)
            .map(|rule| rule.name())
    }

    pub fn rule_level_by_name(&self, rule_name: &str) -> RuleLevel {
        if self.warning_rule_names().any(|name| name == rule_name) {
            RuleLevel::Warning
        } else {
            RuleLevel::Error
        }
    }

    /// Collects all rules and creates a `CstRuleStore`.
    ///
    /// This method may add warnings to the warning list of this `Config`.
    pub fn rules_store(&self) -> CstRuleStore {
        let rule_cfg = match &self.repr.rules {
            Some(rules) => rules,
            None => return CstRuleStore::new().builtins(),
        };

        let rules = unique_rules(rule_cfg.errors.clone(), rule_cfg.warnings.clone());
        let mut rules = self.intersect_allowed(rules).collect::<Vec<_>>();

        for group in &rule_cfg.groups {
            if let Some(group_rules) = get_group_rules_by_name(group) {
                let list = self.intersect_allowed(group_rules.into_iter());
                let list = list.collect::<Vec<_>>();
                rules = unique_rules(rules, list).collect();
            } else {
                let d = Diagnostic::warning(1, "config", format!("unknown rule group '{}'", group));
                self.warnings.borrow_mut().push(d);
            }
        }

        let mut store = CstRuleStore::new();
        store.load_rules(rules);
        store
    }

    /// Remove any rules which are explicitly allowed by the `allowed` field.
    ///
    /// This method may add warnings to the warning list of this `Config`.
    fn intersect_allowed<'s>(
        &'s self,
        rules: impl Iterator<Item = Box<dyn CstRule>> + 's,
    ) -> impl Iterator<Item = Box<dyn CstRule>> + 's {
        rules.filter(move |rule| {
            let rule_cfg = match self.repr.rules.as_ref() {
                Some(rule_cfg) => rule_cfg,
                None => return true,
            };

            let res = rule_cfg
                .allowed
                .iter()
                .any(|allowed| allowed == rule.name());

            if res {
                let d = Diagnostic::warning(
                    1,
                    "config",
                    format!(
                        "ignoring configuration for '{}' because it is explicitly allowed",
                        rule.name()
                    ),
                );
                self.warnings.borrow_mut().push(d)
            }

            !res
        })
    }
}

fn unique_rules(first: RuleList, mut second: RuleList) -> impl Iterator<Item = Box<dyn CstRule>> {
    second.retain(|rule| !first.iter().any(|prev| prev.name() == rule.name()));
    first.into_iter().chain(second)
}
