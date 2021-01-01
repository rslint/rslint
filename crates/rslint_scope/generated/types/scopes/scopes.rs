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
#[ddlog(rename = "scopes::FunctionLevelScope")]
pub struct FunctionLevelScope {
    pub scope: types__ast::ScopeId,
    pub nearest: types__ast::ScopeId,
    pub file: types__ast::FileId,
    pub id: types__ast::AnyId
}
impl abomonation::Abomonation for FunctionLevelScope{}
impl ::std::fmt::Display for FunctionLevelScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FunctionLevelScope{scope,nearest,file,id} => {
                __formatter.write_str("scopes::FunctionLevelScope{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(nearest, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FunctionLevelScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "scopes::IsHoistable")]
pub struct IsHoistable {
    pub file: types__ast::FileId,
    pub id: types__ast::AnyId,
    pub hoistable: bool
}
impl abomonation::Abomonation for IsHoistable{}
impl ::std::fmt::Display for IsHoistable {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            IsHoistable{file,id,hoistable} => {
                __formatter.write_str("scopes::IsHoistable{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(hoistable, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for IsHoistable {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "scopes::NeedsScopeChildren")]
pub struct NeedsScopeChildren {
    pub scope: types__ast::ScopeId,
    pub file: types__ast::FileId
}
impl abomonation::Abomonation for NeedsScopeChildren{}
impl ::std::fmt::Display for NeedsScopeChildren {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NeedsScopeChildren{scope,file} => {
                __formatter.write_str("scopes::NeedsScopeChildren{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NeedsScopeChildren {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "scopes::NeedsScopeParents")]
pub struct NeedsScopeParents {
    pub scope: types__ast::ScopeId,
    pub file: types__ast::FileId
}
impl abomonation::Abomonation for NeedsScopeParents{}
impl ::std::fmt::Display for NeedsScopeParents {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NeedsScopeParents{scope,file} => {
                __formatter.write_str("scopes::NeedsScopeParents{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NeedsScopeParents {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "scopes::ScopeFamily")]
pub struct ScopeFamily {
    pub parent: types__ast::ScopeId,
    pub child: types__ast::ScopeId,
    pub file: types__ast::FileId
}
impl abomonation::Abomonation for ScopeFamily{}
impl ::std::fmt::Display for ScopeFamily {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ScopeFamily{parent,child,file} => {
                __formatter.write_str("scopes::ScopeFamily{")?;
                ::std::fmt::Debug::fmt(parent, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(child, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ScopeFamily {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "scopes::ScopeOfId")]
pub struct ScopeOfId {
    pub id: types__ast::AnyId,
    pub file: types__ast::FileId,
    pub scope: types__ast::ScopeId
}
impl abomonation::Abomonation for ScopeOfId{}
impl ::std::fmt::Display for ScopeOfId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ScopeOfId{id,file,scope} => {
                __formatter.write_str("scopes::ScopeOfId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ScopeOfId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub static __Arng_scopes_NeedsScopeChildren_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(scopes::NeedsScopeChildren{.scope=(_0: ast::ScopeId), .file=(_1: ast::FileId)}: scopes::NeedsScopeChildren) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NeedsScopeChildren>::from_ddvalue(__v) {
                                                                                                                                            NeedsScopeChildren{scope: ref _0, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_scopes_FunctionLevelScope_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(scopes::FunctionLevelScope{.scope=(_0: ast::ScopeId), .nearest=(_: ast::ScopeId), .file=(_1: ast::FileId), .id=(_: ast::AnyId)}: scopes::FunctionLevelScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < FunctionLevelScope>::from_ddvalue(__v) {
                                                                                                                                            FunctionLevelScope{scope: ref _0, nearest: _, file: ref _1, id: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_scopes_NeedsScopeParents_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                  name: std::borrow::Cow::from(r###"(scopes::NeedsScopeParents{.scope=(_0: ast::ScopeId), .file=(_1: ast::FileId)}: scopes::NeedsScopeParents) /*join*/"###),
                                                                                                                                   afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                   {
                                                                                                                                       let __cloned = __v.clone();
                                                                                                                                       match < NeedsScopeParents>::from_ddvalue(__v) {
                                                                                                                                           NeedsScopeParents{scope: ref _0, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                           _ => None
                                                                                                                                       }.map(|x|(x,__cloned))
                                                                                                                                   }
                                                                                                                                   __f},
                                                                                                                                   queryable: false
                                                                                                                               });
pub static __Arng_scopes_ScopeFamily_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                            name: std::borrow::Cow::from(r###"(scopes::ScopeFamily{.parent=(_0: ast::ScopeId), .child=(_: ast::ScopeId), .file=(_1: ast::FileId)}: scopes::ScopeFamily) /*join*/"###),
                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                             {
                                                                                                                                 let __cloned = __v.clone();
                                                                                                                                 match < ScopeFamily>::from_ddvalue(__v) {
                                                                                                                                     ScopeFamily{parent: ref _0, child: _, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                     _ => None
                                                                                                                                 }.map(|x|(x,__cloned))
                                                                                                                             }
                                                                                                                             __f},
                                                                                                                             queryable: false
                                                                                                                         });
pub static __Arng_scopes_ScopeFamily_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                            name: std::borrow::Cow::from(r###"(scopes::ScopeFamily{.parent=(_: ast::ScopeId), .child=(_0: ast::ScopeId), .file=(_1: ast::FileId)}: scopes::ScopeFamily) /*join*/"###),
                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                             {
                                                                                                                                 let __cloned = __v.clone();
                                                                                                                                 match < ScopeFamily>::from_ddvalue(__v) {
                                                                                                                                     ScopeFamily{parent: _, child: ref _0, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                     _ => None
                                                                                                                                 }.map(|x|(x,__cloned))
                                                                                                                             }
                                                                                                                             __f},
                                                                                                                             queryable: false
                                                                                                                         });
pub static __Arng_scopes_ScopeFamily_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                            name: std::borrow::Cow::from(r###"(scopes::ScopeFamily{.parent=_0, .child=(_: ast::ScopeId), .file=_1}: scopes::ScopeFamily) /*join*/"###),
                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                             {
                                                                                                                                 let __cloned = __v.clone();
                                                                                                                                 match < ScopeFamily>::from_ddvalue(__v) {
                                                                                                                                     ScopeFamily{parent: ref _0, child: _, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                     _ => None
                                                                                                                                 }.map(|x|(x,__cloned))
                                                                                                                             }
                                                                                                                             __f},
                                                                                                                             queryable: true
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdFile{.file=file}: ast::AnyId)}: scopes::FunctionLevelScope)] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(scope: ast::ScopeId)}: inputs::File)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdFile{.file=file}) :- inputs::File(.id=file, .kind=_, .top_level_scope=scope)."),
                                                                                                                             rel: 29,
                                                                                                                             xform: Some(XFormCollection::FilterMap{
                                                                                                                                             description: std::borrow::Cow::from("head of scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdFile{.file=file}) :- inputs::File(.id=file, .kind=_, .top_level_scope=scope)."),
                                                                                                                                             fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                             {
                                                                                                                                                 let (ref file, ref scope) = match *<types__inputs::File>::from_ddvalue_ref(&__v) {
                                                                                                                                                     types__inputs::File{id: ref file, kind: _, top_level_scope: ref scope} => ((*file).clone(), (*scope).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 Some(((FunctionLevelScope{scope: (*scope).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (types__ast::AnyId::AnyIdFile{file: (*file).clone()})})).into_ddvalue())
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(None)
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=body, .nearest=body, .file=file, .id=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: scopes::FunctionLevelScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=body, .nearest=body, .file=file, .id=ast::AnyIdFunc{.func=func}) :- inputs::Function(.id=func, .file=file, .name=_, .scope=_, .body=body, .exported=_)."),
                                                                                                                             rel: 34,
                                                                                                                             xform: Some(XFormCollection::FilterMap{
                                                                                                                                             description: std::borrow::Cow::from("head of scopes::FunctionLevelScope(.scope=body, .nearest=body, .file=file, .id=ast::AnyIdFunc{.func=func}) :- inputs::Function(.id=func, .file=file, .name=_, .scope=_, .body=body, .exported=_)."),
                                                                                                                                             fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                             {
                                                                                                                                                 let (ref func, ref file, ref body) = match *<types__inputs::Function>::from_ddvalue_ref(&__v) {
                                                                                                                                                     types__inputs::Function{id: ref func, file: ref file, name: _, scope: _, body: ref body, exported: _} => ((*func).clone(), (*file).clone(), (*body).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 Some(((FunctionLevelScope{scope: (*body).clone(), nearest: (*body).clone(), file: (*file).clone(), id: (types__ast::AnyId::AnyIdFunc{func: (*func).clone()})})).into_ddvalue())
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(None)
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdClass{.class=class}: ast::AnyId)}: scopes::FunctionLevelScope)] :- inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(ddlog_std::Some{.x=(elements: ddlog_std::Vec<ast::IClassElement>)}: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Class)], var body = FlatMap(((vec::filter_map: function(ddlog_std::Vec<ast::IClassElement>, function(internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>):ddlog_std::Vec<ast::StmtId>)(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{((ast::body: function(ast::ClassElement):ddlog_std::Option<ast::StmtId>)(((internment::ival: function(internment::Intern<ast::ClassElement>):ast::ClassElement)(elem))))})))), inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdClass{.class=class}) :- inputs::Class(.id=class, .file=file, .name=_, .parent=_, .elements=ddlog_std::Some{.x=elements}, .scope=_, .exported=_), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})))), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                             rel: 17,
                                                                                                                             xform: Some(XFormCollection::FlatMap{
                                                                                                                                             description: std::borrow::Cow::from("inputs::Class(.id=class, .file=file, .name=_, .parent=_, .elements=ddlog_std::Some{.x=elements}, .scope=_, .exported=_), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))}))))"),
                                                                                                                                             fmfun: {fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                                             {
                                                                                                                                                 let (ref class, ref file, ref elements) = match *<types__inputs::Class>::from_ddvalue_ref(&__v) {
                                                                                                                                                     types__inputs::Class{id: ref class, file: ref file, name: _, parent: _, elements: ddlog_std::Option::Some{x: ref elements}, scope: _, exported: _} => ((*class).clone(), (*file).clone(), (*elements).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 let __flattened = types__vec::filter_map::<internment::Intern<types__ast::ClassElement>, types__ast::StmtId>(elements, (&{
                                                                                                                                                                                                                                                                              (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                                                                                                                                                                  description: "(function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})",
                                                                                                                                                                                                                                                                                  captured: (),
                                                                                                                                                                                                                                                                                  f: {
                                                                                                                                                                                                                                                                                         fn __f(__args:*const internment::Intern<types__ast::ClassElement>, __captured: &()) -> ddlog_std::Option<types__ast::StmtId>
                                                                                                                                                                                                                                                                                         {
                                                                                                                                                                                                                                                                                             let elem = unsafe{&*__args};
                                                                                                                                                                                                                                                                                             types__ast::body_ast_ClassElement_ddlog_std_Option__ast_StmtId(internment::ival(elem))
                                                                                                                                                                                                                                                                                         }
                                                                                                                                                                                                                                                                                         __f
                                                                                                                                                                                                                                                                                     }
                                                                                                                                                                                                                                                                              }) as Box<dyn ::ddlog_rt::Closure<(*const internment::Intern<types__ast::ClassElement>), ddlog_std::Option<types__ast::StmtId>>>)
                                                                                                                                                                                                                                                                          }));
                                                                                                                                                 let class = (*class).clone();
                                                                                                                                                 let file = (*file).clone();
                                                                                                                                                 Some(Box::new(__flattened.into_iter().map(move |body|(ddlog_std::tuple3(body.clone(), class.clone(), file.clone())).into_ddvalue())))
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                     description: std::borrow::Cow::from("arrange inputs::Class(.id=class, .file=file, .name=_, .parent=_, .elements=ddlog_std::Some{.x=elements}, .scope=_, .exported=_), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})))) by (body, file)"),
                                                                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                     {
                                                                                                                                                                         let ddlog_std::tuple3(ref body, ref class, ref file) = *<ddlog_std::tuple3<types__ast::StmtId, types__ast::ClassId, types__ast::FileId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                         Some(((ddlog_std::tuple2((*body).clone(), (*file).clone())).into_ddvalue(), (ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue()))
                                                                                                                                                                     }
                                                                                                                                                                     __f},
                                                                                                                                                                     next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                        description: std::borrow::Cow::from("inputs::Class(.id=class, .file=file, .name=_, .parent=_, .elements=ddlog_std::Some{.x=elements}, .scope=_, .exported=_), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})))), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                                                                        ffun: None,
                                                                                                                                                                                        arrangement: (48,0),
                                                                                                                                                                                        jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                        {
                                                                                                                                                                                            let ddlog_std::tuple2(ref class, ref file) = *<ddlog_std::tuple2<types__ast::ClassId, types__ast::FileId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                            let ref scope = match *<types__inputs::Statement>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                types__inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                                                                _ => return None
                                                                                                                                                                                            };
                                                                                                                                                                                            Some(((FunctionLevelScope{scope: (*scope).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (types__ast::AnyId::AnyIdClass{class: (*class).clone()})})).into_ddvalue())
                                                                                                                                                                                        }
                                                                                                                                                                                        __f},
                                                                                                                                                                                        next: Box::new(None)
                                                                                                                                                                                    })
                                                                                                                                                                 }))
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_3 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: scopes::FunctionLevelScope)] :- inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .elements=(ddlog_std::Some{.x=(elements: ddlog_std::Vec<ast::IClassElement>)}: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], var body = FlatMap(((vec::filter_map: function(ddlog_std::Vec<ast::IClassElement>, function(internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>):ddlog_std::Vec<ast::StmtId>)(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{((ast::body: function(ast::ClassElement):ddlog_std::Option<ast::StmtId>)(((internment::ival: function(internment::Intern<ast::ClassElement>):ast::ClassElement)(elem))))})))), inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdExpr{.expr=expr}) :- inputs::ClassExpr(.expr_id=expr, .file=file, .elements=ddlog_std::Some{.x=elements}), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})))), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                             rel: 18,
                                                                                                                             xform: Some(XFormCollection::FlatMap{
                                                                                                                                             description: std::borrow::Cow::from("inputs::ClassExpr(.expr_id=expr, .file=file, .elements=ddlog_std::Some{.x=elements}), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))}))))"),
                                                                                                                                             fmfun: {fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                                             {
                                                                                                                                                 let (ref expr, ref file, ref elements) = match *<types__inputs::ClassExpr>::from_ddvalue_ref(&__v) {
                                                                                                                                                     types__inputs::ClassExpr{expr_id: ref expr, file: ref file, elements: ddlog_std::Option::Some{x: ref elements}} => ((*expr).clone(), (*file).clone(), (*elements).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 let __flattened = types__vec::filter_map::<internment::Intern<types__ast::ClassElement>, types__ast::StmtId>(elements, (&{
                                                                                                                                                                                                                                                                              (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                                                                                                                                                                  description: "(function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})",
                                                                                                                                                                                                                                                                                  captured: (),
                                                                                                                                                                                                                                                                                  f: {
                                                                                                                                                                                                                                                                                         fn __f(__args:*const internment::Intern<types__ast::ClassElement>, __captured: &()) -> ddlog_std::Option<types__ast::StmtId>
                                                                                                                                                                                                                                                                                         {
                                                                                                                                                                                                                                                                                             let elem = unsafe{&*__args};
                                                                                                                                                                                                                                                                                             types__ast::body_ast_ClassElement_ddlog_std_Option__ast_StmtId(internment::ival(elem))
                                                                                                                                                                                                                                                                                         }
                                                                                                                                                                                                                                                                                         __f
                                                                                                                                                                                                                                                                                     }
                                                                                                                                                                                                                                                                              }) as Box<dyn ::ddlog_rt::Closure<(*const internment::Intern<types__ast::ClassElement>), ddlog_std::Option<types__ast::StmtId>>>)
                                                                                                                                                                                                                                                                          }));
                                                                                                                                                 let expr = (*expr).clone();
                                                                                                                                                 let file = (*file).clone();
                                                                                                                                                 Some(Box::new(__flattened.into_iter().map(move |body|(ddlog_std::tuple3(body.clone(), expr.clone(), file.clone())).into_ddvalue())))
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                     description: std::borrow::Cow::from("arrange inputs::ClassExpr(.expr_id=expr, .file=file, .elements=ddlog_std::Some{.x=elements}), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})))) by (body, file)"),
                                                                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                     {
                                                                                                                                                                         let ddlog_std::tuple3(ref body, ref expr, ref file) = *<ddlog_std::tuple3<types__ast::StmtId, types__ast::ExprId, types__ast::FileId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                         Some(((ddlog_std::tuple2((*body).clone(), (*file).clone())).into_ddvalue(), (ddlog_std::tuple2((*expr).clone(), (*file).clone())).into_ddvalue()))
                                                                                                                                                                     }
                                                                                                                                                                     __f},
                                                                                                                                                                     next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                        description: std::borrow::Cow::from("inputs::ClassExpr(.expr_id=expr, .file=file, .elements=ddlog_std::Some{.x=elements}), var body = FlatMap((vec::filter_map(elements, (function(elem: internment::Intern<ast::ClassElement>):ddlog_std::Option<ast::StmtId>{(ast::body((internment::ival(elem))))})))), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                                                                        ffun: None,
                                                                                                                                                                                        arrangement: (48,0),
                                                                                                                                                                                        jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                        {
                                                                                                                                                                                            let ddlog_std::tuple2(ref expr, ref file) = *<ddlog_std::tuple2<types__ast::ExprId, types__ast::FileId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                            let ref scope = match *<types__inputs::Statement>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                types__inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                                                                _ => return None
                                                                                                                                                                                            };
                                                                                                                                                                                            Some(((FunctionLevelScope{scope: (*scope).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (types__ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                                                                                                                        }
                                                                                                                                                                                        __f},
                                                                                                                                                                                        next: Box::new(None)
                                                                                                                                                                                    })
                                                                                                                                                                 }))
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_4 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: scopes::FunctionLevelScope)] :- inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                                                                                                                         program::Rule::ArrangementRule {
                                                                                                                             description: std::borrow::Cow::from( "scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdExpr{.expr=expr}) :- inputs::InlineFunc(.expr_id=expr, .file=file, .name=_, .body=ddlog_std::Some{.x=body}), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                             arr: ( 39, 1),
                                                                                                                             xform: XFormArrangement::Join{
                                                                                                                                        description: std::borrow::Cow::from("inputs::InlineFunc(.expr_id=expr, .file=file, .name=_, .body=ddlog_std::Some{.x=body}), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                        ffun: None,
                                                                                                                                        arrangement: (48,0),
                                                                                                                                        jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let (ref expr, ref file, ref body) = match *<types__inputs::InlineFunc>::from_ddvalue_ref(__v1) {
                                                                                                                                                types__inputs::InlineFunc{expr_id: ref expr, file: ref file, name: _, body: ddlog_std::Option::Some{x: ref body}} => ((*expr).clone(), (*file).clone(), (*body).clone()),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            let ref scope = match *<types__inputs::Statement>::from_ddvalue_ref(__v2) {
                                                                                                                                                types__inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some(((FunctionLevelScope{scope: (*scope).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (types__ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(None)
                                                                                                                                    }
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_5 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: scopes::FunctionLevelScope)] :- inputs::Property[(inputs::Property{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .key=(_: ddlog_std::Option<ast::PropertyKey>), .val=(ddlog_std::Some{.x=(val: ast::PropertyVal)}: ddlog_std::Option<ast::PropertyVal>)}: inputs::Property)], ((ddlog_std::Some{.x=(var body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>) = ((ast::body: function(ast::PropertyVal):ddlog_std::Option<ast::StmtId>)(val))), inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdExpr{.expr=expr}) :- inputs::Property(.expr_id=expr, .file=file, .key=_, .val=ddlog_std::Some{.x=val}), (ddlog_std::Some{.x=var body} = (ast::body(val))), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                             rel: 46,
                                                                                                                             xform: Some(XFormCollection::Arrange {
                                                                                                                                             description: std::borrow::Cow::from("arrange inputs::Property(.expr_id=expr, .file=file, .key=_, .val=ddlog_std::Some{.x=val}) by (body, file)"),
                                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                             {
                                                                                                                                                 let (ref expr, ref file, ref val) = match *<types__inputs::Property>::from_ddvalue_ref(&__v) {
                                                                                                                                                     types__inputs::Property{expr_id: ref expr, file: ref file, key: _, val: ddlog_std::Option::Some{x: ref val}} => ((*expr).clone(), (*file).clone(), (*val).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 let ref body: types__ast::StmtId = match types__ast::body_ast_PropertyVal_ddlog_std_Option__ast_StmtId(val) {
                                                                                                                                                     ddlog_std::Option::Some{x: body} => body,
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 Some(((ddlog_std::tuple2((*body).clone(), (*file).clone())).into_ddvalue(), (ddlog_std::tuple2((*expr).clone(), (*file).clone())).into_ddvalue()))
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(XFormArrangement::Join{
                                                                                                                                                                description: std::borrow::Cow::from("inputs::Property(.expr_id=expr, .file=file, .key=_, .val=ddlog_std::Some{.x=val}), (ddlog_std::Some{.x=var body} = (ast::body(val))), inputs::Statement(.id=body, .file=file, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                                                ffun: None,
                                                                                                                                                                arrangement: (48,0),
                                                                                                                                                                jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                {
                                                                                                                                                                    let ddlog_std::tuple2(ref expr, ref file) = *<ddlog_std::tuple2<types__ast::ExprId, types__ast::FileId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                    let ref scope = match *<types__inputs::Statement>::from_ddvalue_ref(__v2) {
                                                                                                                                                                        types__inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                                        _ => return None
                                                                                                                                                                    };
                                                                                                                                                                    Some(((FunctionLevelScope{scope: (*scope).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (types__ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                                                                                                }
                                                                                                                                                                __f},
                                                                                                                                                                next: Box::new(None)
                                                                                                                                                            })
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_6 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: scopes::FunctionLevelScope)] :- inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=((_: ddlog_std::Either<ast::ExprId,ast::StmtId>), (scope: ast::ScopeId))}: ddlog_std::Option<(ddlog_std::Either<ast::ExprId,ast::StmtId>, ast::ScopeId)>)}: inputs::Arrow)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdExpr{.expr=expr}) :- inputs::Arrow(.expr_id=expr, .file=file, .body=ddlog_std::Some{.x=(_, scope)})."),
                                                                                                                             rel: 9,
                                                                                                                             xform: Some(XFormCollection::FilterMap{
                                                                                                                                             description: std::borrow::Cow::from("head of scopes::FunctionLevelScope(.scope=scope, .nearest=scope, .file=file, .id=ast::AnyIdExpr{.expr=expr}) :- inputs::Arrow(.expr_id=expr, .file=file, .body=ddlog_std::Some{.x=(_, scope)})."),
                                                                                                                                             fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                             {
                                                                                                                                                 let (ref expr, ref file, ref scope) = match *<types__inputs::Arrow>::from_ddvalue_ref(&__v) {
                                                                                                                                                     types__inputs::Arrow{expr_id: ref expr, file: ref file, body: ddlog_std::Option::Some{x: ddlog_std::tuple2(_, ref scope)}} => ((*expr).clone(), (*file).clone(), (*scope).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 Some(((FunctionLevelScope{scope: (*scope).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (types__ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(None)
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_scopes_FunctionLevelScope_7 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=child, .nearest=scope, .file=file, .id=id}: scopes::FunctionLevelScope)] :- scopes::FunctionLevelScope[(scopes::FunctionLevelScope{.scope=(parent: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(id: ast::AnyId)}: scopes::FunctionLevelScope)], inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], var __group = (scope, id).group_by((child, file)), (((var scope: ast::ScopeId), (var id: ast::AnyId)) = ((group::arg_max: function(ddlog_std::Group<(ast::ScopeId, ast::FileId),(ast::ScopeId, ast::AnyId)>, function((ast::ScopeId, ast::AnyId)):ast::ScopeId):(ast::ScopeId, ast::AnyId))(__group, (function(scope: (ast::ScopeId, ast::AnyId)):ast::ScopeId{(scope.0)})))). */
                                                                                                                         program::Rule::ArrangementRule {
                                                                                                                             description: std::borrow::Cow::from( "scopes::FunctionLevelScope(.scope=child, .nearest=scope, .file=file, .id=id) :- scopes::FunctionLevelScope(.scope=parent, .nearest=scope, .file=file, .id=id), inputs::InputScope(.parent=parent, .child=child, .file=file), var __group = (scope, id).group_by((child, file)), ((var scope, var id) = (group::arg_max(__group, (function(scope: (ast::ScopeId, ast::AnyId)):ast::ScopeId{(scope.0)}))))."),
                                                                                                                             arr: ( 80, 0),
                                                                                                                             xform: XFormArrangement::Join{
                                                                                                                                        description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=parent, .nearest=scope, .file=file, .id=id), inputs::InputScope(.parent=parent, .child=child, .file=file)"),
                                                                                                                                        ffun: None,
                                                                                                                                        arrangement: (41,1),
                                                                                                                                        jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let (ref parent, ref scope, ref file, ref id) = match *<FunctionLevelScope>::from_ddvalue_ref(__v1) {
                                                                                                                                                FunctionLevelScope{scope: ref parent, nearest: ref scope, file: ref file, id: ref id} => ((*parent).clone(), (*scope).clone(), (*file).clone(), (*id).clone()),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            let ref child = match *<types__inputs::InputScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                types__inputs::InputScope{parent: _, child: ref child, file: _} => (*child).clone(),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some((ddlog_std::tuple5((*parent).clone(), (*scope).clone(), (*file).clone(), (*id).clone(), (*child).clone())).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                description: std::borrow::Cow::from("arrange scopes::FunctionLevelScope(.scope=parent, .nearest=scope, .file=file, .id=id), inputs::InputScope(.parent=parent, .child=child, .file=file) by (child, file)"),
                                                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                {
                                                                                                                                                                    let ddlog_std::tuple5(ref parent, ref scope, ref file, ref id, ref child) = *<ddlog_std::tuple5<types__ast::ScopeId, types__ast::ScopeId, types__ast::FileId, types__ast::AnyId, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                    Some(((ddlog_std::tuple2((*child).clone(), (*file).clone())).into_ddvalue(), (ddlog_std::tuple5((*parent).clone(), (*scope).clone(), (*file).clone(), (*id).clone(), (*child).clone())).into_ddvalue()))
                                                                                                                                                                }
                                                                                                                                                                __f},
                                                                                                                                                                next: Box::new(XFormArrangement::Aggregate{
                                                                                                                                                                                   description: std::borrow::Cow::from("scopes::FunctionLevelScope(.scope=parent, .nearest=scope, .file=file, .id=id), inputs::InputScope(.parent=parent, .child=child, .file=file), var __group = (scope, id).group_by((child, file))"),
                                                                                                                                                                                   ffun: None,
                                                                                                                                                                                   aggfun: {fn __f(__key: &DDValue, __group__: &[(&DDValue, Weight)]) -> Option<DDValue>
                                                                                                                                                                               {
                                                                                                                                                                                   let ddlog_std::tuple2(ref child, ref file) = *<ddlog_std::tuple2<types__ast::ScopeId, types__ast::FileId>>::from_ddvalue_ref( __key );
                                                                                                                                                                                   let ref __group = unsafe{ddlog_std::Group::new_by_ref(ddlog_std::tuple2((*child).clone(), (*file).clone()), __group__, {fn __f(__v: &DDValue) ->  ddlog_std::tuple2<types__ast::ScopeId, types__ast::AnyId>
                                                                                                                                                                                                                                                                                                          {
                                                                                                                                                                                                                                                                                                              let ddlog_std::tuple5(ref parent, ref scope, ref file, ref id, ref child) = *<ddlog_std::tuple5<types__ast::ScopeId, types__ast::ScopeId, types__ast::FileId, types__ast::AnyId, types__ast::ScopeId>>::from_ddvalue_ref( __v );
                                                                                                                                                                                                                                                                                                              ddlog_std::tuple2((*scope).clone(), (*id).clone())
                                                                                                                                                                                                                                                                                                          }
                                                                                                                                                                                                                                                                                                          ::std::sync::Arc::new(__f)})};
                                                                                                                                                                                   let (ref scope, ref id): (types__ast::ScopeId, types__ast::AnyId) = match types__group::arg_max::<ddlog_std::tuple2<types__ast::ScopeId, types__ast::FileId>, ddlog_std::tuple2<types__ast::ScopeId, types__ast::AnyId>, types__ast::ScopeId>(__group, (&{
                                                                                                                                                                                                                                                                                                                                                                                                                                                (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                                                                                                                                                                                                                                                                                                                                    description: "(function(scope: (ast::ScopeId, ast::AnyId)):ast::ScopeId{(scope.0)})",
                                                                                                                                                                                                                                                                                                                                                                                                                                                    captured: (),
                                                                                                                                                                                                                                                                                                                                                                                                                                                    f: {
                                                                                                                                                                                                                                                                                                                                                                                                                                                           fn __f(__args:*const ddlog_std::tuple2<types__ast::ScopeId, types__ast::AnyId>, __captured: &()) -> types__ast::ScopeId
                                                                                                                                                                                                                                                                                                                                                                                                                                                           {
                                                                                                                                                                                                                                                                                                                                                                                                                                                               let scope = unsafe{&*__args};
                                                                                                                                                                                                                                                                                                                                                                                                                                                               (scope.0).clone()
                                                                                                                                                                                                                                                                                                                                                                                                                                                           }
                                                                                                                                                                                                                                                                                                                                                                                                                                                           __f
                                                                                                                                                                                                                                                                                                                                                                                                                                                       }
                                                                                                                                                                                                                                                                                                                                                                                                                                                }) as Box<dyn ::ddlog_rt::Closure<(*const ddlog_std::tuple2<types__ast::ScopeId, types__ast::AnyId>), types__ast::ScopeId>>)
                                                                                                                                                                                                                                                                                                                                                                                                                                            })) {
                                                                                                                                                                                       ddlog_std::tuple2(scope, id) => (scope, id),
                                                                                                                                                                                       _ => return None
                                                                                                                                                                                   };
                                                                                                                                                                                   Some((ddlog_std::tuple4((*child).clone(), (*file).clone(), (*scope).clone(), (*id).clone())).into_ddvalue())
                                                                                                                                                                               }
                                                                                                                                                                               __f},
                                                                                                                                                                                   next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                                           description: std::borrow::Cow::from("head of scopes::FunctionLevelScope(.scope=child, .nearest=scope, .file=file, .id=id) :- scopes::FunctionLevelScope(.scope=parent, .nearest=scope, .file=file, .id=id), inputs::InputScope(.parent=parent, .child=child, .file=file), var __group = (scope, id).group_by((child, file)), ((var scope, var id) = (group::arg_max(__group, (function(scope: (ast::ScopeId, ast::AnyId)):ast::ScopeId{(scope.0)}))))."),
                                                                                                                                                                                                           fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                                           {
                                                                                                                                                                                                               let ddlog_std::tuple4(ref child, ref file, ref scope, ref id) = *<ddlog_std::tuple4<types__ast::ScopeId, types__ast::FileId, types__ast::ScopeId, types__ast::AnyId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                               Some(((FunctionLevelScope{scope: (*child).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (*id).clone()})).into_ddvalue())
                                                                                                                                                                                                           }
                                                                                                                                                                                                           __f},
                                                                                                                                                                                                           next: Box::new(None)
                                                                                                                                                                                                       }))
                                                                                                                                                                               })
                                                                                                                                                            }))
                                                                                                                                    }
                                                                                                                         });
pub static __Rule_scopes_ScopeOfId_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeOfId[(scopes::ScopeOfId{.id=(ast::AnyIdFile{.file=id}: ast::AnyId), .file=id, .scope=scope}: scopes::ScopeOfId)] :- inputs::File[(inputs::File{.id=(id: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(scope: ast::ScopeId)}: inputs::File)]. */
                                                                                                                program::Rule::CollectionRule {
                                                                                                                    description: std::borrow::Cow::from("scopes::ScopeOfId(.id=ast::AnyIdFile{.file=id}, .file=id, .scope=scope) :- inputs::File(.id=id, .kind=_, .top_level_scope=scope)."),
                                                                                                                    rel: 29,
                                                                                                                    xform: Some(XFormCollection::FilterMap{
                                                                                                                                    description: std::borrow::Cow::from("head of scopes::ScopeOfId(.id=ast::AnyIdFile{.file=id}, .file=id, .scope=scope) :- inputs::File(.id=id, .kind=_, .top_level_scope=scope)."),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        let (ref id, ref scope) = match *<types__inputs::File>::from_ddvalue_ref(&__v) {
                                                                                                                                            types__inputs::File{id: ref id, kind: _, top_level_scope: ref scope} => ((*id).clone(), (*scope).clone()),
                                                                                                                                            _ => return None
                                                                                                                                        };
                                                                                                                                        Some(((ScopeOfId{id: (types__ast::AnyId::AnyIdFile{file: (*id).clone()}), file: (*id).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    next: Box::new(None)
                                                                                                                                })
                                                                                                                });
pub static __Rule_scopes_ScopeOfId_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeOfId[(scopes::ScopeOfId{.id=(ast::AnyIdFunc{.func=id}: ast::AnyId), .file=file, .scope=scope}: scopes::ScopeOfId)] :- inputs::Function[(inputs::Function{.id=(id: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)]. */
                                                                                                                program::Rule::CollectionRule {
                                                                                                                    description: std::borrow::Cow::from("scopes::ScopeOfId(.id=ast::AnyIdFunc{.func=id}, .file=file, .scope=scope) :- inputs::Function(.id=id, .file=file, .name=_, .scope=scope, .body=_, .exported=_)."),
                                                                                                                    rel: 34,
                                                                                                                    xform: Some(XFormCollection::FilterMap{
                                                                                                                                    description: std::borrow::Cow::from("head of scopes::ScopeOfId(.id=ast::AnyIdFunc{.func=id}, .file=file, .scope=scope) :- inputs::Function(.id=id, .file=file, .name=_, .scope=scope, .body=_, .exported=_)."),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        let (ref id, ref file, ref scope) = match *<types__inputs::Function>::from_ddvalue_ref(&__v) {
                                                                                                                                            types__inputs::Function{id: ref id, file: ref file, name: _, scope: ref scope, body: _, exported: _} => ((*id).clone(), (*file).clone(), (*scope).clone()),
                                                                                                                                            _ => return None
                                                                                                                                        };
                                                                                                                                        Some(((ScopeOfId{id: (types__ast::AnyId::AnyIdFunc{func: (*id).clone()}), file: (*file).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    next: Box::new(None)
                                                                                                                                })
                                                                                                                });
pub static __Rule_scopes_ScopeOfId_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeOfId[(scopes::ScopeOfId{.id=(ast::AnyIdClass{.class=id}: ast::AnyId), .file=file, .scope=scope}: scopes::ScopeOfId)] :- inputs::Class[(inputs::Class{.id=(id: ast::ClassId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)]. */
                                                                                                                program::Rule::CollectionRule {
                                                                                                                    description: std::borrow::Cow::from("scopes::ScopeOfId(.id=ast::AnyIdClass{.class=id}, .file=file, .scope=scope) :- inputs::Class(.id=id, .file=file, .name=_, .parent=_, .elements=_, .scope=scope, .exported=_)."),
                                                                                                                    rel: 17,
                                                                                                                    xform: Some(XFormCollection::FilterMap{
                                                                                                                                    description: std::borrow::Cow::from("head of scopes::ScopeOfId(.id=ast::AnyIdClass{.class=id}, .file=file, .scope=scope) :- inputs::Class(.id=id, .file=file, .name=_, .parent=_, .elements=_, .scope=scope, .exported=_)."),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        let (ref id, ref file, ref scope) = match *<types__inputs::Class>::from_ddvalue_ref(&__v) {
                                                                                                                                            types__inputs::Class{id: ref id, file: ref file, name: _, parent: _, elements: _, scope: ref scope, exported: _} => ((*id).clone(), (*file).clone(), (*scope).clone()),
                                                                                                                                            _ => return None
                                                                                                                                        };
                                                                                                                                        Some(((ScopeOfId{id: (types__ast::AnyId::AnyIdClass{class: (*id).clone()}), file: (*file).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    next: Box::new(None)
                                                                                                                                })
                                                                                                                });
pub static __Rule_scopes_ScopeOfId_3 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeOfId[(scopes::ScopeOfId{.id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId), .file=file, .scope=scope}: scopes::ScopeOfId)] :- inputs::Statement[(inputs::Statement{.id=(id: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                                                                                                                program::Rule::CollectionRule {
                                                                                                                    description: std::borrow::Cow::from("scopes::ScopeOfId(.id=ast::AnyIdStmt{.stmt=id}, .file=file, .scope=scope) :- inputs::Statement(.id=id, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                    rel: 48,
                                                                                                                    xform: Some(XFormCollection::FilterMap{
                                                                                                                                    description: std::borrow::Cow::from("head of scopes::ScopeOfId(.id=ast::AnyIdStmt{.stmt=id}, .file=file, .scope=scope) :- inputs::Statement(.id=id, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        let (ref id, ref file, ref scope) = match *<types__inputs::Statement>::from_ddvalue_ref(&__v) {
                                                                                                                                            types__inputs::Statement{id: ref id, file: ref file, kind: _, scope: ref scope, span: _} => ((*id).clone(), (*file).clone(), (*scope).clone()),
                                                                                                                                            _ => return None
                                                                                                                                        };
                                                                                                                                        Some(((ScopeOfId{id: (types__ast::AnyId::AnyIdStmt{stmt: (*id).clone()}), file: (*file).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    next: Box::new(None)
                                                                                                                                })
                                                                                                                });
pub static __Rule_scopes_ScopeOfId_4 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeOfId[(scopes::ScopeOfId{.id=(ast::AnyIdExpr{.expr=id}: ast::AnyId), .file=file, .scope=scope}: scopes::ScopeOfId)] :- inputs::Expression[(inputs::Expression{.id=(id: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]. */
                                                                                                                program::Rule::CollectionRule {
                                                                                                                    description: std::borrow::Cow::from("scopes::ScopeOfId(.id=ast::AnyIdExpr{.expr=id}, .file=file, .scope=scope) :- inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                    rel: 28,
                                                                                                                    xform: Some(XFormCollection::FilterMap{
                                                                                                                                    description: std::borrow::Cow::from("head of scopes::ScopeOfId(.id=ast::AnyIdExpr{.expr=id}, .file=file, .scope=scope) :- inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_)."),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        let (ref id, ref file, ref scope) = match *<types__inputs::Expression>::from_ddvalue_ref(&__v) {
                                                                                                                                            types__inputs::Expression{id: ref id, file: ref file, kind: _, scope: ref scope, span: _} => ((*id).clone(), (*file).clone(), (*scope).clone()),
                                                                                                                                            _ => return None
                                                                                                                                        };
                                                                                                                                        Some(((ScopeOfId{id: (types__ast::AnyId::AnyIdExpr{expr: (*id).clone()}), file: (*file).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    next: Box::new(None)
                                                                                                                                })
                                                                                                                });
pub static __Rule_scopes_ScopeOfId_5 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeOfId[(scopes::ScopeOfId{.id=(ast::AnyIdImport{.import_=id}: ast::AnyId), .file=file, .scope=scope}: scopes::ScopeOfId)] :- inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .file=(file: ast::FileId), .clause=(_: ast::ImportClause)}: inputs::ImportDecl)], inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(scope: ast::ScopeId)}: inputs::File)]. */
                                                                                                                program::Rule::ArrangementRule {
                                                                                                                    description: std::borrow::Cow::from( "scopes::ScopeOfId(.id=ast::AnyIdImport{.import_=id}, .file=file, .scope=scope) :- inputs::ImportDecl(.id=id, .file=file, .clause=_), inputs::File(.id=file, .kind=_, .top_level_scope=scope)."),
                                                                                                                    arr: ( 38, 0),
                                                                                                                    xform: XFormArrangement::Join{
                                                                                                                               description: std::borrow::Cow::from("inputs::ImportDecl(.id=id, .file=file, .clause=_), inputs::File(.id=file, .kind=_, .top_level_scope=scope)"),
                                                                                                                               ffun: None,
                                                                                                                               arrangement: (29,0),
                                                                                                                               jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                               {
                                                                                                                                   let (ref id, ref file) = match *<types__inputs::ImportDecl>::from_ddvalue_ref(__v1) {
                                                                                                                                       types__inputs::ImportDecl{id: ref id, file: ref file, clause: _} => ((*id).clone(), (*file).clone()),
                                                                                                                                       _ => return None
                                                                                                                                   };
                                                                                                                                   let ref scope = match *<types__inputs::File>::from_ddvalue_ref(__v2) {
                                                                                                                                       types__inputs::File{id: _, kind: _, top_level_scope: ref scope} => (*scope).clone(),
                                                                                                                                       _ => return None
                                                                                                                                   };
                                                                                                                                   Some(((ScopeOfId{id: (types__ast::AnyId::AnyIdImport{import_: (*id).clone()}), file: (*file).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                               }
                                                                                                                               __f},
                                                                                                                               next: Box::new(None)
                                                                                                                           }
                                                                                                                });
pub static __Rule_scopes_IsHoistable_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::IsHoistable[(scopes::IsHoistable{.file=file, .id=(ast::AnyIdFunc{.func=id}: ast::AnyId), .hoistable=true}: scopes::IsHoistable)] :- inputs::Function[(inputs::Function{.id=(id: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)]. */
                                                                                                                  program::Rule::CollectionRule {
                                                                                                                      description: std::borrow::Cow::from("scopes::IsHoistable(.file=file, .id=ast::AnyIdFunc{.func=id}, .hoistable=true) :- inputs::Function(.id=id, .file=file, .name=_, .scope=_, .body=_, .exported=_)."),
                                                                                                                      rel: 34,
                                                                                                                      xform: Some(XFormCollection::FilterMap{
                                                                                                                                      description: std::borrow::Cow::from("head of scopes::IsHoistable(.file=file, .id=ast::AnyIdFunc{.func=id}, .hoistable=true) :- inputs::Function(.id=id, .file=file, .name=_, .scope=_, .body=_, .exported=_)."),
                                                                                                                                      fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                      {
                                                                                                                                          let (ref id, ref file) = match *<types__inputs::Function>::from_ddvalue_ref(&__v) {
                                                                                                                                              types__inputs::Function{id: ref id, file: ref file, name: _, scope: _, body: _, exported: _} => ((*id).clone(), (*file).clone()),
                                                                                                                                              _ => return None
                                                                                                                                          };
                                                                                                                                          Some(((IsHoistable{file: (*file).clone(), id: (types__ast::AnyId::AnyIdFunc{func: (*id).clone()}), hoistable: true})).into_ddvalue())
                                                                                                                                      }
                                                                                                                                      __f},
                                                                                                                                      next: Box::new(None)
                                                                                                                                  })
                                                                                                                  });
pub static __Rule_scopes_IsHoistable_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::IsHoistable[(scopes::IsHoistable{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId), .hoistable=true}: scopes::IsHoistable)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)]. */
                                                                                                                  program::Rule::CollectionRule {
                                                                                                                      description: std::borrow::Cow::from("scopes::IsHoistable(.file=file, .id=ast::AnyIdStmt{.stmt=id}, .hoistable=true) :- inputs::VarDecl(.stmt_id=id, .file=file, .pattern=_, .value=_, .exported=_)."),
                                                                                                                      rel: 57,
                                                                                                                      xform: Some(XFormCollection::FilterMap{
                                                                                                                                      description: std::borrow::Cow::from("head of scopes::IsHoistable(.file=file, .id=ast::AnyIdStmt{.stmt=id}, .hoistable=true) :- inputs::VarDecl(.stmt_id=id, .file=file, .pattern=_, .value=_, .exported=_)."),
                                                                                                                                      fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                      {
                                                                                                                                          let (ref id, ref file) = match *<types__inputs::VarDecl>::from_ddvalue_ref(&__v) {
                                                                                                                                              types__inputs::VarDecl{stmt_id: ref id, file: ref file, pattern: _, value: _, exported: _} => ((*id).clone(), (*file).clone()),
                                                                                                                                              _ => return None
                                                                                                                                          };
                                                                                                                                          Some(((IsHoistable{file: (*file).clone(), id: (types__ast::AnyId::AnyIdStmt{stmt: (*id).clone()}), hoistable: true})).into_ddvalue())
                                                                                                                                      }
                                                                                                                                      __f},
                                                                                                                                      next: Box::new(None)
                                                                                                                                  })
                                                                                                                  });
pub static __Rule_scopes_ScopeFamily_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeFamily[(scopes::ScopeFamily{.parent=parent, .child=child, .file=file}: scopes::ScopeFamily)] :- scopes::NeedsScopeChildren[(scopes::NeedsScopeChildren{.scope=(parent: ast::ScopeId), .file=(file: ast::FileId)}: scopes::NeedsScopeChildren)], inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], (parent != child). */
                                                                                                                  program::Rule::ArrangementRule {
                                                                                                                      description: std::borrow::Cow::from( "scopes::ScopeFamily(.parent=parent, .child=child, .file=file) :- scopes::NeedsScopeChildren(.scope=parent, .file=file), inputs::InputScope(.parent=parent, .child=child, .file=file), (parent != child)."),
                                                                                                                      arr: ( 82, 0),
                                                                                                                      xform: XFormArrangement::Join{
                                                                                                                                 description: std::borrow::Cow::from("scopes::NeedsScopeChildren(.scope=parent, .file=file), inputs::InputScope(.parent=parent, .child=child, .file=file)"),
                                                                                                                                 ffun: None,
                                                                                                                                 arrangement: (41,1),
                                                                                                                                 jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                 {
                                                                                                                                     let (ref parent, ref file) = match *<NeedsScopeChildren>::from_ddvalue_ref(__v1) {
                                                                                                                                         NeedsScopeChildren{scope: ref parent, file: ref file} => ((*parent).clone(), (*file).clone()),
                                                                                                                                         _ => return None
                                                                                                                                     };
                                                                                                                                     let ref child = match *<types__inputs::InputScope>::from_ddvalue_ref(__v2) {
                                                                                                                                         types__inputs::InputScope{parent: _, child: ref child, file: _} => (*child).clone(),
                                                                                                                                         _ => return None
                                                                                                                                     };
                                                                                                                                     if !((&*parent) != (&*child)) {return None;};
                                                                                                                                     Some(((ScopeFamily{parent: (*parent).clone(), child: (*child).clone(), file: (*file).clone()})).into_ddvalue())
                                                                                                                                 }
                                                                                                                                 __f},
                                                                                                                                 next: Box::new(None)
                                                                                                                             }
                                                                                                                  });
pub static __Rule_scopes_ScopeFamily_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeFamily[(scopes::ScopeFamily{.parent=parent, .child=child, .file=file}: scopes::ScopeFamily)] :- scopes::NeedsScopeParents[(scopes::NeedsScopeParents{.scope=(child: ast::ScopeId), .file=(file: ast::FileId)}: scopes::NeedsScopeParents)], inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], (parent != child). */
                                                                                                                  program::Rule::ArrangementRule {
                                                                                                                      description: std::borrow::Cow::from( "scopes::ScopeFamily(.parent=parent, .child=child, .file=file) :- scopes::NeedsScopeParents(.scope=child, .file=file), inputs::InputScope(.parent=parent, .child=child, .file=file), (parent != child)."),
                                                                                                                      arr: ( 83, 0),
                                                                                                                      xform: XFormArrangement::Join{
                                                                                                                                 description: std::borrow::Cow::from("scopes::NeedsScopeParents(.scope=child, .file=file), inputs::InputScope(.parent=parent, .child=child, .file=file)"),
                                                                                                                                 ffun: None,
                                                                                                                                 arrangement: (41,0),
                                                                                                                                 jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                 {
                                                                                                                                     let (ref child, ref file) = match *<NeedsScopeParents>::from_ddvalue_ref(__v1) {
                                                                                                                                         NeedsScopeParents{scope: ref child, file: ref file} => ((*child).clone(), (*file).clone()),
                                                                                                                                         _ => return None
                                                                                                                                     };
                                                                                                                                     let ref parent = match *<types__inputs::InputScope>::from_ddvalue_ref(__v2) {
                                                                                                                                         types__inputs::InputScope{parent: ref parent, child: _, file: _} => (*parent).clone(),
                                                                                                                                         _ => return None
                                                                                                                                     };
                                                                                                                                     if !((&*parent) != (&*child)) {return None;};
                                                                                                                                     Some(((ScopeFamily{parent: (*parent).clone(), child: (*child).clone(), file: (*file).clone()})).into_ddvalue())
                                                                                                                                 }
                                                                                                                                 __f},
                                                                                                                                 next: Box::new(None)
                                                                                                                             }
                                                                                                                  });
pub static __Rule_scopes_ScopeFamily_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::ScopeFamily[(scopes::ScopeFamily{.parent=parent, .child=child, .file=file}: scopes::ScopeFamily)] :- inputs::InputScope[(inputs::InputScope{.parent=(interum: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], scopes::ScopeFamily[(scopes::ScopeFamily{.parent=(parent: ast::ScopeId), .child=(interum: ast::ScopeId), .file=(file: ast::FileId)}: scopes::ScopeFamily)], (parent != child). */
                                                                                                                  program::Rule::ArrangementRule {
                                                                                                                      description: std::borrow::Cow::from( "scopes::ScopeFamily(.parent=parent, .child=child, .file=file) :- inputs::InputScope(.parent=interum, .child=child, .file=file), scopes::ScopeFamily(.parent=parent, .child=interum, .file=file), (parent != child)."),
                                                                                                                      arr: ( 41, 1),
                                                                                                                      xform: XFormArrangement::Join{
                                                                                                                                 description: std::borrow::Cow::from("inputs::InputScope(.parent=interum, .child=child, .file=file), scopes::ScopeFamily(.parent=parent, .child=interum, .file=file)"),
                                                                                                                                 ffun: None,
                                                                                                                                 arrangement: (84,1),
                                                                                                                                 jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                 {
                                                                                                                                     let (ref interum, ref child, ref file) = match *<types__inputs::InputScope>::from_ddvalue_ref(__v1) {
                                                                                                                                         types__inputs::InputScope{parent: ref interum, child: ref child, file: ref file} => ((*interum).clone(), (*child).clone(), (*file).clone()),
                                                                                                                                         _ => return None
                                                                                                                                     };
                                                                                                                                     let ref parent = match *<ScopeFamily>::from_ddvalue_ref(__v2) {
                                                                                                                                         ScopeFamily{parent: ref parent, child: _, file: _} => (*parent).clone(),
                                                                                                                                         _ => return None
                                                                                                                                     };
                                                                                                                                     if !((&*parent) != (&*child)) {return None;};
                                                                                                                                     Some(((ScopeFamily{parent: (*parent).clone(), child: (*child).clone(), file: (*file).clone()})).into_ddvalue())
                                                                                                                                 }
                                                                                                                                 __f},
                                                                                                                                 next: Box::new(None)
                                                                                                                             }
                                                                                                                  });