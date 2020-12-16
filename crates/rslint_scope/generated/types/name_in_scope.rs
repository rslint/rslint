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
    clippy::unknown_clippy_lints,
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

//use ::serde::de::DeserializeOwned;
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


#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NameInScope {
    pub file: types__ast::FileId,
    pub name: types__ast::Name,
    pub scope: types__ast::ScopeId,
    pub declared: types__ast::AnyId
}
impl abomonation::Abomonation for NameInScope{}
::differential_datalog::decl_struct_from_record!(NameInScope["name_in_scope::NameInScope"]<>, ["name_in_scope::NameInScope"][4]{[0]file["file"]: types__ast::FileId, [1]name["name"]: types__ast::Name, [2]scope["scope"]: types__ast::ScopeId, [3]declared["declared"]: types__ast::AnyId});
::differential_datalog::decl_struct_into_record!(NameInScope, ["name_in_scope::NameInScope"]<>, file, name, scope, declared);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameInScope, <>, file: types__ast::FileId, name: types__ast::Name, scope: types__ast::ScopeId, declared: types__ast::AnyId);
impl ::std::fmt::Display for NameInScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameInScope{file,name,scope,declared} => {
                __formatter.write_str("name_in_scope::NameInScope{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NameOccurance {
    pub name: types__ast::Name,
    pub file: types__ast::FileId
}
impl abomonation::Abomonation for NameOccurance{}
::differential_datalog::decl_struct_from_record!(NameOccurance["name_in_scope::NameOccurance"]<>, ["name_in_scope::NameOccurance"][2]{[0]name["name"]: types__ast::Name, [1]file["file"]: types__ast::FileId});
::differential_datalog::decl_struct_into_record!(NameOccurance, ["name_in_scope::NameOccurance"]<>, name, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameOccurance, <>, name: types__ast::Name, file: types__ast::FileId);
impl ::std::fmt::Display for NameOccurance {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameOccurance{name,file} => {
                __formatter.write_str("name_in_scope::NameOccurance{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameOccurance {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NameOccursInScope {
    pub scope: types__ast::ScopeId,
    pub name: internment::Intern<NameOccurance>
}
impl abomonation::Abomonation for NameOccursInScope{}
::differential_datalog::decl_struct_from_record!(NameOccursInScope["name_in_scope::NameOccursInScope"]<>, ["name_in_scope::NameOccursInScope"][2]{[0]scope["scope"]: types__ast::ScopeId, [1]name["name"]: internment::Intern<NameOccurance>});
::differential_datalog::decl_struct_into_record!(NameOccursInScope, ["name_in_scope::NameOccursInScope"]<>, scope, name);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameOccursInScope, <>, scope: types__ast::ScopeId, name: internment::Intern<NameOccurance>);
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum NameOrigin {
    AutoGlobal,
    Imported,
    UserDefined {
        scope: types__ast::ScopeId
    }
}
impl abomonation::Abomonation for NameOrigin{}
::differential_datalog::decl_enum_from_record!(NameOrigin["name_in_scope::NameOrigin"]<>, AutoGlobal["name_in_scope::AutoGlobal"][0]{}, Imported["name_in_scope::Imported"][0]{}, UserDefined["name_in_scope::UserDefined"][1]{[0]scope["scope"]: types__ast::ScopeId});
::differential_datalog::decl_enum_into_record!(NameOrigin<>, AutoGlobal["name_in_scope::AutoGlobal"]{}, Imported["name_in_scope::Imported"]{}, UserDefined["name_in_scope::UserDefined"]{scope});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(NameOrigin<>, AutoGlobal{}, Imported{}, UserDefined{scope: types__ast::ScopeId});
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ScopeOfDeclName {
    pub file: types__ast::FileId,
    pub name: types__ast::Name,
    pub scope: types__ast::ScopeId,
    pub declared: types__ast::AnyId
}
impl abomonation::Abomonation for ScopeOfDeclName{}
::differential_datalog::decl_struct_from_record!(ScopeOfDeclName["name_in_scope::ScopeOfDeclName"]<>, ["name_in_scope::ScopeOfDeclName"][4]{[0]file["file"]: types__ast::FileId, [1]name["name"]: types__ast::Name, [2]scope["scope"]: types__ast::ScopeId, [3]declared["declared"]: types__ast::AnyId});
::differential_datalog::decl_struct_into_record!(ScopeOfDeclName, ["name_in_scope::ScopeOfDeclName"]<>, file, name, scope, declared);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ScopeOfDeclName, <>, file: types__ast::FileId, name: types__ast::Name, scope: types__ast::ScopeId, declared: types__ast::AnyId);
impl ::std::fmt::Display for ScopeOfDeclName {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ScopeOfDeclName{file,name,scope,declared} => {
                __formatter.write_str("name_in_scope::ScopeOfDeclName{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
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
                                                                                                                                          name: std::borrow::Cow::from(r###"(name_in_scope::NameOccursInScope{.scope=(_0: ast::ScopeId), .name=((&(name_in_scope::NameOccurance{.name=(_1: internment::Intern<string>), .file=(_2: ast::FileId)}: name_in_scope::NameOccurance)): internment::Intern<name_in_scope::NameOccurance>)}: name_in_scope::NameOccursInScope) /*semijoin*/"###),
                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                          {
                                                                                                                                              match < NameOccursInScope>::from_ddvalue(__v) {
                                                                                                                                                  NameOccursInScope{scope: ref _0, name: ref _0_} => match ((*_0_)).deref() {
                                                                                                                                                                                                         NameOccurance{name: _1, file: _2} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                                                                                         _ => None
                                                                                                                                                                                                     },
                                                                                                                                                  _ => None
                                                                                                                                              }
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          distinct: false
                                                                                                                                      });
pub static __Arng_name_in_scope_NameOccursInScope_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                         name: std::borrow::Cow::from(r###"(name_in_scope::NameOccursInScope{.scope=(_2: ast::ScopeId), .name=((&(name_in_scope::NameOccurance{.name=(_1: internment::Intern<string>), .file=(_0: ast::FileId)}: name_in_scope::NameOccurance)): internment::Intern<name_in_scope::NameOccurance>)}: name_in_scope::NameOccursInScope) /*join*/"###),
                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                          {
                                                                                                                                              let __cloned = __v.clone();
                                                                                                                                              match < NameOccursInScope>::from_ddvalue(__v) {
                                                                                                                                                  NameOccursInScope{scope: ref _2, name: ref _0_} => match ((*_0_)).deref() {
                                                                                                                                                                                                         NameOccurance{name: _1, file: _0} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                                                                                         _ => None
                                                                                                                                                                                                     },
                                                                                                                                                  _ => None
                                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          queryable: false
                                                                                                                                      });
pub static __Arng_name_in_scope_NameOccursInScope_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                         name: std::borrow::Cow::from(r###"(name_in_scope::NameOccursInScope{.scope=(_0: ast::ScopeId), .name=((&(name_in_scope::NameOccurance{.name=(_: internment::Intern<string>), .file=(_1: ast::FileId)}: name_in_scope::NameOccurance)): internment::Intern<name_in_scope::NameOccurance>)}: name_in_scope::NameOccursInScope) /*join*/"###),
                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                          {
                                                                                                                                              let __cloned = __v.clone();
                                                                                                                                              match < NameOccursInScope>::from_ddvalue(__v) {
                                                                                                                                                  NameOccursInScope{scope: ref _0, name: ref _0_} => match ((*_0_)).deref() {
                                                                                                                                                                                                         NameOccurance{name: _, file: _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                                                                         _ => None
                                                                                                                                                                                                     },
                                                                                                                                                  _ => None
                                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                                          }
                                                                                                                                          __f},
                                                                                                                                          queryable: false
                                                                                                                                      });
pub static __Arng_name_in_scope_ScopeOfDeclName_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                        name: std::borrow::Cow::from(r###"(name_in_scope::ScopeOfDeclName{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::ScopeOfDeclName) /*antijoin*/"###),
                                                                                                                                        fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            match < ScopeOfDeclName>::from_ddvalue(__v) {
                                                                                                                                                ScopeOfDeclName{file: ref _0, name: ref _1, scope: ref _2, declared: _} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                                _ => None
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        distinct: true
                                                                                                                                    });
pub static __Arng_name_in_scope_ScopeOfDeclName_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                        name: std::borrow::Cow::from(r###"(name_in_scope::ScopeOfDeclName{.file=(_0: ast::FileId), .name=(_: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_2: ast::AnyId)}: name_in_scope::ScopeOfDeclName) /*antijoin*/"###),
                                                                                                                                        fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            match < ScopeOfDeclName>::from_ddvalue(__v) {
                                                                                                                                                ScopeOfDeclName{file: ref _0, name: _, scope: ref _1, declared: ref _2} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                                _ => None
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        distinct: true
                                                                                                                                    });
pub static __Arng_name_in_scope_NameInScope_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: ref _1, scope: ref _2, declared: _} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                    name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: ref _1, scope: ref _2, declared: _} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    distinct: true
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                    name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=_2, .declared=(_3: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###),
                                                                                                                                    fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: ref _1, scope: ref _2, declared: ref _3} => Some((ddlog_std::tuple4((*_0).clone(), (*_1).clone(), (*_2).clone(), (*_3).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    distinct: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .declared=(ast::AnyIdStmt{.stmt=(_: ast::StmtId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: ref _1, scope: ref _2, declared: types__ast::AnyId::AnyIdStmt{stmt: _}} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_4 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .declared=(ast::AnyIdClass{.class=(_: ast::ClassId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: ref _1, scope: ref _2, declared: types__ast::AnyId::AnyIdClass{class: _}} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_5 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .declared=(ast::AnyIdFunc{.func=(_: ast::FuncId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: ref _1, scope: ref _2, declared: types__ast::AnyId::AnyIdFunc{func: _}} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: false
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_6 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=_0, .name=_2, .scope=_1, .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: ref _2, scope: ref _1, declared: _} => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: true
                                                                                                                                });
pub static __Arng_name_in_scope_NameInScope_7 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                   name: std::borrow::Cow::from(r###"(name_in_scope::NameInScope{.file=_0, .name=(_: internment::Intern<string>), .scope=_1, .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###),
                                                                                                                                    afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                    {
                                                                                                                                        let __cloned = __v.clone();
                                                                                                                                        match < NameInScope>::from_ddvalue(__v) {
                                                                                                                                            NameInScope{file: ref _0, name: _, scope: ref _1, declared: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                            _ => None
                                                                                                                                        }.map(|x|(x,__cloned))
                                                                                                                                    }
                                                                                                                                    __f},
                                                                                                                                    queryable: true
                                                                                                                                });
pub static __Rule_name_in_scope_NameOccursInScope_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=scope, .name=interned}: name_in_scope::NameOccursInScope)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(id: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(id: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], ((var interned: internment::Intern<name_in_scope::NameOccurance>) = ((internment::intern: function(name_in_scope::NameOccurance):internment::Intern<name_in_scope::NameOccurance>)((name_in_scope::NameOccurance{.name=name, .file=file}: name_in_scope::NameOccurance)))). */
                                                                                                                               program::Rule::ArrangementRule {
                                                                                                                                   description: std::borrow::Cow::from( "name_in_scope::NameOccursInScope(.scope=scope, .name=interned) :- inputs::NameRef(.expr_id=id, .file=file, .value=name), inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_), (var interned = (internment::intern(name_in_scope::NameOccurance{.name=name, .file=file})))."),
                                                                                                                                   arr: ( 44, 0),
                                                                                                                                   xform: XFormArrangement::Join{
                                                                                                                                              description: std::borrow::Cow::from("inputs::NameRef(.expr_id=id, .file=file, .value=name), inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                              ffun: None,
                                                                                                                                              arrangement: (28,0),
                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref id, ref file, ref name) = match *<types__inputs::NameRef>::from_ddvalue_ref(__v1) {
                                                                                                                                                      types__inputs::NameRef{expr_id: ref id, file: ref file, value: ref name} => ((*id).clone(), (*file).clone(), (*name).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  let ref scope = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                      types__inputs::Expression{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  let ref interned: internment::Intern<NameOccurance> = match internment::intern((&(NameOccurance{name: (*name).clone(), file: (*file).clone()}))) {
                                                                                                                                                      interned => interned,
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some(((NameOccursInScope{scope: (*scope).clone(), name: (*interned).clone()})).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(None)
                                                                                                                                          }
                                                                                                                               });
pub static __Rule_name_in_scope_NameOccursInScope_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=scope, .name=interned}: name_in_scope::NameOccursInScope)] :- inputs::Assign[(inputs::Assign{.expr_id=(id: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pattern: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(id: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((vec::map: function(ddlog_std::Vec<ast::Spanned<ast::Name>>, function(ast::Spanned<ast::Name>):internment::Intern<string>):ddlog_std::Vec<internment::Intern<string>>)(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)})))), ((var interned: internment::Intern<name_in_scope::NameOccurance>) = ((internment::intern: function(name_in_scope::NameOccurance):internment::Intern<name_in_scope::NameOccurance>)((name_in_scope::NameOccurance{.name=name, .file=file}: name_in_scope::NameOccurance)))). */
                                                                                                                               program::Rule::ArrangementRule {
                                                                                                                                   description: std::borrow::Cow::from( "name_in_scope::NameOccursInScope(.scope=scope, .name=interned) :- inputs::Assign(.expr_id=id, .file=file, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_), var name = FlatMap((vec::map((ast::bound_vars(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)})))), (var interned = (internment::intern(name_in_scope::NameOccurance{.name=name, .file=file})))."),
                                                                                                                                   arr: ( 11, 0),
                                                                                                                                   xform: XFormArrangement::Join{
                                                                                                                                              description: std::borrow::Cow::from("inputs::Assign(.expr_id=id, .file=file, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                              ffun: None,
                                                                                                                                              arrangement: (28,0),
                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref id, ref file, ref pattern) = match *<types__inputs::Assign>::from_ddvalue_ref(__v1) {
                                                                                                                                                      types__inputs::Assign{expr_id: ref id, file: ref file, lhs: ddlog_std::Option::Some{x: ddlog_std::Either::Left{l: ref pattern}}, rhs: _, op: _} => ((*id).clone(), (*file).clone(), (*pattern).clone()),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  let ref scope = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                      types__inputs::Expression{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some((ddlog_std::tuple3((*file).clone(), (*pattern).clone(), (*scope).clone())).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(Some(XFormCollection::FlatMap{
                                                                                                                                                                      description: std::borrow::Cow::from("inputs::Assign(.expr_id=id, .file=file, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_), var name = FlatMap((vec::map((ast::bound_vars(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)}))))"),
                                                                                                                                                                      fmfun: {fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                                                                      {
                                                                                                                                                                          let ddlog_std::tuple3(ref file, ref pattern, ref scope) = *<ddlog_std::tuple3<types__ast::FileId, internment::Intern<types__ast::Pattern>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
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
                                                                                                                                                                          let file = (*file).clone();
                                                                                                                                                                          let scope = (*scope).clone();
                                                                                                                                                                          Some(Box::new(__flattened.into_iter().map(move |name|(ddlog_std::tuple3(name.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                                                                                                                      }
                                                                                                                                                                      __f},
                                                                                                                                                                      next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                              description: std::borrow::Cow::from("head of name_in_scope::NameOccursInScope(.scope=scope, .name=interned) :- inputs::Assign(.expr_id=id, .file=file, .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pattern}}, .rhs=_, .op=_), inputs::Expression(.id=id, .file=file, .kind=_, .scope=scope, .span=_), var name = FlatMap((vec::map((ast::bound_vars(pattern)), (function(name: ast::Spanned<ast::Name>):internment::Intern<string>{(name.data)})))), (var interned = (internment::intern(name_in_scope::NameOccurance{.name=name, .file=file})))."),
                                                                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                              {
                                                                                                                                                                                                  let ddlog_std::tuple3(ref name, ref file, ref scope) = *<ddlog_std::tuple3<internment::Intern<String>, types__ast::FileId, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                  let ref interned: internment::Intern<NameOccurance> = match internment::intern((&(NameOccurance{name: (*name).clone(), file: (*file).clone()}))) {
                                                                                                                                                                                                      interned => interned,
                                                                                                                                                                                                      _ => return None
                                                                                                                                                                                                  };
                                                                                                                                                                                                  Some(((NameOccursInScope{scope: (*scope).clone(), name: (*interned).clone()})).into_ddvalue())
                                                                                                                                                                                              }
                                                                                                                                                                                              __f},
                                                                                                                                                                                              next: Box::new(None)
                                                                                                                                                                                          }))
                                                                                                                                                                  }))
                                                                                                                                          }
                                                                                                                               });
pub static __Rule_name_in_scope_NameOccursInScope_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=scope, .name=interned}: name_in_scope::NameOccursInScope)] :- inputs::FileExport[(inputs::FileExport{.file=(file: ast::FileId), .export=(ast::NamedExport{.name=(export_name: ddlog_std::Option<ast::Spanned<ast::Name>>), .alias=(export_alias: ddlog_std::Option<ast::Spanned<ast::Name>>)}: ast::ExportKind), .scope=(scope: ast::ScopeId)}: inputs::FileExport)], ((ddlog_std::Some{.x=(ast::Spanned{.data=(var name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<internment::Intern<string>>>) = ((utils::or_else: function(ddlog_std::Option<ast::Spanned<ast::Name>>, ddlog_std::Option<ast::Spanned<ast::Name>>):ddlog_std::Option<ast::Spanned<internment::Intern<string>>>)(export_alias, export_name))), ((var interned: internment::Intern<name_in_scope::NameOccurance>) = ((internment::intern: function(name_in_scope::NameOccurance):internment::Intern<name_in_scope::NameOccurance>)((name_in_scope::NameOccurance{.name=name, .file=file}: name_in_scope::NameOccurance)))). */
                                                                                                                               program::Rule::CollectionRule {
                                                                                                                                   description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=scope, .name=interned) :- inputs::FileExport(.file=file, .export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name))), (var interned = (internment::intern(name_in_scope::NameOccurance{.name=name, .file=file})))."),
                                                                                                                                   rel: 30,
                                                                                                                                   xform: Some(XFormCollection::FilterMap{
                                                                                                                                                   description: std::borrow::Cow::from("head of name_in_scope::NameOccursInScope(.scope=scope, .name=interned) :- inputs::FileExport(.file=file, .export=ast::NamedExport{.name=export_name, .alias=export_alias}, .scope=scope), (ddlog_std::Some{.x=ast::Spanned{.data=var name, .span=_}} = (utils::or_else(export_alias, export_name))), (var interned = (internment::intern(name_in_scope::NameOccurance{.name=name, .file=file})))."),
                                                                                                                                                   fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                   {
                                                                                                                                                       let (ref file, ref export_name, ref export_alias, ref scope) = match *<types__inputs::FileExport>::from_ddvalue_ref(&__v) {
                                                                                                                                                           types__inputs::FileExport{file: ref file, export: types__ast::ExportKind::NamedExport{name: ref export_name, alias: ref export_alias}, scope: ref scope} => ((*file).clone(), (*export_name).clone(), (*export_alias).clone(), (*scope).clone()),
                                                                                                                                                           _ => return None
                                                                                                                                                       };
                                                                                                                                                       let ref name: internment::Intern<String> = match types__utils::or_else::<types__ast::Spanned<types__ast::Name>>(export_alias, export_name) {
                                                                                                                                                           ddlog_std::Option::Some{x: types__ast::Spanned{data: name, span: _}} => name,
                                                                                                                                                           _ => return None
                                                                                                                                                       };
                                                                                                                                                       let ref interned: internment::Intern<NameOccurance> = match internment::intern((&(NameOccurance{name: (*name).clone(), file: (*file).clone()}))) {
                                                                                                                                                           interned => interned,
                                                                                                                                                           _ => return None
                                                                                                                                                       };
                                                                                                                                                       Some(((NameOccursInScope{scope: (*scope).clone(), name: (*interned).clone()})).into_ddvalue())
                                                                                                                                                   }
                                                                                                                                                   __f},
                                                                                                                                                   next: Box::new(None)
                                                                                                                                               })
                                                                                                                               });
pub static __Rule_name_in_scope_NameOccursInScope_3 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=parent, .name=interned}: name_in_scope::NameOccursInScope)] :- name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=(child: ast::ScopeId), .name=(interned@ ((&(name_in_scope::NameOccurance{.name=(_: internment::Intern<string>), .file=(file: ast::FileId)}: name_in_scope::NameOccurance)): internment::Intern<name_in_scope::NameOccurance>))}: name_in_scope::NameOccursInScope)], inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)]. */
                                                                                                                               program::Rule::ArrangementRule {
                                                                                                                                   description: std::borrow::Cow::from( "name_in_scope::NameOccursInScope(.scope=parent, .name=interned) :- name_in_scope::NameOccursInScope(.scope=child, .name=(interned@ (&name_in_scope::NameOccurance{.name=_, .file=file}))), inputs::InputScope(.parent=parent, .child=child, .file=file)."),
                                                                                                                                   arr: ( 63, 2),
                                                                                                                                   xform: XFormArrangement::Join{
                                                                                                                                              description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=child, .name=(interned@ (&name_in_scope::NameOccurance{.name=_, .file=file}))), inputs::InputScope(.parent=parent, .child=child, .file=file)"),
                                                                                                                                              ffun: None,
                                                                                                                                              arrangement: (41,0),
                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  let (ref child, ref interned, ref file) = match *<NameOccursInScope>::from_ddvalue_ref(__v1) {
                                                                                                                                                      NameOccursInScope{scope: ref child, name: ref interned} => match interned {
                                                                                                                                                                                                                     ref _0_ => match ((*_0_)).deref() {
                                                                                                                                                                                                                                    NameOccurance{name: _, file: file} => ((*child).clone(), (*interned).clone(), (*file).clone()),
                                                                                                                                                                                                                                    _ => return None
                                                                                                                                                                                                                                },
                                                                                                                                                                                                                     _ => return None
                                                                                                                                                                                                                 },
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  let ref parent = match *<types__inputs::InputScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                      types__inputs::InputScope{parent: ref parent, child: _, file: _} => (*parent).clone(),
                                                                                                                                                      _ => return None
                                                                                                                                                  };
                                                                                                                                                  Some(((NameOccursInScope{scope: (*parent).clone(), name: (*interned).clone()})).into_ddvalue())
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              next: Box::new(None)
                                                                                                                                          }
                                                                                                                               });
pub static __Rule_name_in_scope_ScopeOfDeclName_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::ScopeOfDeclName[(name_in_scope::ScopeOfDeclName{.file=file, .name=name, .scope=scope, .declared=declared}: name_in_scope::ScopeOfDeclName)] :- var_decls::VariableDeclarations[(var_decls::VariableDeclarations{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(var_decls::Unhoistable{.scope=(scope: ast::ScopeId)}: var_decls::DeclarationScope), .declared_in=(declared: ast::AnyId), .meta=(_: ddlog_std::Ref<var_decls::VariableMeta>)}: var_decls::VariableDeclarations)]. */
                                                                                                                             program::Rule::CollectionRule {
                                                                                                                                 description: std::borrow::Cow::from("name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.file=file, .name=name, .scope=var_decls::Unhoistable{.scope=scope}, .declared_in=declared, .meta=_)."),
                                                                                                                                 rel: 84,
                                                                                                                                 xform: Some(XFormCollection::FilterMap{
                                                                                                                                                 description: std::borrow::Cow::from("head of name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.file=file, .name=name, .scope=var_decls::Unhoistable{.scope=scope}, .declared_in=declared, .meta=_)."),
                                                                                                                                                 fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                 {
                                                                                                                                                     let (ref file, ref name, ref scope, ref declared) = match *<crate::var_decls::VariableDeclarations>::from_ddvalue_ref(&__v) {
                                                                                                                                                         crate::var_decls::VariableDeclarations{file: ref file, name: ref name, scope: crate::var_decls::DeclarationScope::Unhoistable{scope: ref scope}, declared_in: ref declared, meta: _} => ((*file).clone(), (*name).clone(), (*scope).clone(), (*declared).clone()),
                                                                                                                                                         _ => return None
                                                                                                                                                     };
                                                                                                                                                     Some(((ScopeOfDeclName{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                 }
                                                                                                                                                 __f},
                                                                                                                                                 next: Box::new(None)
                                                                                                                                             })
                                                                                                                             });
pub static __Rule_name_in_scope_ScopeOfDeclName_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::ScopeOfDeclName[(name_in_scope::ScopeOfDeclName{.file=file, .name=name, .scope=scope, .declared=declared}: name_in_scope::ScopeOfDeclName)] :- var_decls::VariableDeclarations[(var_decls::VariableDeclarations{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(var_decls::Hoistable{.hoisted=(scope: ast::ScopeId), .unhoisted=(_: ast::ScopeId)}: var_decls::DeclarationScope), .declared_in=(declared: ast::AnyId), .meta=(_: ddlog_std::Ref<var_decls::VariableMeta>)}: var_decls::VariableDeclarations)]. */
                                                                                                                             program::Rule::CollectionRule {
                                                                                                                                 description: std::borrow::Cow::from("name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.file=file, .name=name, .scope=var_decls::Hoistable{.hoisted=scope, .unhoisted=_}, .declared_in=declared, .meta=_)."),
                                                                                                                                 rel: 84,
                                                                                                                                 xform: Some(XFormCollection::FilterMap{
                                                                                                                                                 description: std::borrow::Cow::from("head of name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=scope, .declared=declared) :- var_decls::VariableDeclarations(.file=file, .name=name, .scope=var_decls::Hoistable{.hoisted=scope, .unhoisted=_}, .declared_in=declared, .meta=_)."),
                                                                                                                                                 fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                 {
                                                                                                                                                     let (ref file, ref name, ref scope, ref declared) = match *<crate::var_decls::VariableDeclarations>::from_ddvalue_ref(&__v) {
                                                                                                                                                         crate::var_decls::VariableDeclarations{file: ref file, name: ref name, scope: crate::var_decls::DeclarationScope::Hoistable{hoisted: ref scope, unhoisted: _}, declared_in: ref declared, meta: _} => ((*file).clone(), (*name).clone(), (*scope).clone(), (*declared).clone()),
                                                                                                                                                         _ => return None
                                                                                                                                                     };
                                                                                                                                                     Some(((ScopeOfDeclName{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                 }
                                                                                                                                                 __f},
                                                                                                                                                 next: Box::new(None)
                                                                                                                                             })
                                                                                                                             });
pub static __Rule_name_in_scope_NameInScope_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameInScope[(name_in_scope::NameInScope{.file=file, .name=name, .scope=variable_scope, .declared=declared}: name_in_scope::NameInScope)] :- var_decls::VariableDeclarations[(var_decls::VariableDeclarations{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: var_decls::DeclarationScope), .declared_in=(declared: ast::AnyId), .meta=(_: ddlog_std::Ref<var_decls::VariableMeta>)}: var_decls::VariableDeclarations)], ((var variable_scope: ast::ScopeId) = (var_decls::hoisted_scope(scope))), name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=(variable_scope: ast::ScopeId), .name=((&(name_in_scope::NameOccurance{.name=(name: internment::Intern<string>), .file=(file: ast::FileId)}: name_in_scope::NameOccurance)): internment::Intern<name_in_scope::NameOccurance>)}: name_in_scope::NameOccursInScope)]. */
                                                                                                                         program::Rule::CollectionRule {
                                                                                                                             description: std::borrow::Cow::from("name_in_scope::NameInScope(.file=file, .name=name, .scope=variable_scope, .declared=declared) :- var_decls::VariableDeclarations(.file=file, .name=name, .scope=scope, .declared_in=declared, .meta=_), (var variable_scope = (var_decls::hoisted_scope(scope))), name_in_scope::NameOccursInScope(.scope=variable_scope, .name=(&name_in_scope::NameOccurance{.name=name, .file=file}))."),
                                                                                                                             rel: 84,
                                                                                                                             xform: Some(XFormCollection::Arrange {
                                                                                                                                             description: std::borrow::Cow::from("arrange var_decls::VariableDeclarations(.file=file, .name=name, .scope=scope, .declared_in=declared, .meta=_) by (variable_scope, name, file)"),
                                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                             {
                                                                                                                                                 let (ref file, ref name, ref scope, ref declared) = match *<crate::var_decls::VariableDeclarations>::from_ddvalue_ref(&__v) {
                                                                                                                                                     crate::var_decls::VariableDeclarations{file: ref file, name: ref name, scope: ref scope, declared_in: ref declared, meta: _} => ((*file).clone(), (*name).clone(), (*scope).clone(), (*declared).clone()),
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 let ref variable_scope: types__ast::ScopeId = match crate::var_decls::hoisted_scope(scope) {
                                                                                                                                                     variable_scope => variable_scope,
                                                                                                                                                     _ => return None
                                                                                                                                                 };
                                                                                                                                                 Some(((ddlog_std::tuple3((*variable_scope).clone(), (*name).clone(), (*file).clone())).into_ddvalue(), (ddlog_std::tuple4((*file).clone(), (*name).clone(), (*declared).clone(), (*variable_scope).clone())).into_ddvalue()))
                                                                                                                                             }
                                                                                                                                             __f},
                                                                                                                                             next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                description: std::borrow::Cow::from("var_decls::VariableDeclarations(.file=file, .name=name, .scope=scope, .declared_in=declared, .meta=_), (var variable_scope = (var_decls::hoisted_scope(scope))), name_in_scope::NameOccursInScope(.scope=variable_scope, .name=(&name_in_scope::NameOccurance{.name=name, .file=file}))"),
                                                                                                                                                                ffun: None,
                                                                                                                                                                arrangement: (63,0),
                                                                                                                                                                jfun: {fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                {
                                                                                                                                                                    let ddlog_std::tuple4(ref file, ref name, ref declared, ref variable_scope) = *<ddlog_std::tuple4<types__ast::FileId, internment::Intern<String>, types__ast::AnyId, types__ast::ScopeId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                    Some(((NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*variable_scope).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                                }
                                                                                                                                                                __f},
                                                                                                                                                                next: Box::new(None)
                                                                                                                                                            })
                                                                                                                                         })
                                                                                                                         });
pub static __Rule_name_in_scope_NameInScope_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* name_in_scope::NameInScope[(name_in_scope::NameInScope{.file=file, .name=name, .scope=to, .declared=declared}: name_in_scope::NameInScope)] :- name_in_scope::NameOccursInScope[(name_in_scope::NameOccursInScope{.scope=(to: ast::ScopeId), .name=((&(name_in_scope::NameOccurance{.name=(name: internment::Intern<string>), .file=(file: ast::FileId)}: name_in_scope::NameOccurance)): internment::Intern<name_in_scope::NameOccurance>)}: name_in_scope::NameOccursInScope)], not name_in_scope::ScopeOfDeclName[(name_in_scope::ScopeOfDeclName{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(to: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::ScopeOfDeclName)], inputs::InputScope[(inputs::InputScope{.parent=(from: ast::ScopeId), .child=(to: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], name_in_scope::NameInScope[(name_in_scope::NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(from: ast::ScopeId), .declared=(declared: ast::AnyId)}: name_in_scope::NameInScope)]. */
                                                                                                                         program::Rule::ArrangementRule {
                                                                                                                             description: std::borrow::Cow::from( "name_in_scope::NameInScope(.file=file, .name=name, .scope=to, .declared=declared) :- name_in_scope::NameOccursInScope(.scope=to, .name=(&name_in_scope::NameOccurance{.name=name, .file=file})), not name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=to, .declared=_), inputs::InputScope(.parent=from, .child=to, .file=file), name_in_scope::NameInScope(.file=file, .name=name, .scope=from, .declared=declared)."),
                                                                                                                             arr: ( 63, 1),
                                                                                                                             xform: XFormArrangement::Antijoin {
                                                                                                                                        description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=to, .name=(&name_in_scope::NameOccurance{.name=name, .file=file})), not name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=to, .declared=_)"),
                                                                                                                                        ffun: None,
                                                                                                                                        arrangement: (64,0),
                                                                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                description: std::borrow::Cow::from("arrange name_in_scope::NameOccursInScope(.scope=to, .name=(&name_in_scope::NameOccurance{.name=name, .file=file})), not name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=to, .declared=_) by (to, file)"),
                                                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                {
                                                                                                                                                                    let (ref to, ref name, ref file) = match *<NameOccursInScope>::from_ddvalue_ref(&__v) {
                                                                                                                                                                        NameOccursInScope{scope: ref to, name: ref _0_} => match ((*_0_)).deref() {
                                                                                                                                                                                                                               NameOccurance{name: name, file: file} => ((*to).clone(), (*name).clone(), (*file).clone()),
                                                                                                                                                                                                                               _ => return None
                                                                                                                                                                                                                           },
                                                                                                                                                                        _ => return None
                                                                                                                                                                    };
                                                                                                                                                                    Some(((ddlog_std::tuple2((*to).clone(), (*file).clone())).into_ddvalue(), (ddlog_std::tuple3((*to).clone(), (*name).clone(), (*file).clone())).into_ddvalue()))
                                                                                                                                                                }
                                                                                                                                                                __f},
                                                                                                                                                                next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                   description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=to, .name=(&name_in_scope::NameOccurance{.name=name, .file=file})), not name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=to, .declared=_), inputs::InputScope(.parent=from, .child=to, .file=file)"),
                                                                                                                                                                                   ffun: None,
                                                                                                                                                                                   arrangement: (41,0),
                                                                                                                                                                                   jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                   {
                                                                                                                                                                                       let ddlog_std::tuple3(ref to, ref name, ref file) = *<ddlog_std::tuple3<types__ast::ScopeId, internment::Intern<String>, types__ast::FileId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                       let ref from = match *<types__inputs::InputScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                           types__inputs::InputScope{parent: ref from, child: _, file: _} => (*from).clone(),
                                                                                                                                                                                           _ => return None
                                                                                                                                                                                       };
                                                                                                                                                                                       Some((ddlog_std::tuple4((*to).clone(), (*name).clone(), (*file).clone(), (*from).clone())).into_ddvalue())
                                                                                                                                                                                   }
                                                                                                                                                                                   __f},
                                                                                                                                                                                   next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                           description: std::borrow::Cow::from("arrange name_in_scope::NameOccursInScope(.scope=to, .name=(&name_in_scope::NameOccurance{.name=name, .file=file})), not name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=to, .declared=_), inputs::InputScope(.parent=from, .child=to, .file=file) by (file, name, from)"),
                                                                                                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                           {
                                                                                                                                                                                                               let ddlog_std::tuple4(ref to, ref name, ref file, ref from) = *<ddlog_std::tuple4<types__ast::ScopeId, internment::Intern<String>, types__ast::FileId, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                               Some(((ddlog_std::tuple3((*file).clone(), (*name).clone(), (*from).clone())).into_ddvalue(), (ddlog_std::tuple3((*to).clone(), (*name).clone(), (*file).clone())).into_ddvalue()))
                                                                                                                                                                                                           }
                                                                                                                                                                                                           __f},
                                                                                                                                                                                                           next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                                                              description: std::borrow::Cow::from("name_in_scope::NameOccursInScope(.scope=to, .name=(&name_in_scope::NameOccurance{.name=name, .file=file})), not name_in_scope::ScopeOfDeclName(.file=file, .name=name, .scope=to, .declared=_), inputs::InputScope(.parent=from, .child=to, .file=file), name_in_scope::NameInScope(.file=file, .name=name, .scope=from, .declared=declared)"),
                                                                                                                                                                                                                              ffun: None,
                                                                                                                                                                                                                              arrangement: (62,0),
                                                                                                                                                                                                                              jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                                                              {
                                                                                                                                                                                                                                  let ddlog_std::tuple3(ref to, ref name, ref file) = *<ddlog_std::tuple3<types__ast::ScopeId, internment::Intern<String>, types__ast::FileId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                                                  let ref declared = match *<NameInScope>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                                                      NameInScope{file: _, name: _, scope: _, declared: ref declared} => (*declared).clone(),
                                                                                                                                                                                                                                      _ => return None
                                                                                                                                                                                                                                  };
                                                                                                                                                                                                                                  Some(((NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*to).clone(), declared: (*declared).clone()})).into_ddvalue())
                                                                                                                                                                                                                              }
                                                                                                                                                                                                                              __f},
                                                                                                                                                                                                                              next: Box::new(None)
                                                                                                                                                                                                                          })
                                                                                                                                                                                                       }))
                                                                                                                                                                               })
                                                                                                                                                            }))
                                                                                                                                    }
                                                                                                                         });