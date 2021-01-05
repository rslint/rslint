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
#[ddlog(rename = "outputs::no_unused_labels::LabelUsage")]
pub struct LabelUsage {
    pub stmt: types__ast::StmtId,
    pub label_name: types__ast::Name,
    pub scope: types__ast::ScopeId
}
impl abomonation::Abomonation for LabelUsage{}
impl ::std::fmt::Display for LabelUsage {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            LabelUsage{stmt,label_name,scope} => {
                __formatter.write_str("outputs::no_unused_labels::LabelUsage{")?;
                ::std::fmt::Debug::fmt(stmt, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label_name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for LabelUsage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "outputs::no_unused_labels::NoUnusedLabels")]
pub struct NoUnusedLabels {
    pub stmt_id: types__ast::StmtId,
    pub label_name: types__ast::Spanned<types__ast::Name>
}
impl abomonation::Abomonation for NoUnusedLabels{}
impl ::std::fmt::Display for NoUnusedLabels {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NoUnusedLabels{stmt_id,label_name} => {
                __formatter.write_str("outputs::no_unused_labels::NoUnusedLabels{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label_name, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NoUnusedLabels {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "outputs::no_unused_labels::UsedLabels")]
pub struct UsedLabels {
    pub stmt_id: types__ast::StmtId,
    pub label_name: types__ast::Name
}
impl abomonation::Abomonation for UsedLabels{}
impl ::std::fmt::Display for UsedLabels {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            UsedLabels{stmt_id,label_name} => {
                __formatter.write_str("outputs::no_unused_labels::UsedLabels{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label_name, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for UsedLabels {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub static __Arng_outputs_no_unused_labels___Prefix_1_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                             name: std::borrow::Cow::from(r###"((_: ast::FileId), (_: ast::StmtId), ((ast::Spanned{.data=_0, .span=_}: ast::Spanned{data: ast::Name, span: ast::Span}): ast::Spanned<ast::Name>), (_1: ast::ScopeId)) /*join*/"###),
                                                                                                                                              afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                              {
                                                                                                                                                  let __cloned = __v.clone();
                                                                                                                                                  match < ddlog_std::tuple4<types__ast::FileId, types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>, types__ast::ScopeId>>::from_ddvalue(__v) {
                                                                                                                                                      ddlog_std::tuple4(_, _, types__ast::Spanned{data: ref _0, span: _}, ref _1) => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                      _ => None
                                                                                                                                                  }.map(|x|(x,__cloned))
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              queryable: false
                                                                                                                                          });
pub static __Arng_outputs_no_unused_labels___Prefix_1_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                             name: std::borrow::Cow::from(r###"((_: ast::FileId), (_: ast::StmtId), (_: ast::Spanned<ast::Name>), (_0: ast::ScopeId)) /*join*/"###),
                                                                                                                                              afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                              {
                                                                                                                                                  let __cloned = __v.clone();
                                                                                                                                                  match < ddlog_std::tuple4<types__ast::FileId, types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>, types__ast::ScopeId>>::from_ddvalue(__v) {
                                                                                                                                                      ddlog_std::tuple4(_, _, _, ref _0) => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                                      _ => None
                                                                                                                                                  }.map(|x|(x,__cloned))
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              queryable: false
                                                                                                                                          });
pub static __Arng_outputs_no_unused_labels_LabelUsage_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                              name: std::borrow::Cow::from(r###"(outputs::no_unused_labels::LabelUsage{.stmt=(_: ast::StmtId), .label_name=_0, .scope=(_1: ast::ScopeId)}: outputs::no_unused_labels::LabelUsage) /*semijoin*/"###),
                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  match < LabelUsage>::from_ddvalue(__v) {
                                                                                                                                                      LabelUsage{stmt: _, label_name: ref _0, scope: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                      _ => None
                                                                                                                                                  }
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              distinct: false
                                                                                                                                          });
pub static __Arng_outputs_no_unused_labels_LabelUsage_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                              name: std::borrow::Cow::from(r###"(outputs::no_unused_labels::LabelUsage{.stmt=(_: ast::StmtId), .label_name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId)}: outputs::no_unused_labels::LabelUsage) /*antijoin*/"###),
                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  match < LabelUsage>::from_ddvalue(__v) {
                                                                                                                                                      LabelUsage{stmt: _, label_name: ref _0, scope: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                      _ => None
                                                                                                                                                  }
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              distinct: true
                                                                                                                                          });
pub static __Arng_outputs_no_unused_labels_UsedLabels_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                              name: std::borrow::Cow::from(r###"(outputs::no_unused_labels::UsedLabels{.stmt_id=(_0: ast::StmtId), .label_name=_1}: outputs::no_unused_labels::UsedLabels) /*antijoin*/"###),
                                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                              {
                                                                                                                                                  match < UsedLabels>::from_ddvalue(__v) {
                                                                                                                                                      UsedLabels{stmt_id: ref _0, label_name: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                                      _ => None
                                                                                                                                                  }
                                                                                                                                              }
                                                                                                                                              __f},
                                                                                                                                              distinct: false
                                                                                                                                          });
pub static __Rule_outputs_no_unused_labels___Prefix_1_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_unused_labels::__Prefix_1[((file: ast::FileId), (stmt: ast::StmtId), (name: ast::Spanned<ast::Name>), (body_scope: ast::ScopeId))] :- config::EnableNoUnusedLabels[(config::EnableNoUnusedLabels{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedLabelsConfig>)}: config::EnableNoUnusedLabels)], inputs::Label[(inputs::Label{.stmt_id=(stmt@ (ast::StmtId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::StmtId)), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>), .body_scope=(body_scope: ast::ScopeId)}: inputs::Label)]. */
                                                                                                                                   program::Rule::ArrangementRule {
                                                                                                                                       description: std::borrow::Cow::from( "outputs::no_unused_labels::__Prefix_1[(file, stmt, name, body_scope)] :- config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .name=ddlog_std::Some{.x=name}, .body=_, .body_scope=body_scope)."),
                                                                                                                                       arr: ( 4, 0),
                                                                                                                                       xform: XFormArrangement::Join{
                                                                                                                                                  description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .name=ddlog_std::Some{.x=name}, .body=_, .body_scope=body_scope)"),
                                                                                                                                                  ffun: None,
                                                                                                                                                  arrangement: (41,0),
                                                                                                                                                  jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                  {
                                                                                                                                                      let ref file = match *<types__config::EnableNoUnusedLabels>::from_ddvalue_ref(__v1) {
                                                                                                                                                          types__config::EnableNoUnusedLabels{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      let (ref stmt, ref name, ref body_scope) = match *<types__inputs::Label>::from_ddvalue_ref(__v2) {
                                                                                                                                                          types__inputs::Label{stmt_id: ref stmt, name: ddlog_std::Option::Some{x: ref name}, body: _, body_scope: ref body_scope} => match stmt {
                                                                                                                                                                                                                                                                                          types__ast::StmtId{id: _, file: _} => ((*stmt).clone(), (*name).clone(), (*body_scope).clone()),
                                                                                                                                                                                                                                                                                          _ => return None
                                                                                                                                                                                                                                                                                      },
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      Some((ddlog_std::tuple4((*file).clone(), (*stmt).clone(), (*name).clone(), (*body_scope).clone())).into_ddvalue())
                                                                                                                                                  }
                                                                                                                                                  __f},
                                                                                                                                                  next: Box::new(None)
                                                                                                                                              }
                                                                                                                                   });
pub static __Rule_outputs_no_unused_labels_LabelUsage_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_unused_labels::LabelUsage[(outputs::no_unused_labels::LabelUsage{.stmt=stmt, .label_name=name, .scope=scope}: outputs::no_unused_labels::LabelUsage)] :- config::EnableNoUnusedLabels[(config::EnableNoUnusedLabels{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedLabelsConfig>)}: config::EnableNoUnusedLabels)], inputs::Break[(inputs::Break{.stmt_id=(stmt@ (ast::StmtId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::StmtId)), .label=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::Break)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                                                                                                                                   program::Rule::ArrangementRule {
                                                                                                                                       description: std::borrow::Cow::from( "outputs::no_unused_labels::LabelUsage(.stmt=stmt, .label_name=name, .scope=scope) :- config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Break(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}), inputs::Statement(.id=stmt, .kind=_, .scope=scope, .span=_)."),
                                                                                                                                       arr: ( 4, 0),
                                                                                                                                       xform: XFormArrangement::Join{
                                                                                                                                                  description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Break(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}})"),
                                                                                                                                                  ffun: None,
                                                                                                                                                  arrangement: (14,0),
                                                                                                                                                  jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                  {
                                                                                                                                                      let ref file = match *<types__config::EnableNoUnusedLabels>::from_ddvalue_ref(__v1) {
                                                                                                                                                          types__config::EnableNoUnusedLabels{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      let (ref stmt, ref name) = match *<types__inputs::Break>::from_ddvalue_ref(__v2) {
                                                                                                                                                          types__inputs::Break{stmt_id: ref stmt, label: ddlog_std::Option::Some{x: types__ast::Spanned{data: ref name, span: _}}} => match stmt {
                                                                                                                                                                                                                                                                                          types__ast::StmtId{id: _, file: _} => ((*stmt).clone(), (*name).clone()),
                                                                                                                                                                                                                                                                                          _ => return None
                                                                                                                                                                                                                                                                                      },
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      Some((ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue())
                                                                                                                                                  }
                                                                                                                                                  __f},
                                                                                                                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                          description: std::borrow::Cow::from("arrange config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Break(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}) by (stmt)"),
                                                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                          {
                                                                                                                                                                              let ddlog_std::tuple2(ref stmt, ref name) = *<ddlog_std::tuple2<types__ast::StmtId, internment::Intern<String>>>::from_ddvalue_ref( &__v );
                                                                                                                                                                              Some((((*stmt).clone()).into_ddvalue(), (ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                                          }
                                                                                                                                                                          __f},
                                                                                                                                                                          next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                             description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Break(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}), inputs::Statement(.id=stmt, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                                                                             ffun: None,
                                                                                                                                                                                             arrangement: (47,0),
                                                                                                                                                                                             jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                             {
                                                                                                                                                                                                 let ddlog_std::tuple2(ref stmt, ref name) = *<ddlog_std::tuple2<types__ast::StmtId, internment::Intern<String>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                 let ref scope = match *<types__inputs::Statement>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                     types__inputs::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                                                                     _ => return None
                                                                                                                                                                                                 };
                                                                                                                                                                                                 Some(((LabelUsage{stmt: (*stmt).clone(), label_name: (*name).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                                                                             }
                                                                                                                                                                                             __f},
                                                                                                                                                                                             next: Box::new(None)
                                                                                                                                                                                         })
                                                                                                                                                                      }))
                                                                                                                                              }
                                                                                                                                   });
pub static __Rule_outputs_no_unused_labels_LabelUsage_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_unused_labels::LabelUsage[(outputs::no_unused_labels::LabelUsage{.stmt=stmt, .label_name=name, .scope=scope}: outputs::no_unused_labels::LabelUsage)] :- config::EnableNoUnusedLabels[(config::EnableNoUnusedLabels{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedLabelsConfig>)}: config::EnableNoUnusedLabels)], inputs::Continue[(inputs::Continue{.stmt_id=(stmt@ (ast::StmtId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::StmtId)), .label=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::Continue)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                                                                                                                                   program::Rule::ArrangementRule {
                                                                                                                                       description: std::borrow::Cow::from( "outputs::no_unused_labels::LabelUsage(.stmt=stmt, .label_name=name, .scope=scope) :- config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Continue(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}), inputs::Statement(.id=stmt, .kind=_, .scope=scope, .span=_)."),
                                                                                                                                       arr: ( 4, 0),
                                                                                                                                       xform: XFormArrangement::Join{
                                                                                                                                                  description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Continue(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}})"),
                                                                                                                                                  ffun: None,
                                                                                                                                                  arrangement: (19,0),
                                                                                                                                                  jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                  {
                                                                                                                                                      let ref file = match *<types__config::EnableNoUnusedLabels>::from_ddvalue_ref(__v1) {
                                                                                                                                                          types__config::EnableNoUnusedLabels{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      let (ref stmt, ref name) = match *<types__inputs::Continue>::from_ddvalue_ref(__v2) {
                                                                                                                                                          types__inputs::Continue{stmt_id: ref stmt, label: ddlog_std::Option::Some{x: types__ast::Spanned{data: ref name, span: _}}} => match stmt {
                                                                                                                                                                                                                                                                                             types__ast::StmtId{id: _, file: _} => ((*stmt).clone(), (*name).clone()),
                                                                                                                                                                                                                                                                                             _ => return None
                                                                                                                                                                                                                                                                                         },
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      Some((ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue())
                                                                                                                                                  }
                                                                                                                                                  __f},
                                                                                                                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                          description: std::borrow::Cow::from("arrange config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Continue(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}) by (stmt)"),
                                                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                          {
                                                                                                                                                                              let ddlog_std::tuple2(ref stmt, ref name) = *<ddlog_std::tuple2<types__ast::StmtId, internment::Intern<String>>>::from_ddvalue_ref( &__v );
                                                                                                                                                                              Some((((*stmt).clone()).into_ddvalue(), (ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                                          }
                                                                                                                                                                          __f},
                                                                                                                                                                          next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                             description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Continue(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .label=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}), inputs::Statement(.id=stmt, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                                                                             ffun: None,
                                                                                                                                                                                             arrangement: (47,0),
                                                                                                                                                                                             jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                             {
                                                                                                                                                                                                 let ddlog_std::tuple2(ref stmt, ref name) = *<ddlog_std::tuple2<types__ast::StmtId, internment::Intern<String>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                 let ref scope = match *<types__inputs::Statement>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                     types__inputs::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                                                                     _ => return None
                                                                                                                                                                                                 };
                                                                                                                                                                                                 Some(((LabelUsage{stmt: (*stmt).clone(), label_name: (*name).clone(), scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                                                                             }
                                                                                                                                                                                             __f},
                                                                                                                                                                                             next: Box::new(None)
                                                                                                                                                                                         })
                                                                                                                                                                      }))
                                                                                                                                              }
                                                                                                                                   });
pub static __Rule_scopes_NeedsScopeChildren_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* scopes::NeedsScopeChildren[(scopes::NeedsScopeChildren{.scope=scope}: scopes::NeedsScopeChildren)] :- config::EnableNoUnusedLabels[(config::EnableNoUnusedLabels{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedLabelsConfig>)}: config::EnableNoUnusedLabels)], inputs::Label[(inputs::Label{.stmt_id=(ast::StmtId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::StmtId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>), .body_scope=(scope: ast::ScopeId)}: inputs::Label)], not outputs::no_unused_labels::LabelUsage[(outputs::no_unused_labels::LabelUsage{.stmt=(_: ast::StmtId), .label_name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId)}: outputs::no_unused_labels::LabelUsage)]. */
                                                                                                                         program::Rule::ArrangementRule {
                                                                                                                             description: std::borrow::Cow::from( "scopes::NeedsScopeChildren(.scope=scope) :- config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=ast::StmtId{.id=_, .file=file}, .name=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}, .body=_, .body_scope=scope), not outputs::no_unused_labels::LabelUsage(.stmt=_, .label_name=name, .scope=scope)."),
                                                                                                                             arr: ( 4, 0),
                                                                                                                             xform: XFormArrangement::Join{
                                                                                                                                        description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=ast::StmtId{.id=_, .file=file}, .name=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}, .body=_, .body_scope=scope)"),
                                                                                                                                        ffun: None,
                                                                                                                                        arrangement: (41,1),
                                                                                                                                        jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let ref file = match *<types__config::EnableNoUnusedLabels>::from_ddvalue_ref(__v1) {
                                                                                                                                                types__config::EnableNoUnusedLabels{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            let (ref name, ref scope) = match *<types__inputs::Label>::from_ddvalue_ref(__v2) {
                                                                                                                                                types__inputs::Label{stmt_id: types__ast::StmtId{id: _, file: _}, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: ref name, span: _}}, body: _, body_scope: ref scope} => ((*name).clone(), (*scope).clone()),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some((ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                description: std::borrow::Cow::from("arrange config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=ast::StmtId{.id=_, .file=file}, .name=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}, .body=_, .body_scope=scope) by (name, scope)"),
                                                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                {
                                                                                                                                                                    let ddlog_std::tuple2(ref name, ref scope) = *<ddlog_std::tuple2<internment::Intern<String>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                    Some(((ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), ((*scope).clone()).into_ddvalue()))
                                                                                                                                                                }
                                                                                                                                                                __f},
                                                                                                                                                                next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                                   description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=ast::StmtId{.id=_, .file=file}, .name=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}, .body=_, .body_scope=scope), not outputs::no_unused_labels::LabelUsage(.stmt=_, .label_name=name, .scope=scope)"),
                                                                                                                                                                                   ffun: None,
                                                                                                                                                                                   arrangement: (73,1),
                                                                                                                                                                                   next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                                           description: std::borrow::Cow::from("head of scopes::NeedsScopeChildren(.scope=scope) :- config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=ast::StmtId{.id=_, .file=file}, .name=ddlog_std::Some{.x=ast::Spanned{.data=name, .span=_}}, .body=_, .body_scope=scope), not outputs::no_unused_labels::LabelUsage(.stmt=_, .label_name=name, .scope=scope)."),
                                                                                                                                                                                                           fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                                           {
                                                                                                                                                                                                               let ref scope = *<types__ast::ScopeId>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                               Some(((types__scopes::NeedsScopeChildren{scope: (*scope).clone()})).into_ddvalue())
                                                                                                                                                                                                           }
                                                                                                                                                                                                           __f},
                                                                                                                                                                                                           next: Box::new(None)
                                                                                                                                                                                                       }))
                                                                                                                                                                               })
                                                                                                                                                            }))
                                                                                                                                    }
                                                                                                                         });
pub static __Rule_outputs_no_unused_labels_UsedLabels_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_unused_labels::UsedLabels[(outputs::no_unused_labels::UsedLabels{.stmt_id=stmt, .label_name=(name.data)}: outputs::no_unused_labels::UsedLabels)] :- outputs::no_unused_labels::__Prefix_1[((file: ast::FileId), (stmt: ast::StmtId), (name: ast::Spanned<ast::Name>), (body_scope: ast::ScopeId))], outputs::no_unused_labels::LabelUsage[(outputs::no_unused_labels::LabelUsage{.stmt=(_: ast::StmtId), .label_name=(name.data), .scope=(body_scope: ast::ScopeId)}: outputs::no_unused_labels::LabelUsage)]. */
                                                                                                                                   program::Rule::ArrangementRule {
                                                                                                                                       description: std::borrow::Cow::from( "outputs::no_unused_labels::UsedLabels(.stmt_id=stmt, .label_name=(name.data)) :- outputs::no_unused_labels::__Prefix_1[(file, stmt, name, body_scope)], outputs::no_unused_labels::LabelUsage(.stmt=_, .label_name=(name.data), .scope=body_scope)."),
                                                                                                                                       arr: ( 76, 0),
                                                                                                                                       xform: XFormArrangement::Semijoin{
                                                                                                                                                  description: std::borrow::Cow::from("outputs::no_unused_labels::__Prefix_1[(file, stmt, name, body_scope)], outputs::no_unused_labels::LabelUsage(.stmt=_, .label_name=(name.data), .scope=body_scope)"),
                                                                                                                                                  ffun: None,
                                                                                                                                                  arrangement: (73,0),
                                                                                                                                                  jfun: {fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                  {
                                                                                                                                                      let (ref file, ref stmt, ref name, ref body_scope) = match *<ddlog_std::tuple4<types__ast::FileId, types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>, types__ast::ScopeId>>::from_ddvalue_ref(__v1) {
                                                                                                                                                          ddlog_std::tuple4(ref file, ref stmt, ref name, ref body_scope) => ((*file).clone(), (*stmt).clone(), (*name).clone(), (*body_scope).clone()),
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      Some(((UsedLabels{stmt_id: (*stmt).clone(), label_name: name.data.clone()})).into_ddvalue())
                                                                                                                                                  }
                                                                                                                                                  __f},
                                                                                                                                                  next: Box::new(None)
                                                                                                                                              }
                                                                                                                                   });
pub static __Rule_outputs_no_unused_labels_UsedLabels_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_unused_labels::UsedLabels[(outputs::no_unused_labels::UsedLabels{.stmt_id=stmt, .label_name=(name.data)}: outputs::no_unused_labels::UsedLabels)] :- outputs::no_unused_labels::__Prefix_1[((file: ast::FileId), (stmt: ast::StmtId), (name: ast::Spanned<ast::Name>), (body_scope: ast::ScopeId))], scopes::ScopeFamily[(scopes::ScopeFamily{.parent=(body_scope: ast::ScopeId), .child=(child_scope: ast::ScopeId)}: scopes::ScopeFamily)], outputs::no_unused_labels::LabelUsage[(outputs::no_unused_labels::LabelUsage{.stmt=(_: ast::StmtId), .label_name=(name.data), .scope=(child_scope: ast::ScopeId)}: outputs::no_unused_labels::LabelUsage)]. */
                                                                                                                                   program::Rule::ArrangementRule {
                                                                                                                                       description: std::borrow::Cow::from( "outputs::no_unused_labels::UsedLabels(.stmt_id=stmt, .label_name=(name.data)) :- outputs::no_unused_labels::__Prefix_1[(file, stmt, name, body_scope)], scopes::ScopeFamily(.parent=body_scope, .child=child_scope), outputs::no_unused_labels::LabelUsage(.stmt=_, .label_name=(name.data), .scope=child_scope)."),
                                                                                                                                       arr: ( 76, 1),
                                                                                                                                       xform: XFormArrangement::Join{
                                                                                                                                                  description: std::borrow::Cow::from("outputs::no_unused_labels::__Prefix_1[(file, stmt, name, body_scope)], scopes::ScopeFamily(.parent=body_scope, .child=child_scope)"),
                                                                                                                                                  ffun: None,
                                                                                                                                                  arrangement: (84,0),
                                                                                                                                                  jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                  {
                                                                                                                                                      let (ref file, ref stmt, ref name, ref body_scope) = match *<ddlog_std::tuple4<types__ast::FileId, types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>, types__ast::ScopeId>>::from_ddvalue_ref(__v1) {
                                                                                                                                                          ddlog_std::tuple4(ref file, ref stmt, ref name, ref body_scope) => ((*file).clone(), (*stmt).clone(), (*name).clone(), (*body_scope).clone()),
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      let ref child_scope = match *<types__scopes::ScopeFamily>::from_ddvalue_ref(__v2) {
                                                                                                                                                          types__scopes::ScopeFamily{parent: _, child: ref child_scope} => (*child_scope).clone(),
                                                                                                                                                          _ => return None
                                                                                                                                                      };
                                                                                                                                                      Some((ddlog_std::tuple3((*stmt).clone(), (*name).clone(), (*child_scope).clone())).into_ddvalue())
                                                                                                                                                  }
                                                                                                                                                  __f},
                                                                                                                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                          description: std::borrow::Cow::from("arrange outputs::no_unused_labels::__Prefix_1[(file, stmt, name, body_scope)], scopes::ScopeFamily(.parent=body_scope, .child=child_scope) by ((name.data), child_scope)"),
                                                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                          {
                                                                                                                                                                              let ddlog_std::tuple3(ref stmt, ref name, ref child_scope) = *<ddlog_std::tuple3<types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                              Some(((ddlog_std::tuple2(name.data.clone(), (*child_scope).clone())).into_ddvalue(), (ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                                          }
                                                                                                                                                                          __f},
                                                                                                                                                                          next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                             description: std::borrow::Cow::from("outputs::no_unused_labels::__Prefix_1[(file, stmt, name, body_scope)], scopes::ScopeFamily(.parent=body_scope, .child=child_scope), outputs::no_unused_labels::LabelUsage(.stmt=_, .label_name=(name.data), .scope=child_scope)"),
                                                                                                                                                                                             ffun: None,
                                                                                                                                                                                             arrangement: (73,0),
                                                                                                                                                                                             jfun: {fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                             {
                                                                                                                                                                                                 let ddlog_std::tuple2(ref stmt, ref name) = *<ddlog_std::tuple2<types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                 Some(((UsedLabels{stmt_id: (*stmt).clone(), label_name: name.data.clone()})).into_ddvalue())
                                                                                                                                                                                             }
                                                                                                                                                                                             __f},
                                                                                                                                                                                             next: Box::new(None)
                                                                                                                                                                                         })
                                                                                                                                                                      }))
                                                                                                                                              }
                                                                                                                                   });
pub static __Rule_outputs_no_unused_labels_NoUnusedLabels_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_unused_labels::NoUnusedLabels[(outputs::no_unused_labels::NoUnusedLabels{.stmt_id=stmt, .label_name=name}: outputs::no_unused_labels::NoUnusedLabels)] :- config::EnableNoUnusedLabels[(config::EnableNoUnusedLabels{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedLabelsConfig>)}: config::EnableNoUnusedLabels)], inputs::Label[(inputs::Label{.stmt_id=(stmt@ (ast::StmtId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::StmtId)), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>), .body_scope=(_: ast::ScopeId)}: inputs::Label)], not outputs::no_unused_labels::UsedLabels[(outputs::no_unused_labels::UsedLabels{.stmt_id=(stmt: ast::StmtId), .label_name=(name.data)}: outputs::no_unused_labels::UsedLabels)]. */
                                                                                                                                       program::Rule::ArrangementRule {
                                                                                                                                           description: std::borrow::Cow::from( "outputs::no_unused_labels::NoUnusedLabels(.stmt_id=stmt, .label_name=name) :- config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .name=ddlog_std::Some{.x=name}, .body=_, .body_scope=_), not outputs::no_unused_labels::UsedLabels(.stmt_id=stmt, .label_name=(name.data))."),
                                                                                                                                           arr: ( 4, 0),
                                                                                                                                           xform: XFormArrangement::Join{
                                                                                                                                                      description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .name=ddlog_std::Some{.x=name}, .body=_, .body_scope=_)"),
                                                                                                                                                      ffun: None,
                                                                                                                                                      arrangement: (41,0),
                                                                                                                                                      jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                      {
                                                                                                                                                          let ref file = match *<types__config::EnableNoUnusedLabels>::from_ddvalue_ref(__v1) {
                                                                                                                                                              types__config::EnableNoUnusedLabels{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                              _ => return None
                                                                                                                                                          };
                                                                                                                                                          let (ref stmt, ref name) = match *<types__inputs::Label>::from_ddvalue_ref(__v2) {
                                                                                                                                                              types__inputs::Label{stmt_id: ref stmt, name: ddlog_std::Option::Some{x: ref name}, body: _, body_scope: _} => match stmt {
                                                                                                                                                                                                                                                                                 types__ast::StmtId{id: _, file: _} => ((*stmt).clone(), (*name).clone()),
                                                                                                                                                                                                                                                                                 _ => return None
                                                                                                                                                                                                                                                                             },
                                                                                                                                                              _ => return None
                                                                                                                                                          };
                                                                                                                                                          Some((ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue())
                                                                                                                                                      }
                                                                                                                                                      __f},
                                                                                                                                                      next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                              description: std::borrow::Cow::from("arrange config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .name=ddlog_std::Some{.x=name}, .body=_, .body_scope=_) by (stmt, (name.data))"),
                                                                                                                                                                              afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                              {
                                                                                                                                                                                  let ddlog_std::tuple2(ref stmt, ref name) = *<ddlog_std::tuple2<types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                  Some(((ddlog_std::tuple2((*stmt).clone(), name.data.clone())).into_ddvalue(), (ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                                              }
                                                                                                                                                                              __f},
                                                                                                                                                                              next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                                                 description: std::borrow::Cow::from("config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .name=ddlog_std::Some{.x=name}, .body=_, .body_scope=_), not outputs::no_unused_labels::UsedLabels(.stmt_id=stmt, .label_name=(name.data))"),
                                                                                                                                                                                                 ffun: None,
                                                                                                                                                                                                 arrangement: (75,0),
                                                                                                                                                                                                 next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                                                         description: std::borrow::Cow::from("head of outputs::no_unused_labels::NoUnusedLabels(.stmt_id=stmt, .label_name=name) :- config::EnableNoUnusedLabels(.file=file, .config=_), inputs::Label(.stmt_id=(stmt@ ast::StmtId{.id=_, .file=file}), .name=ddlog_std::Some{.x=name}, .body=_, .body_scope=_), not outputs::no_unused_labels::UsedLabels(.stmt_id=stmt, .label_name=(name.data))."),
                                                                                                                                                                                                                         fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                                                         {
                                                                                                                                                                                                                             let ddlog_std::tuple2(ref stmt, ref name) = *<ddlog_std::tuple2<types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                             Some(((NoUnusedLabels{stmt_id: (*stmt).clone(), label_name: (*name).clone()})).into_ddvalue())
                                                                                                                                                                                                                         }
                                                                                                                                                                                                                         __f},
                                                                                                                                                                                                                         next: Box::new(None)
                                                                                                                                                                                                                     }))
                                                                                                                                                                                             })
                                                                                                                                                                          }))
                                                                                                                                                  }
                                                                                                                                       });