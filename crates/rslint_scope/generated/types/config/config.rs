#![allow(
    path_statements,
    unused_imports,
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused_parens,
    non_shorthand_field_patterns,
    dead_code,
    overflowing_literals,
    unreachable_patterns,
    unused_variables,
    clippy::missing_safety_doc,
    clippy::match_single_binding,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::needless_lifetimes,
    clippy::borrowed_box,
    clippy::map_clone,
    clippy::toplevel_ref_arg,
    clippy::double_parens,
    clippy::collapsible_if,
    clippy::clone_on_copy,
    clippy::unused_unit,
    clippy::deref_addrof,
    clippy::clone_on_copy,
    clippy::needless_return,
    clippy::op_ref,
    clippy::match_like_matches_macro,
    clippy::comparison_chain,
    clippy::len_zero,
    clippy::extra_unused_lifetimes
)]

use ::num::One;
use ::std::ops::Deref;

use ::differential_dataflow::collection;
use ::timely::communication;
use ::timely::dataflow::scopes;
use ::timely::worker;

use ::ddlog_derive::{FromRecord, IntoRecord, Mutator};
use ::differential_datalog::ddval::DDValue;
use ::differential_datalog::ddval::DDValConvert;
use ::differential_datalog::program;
use ::differential_datalog::program::TupleTS;
use ::differential_datalog::program::XFormArrangement;
use ::differential_datalog::program::XFormCollection;
use ::differential_datalog::program::Weight;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;


// `usize` and `isize` are builtin Rust types; we therefore declare an alias to DDlog's `usize` and
// `isize`.
pub type std_usize = u64;
pub type std_isize = i64;


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

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "config::EnableNoShadow")]
pub struct EnableNoShadow {
    pub file: types__ast::FileId,
    pub config: ddlog_std::Ref<NoShadowConfig>
}
impl abomonation::Abomonation for EnableNoShadow{}
impl ::std::fmt::Display for EnableNoShadow {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EnableNoShadow{file,config} => {
                __formatter.write_str("config::EnableNoShadow{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(config, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for EnableNoShadow {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "config::EnableNoTypeofUndef")]
pub struct EnableNoTypeofUndef {
    pub file: types__ast::FileId,
    pub config: ddlog_std::Ref<NoTypeofUndefConfig>
}
impl abomonation::Abomonation for EnableNoTypeofUndef{}
impl ::std::fmt::Display for EnableNoTypeofUndef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EnableNoTypeofUndef{file,config} => {
                __formatter.write_str("config::EnableNoTypeofUndef{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(config, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for EnableNoTypeofUndef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "config::EnableNoUndef")]
pub struct EnableNoUndef {
    pub file: types__ast::FileId,
    pub config: ddlog_std::Ref<NoUndefConfig>
}
impl abomonation::Abomonation for EnableNoUndef{}
impl ::std::fmt::Display for EnableNoUndef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EnableNoUndef{file,config} => {
                __formatter.write_str("config::EnableNoUndef{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(config, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for EnableNoUndef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "config::EnableNoUnusedLabels")]
pub struct EnableNoUnusedLabels {
    pub file: types__ast::FileId,
    pub config: ddlog_std::Ref<NoUnusedLabelsConfig>
}
impl abomonation::Abomonation for EnableNoUnusedLabels{}
impl ::std::fmt::Display for EnableNoUnusedLabels {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EnableNoUnusedLabels{file,config} => {
                __formatter.write_str("config::EnableNoUnusedLabels{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(config, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for EnableNoUnusedLabels {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "config::EnableNoUnusedVars")]
pub struct EnableNoUnusedVars {
    pub file: types__ast::FileId,
    pub config: ddlog_std::Ref<NoUnusedVarsConfig>
}
impl abomonation::Abomonation for EnableNoUnusedVars{}
impl ::std::fmt::Display for EnableNoUnusedVars {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EnableNoUnusedVars{file,config} => {
                __formatter.write_str("config::EnableNoUnusedVars{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(config, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for EnableNoUnusedVars {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "config::EnableNoUseBeforeDef")]
pub struct EnableNoUseBeforeDef {
    pub file: types__ast::FileId,
    pub config: ddlog_std::Ref<NoUseBeforeDefConfig>
}
impl abomonation::Abomonation for EnableNoUseBeforeDef{}
impl ::std::fmt::Display for EnableNoUseBeforeDef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EnableNoUseBeforeDef{file,config} => {
                __formatter.write_str("config::EnableNoUseBeforeDef{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(config, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for EnableNoUseBeforeDef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
/* fn hoisting_always(config: & NoShadowConfig) -> bool */
/* fn hoisting_enabled(config: & NoShadowConfig) -> bool */
/* fn hoisting_functions(config: & NoShadowConfig) -> bool */
/* fn hoisting_never(config: & NoShadowConfig) -> bool */
/* fn ignored_patterns(config: & NoUnusedVarsConfig) -> types__regex::RegexSet */
pub fn __Key_config_EnableNoShadow(__key: &DDValue) -> DDValue {
    let ref conf = *{<EnableNoShadow>::from_ddvalue_ref(__key) };
    (conf.file.clone()).into_ddvalue()
}
pub fn __Key_config_EnableNoTypeofUndef(__key: &DDValue) -> DDValue {
    let ref conf = *{<EnableNoTypeofUndef>::from_ddvalue_ref(__key) };
    (conf.file.clone()).into_ddvalue()
}
pub fn __Key_config_EnableNoUndef(__key: &DDValue) -> DDValue {
    let ref conf = *{<EnableNoUndef>::from_ddvalue_ref(__key) };
    (conf.file.clone()).into_ddvalue()
}
pub fn __Key_config_EnableNoUnusedLabels(__key: &DDValue) -> DDValue {
    let ref conf = *{<EnableNoUnusedLabels>::from_ddvalue_ref(__key) };
    (conf.file.clone()).into_ddvalue()
}
pub fn __Key_config_EnableNoUnusedVars(__key: &DDValue) -> DDValue {
    let ref conf = *{<EnableNoUnusedVars>::from_ddvalue_ref(__key) };
    (conf.file.clone()).into_ddvalue()
}
pub fn __Key_config_EnableNoUseBeforeDef(__key: &DDValue) -> DDValue {
    let ref conf = *{<EnableNoUseBeforeDef>::from_ddvalue_ref(__key) };
    (conf.file.clone()).into_ddvalue()
}
pub static __Arng_config_EnableNoShadow_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                               name: std::borrow::Cow::from(r###"(config::EnableNoShadow{.file=(_0: ast::FileId), .config=(_: ddlog_std::Ref<config::NoShadowConfig>)}: config::EnableNoShadow) /*join*/"###),
                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                {
                                                                                                                                    let __cloned = __v.clone();
                                                                                                                                    match <EnableNoShadow>::from_ddvalue(__v) {
                                                                                                                                        EnableNoShadow{file: ref _0, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                        _ => None
                                                                                                                                    }.map(|x|(x,__cloned))
                                                                                                                                }
                                                                                                                                __f},
                                                                                                                                queryable: false
                                                                                                                            });
pub static __Arng_config_EnableNoShadow_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                               name: std::borrow::Cow::from(r###"(config::EnableNoShadow{.file=(_: ast::FileId), .config=(_: ddlog_std::Ref<config::NoShadowConfig>)}: config::EnableNoShadow) /*join*/"###),
                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                {
                                                                                                                                    let __cloned = __v.clone();
                                                                                                                                    match <EnableNoShadow>::from_ddvalue(__v) {
                                                                                                                                        EnableNoShadow{file: _, config: _} => Some((()).into_ddvalue()),
                                                                                                                                        _ => None
                                                                                                                                    }.map(|x|(x,__cloned))
                                                                                                                                }
                                                                                                                                __f},
                                                                                                                                queryable: false
                                                                                                                            });
pub static __Arng_config_EnableNoTypeofUndef_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                    name: std::borrow::Cow::from(r###"(config::EnableNoTypeofUndef{.file=(_0: ast::FileId), .config=(_: ddlog_std::Ref<config::NoTypeofUndefConfig>)}: config::EnableNoTypeofUndef) /*join*/"###),
                                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                     {
                                                                                                                                         let __cloned = __v.clone();
                                                                                                                                         match <EnableNoTypeofUndef>::from_ddvalue(__v) {
                                                                                                                                             EnableNoTypeofUndef{file: ref _0, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                             _ => None
                                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     queryable: false
                                                                                                                                 });
pub static __Arng_config_EnableNoUndef_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                              name: std::borrow::Cow::from(r###"(config::EnableNoUndef{.file=(_0: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUndefConfig>)}: config::EnableNoUndef) /*join*/"###),
                                                                                                                               afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                               {
                                                                                                                                   let __cloned = __v.clone();
                                                                                                                                   match <EnableNoUndef>::from_ddvalue(__v) {
                                                                                                                                       EnableNoUndef{file: ref _0, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                       _ => None
                                                                                                                                   }.map(|x|(x,__cloned))
                                                                                                                               }
                                                                                                                               __f},
                                                                                                                               queryable: false
                                                                                                                           });
pub static __Arng_config_EnableNoUnusedLabels_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                     name: std::borrow::Cow::from(r###"(config::EnableNoUnusedLabels{.file=(_0: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedLabelsConfig>)}: config::EnableNoUnusedLabels) /*join*/"###),
                                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                      {
                                                                                                                                          let __cloned = __v.clone();
                                                                                                                                          match <EnableNoUnusedLabels>::from_ddvalue(__v) {
                                                                                                                                              EnableNoUnusedLabels{file: ref _0, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                              _ => None
                                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                                      }
                                                                                                                                      __f},
                                                                                                                                      queryable: false
                                                                                                                                  });
pub static __Arng_config_EnableNoUnusedVars_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(config::EnableNoUnusedVars{.file=(_0: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedVarsConfig>)}: config::EnableNoUnusedVars) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match <EnableNoUnusedVars>::from_ddvalue(__v) {
                                                                                                                                            EnableNoUnusedVars{file: ref _0, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_config_EnableNoUnusedVars_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(config::EnableNoUnusedVars{.file=(_: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedVarsConfig>)}: config::EnableNoUnusedVars) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match <EnableNoUnusedVars>::from_ddvalue(__v) {
                                                                                                                                            EnableNoUnusedVars{file: _, config: _} => Some((()).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_config_EnableNoUseBeforeDef_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                     name: std::borrow::Cow::from(r###"(config::EnableNoUseBeforeDef{.file=(_0: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUseBeforeDefConfig>)}: config::EnableNoUseBeforeDef) /*join*/"###),
                                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                      {
                                                                                                                                          let __cloned = __v.clone();
                                                                                                                                          match <EnableNoUseBeforeDef>::from_ddvalue(__v) {
                                                                                                                                              EnableNoUseBeforeDef{file: ref _0, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                              _ => None
                                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                                      }
                                                                                                                                      __f},
                                                                                                                                      queryable: false
                                                                                                                                  });