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


#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "variable_decl::VariableDecl")]
pub struct VariableDecl {
    pub stmt: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub kind: VariableDeclKind,
    pub pattern: ddlog_std::Option<types__ast::IPattern>,
    pub value: ddlog_std::Option<types__ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for VariableDecl{}
impl ::std::fmt::Display for VariableDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            VariableDecl{stmt,file,kind,pattern,value,exported} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "variable_decl::VariableDeclKind")]
pub enum VariableDeclKind {
    #[ddlog(rename = "variable_decl::VarDeclVar")]
    VarDeclVar,
    #[ddlog(rename = "variable_decl::VarDeclLet")]
    VarDeclLet,
    #[ddlog(rename = "variable_decl::VarDeclConst")]
    VarDeclConst
}
impl abomonation::Abomonation for VariableDeclKind{}
impl ::std::fmt::Display for VariableDeclKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            VariableDeclKind::VarDeclVar{} => {
                __formatter.write_str("variable_decl::VarDeclVar{")?;
                __formatter.write_str("}")
            },
            VariableDeclKind::VarDeclLet{} => {
                __formatter.write_str("variable_decl::VarDeclLet{")?;
                __formatter.write_str("}")
            },
            VariableDeclKind::VarDeclConst{} => {
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
        VariableDeclKind::VarDeclVar{}
    }
}
pub static __Arng_variable_decl_VariableDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                    name: std::borrow::Cow::from(r###"(variable_decl::VariableDecl{.stmt=(_0: ast::StmtId), .file=(_1: ast::FileId), .kind=(_: variable_decl::VariableDeclKind), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: variable_decl::VariableDecl) /*join*/"###),
                                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                     {
                                                                                                                                         let __cloned = __v.clone();
                                                                                                                                         match < VariableDecl>::from_ddvalue(__v) {
                                                                                                                                             VariableDecl{stmt: ref _0, file: ref _1, kind: _, pattern: _, value: ddlog_std::Option::Some{x: _}, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                             _ => None
                                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     queryable: false
                                                                                                                                 });
pub static __Rule_variable_decl_VariableDecl_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* variable_decl::VariableDecl[(variable_decl::VariableDecl{.stmt=stmt, .file=file, .kind=(variable_decl::VarDeclVar{}: variable_decl::VariableDeclKind), .pattern=pattern, .value=value, .exported=exported}: variable_decl::VariableDecl)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(pattern: ddlog_std::Option<ast::IPattern>), .value=(value: ddlog_std::Option<ast::ExprId>), .exported=(exported: bool)}: inputs::VarDecl)]. */
                                                                                                                          program::Rule::CollectionRule {
                                                                                                                              description: std::borrow::Cow::from("variable_decl::VariableDecl(.stmt=stmt, .file=file, .kind=variable_decl::VarDeclVar{}, .pattern=pattern, .value=value, .exported=exported) :- inputs::VarDecl(.stmt_id=stmt, .file=file, .pattern=pattern, .value=value, .exported=exported)."),
                                                                                                                              rel: 57,
                                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                                              description: std::borrow::Cow::from("head of variable_decl::VariableDecl(.stmt=stmt, .file=file, .kind=variable_decl::VarDeclVar{}, .pattern=pattern, .value=value, .exported=exported) :- inputs::VarDecl(.stmt_id=stmt, .file=file, .pattern=pattern, .value=value, .exported=exported)."),
                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref stmt, ref file, ref pattern, ref value, ref exported) = match *<types__inputs::VarDecl>::from_ddvalue_ref(&__v) {
                                                                                                                                                      types__inputs::VarDecl{stmt_id: ref stmt, file: ref file, pattern: ref pattern, value: ref value, exported: ref exported} => ((*stmt).clone(), (*file).clone(), (*pattern).clone(), (*value).clone(), (*exported).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some(((VariableDecl{stmt: (*stmt).clone(), file: (*file).clone(), kind: (VariableDeclKind::VarDeclVar{}), pattern: (*pattern).clone(), value: (*value).clone(), exported: (*exported).clone()})).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(None)
                                                                                                                                          })
                                                                                                                          });
pub static __Rule_variable_decl_VariableDecl_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* variable_decl::VariableDecl[(variable_decl::VariableDecl{.stmt=stmt, .file=file, .kind=(variable_decl::VarDeclLet{}: variable_decl::VariableDeclKind), .pattern=pattern, .value=value, .exported=exported}: variable_decl::VariableDecl)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(pattern: ddlog_std::Option<ast::IPattern>), .value=(value: ddlog_std::Option<ast::ExprId>), .exported=(exported: bool)}: inputs::LetDecl)]. */
                                                                                                                          program::Rule::CollectionRule {
                                                                                                                              description: std::borrow::Cow::from("variable_decl::VariableDecl(.stmt=stmt, .file=file, .kind=variable_decl::VarDeclLet{}, .pattern=pattern, .value=value, .exported=exported) :- inputs::LetDecl(.stmt_id=stmt, .file=file, .pattern=pattern, .value=value, .exported=exported)."),
                                                                                                                              rel: 43,
                                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                                              description: std::borrow::Cow::from("head of variable_decl::VariableDecl(.stmt=stmt, .file=file, .kind=variable_decl::VarDeclLet{}, .pattern=pattern, .value=value, .exported=exported) :- inputs::LetDecl(.stmt_id=stmt, .file=file, .pattern=pattern, .value=value, .exported=exported)."),
                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref stmt, ref file, ref pattern, ref value, ref exported) = match *<types__inputs::LetDecl>::from_ddvalue_ref(&__v) {
                                                                                                                                                      types__inputs::LetDecl{stmt_id: ref stmt, file: ref file, pattern: ref pattern, value: ref value, exported: ref exported} => ((*stmt).clone(), (*file).clone(), (*pattern).clone(), (*value).clone(), (*exported).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some(((VariableDecl{stmt: (*stmt).clone(), file: (*file).clone(), kind: (VariableDeclKind::VarDeclLet{}), pattern: (*pattern).clone(), value: (*value).clone(), exported: (*exported).clone()})).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(None)
                                                                                                                                          })
                                                                                                                          });
pub static __Rule_variable_decl_VariableDecl_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* variable_decl::VariableDecl[(variable_decl::VariableDecl{.stmt=stmt, .file=file, .kind=(variable_decl::VarDeclConst{}: variable_decl::VariableDeclKind), .pattern=pattern, .value=value, .exported=exported}: variable_decl::VariableDecl)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(pattern: ddlog_std::Option<ast::IPattern>), .value=(value: ddlog_std::Option<ast::ExprId>), .exported=(exported: bool)}: inputs::ConstDecl)]. */
                                                                                                                          program::Rule::CollectionRule {
                                                                                                                              description: std::borrow::Cow::from("variable_decl::VariableDecl(.stmt=stmt, .file=file, .kind=variable_decl::VarDeclConst{}, .pattern=pattern, .value=value, .exported=exported) :- inputs::ConstDecl(.stmt_id=stmt, .file=file, .pattern=pattern, .value=value, .exported=exported)."),
                                                                                                                              rel: 19,
                                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                                              description: std::borrow::Cow::from("head of variable_decl::VariableDecl(.stmt=stmt, .file=file, .kind=variable_decl::VarDeclConst{}, .pattern=pattern, .value=value, .exported=exported) :- inputs::ConstDecl(.stmt_id=stmt, .file=file, .pattern=pattern, .value=value, .exported=exported)."),
                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref stmt, ref file, ref pattern, ref value, ref exported) = match *<types__inputs::ConstDecl>::from_ddvalue_ref(&__v) {
                                                                                                                                                      types__inputs::ConstDecl{stmt_id: ref stmt, file: ref file, pattern: ref pattern, value: ref value, exported: ref exported} => ((*stmt).clone(), (*file).clone(), (*pattern).clone(), (*value).clone(), (*exported).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some(((VariableDecl{stmt: (*stmt).clone(), file: (*file).clone(), kind: (VariableDeclKind::VarDeclConst{}), pattern: (*pattern).clone(), value: (*value).clone(), exported: (*exported).clone()})).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(None)
                                                                                                                                          })
                                                                                                                          });