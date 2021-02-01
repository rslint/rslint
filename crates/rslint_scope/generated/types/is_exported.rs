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
#[ddlog(rename = "is_exported::IsExported")]
pub struct IsExported {
    pub id: types__ast::AnyId
}
impl abomonation::Abomonation for IsExported{}
impl ::std::fmt::Display for IsExported {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            IsExported{id} => {
                __formatter.write_str("is_exported::IsExported{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for IsExported {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub static __Arng_is_exported_IsExported_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                 name: std::borrow::Cow::from(r###"(is_exported::IsExported{.id=(_0: ast::AnyId)}: is_exported::IsExported) /*antijoin*/"###),
                                                                                                                                 fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                 {
                                                                                                                                     match <IsExported>::from_ddvalue(__v) {
                                                                                                                                         IsExported{id: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                         _ => None
                                                                                                                                     }
                                                                                                                                 }
                                                                                                                                 __f},
                                                                                                                                 distinct: true
                                                                                                                             });
pub static __Rule_is_exported_IsExported_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* is_exported::IsExported[(is_exported::IsExported{.id=(ast::AnyIdFunc{.func=id}: ast::AnyId)}: is_exported::IsExported)] :- inputs::Function[(inputs::Function{.id=(id: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=true}: inputs::Function)]. */
                                                                                                                      program::Rule::CollectionRule {
                                                                                                                          description: std::borrow::Cow::from("is_exported::IsExported(.id=ast::AnyIdFunc{.func=id}) :- inputs::Function(.id=id, .name=_, .scope=_, .body=_, .exported=true)."),
                                                                                                                          rel: 33,
                                                                                                                          xform: Some(XFormCollection::FilterMap{
                                                                                                                                          description: std::borrow::Cow::from("head of is_exported::IsExported(.id=ast::AnyIdFunc{.func=id}) :- inputs::Function(.id=id, .name=_, .scope=_, .body=_, .exported=true)."),
                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                          {
                                                                                                                                              let ref id = match *<types__inputs::Function>::from_ddvalue_ref(&__v) {
                                                                                                                                                  types__inputs::Function{id: ref id, name: _, scope: _, body: _, exported: true} => (*id).clone(),
                                                                                                                                                  _ => return None
                                                                                                                                              };
                                                                                                                                              Some(((IsExported{id: (types__ast::AnyId::AnyIdFunc{func: (*id).clone()})})).into_ddvalue())
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          next: Box::new(None)
                                                                                                                                      })
                                                                                                                      });
pub static __Rule_is_exported_IsExported_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* is_exported::IsExported[(is_exported::IsExported{.id=(ast::AnyIdClass{.class=id}: ast::AnyId)}: is_exported::IsExported)] :- inputs::Class[(inputs::Class{.id=(id: ast::ClassId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=true}: inputs::Class)]. */
                                                                                                                      program::Rule::CollectionRule {
                                                                                                                          description: std::borrow::Cow::from("is_exported::IsExported(.id=ast::AnyIdClass{.class=id}) :- inputs::Class(.id=id, .name=_, .parent=_, .elements=_, .scope=_, .exported=true)."),
                                                                                                                          rel: 16,
                                                                                                                          xform: Some(XFormCollection::FilterMap{
                                                                                                                                          description: std::borrow::Cow::from("head of is_exported::IsExported(.id=ast::AnyIdClass{.class=id}) :- inputs::Class(.id=id, .name=_, .parent=_, .elements=_, .scope=_, .exported=true)."),
                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                          {
                                                                                                                                              let ref id = match *<types__inputs::Class>::from_ddvalue_ref(&__v) {
                                                                                                                                                  types__inputs::Class{id: ref id, name: _, parent: _, elements: _, scope: _, exported: true} => (*id).clone(),
                                                                                                                                                  _ => return None
                                                                                                                                              };
                                                                                                                                              Some(((IsExported{id: (types__ast::AnyId::AnyIdClass{class: (*id).clone()})})).into_ddvalue())
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          next: Box::new(None)
                                                                                                                                      })
                                                                                                                      });
pub static __Rule_is_exported_IsExported_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* is_exported::IsExported[(is_exported::IsExported{.id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: is_exported::IsExported)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(id: ast::StmtId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::VarDecl)]. */
                                                                                                                      program::Rule::CollectionRule {
                                                                                                                          description: std::borrow::Cow::from("is_exported::IsExported(.id=ast::AnyIdStmt{.stmt=id}) :- inputs::VarDecl(.stmt_id=id, .pattern=_, .value=_, .exported=true)."),
                                                                                                                          rel: 56,
                                                                                                                          xform: Some(XFormCollection::FilterMap{
                                                                                                                                          description: std::borrow::Cow::from("head of is_exported::IsExported(.id=ast::AnyIdStmt{.stmt=id}) :- inputs::VarDecl(.stmt_id=id, .pattern=_, .value=_, .exported=true)."),
                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                          {
                                                                                                                                              let ref id = match *<types__inputs::VarDecl>::from_ddvalue_ref(&__v) {
                                                                                                                                                  types__inputs::VarDecl{stmt_id: ref id, pattern: _, value: _, exported: true} => (*id).clone(),
                                                                                                                                                  _ => return None
                                                                                                                                              };
                                                                                                                                              Some(((IsExported{id: (types__ast::AnyId::AnyIdStmt{stmt: (*id).clone()})})).into_ddvalue())
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          next: Box::new(None)
                                                                                                                                      })
                                                                                                                      });
pub static __Rule_is_exported_IsExported_3 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* is_exported::IsExported[(is_exported::IsExported{.id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: is_exported::IsExported)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(id: ast::StmtId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::LetDecl)]. */
                                                                                                                      program::Rule::CollectionRule {
                                                                                                                          description: std::borrow::Cow::from("is_exported::IsExported(.id=ast::AnyIdStmt{.stmt=id}) :- inputs::LetDecl(.stmt_id=id, .pattern=_, .value=_, .exported=true)."),
                                                                                                                          rel: 42,
                                                                                                                          xform: Some(XFormCollection::FilterMap{
                                                                                                                                          description: std::borrow::Cow::from("head of is_exported::IsExported(.id=ast::AnyIdStmt{.stmt=id}) :- inputs::LetDecl(.stmt_id=id, .pattern=_, .value=_, .exported=true)."),
                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                          {
                                                                                                                                              let ref id = match *<types__inputs::LetDecl>::from_ddvalue_ref(&__v) {
                                                                                                                                                  types__inputs::LetDecl{stmt_id: ref id, pattern: _, value: _, exported: true} => (*id).clone(),
                                                                                                                                                  _ => return None
                                                                                                                                              };
                                                                                                                                              Some(((IsExported{id: (types__ast::AnyId::AnyIdStmt{stmt: (*id).clone()})})).into_ddvalue())
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          next: Box::new(None)
                                                                                                                                      })
                                                                                                                      });
pub static __Rule_is_exported_IsExported_4 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* is_exported::IsExported[(is_exported::IsExported{.id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: is_exported::IsExported)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(id: ast::StmtId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::ConstDecl)]. */
                                                                                                                      program::Rule::CollectionRule {
                                                                                                                          description: std::borrow::Cow::from("is_exported::IsExported(.id=ast::AnyIdStmt{.stmt=id}) :- inputs::ConstDecl(.stmt_id=id, .pattern=_, .value=_, .exported=true)."),
                                                                                                                          rel: 18,
                                                                                                                          xform: Some(XFormCollection::FilterMap{
                                                                                                                                          description: std::borrow::Cow::from("head of is_exported::IsExported(.id=ast::AnyIdStmt{.stmt=id}) :- inputs::ConstDecl(.stmt_id=id, .pattern=_, .value=_, .exported=true)."),
                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                          {
                                                                                                                                              let ref id = match *<types__inputs::ConstDecl>::from_ddvalue_ref(&__v) {
                                                                                                                                                  types__inputs::ConstDecl{stmt_id: ref id, pattern: _, value: _, exported: true} => (*id).clone(),
                                                                                                                                                  _ => return None
                                                                                                                                              };
                                                                                                                                              Some(((IsExported{id: (types__ast::AnyId::AnyIdStmt{stmt: (*id).clone()})})).into_ddvalue())
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          next: Box::new(None)
                                                                                                                                      })
                                                                                                                      });
pub static __Rule_is_exported_IsExported_5 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* is_exported::IsExported[(is_exported::IsExported{.id=id}: is_exported::IsExported)] :- inputs::FileExport[(inputs::FileExport{.export=(ast::NamedExport{.name=(export_name: ddlog_std::Option<ast::Spanned<ast::Name>>), .alias=(export_alias: ddlog_std::Option<ast::Spanned<ast::Name>>)}: ast::ExportKind), .scope=(export_scope: ast::ScopeId)}: inputs::FileExport)], ((ddlog_std::Some{.x=(ast::Spanned{.data=(var name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<internment::Intern<string>>>) = ((utils::or_else: function(ddlog_std::Option<ast::Spanned<ast::Name>>, ddlog_std::Option<ast::Spanned<ast::Name>>):ddlog_std::Option<ast::Spanned<internment::Intern<string>>>)(export_alias, export_name))), name_in_scope::NameInScope[(name_in_scope::NameInScope{.name=(name: internment::Intern<string>), .scope=(export_scope: ast::ScopeId), .declared=(id: ast::AnyId)}: name_in_scope::NameInScope)], var_decls::VariableDeclarations[(var_decls::VariableDeclarations{.name=(name: internment::Intern<string>), .scope=(scope: var_decls::DeclarationScope), .declared_in=(id: ast::AnyId), .meta=(_: ddlog_std::Ref<var_decls::VariableMeta>)}: var_decls::VariableDeclarations)], ((var_decls::hoisted_scope(scope)) == export_scope). */
                                                                                                                      program::Rule::CollectionRule {
                                                                                                                          description: std::borrow::Cow::from("is_exported::IsExported(.id=id) :- inputs::FileExport(.export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=export_scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name))), name_in_scope::NameInScope(.name=name, .scope=export_scope, .declared=id), var_decls::VariableDeclarations(.name=name, .scope=scope, .declared_in=id, .meta=_), ((var_decls::hoisted_scope(scope)) == export_scope)."),
                                                                                                                          rel: 29,
                                                                                                                          xform: Some(XFormCollection::Arrange {
                                                                                                                                          description: std::borrow::Cow::from("arrange inputs::FileExport(.export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=export_scope) by (name, export_scope)"),
                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                          {
                                                                                                                                              let (ref export_name, ref export_alias, ref export_scope) = match *<types__inputs::FileExport>::from_ddvalue_ref(&__v) {
                                                                                                                                                  types__inputs::FileExport{export: types__ast::ExportKind::NamedExport{name: ref export_name, alias: ref export_alias}, scope: ref export_scope} => ((*export_name).clone(), (*export_alias).clone(), (*export_scope).clone()),
                                                                                                                                                  _ => return None
                                                                                                                                              };
                                                                                                                                              let ref name: internment::Intern<String> = match types__utils::or_else::<types__ast::Spanned<types__ast::Name>>(export_alias, export_name) {
                                                                                                                                                  ddlog_std::Option::Some{x: types__ast::Spanned{data: name, span: _}} => name,
                                                                                                                                                  _ => return None
                                                                                                                                              };
                                                                                                                                              Some(((ddlog_std::tuple2((*name).clone(), (*export_scope).clone())).into_ddvalue(), (ddlog_std::tuple2((*export_scope).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          next: Box::new(XFormArrangement::Join{
                                                                                                                                                             description: std::borrow::Cow::from("inputs::FileExport(.export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=export_scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name))), name_in_scope::NameInScope(.name=name, .scope=export_scope, .declared=id)"),
                                                                                                                                                             ffun: None,
                                                                                                                                                             arrangement: (61,0),
                                                                                                                                                             jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                             {
                                                                                                                                                                 let ddlog_std::tuple2(ref export_scope, ref name) = *<ddlog_std::tuple2<types__ast::ScopeId, internment::Intern<String>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                 let ref id = match *<crate::name_in_scope::NameInScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                                     crate::name_in_scope::NameInScope{name: _, scope: _, declared: ref id} => (*id).clone(),
                                                                                                                                                                     _ => return None
                                                                                                                                                                 };
                                                                                                                                                                 Some((ddlog_std::tuple3((*export_scope).clone(), (*name).clone(), (*id).clone())).into_ddvalue())
                                                                                                                                                             }
                                                                                                                                                             __f},
                                                                                                                                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                     description: std::borrow::Cow::from("arrange inputs::FileExport(.export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=export_scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name))), name_in_scope::NameInScope(.name=name, .scope=export_scope, .declared=id) by (name, id)"),
                                                                                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                     {
                                                                                                                                                                                         let ddlog_std::tuple3(ref export_scope, ref name, ref id) = *<ddlog_std::tuple3<types__ast::ScopeId, internment::Intern<String>, types__ast::AnyId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                         Some(((ddlog_std::tuple2((*name).clone(), (*id).clone())).into_ddvalue(), (ddlog_std::tuple2((*export_scope).clone(), (*id).clone())).into_ddvalue()))
                                                                                                                                                                                     }
                                                                                                                                                                                     __f},
                                                                                                                                                                                     next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                                        description: std::borrow::Cow::from("inputs::FileExport(.export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=export_scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name))), name_in_scope::NameInScope(.name=name, .scope=export_scope, .declared=id), var_decls::VariableDeclarations(.name=name, .scope=scope, .declared_in=id, .meta=_)"),
                                                                                                                                                                                                        ffun: None,
                                                                                                                                                                                                        arrangement: (86,0),
                                                                                                                                                                                                        jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                                        {
                                                                                                                                                                                                            let ddlog_std::tuple2(ref export_scope, ref id) = *<ddlog_std::tuple2<types__ast::ScopeId, types__ast::AnyId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                            let ref scope = match *<crate::var_decls::VariableDeclarations>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                                crate::var_decls::VariableDeclarations{name: _, scope: ref scope, declared_in: _, meta: _} => (*scope).clone(),
                                                                                                                                                                                                                _ => return None
                                                                                                                                                                                                            };
                                                                                                                                                                                                            if !((&*(&crate::var_decls::hoisted_scope(scope))) == (&*export_scope)) {return None;};
                                                                                                                                                                                                            Some(((IsExported{id: (*id).clone()})).into_ddvalue())
                                                                                                                                                                                                        }
                                                                                                                                                                                                        __f},
                                                                                                                                                                                                        next: Box::new(None)
                                                                                                                                                                                                    })
                                                                                                                                                                                 }))
                                                                                                                                                         })
                                                                                                                                      })
                                                                                                                      });