//! Configuration file support.

// FIXME: Workaround for https://github.com/GREsau/schemars/pull/65
#![allow(clippy::field_reassign_with_default)]

mod de;
use dirs_next::config_dir;
use rslint_core::{get_group_rules_by_name, CstRule, CstRuleStore, Diagnostic, RuleLevel};
use rslint_errors::file::{Files, SimpleFile};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    env,
    fs::read_to_string,
    path::{Path, PathBuf},
};

/// The name of the config files to search for.
pub const CONFIG_NAMES: [&str; 2] = ["rslintrc.json", "rslintrc.toml"];

/// A list of boxed rule implementations.
pub type RuleList = Vec<Box<dyn CstRule>>;

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Debug, Deserialize, Serialize)]
pub struct ConfigRepr {
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

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(default)]
struct RulesConfigRepr {
    #[serde(deserialize_with = "de::from_rule_objects")]
    errors: RuleList,

    #[serde(deserialize_with = "de::from_rule_objects")]
    warnings: RuleList,

    groups: Vec<String>,
    allowed: Vec<String>,
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for RulesConfigRepr {
    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;
        use schemars::*;

        macro_rules! string_schema {
            ($val:expr) => {
                Schema::Object(SchemaObject {
                    string: Some(Box::new(StringValidation {
                        pattern: Some($val.to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                })
            };
            ($val:expr, $rule_desc:expr) => {{
                let mut split = $rule_desc.split('\n');
                let header = split.next().unwrap_or("");
                let body = split.next().unwrap_or("").to_string();

                Schema::Object(SchemaObject {
                    string: Some(Box::new(StringValidation {
                        pattern: Some($val.to_string()),
                        ..Default::default()
                    })),
                    metadata: Some(Box::new(Metadata {
                        title: Some(header.to_string()),
                        description: Some(body),
                        ..Default::default()
                    })),
                    ..Default::default()
                })
            }};
        }

        let rules = CstRuleStore::new().builtins().rules;
        let mut rule_items = vec![];

        for rule in &rules {
            rule_items.push(string_schema!(rule.name(), rule.docs()));
        }
        let rule_items_schema = Schema::Object(SchemaObject {
            array: Some(Box::new(ArrayValidation {
                items: Some(SingleOrVec::Vec(rule_items)),
                ..Default::default()
            })),
            ..Default::default()
        });

        // TODO(RDambrosio016): dont hardcode it like this
        let group_items = vec![
            string_schema!("errors"),
            string_schema!("style"),
            string_schema!("regex"),
        ];

        let groups_schema = Schema::Object(SchemaObject {
            array: Some(Box::new(ArrayValidation {
                items: Some(SingleOrVec::Vec(group_items)),
                ..Default::default()
            })),
            ..Default::default()
        });

        let mut rule_obj_items = Map::new();
        for rule in &rules {
            if let Some(schema) = rule.schema() {
                rule_obj_items.insert(rule.name().to_string(), Schema::Object(schema.schema));
            }
        }
        let rules_schema = Schema::Object(SchemaObject {
            object: Some(Box::new(ObjectValidation {
                properties: rule_obj_items,
                ..Default::default()
            })),
            ..Default::default()
        });

        let mut map = Map::new();
        map.insert("groups".to_string(), groups_schema);
        map.insert("allowed".to_string(), rule_items_schema);
        map.insert("errors".to_string(), rules_schema.clone());
        map.insert("warnings".to_string(), rules_schema);

        Schema::Object(SchemaObject {
            object: Some(Box::new(ObjectValidation {
                properties: map,
                ..Default::default()
            })),
            ..Default::default()
        })
    }

    fn schema_name() -> String {
        "rules".to_string()
    }
}

#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
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

enum ConfigStyle {
    Toml,
    Json,
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
    pub fn new(no_global_config: bool, emit_diagnostic: fn(SimpleFile, Diagnostic)) -> Self {
        let path = Self::find_config(no_global_config);
        let (source, (path, style)) = match path
            .as_ref()
            .and_then(|(path, _)| read_to_string(path).ok())
        {
            Some(source) => (source, path.unwrap()),
            None => return Default::default(),
        };

        match style {
            ConfigStyle::Json => match serde_json::from_str::<ConfigRepr>(&source) {
                Ok(repr) => Self {
                    repr,
                    warnings: Default::default(),
                },
                Err(err) => {
                    let config_file = SimpleFile::new(path.to_string_lossy().into(), source);
                    let (line, col) = (err.line() - 1, err.column() - 1);
                    let idx = config_file
                        .line_range(0, line)
                        .expect("serde_json yielded an invalid line range")
                        .start
                        + col;

                    let diag =
                        Diagnostic::error(1, "config", err.to_string()).primary(idx..idx, "");
                    emit_diagnostic(config_file, diag);
                    Default::default()
                }
            },
            ConfigStyle::Toml => match toml::from_str::<ConfigRepr>(&source) {
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
            },
        }
    }

    fn find_config(global_config: bool) -> Option<(PathBuf, ConfigStyle)> {
        let path = env::current_dir().ok()?;
        fn search_path(path: &Path) -> Option<(PathBuf, ConfigStyle)> {
            for config_name in CONFIG_NAMES.iter() {
                let new_path = path.join(config_name);
                let style = if config_name.ends_with("json") {
                    ConfigStyle::Json
                } else {
                    ConfigStyle::Toml
                };

                if new_path.exists() {
                    return Some((new_path, style));
                }
            }
            None
        }

        for path in path.ancestors() {
            if let Some(res) = search_path(path) {
                return Some(res);
            }
        }

        let path = config_dir()?;
        if let Some(res) = search_path(&path).filter(|_| global_config) {
            return Some(res);
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
            None => return CstRuleStore::new().recommended(),
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
