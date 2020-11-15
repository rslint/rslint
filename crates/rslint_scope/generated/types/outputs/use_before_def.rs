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
pub struct UseBeforeDef {
    pub name: crate::ast::Name,
    pub used: crate::ast::ExprId,
    pub used_in: crate::ast::Span,
    pub declared: crate::ast::AnyId,
    pub declared_in: crate::ast::Span,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for UseBeforeDef{}
::differential_datalog::decl_struct_from_record!(UseBeforeDef["outputs::use_before_def::UseBeforeDef"]<>, ["outputs::use_before_def::UseBeforeDef"][6]{[0]name["name"]: crate::ast::Name, [1]used["used"]: crate::ast::ExprId, [2]used_in["used_in"]: crate::ast::Span, [3]declared["declared"]: crate::ast::AnyId, [4]declared_in["declared_in"]: crate::ast::Span, [5]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(UseBeforeDef, ["outputs::use_before_def::UseBeforeDef"]<>, name, used, used_in, declared, declared_in, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(UseBeforeDef, <>, name: crate::ast::Name, used: crate::ast::ExprId, used_in: crate::ast::Span, declared: crate::ast::AnyId, declared_in: crate::ast::Span, file: crate::ast::FileId);
impl ::std::fmt::Display for UseBeforeDef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::use_before_def::UseBeforeDef{name,used,used_in,declared,declared_in,file} => {
                __formatter.write_str("outputs::use_before_def::UseBeforeDef{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(used, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(used_in, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared_in, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for UseBeforeDef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}