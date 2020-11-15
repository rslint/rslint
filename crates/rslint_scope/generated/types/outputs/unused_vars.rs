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
pub struct UnusedVariables {
    pub name: crate::ast::Name,
    pub declared: crate::ast::AnyId,
    pub span: crate::ast::Span,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for UnusedVariables{}
::differential_datalog::decl_struct_from_record!(UnusedVariables["outputs::unused_vars::UnusedVariables"]<>, ["outputs::unused_vars::UnusedVariables"][4]{[0]name["name"]: crate::ast::Name, [1]declared["declared"]: crate::ast::AnyId, [2]span["span"]: crate::ast::Span, [3]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(UnusedVariables, ["outputs::unused_vars::UnusedVariables"]<>, name, declared, span, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(UnusedVariables, <>, name: crate::ast::Name, declared: crate::ast::AnyId, span: crate::ast::Span, file: crate::ast::FileId);
impl ::std::fmt::Display for UnusedVariables {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::unused_vars::UnusedVariables{name,declared,span,file} => {
                __formatter.write_str("outputs::unused_vars::UnusedVariables{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for UnusedVariables {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct VariableUsages {
    pub file: crate::ast::FileId,
    pub name: crate::ast::Name,
    pub scope: crate::ast::ScopeId,
    pub declared_in: crate::ast::AnyId
}
impl abomonation::Abomonation for VariableUsages{}
::differential_datalog::decl_struct_from_record!(VariableUsages["outputs::unused_vars::VariableUsages"]<>, ["outputs::unused_vars::VariableUsages"][4]{[0]file["file"]: crate::ast::FileId, [1]name["name"]: crate::ast::Name, [2]scope["scope"]: crate::ast::ScopeId, [3]declared_in["declared_in"]: crate::ast::AnyId});
::differential_datalog::decl_struct_into_record!(VariableUsages, ["outputs::unused_vars::VariableUsages"]<>, file, name, scope, declared_in);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(VariableUsages, <>, file: crate::ast::FileId, name: crate::ast::Name, scope: crate::ast::ScopeId, declared_in: crate::ast::AnyId);
impl ::std::fmt::Display for VariableUsages {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::unused_vars::VariableUsages{file,name,scope,declared_in} => {
                __formatter.write_str("outputs::unused_vars::VariableUsages{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared_in, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for VariableUsages {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}