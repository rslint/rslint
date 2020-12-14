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

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum DeclarationScope {
    Unhoistable {
        scope: crate::ast::ScopeId
    },
    Hoistable {
        hoisted: crate::ast::ScopeId,
        unhoisted: crate::ast::ScopeId
    }
}
impl abomonation::Abomonation for DeclarationScope{}
::differential_datalog::decl_enum_from_record!(DeclarationScope["var_decls::DeclarationScope"]<>, Unhoistable["var_decls::Unhoistable"][1]{[0]scope["scope"]: crate::ast::ScopeId}, Hoistable["var_decls::Hoistable"][2]{[0]hoisted["hoisted"]: crate::ast::ScopeId, [1]unhoisted["unhoisted"]: crate::ast::ScopeId});
::differential_datalog::decl_enum_into_record!(DeclarationScope<>, Unhoistable["var_decls::Unhoistable"]{scope}, Hoistable["var_decls::Hoistable"]{hoisted, unhoisted});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(DeclarationScope<>, Unhoistable{scope: crate::ast::ScopeId}, Hoistable{hoisted: crate::ast::ScopeId, unhoisted: crate::ast::ScopeId});
impl ::std::fmt::Display for DeclarationScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::var_decls::DeclarationScope::Unhoistable{scope} => {
                __formatter.write_str("var_decls::Unhoistable{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            },
            crate::var_decls::DeclarationScope::Hoistable{hoisted,unhoisted} => {
                __formatter.write_str("var_decls::Hoistable{")?;
                ::std::fmt::Debug::fmt(hoisted, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(unhoisted, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for DeclarationScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for DeclarationScope {
    fn default() -> Self {
        crate::var_decls::DeclarationScope::Unhoistable{scope : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct VariableDeclarations {
    pub file: crate::ast::FileId,
    pub name: crate::ast::Name,
    pub scope: crate::var_decls::DeclarationScope,
    pub declared_in: crate::ast::AnyId,
    pub meta: crate::ddlog_std::Ref<crate::var_decls::VariableMeta>
}
impl abomonation::Abomonation for VariableDeclarations{}
::differential_datalog::decl_struct_from_record!(VariableDeclarations["var_decls::VariableDeclarations"]<>, ["var_decls::VariableDeclarations"][5]{[0]file["file"]: crate::ast::FileId, [1]name["name"]: crate::ast::Name, [2]scope["scope"]: crate::var_decls::DeclarationScope, [3]declared_in["declared_in"]: crate::ast::AnyId, [4]meta["meta"]: crate::ddlog_std::Ref<crate::var_decls::VariableMeta>});
::differential_datalog::decl_struct_into_record!(VariableDeclarations, ["var_decls::VariableDeclarations"]<>, file, name, scope, declared_in, meta);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(VariableDeclarations, <>, file: crate::ast::FileId, name: crate::ast::Name, scope: crate::var_decls::DeclarationScope, declared_in: crate::ast::AnyId, meta: crate::ddlog_std::Ref<crate::var_decls::VariableMeta>);
impl ::std::fmt::Display for VariableDeclarations {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::var_decls::VariableDeclarations{file,name,scope,declared_in,meta} => {
                __formatter.write_str("var_decls::VariableDeclarations{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared_in, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(meta, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct VariableMeta {
    pub is_function_argument: bool,
    pub implicitly_declared: bool,
    pub declaration_span: crate::ddlog_std::Option<crate::ast::Span>
}
impl abomonation::Abomonation for VariableMeta{}
::differential_datalog::decl_struct_from_record!(VariableMeta["var_decls::VariableMeta"]<>, ["var_decls::VariableMeta"][3]{[0]is_function_argument["is_function_argument"]: bool, [1]implicitly_declared["implicitly_declared"]: bool, [2]declaration_span["declaration_span"]: crate::ddlog_std::Option<crate::ast::Span>});
::differential_datalog::decl_struct_into_record!(VariableMeta, ["var_decls::VariableMeta"]<>, is_function_argument, implicitly_declared, declaration_span);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(VariableMeta, <>, is_function_argument: bool, implicitly_declared: bool, declaration_span: crate::ddlog_std::Option<crate::ast::Span>);
impl ::std::fmt::Display for VariableMeta {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::var_decls::VariableMeta{is_function_argument,implicitly_declared,declaration_span} => {
                __formatter.write_str("var_decls::VariableMeta{")?;
                ::std::fmt::Debug::fmt(is_function_argument, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(implicitly_declared, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declaration_span, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for VariableMeta {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub fn hoisted_scope(scope: & crate::var_decls::DeclarationScope) -> crate::ast::ScopeId
{   match (*scope) {
        crate::var_decls::DeclarationScope::Unhoistable{scope: ref scope} => (*scope).clone(),
        crate::var_decls::DeclarationScope::Hoistable{hoisted: ref hoisted, unhoisted: _} => (*hoisted).clone()
    }
}
pub fn is_hoistable(scope: & crate::var_decls::DeclarationScope) -> bool
{   match (*scope) {
        crate::var_decls::DeclarationScope::Unhoistable{scope: _} => false,
        crate::var_decls::DeclarationScope::Hoistable{hoisted: _, unhoisted: _} => true
    }
}
pub fn is_unhoistable(scope: & crate::var_decls::DeclarationScope) -> bool
{   match (*scope) {
        crate::var_decls::DeclarationScope::Unhoistable{scope: _} => true,
        crate::var_decls::DeclarationScope::Hoistable{hoisted: _, unhoisted: _} => false
    }
}
pub fn unhoisted_scope(scope: & crate::var_decls::DeclarationScope) -> crate::ast::ScopeId
{   match (*scope) {
        crate::var_decls::DeclarationScope::Unhoistable{scope: ref scope} => (*scope).clone(),
        crate::var_decls::DeclarationScope::Hoistable{hoisted: _, unhoisted: ref unhoisted} => (*unhoisted).clone()
    }
}