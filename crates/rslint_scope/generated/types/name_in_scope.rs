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
#[ddlog(rename = "name_in_scope::NameInScope")]
pub struct NameInScope {
    pub name: types__ast::Name,
    pub scope: types__ast::ScopeId,
    pub declared: types__ast::AnyId
}
impl abomonation::Abomonation for NameInScope{}
impl ::std::fmt::Display for NameInScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameInScope{name,scope,declared} => {
                __formatter.write_str("name_in_scope::NameInScope{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameInScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "name_in_scope::NameOccursInScope")]
pub struct NameOccursInScope {
    pub scope: types__ast::ScopeId,
    pub name: types__ast::Name
}
impl abomonation::Abomonation for NameOccursInScope{}
impl ::std::fmt::Display for NameOccursInScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameOccursInScope{scope,name} => {
                __formatter.write_str("name_in_scope::NameOccursInScope{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameOccursInScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "name_in_scope::NameOrigin")]
pub enum NameOrigin {
    #[ddlog(rename = "name_in_scope::AutoGlobal")]
    AutoGlobal,
    #[ddlog(rename = "name_in_scope::Imported")]
    Imported,
    #[ddlog(rename = "name_in_scope::UserDefined")]
    UserDefined {
        scope: types__ast::ScopeId
    }
}
impl abomonation::Abomonation for NameOrigin{}
impl ::std::fmt::Display for NameOrigin {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameOrigin::AutoGlobal{} => {
                __formatter.write_str("name_in_scope::AutoGlobal{")?;
                __formatter.write_str("}")
            },
            NameOrigin::Imported{} => {
                __formatter.write_str("name_in_scope::Imported{")?;
                __formatter.write_str("}")
            },
            NameOrigin::UserDefined{scope} => {
                __formatter.write_str("name_in_scope::UserDefined{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameOrigin {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for NameOrigin {
    fn default() -> Self {
        NameOrigin::AutoGlobal{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "name_in_scope::ScopeOfDeclName")]
pub struct ScopeOfDeclName {
    pub name: types__ast::Name,
    pub scope: types__ast::ScopeId,
    pub declared: types__ast::AnyId
}
impl abomonation::Abomonation for ScopeOfDeclName{}
impl ::std::fmt::Display for ScopeOfDeclName {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ScopeOfDeclName{name,scope,declared} => {
                __formatter.write_str("name_in_scope::ScopeOfDeclName{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ScopeOfDeclName {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub static __Arng_name_in_scope_NameOccursInScope_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                          name: std::borrow::Cow::from(r###"(name_in_scope::NameOccursInScope{.scope=(_0: ast::ScopeId), .name=(_1: internment::Intern<string>)}: name_in_scope::NameOccursInScope) /*semijoin*/"###),
                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                          {
                                                                                                                                              match < NameOccursInScope>::from_ddvalue(__v) {
                                                                                                                                                  NameOccursInScope{scope: ref _0, name: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                  _ => None
                                                                                                                                              }
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          distinct: false
                                                                                                                                      });
pub static __Arng_name_in_scope_NameOccursInScope_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                         name: std::borrow::Cow::from(r###"(name_in_scope::NameOccursInScope{.scope=(_1: ast::ScopeId), .name=(_0: internment::Intern<string>)}: name_in_scope::NameOccursInScope) /*join*/"###),
                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                          {
                                                                                                                                              let __cloned = __v.clone();
                                                                                                                                              match < NameOccursInScope>::from_ddvalue(__v) {
                                                                                                                                                  NameOccursInScope{scope: ref _1, name: ref _0} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                  _ => None
                                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          queryable: false
                                                                                                                                      });
pub static __Arng_name_in_scope_NameOccursInScope_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                         name: std::borrow::Cow::from(r###"(name_in_scope::NameOccursInScope{.scope=(_0: ast::ScopeId), .name=(_: internment::Intern<string>)}: name_in_scope::NameOccursInScope) /*join*/"###),
                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                          {
                                                                                                                                              let __cloned = __v.clone();
                                                                                                                                              match < NameOccursInScope>::from_ddvalue(__v) {
                                                                                                                                                  NameOccursInScope{scope: ref _0, name: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                                  _ => None
                                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          queryable: false
                                                                                                                                      });
pub static __Arng_name_in_scope_ScopeOfDeclName_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                        name: std::borrow::Cow::from(r###"(name_in_scope::ScopeOfDeclName{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::ScopeOfDeclName) /*antijoin*/"###),
                                                                                                                                        fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            match < ScopeOfDeclName>::from_ddvalue(__v) {
                                                                                                                                                ScopeOfDeclName{name: ref _0, scope: ref _1, declared: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                _ => None
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        distinct: true
                                                                                                                                    });
pub static __Arng_name_in_scope_NameInScope_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _0, scope: ref _1, declared: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                    name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _0, scope: ref _1, declared: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    distinct: true
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(ast::AnyIdStmt{.stmt=(_: ast::StmtId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _0, scope: ref _1, declared: types__ast::AnyId::AnyIdStmt{stmt: _}} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(ast::AnyIdClass{.class=(_: ast::ClassId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _0, scope: ref _1, declared: types__ast::AnyId::AnyIdClass{class: _}} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_4 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(ast::AnyIdFunc{.func=(_: ast::FuncId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _0, scope: ref _1, declared: types__ast::AnyId::AnyIdFunc{func: _}} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_5 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                    name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=_1, .declared=(_2: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _0, scope: ref _1, declared: ref _2} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    distinct: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_6 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                    name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_2: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _0, scope: ref _1, declared: ref _2} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    distinct: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_7 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=_1, .scope=_0, .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: ref _1, scope: ref _0, declared: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: true
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_8 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.name=(_: internment::Intern<string>), .scope=_0, .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{name: _, scope: ref _0, declared: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: true
                                                                                                                                });
pub static __Rule_name_in_scope_NameOccursInScope_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=scope, .name=name}: name_in_scope::NameOccursInScope)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(id: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(id: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]. */
                                                                                                                               program::Rule::ArrangementRule {
                                                                                                                                   description: std::borrow::Cow::from( "name_in_scope::NameOccursInScope(.scope=scope, .name=name) :- inputs::NameRef(.expr_id=id, .value=name), inputs::Expression(.id=id, .kind=_, .scope=scope, .span=_)."),
                                                                                                                                   arr: ( 43, 0),
                                                                                                                                   xform: XFormArrangement::Join{
                                                                                                                                              description: std::borrow::Cow::from("inputs::NameRef(.expr_id=id, .value=name), inputs::Expression(.id=id, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                              ffun: None,
                                                                                                                                              arrangement: (27,0),
                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref id, ref name) = match *<types__inputs::NameRef>::from_ddvalue_ref(__v1) {
                                                                                                                                                      types__inputs::NameRef{expr_id: ref id, value: ref name} => ((*id).clone(), (*name).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  let ref scope = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                      types__inputs::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some(((NameOccursInScope{scope: (*scope).clone(), name: (*name).clone()})).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(None)
                                                                                                                                          }
                                                                                                                               });
pub static __Rule_name_in_scope_NameOccursInScope_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=scope, .name=name}: name_in_scope::NameOccursInScope)] :- inputs::Assign[(inputs::Assign{.expr_id=(id: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pattern: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(id: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((vec::map: function(ddlog_std::Vec<ast::Spanned<ast::Name>>, function(ast::Spanned<ast::Name>):internment::Intern<string>):ddlog_std::Vec<internment::Intern<string>>)(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)})))). */
                                                                                                                               program::Rule::ArrangementRule {
                                                                                                                                   description: std::borrow::Cow::from( "name_in_scope::NameOccursInScope(.scope=scope, .name=name) :- inputs::Assign(.expr_id=id, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .kind=_, .scope=scope, .span=_), var name = FlatMap((vec::map((ast::bound_vars(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)}))))."),
                                                                                                                                   arr: ( 10, 0),
                                                                                                                                   xform: XFormArrangement::Join{
                                                                                                                                              description: std::borrow::Cow::from("inputs::Assign(.expr_id=id, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                              ffun: None,
                                                                                                                                              arrangement: (27,0),
                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref id, ref pattern) = match *<types__inputs::Assign>::from_ddvalue_ref(__v1) {
                                                                                                                                                      types__inputs::Assign{expr_id: ref id, lhs: ddlog_std::Option::Some{x: ddlog_std::Either::Left{l: ref pattern}}, rhs: _, op: _} => ((*id).clone(), (*pattern).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  let ref scope = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                      types__inputs::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some((ddlog_std::tuple2((*pattern).clone(), (*scope).clone())).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(Some(XFormCollection::FlatMap{
                                                                                                                                                                      description: std::borrow::Cow::from("inputs::Assign(.expr_id=id, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .kind=_, .scope=scope, .span=_), var name = FlatMap((vec::map((ast::bound_vars(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)}))))"),
                                                                                                                                                                      fmfun: {fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                                                                      {
                                                                                                                                                                          let ddlog_std::tuple2(ref pattern, ref scope) = *<ddlog_std::tuple2<internment::Intern<types__ast::Pattern>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                          let __flattened = types__vec::map::<types__ast::Spanned<types__ast::Name>, internment::Intern<String>>((&types__ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pattern)), (&{
                                                                                                                                                                                                                                                                                                                                                                                                                     (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                                                                                                                                                                                                                                                                                                         description: "(function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)})",
                                                                                                                                                                                                                                                                                                                                                                                                                         captured: (),
                                                                                                                                                                                                                                                                                                                                                                                                                         f: {
                                                                                                                                                                                                                                                                                                                                                                                                                                fn __f(__args:*const types__ast::Spanned<types__ast::Name>, __captured: &()) -> internment::Intern<String>
                                                                                                                                                                                                                                                                                                                                                                                                                                {
                                                                                                                                                                                                                                                                                                                                                                                                                                    let name = unsafe{&*__args};
                                                                                                                                                                                                                                                                                                                                                                                                                                    name.data.clone()
                                                                                                                                                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                                                                                                                                                __f
                                                                                                                                                                                                                                                                                                                                                                                                                            }
                                                                                                                                                                                                                                                                                                                                                                                                                     }) as Box<dyn ::ddlog_rt::Closure<(*const types__ast::Spanned<types__ast::Name>), internment::Intern<String>>>)
                                                                                                                                                                                                                                                                                                                                                                                                                 }));
                                                                                                                                                                          let scope = (*scope).clone();
                                                                                                                                                                          Some(Box::new(__flattened.into_iter().map(move |name|(ddlog_std::tuple2(name.clone(), scope.clone())).into_ddvalue())))
                                                                                                                                                                      }
                                                                                                                                                                      __f},
                                                                                                                                                                      next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                              description: std::borrow::Cow::from("head of name_in_scope::NameOccursInScope(.scope=scope, .name=name) :- inputs::Assign(.expr_id=id, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .kind=_, .scope=scope, .span=_), var name = FlatMap((vec::map((ast::bound_vars(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)}))))."),
                                                                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                              {
                                                                                                                                                                                                  let ddlog_std::tuple2(ref name, ref scope) = *<ddlog_std::tuple2<internment::Intern<String>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                  Some(((NameOccursInScope{scope: (*scope).clone(), name: (*name).clone()})).into_ddvalue())
                                                                                                                                                                                              }
                                                                                                                                                                                              __f},
                                                                                                                                                                                              next: Box::new(None)
                                                                                                                                                                                          }))
                                                                                                                                                                  }))
                                                                                                                                          }
                                                                                                                               });
pub static __Rule_name_in_scope_NameOccursInScope_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=scope, .name=name}: name_in_scope::NameOccursInScope)] :- inputs::FileExport[(inputs::FileExport{.export=(ast::NamedExport{.name=(export_name: ddlog_std::Option<ast::Spanned<ast::Name>>), .alias=(export_alias: ddlog_std::Option<ast::Spanned<ast::Name>>)}: ast::ExportKind), .scope=(scope: ast::ScopeId)}: inputs::FileExport)], ((ddlog_std::Some{.x=(ast::Spanned{.data=(var name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<internment::Intern<string>>>) = ((utils::or_else: function(ddlog_std::Option<ast::Spanned<ast::Name>>, ddlog_std::Option<ast::Spanned<ast::Name>>):ddlog_std::Option<ast::Spanned<internment::Intern<string>>>)(export_alias, export_name))). */
                                                                                                                               program::Rule::CollectionRule {
                                                                                                                                   description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=scope, .name=name) :- inputs::FileExport(.export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name)))."),
                                                                                                                                   rel: 29,
                                                                                                                                   xform: Some(XFormCollection::FilterMap{
                                                                                                                                                   description: std::borrow::Cow::from("head of name_in_scope::NameOccursInScope(.scope=scope, .name=name) :- inputs::FileExport(.export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name)))."),
                                                                                                                                                   fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                   {
                                                                                                                                                       let (ref export_name, ref export_alias, ref scope) = match *<types__inputs::FileExport>::from_ddvalue_ref(&__v) {
                                                                                                                                                           types__inputs::FileExport{export: types__ast::ExportKind::NamedExport{name: ref export_name, alias: ref export_alias}, scope: ref scope} => ((*export_name).clone(), (*export_alias).clone(), (*scope).clone()),
                                                                                                                                                           _ => return None
                                                                                                                                                       };
                                                                                                                                                       let ref name: internment::Intern<String> = match types__utils::or_else::<types__ast::Spanned<types__ast::Name>>(export_alias, export_name) {
                                                                                                                                                           ddlog_std::Option::Some{x: types__ast::Spanned{data: name, span: _}} => name,
                                                                                                                                                           _ => return None
                                                                                                                                                       };
                                                                                                                                                       Some(((NameOccursInScope{scope: (*scope).clone(), name: (*name).clone()})).into_ddvalue())
                                                                                                                                                   }
                                                                                                                                                   __f},
                                                                                                                                                   next: Box::new(None)
                                                                                                                                               })
                                                                                                                               });
pub static __Rule_name_in_scope_NameOccursInScope_3 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=parent, .name=name}: name_in_scope::NameOccursInScope)] :- name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=(child: ast::ScopeId), .name=(name: internment::Intern<string>)}: name_in_scope::NameOccursInScope)], inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId)}: inputs::InputScope)]. */
                                                                                                                               program::Rule::ArrangementRule {
                                                                                                                                   description: std::borrow::Cow::from( "name_in_scope::NameOccursInScope(.scope=parent, .name=name) :- name_in_scope::NameOccursInScope(.scope=child, .name=name), inputs::InputScope(.parent=parent, .child=child)."),
                                                                                                                                   arr: ( 62, 2),
                                                                                                                                   xform: XFormArrangement::Join{
                                                                                                                                              description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=child, .name=name), inputs::InputScope(.parent=parent, .child=child)"),
                                                                                                                                              ffun: None,
                                                                                                                                              arrangement: (40,0),
                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref child, ref name) = match *<NameOccursInScope>::from_ddvalue_ref(__v1) {
                                                                                                                                                      NameOccursInScope{scope: ref child, name: ref name} => ((*child).clone(), (*name).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  let ref parent = match *<types__inputs::InputScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                      types__inputs::InputScope{parent: ref parent, child: _} => (*parent).clone(),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some(((NameOccursInScope{scope: (*parent).clone(), name: (*name).clone()})).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(None)
                                                                                                                                          }
                                                                                                                               });
pub static __Rule_name_in_scope_ScopeOfDeclName_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::ScopeOfDeclName[(name_in_scope::ScopeOfDeclName{.name=name, .scope=scope, .declared=declared}: name_in_scope::ScopeOfDeclName)] :- var_decls::VariableDeclarations[(var_decls::VariableDeclarations{.name=(name: internment::Intern<string>), .scope=(var_decls::Unhoistable{.scope=(scope: ast::ScopeId)}: var_decls::DeclarationScope), .declared_in=(declared: ast::AnyId), .meta=(_: ddlog_std::Ref<var_decls::VariableMeta>)}: var_decls::VariableDeclarations)]. */
                                                                                                                             program::Rule::CollectionRule {
                                                                                                                                 description: std::borrow::Cow::from("name_in_scope::ScopeOfDeclName(.name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.name=name, .scope=var_decls::Unhoistable{.scope=scope}, .declared_in=declared, .meta=_)."),
                                                                                                                                 rel: 86,
                                                                                                                                 xform: Some(XFormCollection::FilterMap{
                                                                                                                                                 description: std::borrow::Cow::from("head of name_in_scope::ScopeOfDeclName(.name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.name=name, .scope=var_decls::Unhoistable{.scope=scope}, .declared_in=declared, .meta=_)."),
                                                                                                                                                 fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                 {
                                                                                                                                                     let (ref name, ref scope, ref declared) = match *<crate::var_decls::VariableDeclarations>::from_ddvalue_ref(&__v) {
                                                                                                                                                         crate::var_decls::VariableDeclarations{name: ref name, scope: crate::var_decls::DeclarationScope::Unhoistable{scope: ref scope}, declared_in: ref declared, meta: _} => ((*name).clone(), (*scope).clone(), (*declared).clone()),
                                                                                                                                                         _ => return None
                                                                                                                                                     };
                                                                                                                                                     Some(((ScopeOfDeclName{name: (*name).clone(), scope: (*scope).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                 }
                                                                                                                                                 __f},
                                                                                                                                                 next: Box::new(None)
                                                                                                                                             })
                                                                                                                             });
pub static __Rule_name_in_scope_ScopeOfDeclName_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::ScopeOfDeclName[(name_in_scope::ScopeOfDeclName{.name=name, .scope=scope, .declared=declared}: name_in_scope::ScopeOfDeclName)] :- var_decls::VariableDeclarations[(var_decls::VariableDeclarations{.name=(name: internment::Intern<string>), .scope=(var_decls::Hoistable{.hoisted=(scope: ast::ScopeId), .unhoisted=(_: ast::ScopeId)}: var_decls::DeclarationScope), .declared_in=(declared: ast::AnyId), .meta=(_: ddlog_std::Ref<var_decls::VariableMeta>)}: var_decls::VariableDeclarations)]. */
                                                                                                                             program::Rule::CollectionRule {
                                                                                                                                 description: std::borrow::Cow::from("name_in_scope::ScopeOfDeclName(.name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.name=name, .scope=var_decls::Hoistable{.hoisted=scope, .unhoisted=_}, .declared_in=declared, .meta=_)."),
                                                                                                                                 rel: 86,
                                                                                                                                 xform: Some(XFormCollection::FilterMap{
                                                                                                                                                 description: std::borrow::Cow::from("head of name_in_scope::ScopeOfDeclName(.name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.name=name, .scope=var_decls::Hoistable{.hoisted=scope, .unhoisted=_}, .declared_in=declared, .meta=_)."),
                                                                                                                                                 fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                 {
                                                                                                                                                     let (ref name, ref scope, ref declared) = match *<crate::var_decls::VariableDeclarations>::from_ddvalue_ref(&__v) {
                                                                                                                                                         crate::var_decls::VariableDeclarations{name: ref name, scope: crate::var_decls::DeclarationScope::Hoistable{hoisted: ref scope, unhoisted: _}, declared_in: ref declared, meta: _} => ((*name).clone(), (*scope).clone(), (*declared).clone()),
                                                                                                                                                         _ => return None
                                                                                                                                                     };
                                                                                                                                                     Some(((ScopeOfDeclName{name: (*name).clone(), scope: (*scope).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                 }
                                                                                                                                                 __f},
                                                                                                                                                 next: Box::new(None)
                                                                                                                                             })
                                                                                                                             });
pub static __Rule_name_in_scope_NameInScope_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameInScope[(name_in_scope::NameInScope{.name=name, .scope=variable_scope, .declared=declared}: name_in_scope::NameInScope)] :- var_decls::VariableDeclarations[(var_decls::VariableDeclarations{.name=(name: internment::Intern<string>), .scope=(scope: var_decls::DeclarationScope), .declared_in=(declared: ast::AnyId), .meta=(_: ddlog_std::Ref<var_decls::VariableMeta>)}: var_decls::VariableDeclarations)], ((var variable_scope: ast::ScopeId) = (var_decls::hoisted_scope(scope))), name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=(variable_scope: ast::ScopeId), .name=(name: internment::Intern<string>)}: name_in_scope::NameOccursInScope)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("name_in_scope::NameInScope(.name=name, .scope=variable_scope, .declared=declared) :- var_decls::VariableDeclarations(.name=name, .scope=scope, .declared_in=declared, .meta=_), (var variable_scope = (var_decls::hoisted_scope(scope))), name_in_scope::NameOccursInScope(.scope=variable_scope, .name=name)."),
                                                                                                                             rel: 86,
                                                                                                                             xform: Some(XFormCollection::Arrange {
                                                                                                                                             description: std::borrow::Cow::from("arrange var_decls::VariableDeclarations(.name=name, .scope=scope, .declared_in=declared, .meta=_) by (variable_scope, name)"),
                                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                             {
                                                                                                                                                 let (ref name, ref scope, ref declared) = match *<crate::var_decls::VariableDeclarations>::from_ddvalue_ref(&__v) {
                                                                                                                                                     crate::var_decls::VariableDeclarations{name: ref name, scope: ref scope, declared_in: ref declared, meta: _} => ((*name).clone(), (*scope).clone(), (*declared).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 let ref variable_scope: types__ast::ScopeId = match crate::var_decls::hoisted_scope(scope) {
                                                                                                                                                     variable_scope => variable_scope,
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 Some(((ddlog_std::tuple2((*variable_scope).clone(), (*name).clone())).into_ddvalue(), (ddlog_std::tuple3((*name).clone(), (*declared).clone(), (*variable_scope).clone())).into_ddvalue()))
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                description: std::borrow::Cow::from("var_decls::VariableDeclarations(.name=name, .scope=scope, .declared_in=declared, .meta=_), (var variable_scope = (var_decls::hoisted_scope(scope))), name_in_scope::NameOccursInScope(.scope=variable_scope, .name=name)"),
                                                                                                                                                                ffun: None,
                                                                                                                                                                arrangement: (62,0),
                                                                                                                                                                jfun: {fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                {
                                                                                                                                                                    let ddlog_std::tuple3(ref name, ref declared, ref variable_scope) = *<ddlog_std::tuple3<internment::Intern<String>, types__ast::AnyId, types__ast::ScopeId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                    Some(((NameInScope{name: (*name).clone(), scope: (*variable_scope).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                                }
                                                                                                                                                                __f},
                                                                                                                                                                next: Box::new(None)
                                                                                                                                                            })
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_name_in_scope_NameInScope_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameInScope[(name_in_scope::NameInScope{.name=name, .scope=child, .declared=declared}: name_in_scope::NameInScope)] :- name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=(child: ast::ScopeId), .name=(name: internment::Intern<string>)}: name_in_scope::NameOccursInScope)], not name_in_scope::ScopeOfDeclName[(name_in_scope::ScopeOfDeclName{.name=(name: internment::Intern<string>), .scope=(child: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::ScopeOfDeclName)], inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId)}: inputs::InputScope)], name_in_scope::NameInScope[(name_in_scope::NameInScope{.name=(name: internment::Intern<string>), .scope=(parent: ast::ScopeId), .declared=(declared: ast::AnyId)}: name_in_scope::NameInScope)]. */
                                                                                                                         program::Rule::ArrangementRule {
                                                                                                                             description: std::borrow::Cow::from( "name_in_scope::NameInScope(.name=name, .scope=child, .declared=declared) :- name_in_scope::NameOccursInScope(.scope=child, .name=name), not name_in_scope::ScopeOfDeclName(.name=name, .scope=child, .declared=_), inputs::InputScope(.parent=parent, .child=child), name_in_scope::NameInScope(.name=name, .scope=parent, .declared=declared)."),
                                                                                                                             arr: ( 62, 1),
                                                                                                                             xform: XFormArrangement::Antijoin {
                                                                                                                                        description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=child, .name=name), not name_in_scope::ScopeOfDeclName(.name=name, .scope=child, .declared=_)"),
                                                                                                                                        ffun: None,
                                                                                                                                        arrangement: (63,0),
                                                                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                description: std::borrow::Cow::from("arrange name_in_scope::NameOccursInScope(.scope=child, .name=name), not name_in_scope::ScopeOfDeclName(.name=name, .scope=child, .declared=_) by (child)"),
                                                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                {
                                                                                                                                                                    let (ref child, ref name) = match *<NameOccursInScope>::from_ddvalue_ref(&__v) {
                                                                                                                                                                        NameOccursInScope{scope: ref child, name: ref name} => ((*child).clone(), (*name).clone()),
                                                                                                                                                                        _ => return None
                                                                                                                                                                    };
                                                                                                                                                                    Some((((*child).clone()).into_ddvalue(), (ddlog_std::tuple2((*child).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                                }
                                                                                                                                                                __f},
                                                                                                                                                                next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                   description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=child, .name=name), not name_in_scope::ScopeOfDeclName(.name=name, .scope=child, .declared=_), inputs::InputScope(.parent=parent, .child=child)"),
                                                                                                                                                                                   ffun: None,
                                                                                                                                                                                   arrangement: (40,0),
                                                                                                                                                                                   jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                   {
                                                                                                                                                                                       let ddlog_std::tuple2(ref child, ref name) = *<ddlog_std::tuple2<types__ast::ScopeId, internment::Intern<String>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                       let ref parent = match *<types__inputs::InputScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                           types__inputs::InputScope{parent: ref parent, child: _} => (*parent).clone(),
                                                                                                                                                                                           _ => return None
                                                                                                                                                                                       };
                                                                                                                                                                                       Some((ddlog_std::tuple3((*child).clone(), (*name).clone(), (*parent).clone())).into_ddvalue())
                                                                                                                                                                                   }
                                                                                                                                                                                   __f},
                                                                                                                                                                                   next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                           description: std::borrow::Cow::from("arrange name_in_scope::NameOccursInScope(.scope=child, .name=name), not name_in_scope::ScopeOfDeclName(.name=name, .scope=child, .declared=_), inputs::InputScope(.parent=parent, .child=child) by (name, parent)"),
                                                                                                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                           {
                                                                                                                                                                                                               let ddlog_std::tuple3(ref child, ref name, ref parent) = *<ddlog_std::tuple3<types__ast::ScopeId, internment::Intern<String>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                               Some(((ddlog_std::tuple2((*name).clone(), (*parent).clone())).into_ddvalue(), (ddlog_std::tuple2((*child).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                                                                           }
                                                                                                                                                                                                           __f},
                                                                                                                                                                                                           next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                                                              description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=child, .name=name), not name_in_scope::ScopeOfDeclName(.name=name, .scope=child, .declared=_), inputs::InputScope(.parent=parent, .child=child), name_in_scope::NameInScope(.name=name, .scope=parent, .declared=declared)"),
                                                                                                                                                                                                                              ffun: None,
                                                                                                                                                                                                                              arrangement: (61,0),
                                                                                                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                                                              {
                                                                                                                                                                                                                                  let ddlog_std::tuple2(ref child, ref name) = *<ddlog_std::tuple2<types__ast::ScopeId, internment::Intern<String>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                                                  let ref declared = match *<NameInScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                                                      NameInScope{name: _, scope: _, declared: ref declared} => (*declared).clone(),
                                                                                                                                                                                                                                      _ => return None
                                                                                                                                                                                                                                  };
                                                                                                                                                                                                                                  Some(((NameInScope{name: (*name).clone(), scope: (*child).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                                                                                              }
                                                                                                                                                                                                                              __f},
                                                                                                                                                                                                                              next: Box::new(None)
                                                                                                                                                                                                                          })
                                                                                                                                                                                                       }))
                                                                                                                                                                               })
                                                                                                                                                            }))
                                                                                                                                    }
                                                                                                                         });