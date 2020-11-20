#![allow(
    path_statements,
    //unused_imports,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_parens,
    non_shorthand_field_patterns,
    dead_code,
    overflowing_literals,
    unreachable_patterns,
    unused_variables,
    clippy::unknown_clippy_lints,
    clippy::missing_safety_doc,
    clippy::match_single_binding
)]

// Required for #[derive(Serialize, Deserialize)].
use ::serde::Deserialize;
use ::serde::Serialize;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;

use crate::string_append_str;
use crate::string_append;
use crate::std_usize;
use crate::closure;

//
// use crate::ddlog_std;

use differential_datalog::{
    decl_enum_into_record, decl_record_mutator_enum, decl_record_mutator_struct,
    decl_struct_from_record, decl_struct_into_record, record::Record,
};
use schemars::JsonSchema;
use std::fmt::{self, Debug, Display, Formatter};

#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    pub no_shadow: bool,
    pub no_shadow_hoisting: NoShadowHoisting,
    pub no_undef: bool,
    pub no_unused_labels: bool,
    pub no_typeof_undef: bool,
    pub no_unused_vars: bool,
    pub no_use_before_def: bool,
}

impl Config {
    pub fn empty() -> Self {
        Self {
            no_shadow: false,
            no_shadow_hoisting: NoShadowHoisting::Never,
            no_undef: false,
            no_unused_labels: false,
            no_typeof_undef: false,
            no_unused_vars: false,
            no_use_before_def: false,
        }
    }

    pub fn no_shadow(mut self, no_shadow: bool) -> Self {
        self.no_shadow = no_shadow;
        self
    }

    pub fn no_shadow_hoisting(mut self, no_shadow_hoisting: NoShadowHoisting) -> Self {
        self.no_shadow_hoisting = no_shadow_hoisting;
        self
    }

    pub fn no_undef(mut self, no_undef: bool) -> Self {
        self.no_undef = no_undef;
        self
    }

    pub fn no_unused_labels(mut self, no_unused_labels: bool) -> Self {
        self.no_unused_labels = no_unused_labels;
        self
    }

    pub fn no_typeof_undef(mut self, no_typeof_undef: bool) -> Self {
        self.no_typeof_undef = no_typeof_undef;
        self
    }

    pub fn no_unused_vars(mut self, no_unused_vars: bool) -> Self {
        self.no_unused_vars = no_unused_vars;
        self
    }

    pub fn no_use_before_def(mut self, no_use_before_def: bool) -> Self {
        self.no_use_before_def = no_use_before_def;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            no_shadow: true,
            no_shadow_hoisting: NoShadowHoisting::default(),
            no_undef: true,
            no_unused_labels: true,
            no_typeof_undef: true,
            no_unused_vars: true,
            no_use_before_def: true,
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

decl_struct_into_record!(
    Config,
    ["Config"]<>,
    no_shadow,
    no_shadow_hoisting,
    no_undef,
    no_unused_labels,
    no_typeof_undef,
    no_unused_vars,
    no_use_before_def
);

decl_struct_from_record!(
    Config["Config"]<>,
    ["Config"][7]{
        [0] no_shadow["no_shadow"]: bool,
        [1] no_shadow_hoisting["no_shadow_hoisting"]: NoShadowHoisting,
        [2] no_undef["no_undef"]: bool,
        [3] no_unused_labels["no_unused_labels"]: bool,
        [4] no_typeof_undef["no_typeof_undef"]: bool,
        [5] no_unused_vars["no_unused_vars"]: bool,
        [6] no_use_before_def["no_use_before_def"]: bool
    }
);

decl_record_mutator_struct!(
    Config, <>,
    no_shadow: bool,
    no_shadow_hoisting: NoShadowHoisting,
    no_undef: bool,
    no_unused_labels: bool,
    no_typeof_undef: bool,
    no_unused_vars: bool,
    no_use_before_def: bool
);

#[cfg_attr(feature = "schema", derive(JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NoShadowHoisting {
    Never,
    Always,
    Functions,
}

impl Default for NoShadowHoisting {
    fn default() -> Self {
        Self::Functions
    }
}

impl Display for NoShadowHoisting {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Never => f.write_str("never"),
            Self::Always => f.write_str("always"),
            Self::Functions => f.write_str("functions"),
        }
    }
}

impl FromRecord for NoShadowHoisting {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::PosStruct(constr, args) if args.len() == 0 => match constr.as_ref() {
                "Never" => Ok(Self::Never),
                "Always" => Ok(Self::Always),
                "Functions" => Ok(Self::Functions),
                c => Result::Err(format!(
                    "unknown constructor {} of type `NoShadowHoisting` in {:?}",
                    c, *val,
                )),
            },
            Record::NamedStruct(constr, args) if args.len() == 0 => match constr.as_ref() {
                "Never" => Ok(Self::Never),
                "Always" => Ok(Self::Always),
                "Functions" => Ok(Self::Functions),
                c => Result::Err(format!(
                    "unknown constructor {} of type `NoShadowHoisting` in {:?}",
                    c, *val,
                )),
            },

            v => Err(format!("not an instance of `NoShadowHoisting` {:?}", *v)),
        }
    }
}

decl_enum_into_record!(
    NoShadowHoisting<>,
    Never["Never"]{},
    Always["Always"]{},
    Functions["Functions"]{}
);

#[rustfmt::skip]
decl_record_mutator_enum!(NoShadowHoisting<>, Never {}, Always {}, Functions {});

// DDlog bridge functions
pub fn no_shadow_enabled(config: &Config) -> bool {
    config.no_shadow
}

pub fn no_shadow_hoisting(config: &Config) -> bool {
    matches!(
        config.no_shadow_hoisting,
        NoShadowHoisting::Always | NoShadowHoisting::Functions
    )
}

pub fn no_shadow_hoist_functions(config: &Config) -> bool {
    matches!(config.no_shadow_hoisting, NoShadowHoisting::Functions)
}

pub fn no_undef_enabled(config: &Config) -> bool {
    config.no_undef
}

pub fn no_unused_labels_enabled(config: &Config) -> bool {
    config.no_unused_labels
}

pub fn no_typeof_undef_enabled(config: &Config) -> bool {
    config.no_typeof_undef
}

pub fn no_unused_vars_enabled(config: &Config) -> bool {
    config.no_unused_vars
}

pub fn no_use_before_def_enabled(config: &Config) -> bool {
    config.no_use_before_def
}

/* fn no_shadow_enabled(config: & crate::config::Config) -> bool */
/* fn no_shadow_hoist_functions(config: & crate::config::Config) -> bool */
/* fn no_shadow_hoisting(config: & crate::config::Config) -> bool */
/* fn no_typeof_undef_enabled(config: & crate::config::Config) -> bool */
/* fn no_undef_enabled(config: & crate::config::Config) -> bool */
/* fn no_unused_labels_enabled(config: & crate::config::Config) -> bool */
/* fn no_unused_vars_enabled(config: & crate::config::Config) -> bool */
/* fn no_use_before_def_enabled(config: & crate::config::Config) -> bool */