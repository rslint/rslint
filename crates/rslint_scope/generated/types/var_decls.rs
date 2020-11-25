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
pub struct VariableDeclarations {
    pub file: crate::ast::FileId,
    pub name: crate::ast::Name,
    pub scope: crate::ast::ScopeId,
    pub span: crate::ddlog_std::Option<crate::ast::Span>,
    pub declared_in: crate::ast::AnyId,
    pub implicit: bool,
    pub is_arg: bool,
    pub origin: crate::name_in_scope::NameOrigin
}
impl abomonation::Abomonation for VariableDeclarations{}
::differential_datalog::decl_struct_from_record!(VariableDeclarations["var_decls::VariableDeclarations"]<>, ["var_decls::VariableDeclarations"][8]{[0]file["file"]: crate::ast::FileId, [1]name["name"]: crate::ast::Name, [2]scope["scope"]: crate::ast::ScopeId, [3]span["span"]: crate::ddlog_std::Option<crate::ast::Span>, [4]declared_in["declared_in"]: crate::ast::AnyId, [5]implicit["implicit"]: bool, [6]is_arg["is_arg"]: bool, [7]origin["origin"]: crate::name_in_scope::NameOrigin});
::differential_datalog::decl_struct_into_record!(VariableDeclarations, ["var_decls::VariableDeclarations"]<>, file, name, scope, span, declared_in, implicit, is_arg, origin);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(VariableDeclarations, <>, file: crate::ast::FileId, name: crate::ast::Name, scope: crate::ast::ScopeId, span: crate::ddlog_std::Option<crate::ast::Span>, declared_in: crate::ast::AnyId, implicit: bool, is_arg: bool, origin: crate::name_in_scope::NameOrigin);
impl ::std::fmt::Display for VariableDeclarations {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::var_decls::VariableDeclarations{file,name,scope,span,declared_in,implicit,is_arg,origin} => {
                __formatter.write_str("var_decls::VariableDeclarations{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared_in, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(implicit, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(is_arg, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(origin, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for VariableDeclarations {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}