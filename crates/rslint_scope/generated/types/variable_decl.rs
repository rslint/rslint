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
pub struct VariableDecl {
    pub stmt: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub kind: crate::variable_decl::VariableDeclKind,
    pub pattern: crate::ddlog_std::Option<crate::ast::IPattern>,
    pub value: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for VariableDecl{}
::differential_datalog::decl_struct_from_record!(VariableDecl["variable_decl::VariableDecl"]<>, ["variable_decl::VariableDecl"][6]{[0]stmt["stmt"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]kind["kind"]: crate::variable_decl::VariableDeclKind, [3]pattern["pattern"]: crate::ddlog_std::Option<crate::ast::IPattern>, [4]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>, [5]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(VariableDecl, ["variable_decl::VariableDecl"]<>, stmt, file, kind, pattern, value, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(VariableDecl, <>, stmt: crate::ast::StmtId, file: crate::ast::FileId, kind: crate::variable_decl::VariableDeclKind, pattern: crate::ddlog_std::Option<crate::ast::IPattern>, value: crate::ddlog_std::Option<crate::ast::ExprId>, exported: bool);
impl ::std::fmt::Display for VariableDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::variable_decl::VariableDecl{stmt,file,kind,pattern,value,exported} => {
                __formatter.write_str("variable_decl::VariableDecl{")?;
                ::std::fmt::Debug::fmt(stmt, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(pattern, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(exported, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for VariableDecl {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum VariableDeclKind {
    VarDeclVar,
    VarDeclLet,
    VarDeclConst
}
impl abomonation::Abomonation for VariableDeclKind{}
::differential_datalog::decl_enum_from_record!(VariableDeclKind["variable_decl::VariableDeclKind"]<>, VarDeclVar["variable_decl::VarDeclVar"][0]{}, VarDeclLet["variable_decl::VarDeclLet"][0]{}, VarDeclConst["variable_decl::VarDeclConst"][0]{});
::differential_datalog::decl_enum_into_record!(VariableDeclKind<>, VarDeclVar["variable_decl::VarDeclVar"]{}, VarDeclLet["variable_decl::VarDeclLet"]{}, VarDeclConst["variable_decl::VarDeclConst"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(VariableDeclKind<>, VarDeclVar{}, VarDeclLet{}, VarDeclConst{});
impl ::std::fmt::Display for VariableDeclKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::variable_decl::VariableDeclKind::VarDeclVar{} => {
                __formatter.write_str("variable_decl::VarDeclVar{")?;
                __formatter.write_str("}")
            },
            crate::variable_decl::VariableDeclKind::VarDeclLet{} => {
                __formatter.write_str("variable_decl::VarDeclLet{")?;
                __formatter.write_str("}")
            },
            crate::variable_decl::VariableDeclKind::VarDeclConst{} => {
                __formatter.write_str("variable_decl::VarDeclConst{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for VariableDeclKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for VariableDeclKind {
    fn default() -> Self {
        crate::variable_decl::VariableDeclKind::VarDeclVar{}
    }
}