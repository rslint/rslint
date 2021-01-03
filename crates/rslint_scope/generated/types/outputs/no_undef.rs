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
#[ddlog(rename = "outputs::no_undef::ChainedWith")]
pub struct ChainedWith {
    pub object: types__ast::ExprId,
    pub property: types__ast::ExprId
}
impl abomonation::Abomonation for ChainedWith{}
impl ::std::fmt::Display for ChainedWith {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ChainedWith{object,property} => {
                __formatter.write_str("outputs::no_undef::ChainedWith{")?;
                ::std::fmt::Debug::fmt(object, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(property, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ChainedWith {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "outputs::no_undef::NoUndef")]
pub struct NoUndef {
    pub name: types__ast::Name,
    pub scope: types__ast::ScopeId,
    pub span: types__ast::Span
}
impl abomonation::Abomonation for NoUndef{}
impl ::std::fmt::Display for NoUndef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NoUndef{name,scope,span} => {
                __formatter.write_str("outputs::no_undef::NoUndef{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NoUndef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub static __Arng_outputs_no_undef_ChainedWith_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                      name: std::borrow::Cow::from(r###"(outputs::no_undef::ChainedWith{.object=(_: ast::ExprId), .property=(_0: ast::ExprId)}: outputs::no_undef::ChainedWith) /*join*/"###),
                                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                       {
                                                                                                                                           let __cloned = __v.clone();
                                                                                                                                           match < ChainedWith>::from_ddvalue(__v) {
                                                                                                                                               ChainedWith{object: _, property: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                               _ => None
                                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                                       }
                                                                                                                                       __f},
                                                                                                                                       queryable: false
                                                                                                                                   });
pub static __Arng_outputs_no_undef_ChainedWith_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                      name: std::borrow::Cow::from(r###"(outputs::no_undef::ChainedWith{.object=(_0: ast::ExprId), .property=(_: ast::ExprId)}: outputs::no_undef::ChainedWith) /*join*/"###),
                                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                       {
                                                                                                                                           let __cloned = __v.clone();
                                                                                                                                           match < ChainedWith>::from_ddvalue(__v) {
                                                                                                                                               ChainedWith{object: ref _0, property: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                               _ => None
                                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                                       }
                                                                                                                                       __f},
                                                                                                                                       queryable: false
                                                                                                                                   });
pub static __Arng_outputs_no_undef_ChainedWith_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                       name: std::borrow::Cow::from(r###"(outputs::no_undef::ChainedWith{.object=(_: ast::ExprId), .property=(_0: ast::ExprId)}: outputs::no_undef::ChainedWith) /*antijoin*/"###),
                                                                                                                                       fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                       {
                                                                                                                                           match < ChainedWith>::from_ddvalue(__v) {
                                                                                                                                               ChainedWith{object: _, property: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                               _ => None
                                                                                                                                           }
                                                                                                                                       }
                                                                                                                                       __f},
                                                                                                                                       distinct: true
                                                                                                                                   });
pub static __Rule_outputs_no_undef_ChainedWith_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_undef::ChainedWith[(outputs::no_undef::ChainedWith{.object=object, .property=property}: outputs::no_undef::ChainedWith)] :- config::EnableNoUndef[(config::EnableNoUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUndefConfig>)}: config::EnableNoUndef)], inputs::BracketAccess[(inputs::BracketAccess{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::ExprId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(property: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess)]. */
                                                                                                                            program::Rule::ArrangementRule {
                                                                                                                                description: std::borrow::Cow::from( "outputs::no_undef::ChainedWith(.object=object, .property=property) :- config::EnableNoUndef(.file=file, .config=_), inputs::BracketAccess(.expr_id=ast::ExprId{.id=_, .file=file}, .object=ddlog_std::Some{.x=object}, .prop=ddlog_std::Some{.x=property})."),
                                                                                                                                arr: ( 3, 0),
                                                                                                                                xform: XFormArrangement::Join{
                                                                                                                                           description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::BracketAccess(.expr_id=ast::ExprId{.id=_, .file=file}, .object=ddlog_std::Some{.x=object}, .prop=ddlog_std::Some{.x=property})"),
                                                                                                                                           ffun: None,
                                                                                                                                           arrangement: (13,0),
                                                                                                                                           jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                           {
                                                                                                                                               let ref file = match *<types__config::EnableNoUndef>::from_ddvalue_ref(__v1) {
                                                                                                                                                   types__config::EnableNoUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                   _ => return None
                                                                                                                                               };
                                                                                                                                               let (ref object, ref property) = match *<types__inputs::BracketAccess>::from_ddvalue_ref(__v2) {
                                                                                                                                                   types__inputs::BracketAccess{expr_id: types__ast::ExprId{id: _, file: _}, object: ddlog_std::Option::Some{x: ref object}, prop: ddlog_std::Option::Some{x: ref property}} => ((*object).clone(), (*property).clone()),
                                                                                                                                                   _ => return None
                                                                                                                                               };
                                                                                                                                               Some(((ChainedWith{object: (*object).clone(), property: (*property).clone()})).into_ddvalue())
                                                                                                                                           }
                                                                                                                                           __f},
                                                                                                                                           next: Box::new(None)
                                                                                                                                       }
                                                                                                                            });
pub static __Rule_outputs_no_undef_ChainedWith_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_undef::ChainedWith[(outputs::no_undef::ChainedWith{.object=object, .property=property}: outputs::no_undef::ChainedWith)] :- config::EnableNoUndef[(config::EnableNoUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUndefConfig>)}: config::EnableNoUndef)], inputs::DotAccess[(inputs::DotAccess{.expr_id=(property@ (ast::ExprId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::ExprId)), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess)]. */
                                                                                                                            program::Rule::ArrangementRule {
                                                                                                                                description: std::borrow::Cow::from( "outputs::no_undef::ChainedWith(.object=object, .property=property) :- config::EnableNoUndef(.file=file, .config=_), inputs::DotAccess(.expr_id=(property@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .prop=_)."),
                                                                                                                                arr: ( 3, 0),
                                                                                                                                xform: XFormArrangement::Join{
                                                                                                                                           description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::DotAccess(.expr_id=(property@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .prop=_)"),
                                                                                                                                           ffun: None,
                                                                                                                                           arrangement: (21,0),
                                                                                                                                           jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                           {
                                                                                                                                               let ref file = match *<types__config::EnableNoUndef>::from_ddvalue_ref(__v1) {
                                                                                                                                                   types__config::EnableNoUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                   _ => return None
                                                                                                                                               };
                                                                                                                                               let (ref property, ref object) = match *<types__inputs::DotAccess>::from_ddvalue_ref(__v2) {
                                                                                                                                                   types__inputs::DotAccess{expr_id: ref property, object: ddlog_std::Option::Some{x: ref object}, prop: _} => match property {
                                                                                                                                                                                                                                                                   types__ast::ExprId{id: _, file: _} => ((*property).clone(), (*object).clone()),
                                                                                                                                                                                                                                                                   _ => return None
                                                                                                                                                                                                                                                               },
                                                                                                                                                   _ => return None
                                                                                                                                               };
                                                                                                                                               Some(((ChainedWith{object: (*object).clone(), property: (*property).clone()})).into_ddvalue())
                                                                                                                                           }
                                                                                                                                           __f},
                                                                                                                                           next: Box::new(None)
                                                                                                                                       }
                                                                                                                            });
pub static __Rule_outputs_no_undef_ChainedWith_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_undef::ChainedWith[(outputs::no_undef::ChainedWith{.object=object, .property=property}: outputs::no_undef::ChainedWith)] :- outputs::no_undef::ChainedWith[(outputs::no_undef::ChainedWith{.object=(object: ast::ExprId), .property=(interum: ast::ExprId)}: outputs::no_undef::ChainedWith)], outputs::no_undef::ChainedWith[(outputs::no_undef::ChainedWith{.object=(interum: ast::ExprId), .property=(property: ast::ExprId)}: outputs::no_undef::ChainedWith)]. */
                                                                                                                            program::Rule::ArrangementRule {
                                                                                                                                description: std::borrow::Cow::from( "outputs::no_undef::ChainedWith(.object=object, .property=property) :- outputs::no_undef::ChainedWith(.object=object, .property=interum), outputs::no_undef::ChainedWith(.object=interum, .property=property)."),
                                                                                                                                arr: ( 70, 0),
                                                                                                                                xform: XFormArrangement::Join{
                                                                                                                                           description: std::borrow::Cow::from("outputs::no_undef::ChainedWith(.object=object, .property=interum), outputs::no_undef::ChainedWith(.object=interum, .property=property)"),
                                                                                                                                           ffun: None,
                                                                                                                                           arrangement: (70,1),
                                                                                                                                           jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                           {
                                                                                                                                               let (ref object, ref interum) = match *<ChainedWith>::from_ddvalue_ref(__v1) {
                                                                                                                                                   ChainedWith{object: ref object, property: ref interum} => ((*object).clone(), (*interum).clone()),
                                                                                                                                                   _ => return None
                                                                                                                                               };
                                                                                                                                               let ref property = match *<ChainedWith>::from_ddvalue_ref(__v2) {
                                                                                                                                                   ChainedWith{object: _, property: ref property} => (*property).clone(),
                                                                                                                                                   _ => return None
                                                                                                                                               };
                                                                                                                                               Some(((ChainedWith{object: (*object).clone(), property: (*property).clone()})).into_ddvalue())
                                                                                                                                           }
                                                                                                                                           __f},
                                                                                                                                           next: Box::new(None)
                                                                                                                                       }
                                                                                                                            });
pub static __Rule_outputs_no_undef_NoUndef_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_undef::NoUndef[(outputs::no_undef::NoUndef{.name=name, .scope=scope, .span=span}: outputs::no_undef::NoUndef)] :- config::EnableNoUndef[(config::EnableNoUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUndefConfig>)}: config::EnableNoUndef)], inputs::NameRef[(inputs::NameRef{.expr_id=(expr@ (ast::ExprId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::ExprId)), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not outputs::no_typeof_undef::WithinTypeofExpr[(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId)}: outputs::no_typeof_undef::WithinTypeofExpr)], not outputs::no_undef::ChainedWith[(outputs::no_undef::ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId)}: outputs::no_undef::ChainedWith)], not name_in_scope::NameInScope[(name_in_scope::NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope)]. */
                                                                                                                        program::Rule::ArrangementRule {
                                                                                                                            description: std::borrow::Cow::from( "outputs::no_undef::NoUndef(.name=name, .scope=scope, .span=span) :- config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span), not outputs::no_typeof_undef::WithinTypeofExpr(.type_of=_, .expr=expr), not outputs::no_undef::ChainedWith(.object=_, .property=expr), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)."),
                                                                                                                            arr: ( 3, 0),
                                                                                                                            xform: XFormArrangement::Join{
                                                                                                                                       description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name)"),
                                                                                                                                       ffun: None,
                                                                                                                                       arrangement: (43,1),
                                                                                                                                       jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                       {
                                                                                                                                           let ref file = match *<types__config::EnableNoUndef>::from_ddvalue_ref(__v1) {
                                                                                                                                               types__config::EnableNoUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                               _ => return None
                                                                                                                                           };
                                                                                                                                           let (ref expr, ref name) = match *<types__inputs::NameRef>::from_ddvalue_ref(__v2) {
                                                                                                                                               types__inputs::NameRef{expr_id: ref expr, value: ref name} => match expr {
                                                                                                                                                                                                                 types__ast::ExprId{id: _, file: _} => ((*expr).clone(), (*name).clone()),
                                                                                                                                                                                                                 _ => return None
                                                                                                                                                                                                             },
                                                                                                                                               _ => return None
                                                                                                                                           };
                                                                                                                                           Some((ddlog_std::tuple2((*expr).clone(), (*name).clone())).into_ddvalue())
                                                                                                                                       }
                                                                                                                                       __f},
                                                                                                                                       next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                               description: std::borrow::Cow::from("arrange config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name) by (expr)"),
                                                                                                                                                               afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                               {
                                                                                                                                                                   let ddlog_std::tuple2(ref expr, ref name) = *<ddlog_std::tuple2<types__ast::ExprId, internment::Intern<String>>>::from_ddvalue_ref( &__v );
                                                                                                                                                                   Some((((*expr).clone()).into_ddvalue(), (ddlog_std::tuple2((*expr).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                               }
                                                                                                                                                               __f},
                                                                                                                                                               next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                  description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span)"),
                                                                                                                                                                                  ffun: None,
                                                                                                                                                                                  arrangement: (27,3),
                                                                                                                                                                                  jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                  {
                                                                                                                                                                                      let ddlog_std::tuple2(ref expr, ref name) = *<ddlog_std::tuple2<types__ast::ExprId, internment::Intern<String>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                      let (ref scope, ref span) = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                          types__inputs::Expression{id: _, kind: types__ast::ExprKind::ExprNameRef{}, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                                                                                                                                                          _ => return None
                                                                                                                                                                                      };
                                                                                                                                                                                      Some((ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue())
                                                                                                                                                                                  }
                                                                                                                                                                                  __f},
                                                                                                                                                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                          description: std::borrow::Cow::from("arrange config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span) by (expr)"),
                                                                                                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                          {
                                                                                                                                                                                                              let ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) = *<ddlog_std::tuple4<types__ast::ExprId, internment::Intern<String>, types__ast::ScopeId, types__ast::Span>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                              Some((((*expr).clone()).into_ddvalue(), (ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                                                                                                                                                          }
                                                                                                                                                                                                          __f},
                                                                                                                                                                                                          next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                                                                             description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span), not outputs::no_typeof_undef::WithinTypeofExpr(.type_of=_, .expr=expr)"),
                                                                                                                                                                                                                             ffun: None,
                                                                                                                                                                                                                             arrangement: (69,1),
                                                                                                                                                                                                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                                                                     description: std::borrow::Cow::from("arrange config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span), not outputs::no_typeof_undef::WithinTypeofExpr(.type_of=_, .expr=expr) by (expr)"),
                                                                                                                                                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                                                                     {
                                                                                                                                                                                                                                                         let ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) = *<ddlog_std::tuple4<types__ast::ExprId, internment::Intern<String>, types__ast::ScopeId, types__ast::Span>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                                                         Some((((*expr).clone()).into_ddvalue(), (ddlog_std::tuple3((*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                                                                                                                                                                                                     }
                                                                                                                                                                                                                                                     __f},
                                                                                                                                                                                                                                                     next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                                                                                                                        description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span), not outputs::no_typeof_undef::WithinTypeofExpr(.type_of=_, .expr=expr), not outputs::no_undef::ChainedWith(.object=_, .property=expr)"),
                                                                                                                                                                                                                                                                        ffun: None,
                                                                                                                                                                                                                                                                        arrangement: (70,2),
                                                                                                                                                                                                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                                                                                                                description: std::borrow::Cow::from("arrange config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span), not outputs::no_typeof_undef::WithinTypeofExpr(.type_of=_, .expr=expr), not outputs::no_undef::ChainedWith(.object=_, .property=expr) by (name, scope)"),
                                                                                                                                                                                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                                                                                                                {
                                                                                                                                                                                                                                                                                                    let ddlog_std::tuple3(ref name, ref scope, ref span) = *<ddlog_std::tuple3<internment::Intern<String>, types__ast::ScopeId, types__ast::Span>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                                                                                                    Some(((ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), (ddlog_std::tuple3((*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                                                                                                                                                                                                                                                }
                                                                                                                                                                                                                                                                                                __f},
                                                                                                                                                                                                                                                                                                next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                                                                                                                                                                   description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span), not outputs::no_typeof_undef::WithinTypeofExpr(.type_of=_, .expr=expr), not outputs::no_undef::ChainedWith(.object=_, .property=expr), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)"),
                                                                                                                                                                                                                                                                                                                   ffun: None,
                                                                                                                                                                                                                                                                                                                   arrangement: (61,1),
                                                                                                                                                                                                                                                                                                                   next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                                                                                                                                                                           description: std::borrow::Cow::from("head of outputs::no_undef::NoUndef(.name=name, .scope=scope, .span=span) :- config::EnableNoUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .value=name), inputs::Expression(.id=expr, .kind=ast::ExprNameRef{}, .scope=scope, .span=span), not outputs::no_typeof_undef::WithinTypeofExpr(.type_of=_, .expr=expr), not outputs::no_undef::ChainedWith(.object=_, .property=expr), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)."),
                                                                                                                                                                                                                                                                                                                                           fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                                                                                                                                                                           {
                                                                                                                                                                                                                                                                                                                                               let ddlog_std::tuple3(ref name, ref scope, ref span) = *<ddlog_std::tuple3<internment::Intern<String>, types__ast::ScopeId, types__ast::Span>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                                                                                                                                               Some(((NoUndef{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone()})).into_ddvalue())
                                                                                                                                                                                                                                                                                                                                           }
                                                                                                                                                                                                                                                                                                                                           __f},
                                                                                                                                                                                                                                                                                                                                           next: Box::new(None)
                                                                                                                                                                                                                                                                                                                                       }))
                                                                                                                                                                                                                                                                                                               })
                                                                                                                                                                                                                                                                                            }))
                                                                                                                                                                                                                                                                    })
                                                                                                                                                                                                                                                 }))
                                                                                                                                                                                                                         })
                                                                                                                                                                                                      }))
                                                                                                                                                                              })
                                                                                                                                                           }))
                                                                                                                                   }
                                                                                                                        });
pub static __Rule_outputs_no_undef_NoUndef_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_undef::NoUndef[(outputs::no_undef::NoUndef{.name=name, .scope=scope, .span=span}: outputs::no_undef::NoUndef)] :- config::EnableNoUndef[(config::EnableNoUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUndefConfig>)}: config::EnableNoUndef)], inputs::Assign[(inputs::Assign{.expr_id=(expr@ (ast::ExprId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::ExprId)), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var), not name_in_scope::NameInScope[(name_in_scope::NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope)]. */
                                                                                                                        program::Rule::ArrangementRule {
                                                                                                                            description: std::borrow::Cow::from( "outputs::no_undef::NoUndef(.name=name, .scope=scope, .span=span) :- config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_), inputs::Expression(.id=expr, .kind=_, .scope=scope, .span=_), var bound_var = FlatMap((ast::bound_vars(pat))), (ast::Spanned{.data=var name, .span=var span} = bound_var), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)."),
                                                                                                                            arr: ( 3, 0),
                                                                                                                            xform: XFormArrangement::Join{
                                                                                                                                       description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_)"),
                                                                                                                                       ffun: None,
                                                                                                                                       arrangement: (10,1),
                                                                                                                                       jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                       {
                                                                                                                                           let ref file = match *<types__config::EnableNoUndef>::from_ddvalue_ref(__v1) {
                                                                                                                                               types__config::EnableNoUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                               _ => return None
                                                                                                                                           };
                                                                                                                                           let (ref expr, ref pat) = match *<types__inputs::Assign>::from_ddvalue_ref(__v2) {
                                                                                                                                               types__inputs::Assign{expr_id: ref expr, lhs: ddlog_std::Option::Some{x: ddlog_std::Either::Left{l: ref pat}}, rhs: _, op: _} => match expr {
                                                                                                                                                                                                                                                                                    types__ast::ExprId{id: _, file: _} => ((*expr).clone(), (*pat).clone()),
                                                                                                                                                                                                                                                                                    _ => return None
                                                                                                                                                                                                                                                                                },
                                                                                                                                               _ => return None
                                                                                                                                           };
                                                                                                                                           Some((ddlog_std::tuple2((*expr).clone(), (*pat).clone())).into_ddvalue())
                                                                                                                                       }
                                                                                                                                       __f},
                                                                                                                                       next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                               description: std::borrow::Cow::from("arrange config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_) by (expr)"),
                                                                                                                                                               afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                               {
                                                                                                                                                                   let ddlog_std::tuple2(ref expr, ref pat) = *<ddlog_std::tuple2<types__ast::ExprId, internment::Intern<types__ast::Pattern>>>::from_ddvalue_ref( &__v );
                                                                                                                                                                   Some((((*expr).clone()).into_ddvalue(), ((*pat).clone()).into_ddvalue()))
                                                                                                                                                               }
                                                                                                                                                               __f},
                                                                                                                                                               next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                  description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_), inputs::Expression(.id=expr, .kind=_, .scope=scope, .span=_)"),
                                                                                                                                                                                  ffun: None,
                                                                                                                                                                                  arrangement: (27,0),
                                                                                                                                                                                  jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                  {
                                                                                                                                                                                      let ref pat = *<internment::Intern<types__ast::Pattern>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                      let ref scope = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                          types__inputs::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                                                          _ => return None
                                                                                                                                                                                      };
                                                                                                                                                                                      Some((ddlog_std::tuple2((*pat).clone(), (*scope).clone())).into_ddvalue())
                                                                                                                                                                                  }
                                                                                                                                                                                  __f},
                                                                                                                                                                                  next: Box::new(Some(XFormCollection::FlatMap{
                                                                                                                                                                                                          description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_), inputs::Expression(.id=expr, .kind=_, .scope=scope, .span=_), var bound_var = FlatMap((ast::bound_vars(pat)))"),
                                                                                                                                                                                                          fmfun: {fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                                                                                                          {
                                                                                                                                                                                                              let ddlog_std::tuple2(ref pat, ref scope) = *<ddlog_std::tuple2<internment::Intern<types__ast::Pattern>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                              let __flattened = types__ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                                                                                                                                                              let scope = (*scope).clone();
                                                                                                                                                                                                              Some(Box::new(__flattened.into_iter().map(move |bound_var|(ddlog_std::tuple2(bound_var.clone(), scope.clone())).into_ddvalue())))
                                                                                                                                                                                                          }
                                                                                                                                                                                                          __f},
                                                                                                                                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                                                  description: std::borrow::Cow::from("arrange config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_), inputs::Expression(.id=expr, .kind=_, .scope=scope, .span=_), var bound_var = FlatMap((ast::bound_vars(pat))) by (name, scope)"),
                                                                                                                                                                                                                                  afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                                                  {
                                                                                                                                                                                                                                      let ddlog_std::tuple2(ref bound_var, ref scope) = *<ddlog_std::tuple2<types__ast::Spanned<internment::Intern<String>>, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                                      let (ref name, ref span): (internment::Intern<String>, types__ast::Span) = match (*bound_var).clone() {
                                                                                                                                                                                                                                          types__ast::Spanned{data: name, span: span} => (name, span),
                                                                                                                                                                                                                                          _ => return None
                                                                                                                                                                                                                                      };
                                                                                                                                                                                                                                      Some(((ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), (ddlog_std::tuple3((*scope).clone(), (*name).clone(), (*span).clone())).into_ddvalue()))
                                                                                                                                                                                                                                  }
                                                                                                                                                                                                                                  __f},
                                                                                                                                                                                                                                  next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                                                                                                     description: std::borrow::Cow::from("config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_), inputs::Expression(.id=expr, .kind=_, .scope=scope, .span=_), var bound_var = FlatMap((ast::bound_vars(pat))), (ast::Spanned{.data=var name, .span=var span} = bound_var), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)"),
                                                                                                                                                                                                                                                     ffun: None,
                                                                                                                                                                                                                                                     arrangement: (61,1),
                                                                                                                                                                                                                                                     next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                                                                                                             description: std::borrow::Cow::from("head of outputs::no_undef::NoUndef(.name=name, .scope=scope, .span=span) :- config::EnableNoUndef(.file=file, .config=_), inputs::Assign(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .lhs=ddlog_std::Some{.x=ddlog_std::Left{.l=pat}}, .rhs=_, .op=_), inputs::Expression(.id=expr, .kind=_, .scope=scope, .span=_), var bound_var = FlatMap((ast::bound_vars(pat))), (ast::Spanned{.data=var name, .span=var span} = bound_var), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)."),
                                                                                                                                                                                                                                                                             fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                                                                                                             {
                                                                                                                                                                                                                                                                                 let ddlog_std::tuple3(ref scope, ref name, ref span) = *<ddlog_std::tuple3<types__ast::ScopeId, internment::Intern<String>, types__ast::Span>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                                                                                 Some(((NoUndef{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone()})).into_ddvalue())
                                                                                                                                                                                                                                                                             }
                                                                                                                                                                                                                                                                             __f},
                                                                                                                                                                                                                                                                             next: Box::new(None)
                                                                                                                                                                                                                                                                         }))
                                                                                                                                                                                                                                                 })
                                                                                                                                                                                                                              }))
                                                                                                                                                                                                      }))
                                                                                                                                                                              })
                                                                                                                                                           }))
                                                                                                                                   }
                                                                                                                        });