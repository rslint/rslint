use schemars::JsonSchema;
use types__regex::RegexSet as DDlogRegexSet;

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
)]
#[serde(rename_all = "kebab-case")]
pub struct NoUnusedVarsConfig {
    pub ignored_patterns: DDlogRegexSet,
    // pub ignore_args: IgnoreArgs,
    // pub caught_errors: CaughtErrors,
    // pub caught_error_patterns: DDlogRegexSet,
}

impl NoUnusedVarsConfig {
    pub fn ignored_patterns(&self) -> &DDlogRegexSet {
        &self.ignored_patterns
    }

    // pub fn ignore_args(&self) -> IgnoreArgs {
    //     self.ignore_args
    // }
    //
    // pub fn caught_errors(&self) -> CaughtErrors {
    //     self.caught_errors
    // }
    //
    // pub fn caught_error_patterns(&self) -> &DDlogRegexSet {
    //     &self.caught_error_patterns
    // }
}

impl JsonSchema for NoUnusedVarsConfig {
    fn schema_name() -> String {
        "ignored-patterns".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<Vec<String>>()
    }
}

pub fn ignored_patterns(config: &NoUnusedVarsConfig) -> &DDlogRegexSet {
    config.ignored_patterns()
}

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum IgnoreArgs {
    /// Never report an unused argument
    Always,
    /// Unused positional arguments that occur before the last used argument will not be reported,
    /// but all named arguments and all positional arguments after the last used argument will
    AfterLastUsed,
    /// All named arguments must be used
    Never,
}

impl Default for IgnoreArgs {
    fn default() -> Self {
        Self::Never
    }
}

#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum CaughtErrors {
    /// All caught errors must be used
    All,
    /// Do not check that caught errors are used
    None,
}

impl Default for CaughtErrors {
    fn default() -> Self {
        Self::All
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub struct NoUseBeforeDefConfig {}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub struct NoTypeofUndefConfig {}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub struct NoUndefConfig {}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub struct NoUnusedLabelsConfig {}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub struct NoShadowConfig {
    pub hoisting: NoShadowHoisting,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    FromRecord,
    IntoRecord,
    Mutator,
    JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
pub enum NoShadowHoisting {
    Never,
    Always,
    Functions,
}

impl Default for NoShadowHoisting {
    fn default() -> Self {
        Self::Never
    }
}

pub fn hoisting_never(config: &NoShadowConfig) -> bool {
    matches!(config.hoisting, NoShadowHoisting::Never)
}

pub fn hoisting_always(config: &NoShadowConfig) -> bool {
    matches!(config.hoisting, NoShadowHoisting::Always)
}

pub fn hoisting_functions(config: &NoShadowConfig) -> bool {
    matches!(config.hoisting, NoShadowHoisting::Functions)
}

pub fn hoisting_enabled(config: &NoShadowConfig) -> bool {
    matches!(
        config.hoisting,
        NoShadowHoisting::Always | NoShadowHoisting::Functions,
    )
}
