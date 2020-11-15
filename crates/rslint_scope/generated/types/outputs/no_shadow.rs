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

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NoShadow {
    pub variable: crate::ast::Name,
    pub original: crate::ddlog_std::tuple2<crate::ast::AnyId, crate::ast::Span>,
    pub shadower: crate::ddlog_std::tuple2<crate::ast::AnyId, crate::ast::Span>,
    pub implicit: bool,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for NoShadow{}
::differential_datalog::decl_struct_from_record!(NoShadow["outputs::no_shadow::NoShadow"]<>, ["outputs::no_shadow::NoShadow"][5]{[0]variable["variable"]: crate::ast::Name, [1]original["original"]: crate::ddlog_std::tuple2<crate::ast::AnyId, crate::ast::Span>, [2]shadower["shadower"]: crate::ddlog_std::tuple2<crate::ast::AnyId, crate::ast::Span>, [3]implicit["implicit"]: bool, [4]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(NoShadow, ["outputs::no_shadow::NoShadow"]<>, variable, original, shadower, implicit, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NoShadow, <>, variable: crate::ast::Name, original: crate::ddlog_std::tuple2<crate::ast::AnyId, crate::ast::Span>, shadower: crate::ddlog_std::tuple2<crate::ast::AnyId, crate::ast::Span>, implicit: bool, file: crate::ast::FileId);
impl ::std::fmt::Display for NoShadow {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::no_shadow::NoShadow{variable,original,shadower,implicit,file} => {
                __formatter.write_str("outputs::no_shadow::NoShadow{")?;
                ::std::fmt::Debug::fmt(variable, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(original, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(shadower, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(implicit, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NoShadow {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}