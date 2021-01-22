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
#[ddlog(rename = "outputs::no_typeof_undef::NeedsWithinTypeofExpr")]
pub struct NeedsWithinTypeofExpr {
    pub file: types__ast::FileId
}
impl abomonation::Abomonation for NeedsWithinTypeofExpr{}
impl ::std::fmt::Display for NeedsWithinTypeofExpr {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NeedsWithinTypeofExpr{file} => {
                __formatter.write_str("outputs::no_typeof_undef::NeedsWithinTypeofExpr{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NeedsWithinTypeofExpr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "outputs::no_typeof_undef::NoTypeofUndef")]
pub struct NoTypeofUndef {
    pub whole_expr: types__ast::ExprId,
    pub undefined_expr: types__ast::ExprId
}
impl abomonation::Abomonation for NoTypeofUndef{}
impl ::std::fmt::Display for NoTypeofUndef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NoTypeofUndef{whole_expr,undefined_expr} => {
                __formatter.write_str("outputs::no_typeof_undef::NoTypeofUndef{")?;
                ::std::fmt::Debug::fmt(whole_expr, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(undefined_expr, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NoTypeofUndef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "outputs::no_typeof_undef::WithinTypeofExpr")]
pub struct WithinTypeofExpr {
    pub type_of: types__ast::ExprId,
    pub expr: types__ast::ExprId
}
impl abomonation::Abomonation for WithinTypeofExpr{}
impl ::std::fmt::Display for WithinTypeofExpr {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            WithinTypeofExpr{type_of,expr} => {
                __formatter.write_str("outputs::no_typeof_undef::WithinTypeofExpr{")?;
                ::std::fmt::Debug::fmt(type_of, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for WithinTypeofExpr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub static __Arng_outputs_no_typeof_undef_NeedsWithinTypeofExpr_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                                       name: std::borrow::Cow::from(r###"(outputs::no_typeof_undef::NeedsWithinTypeofExpr{.file=(_0: ast::FileId)}: outputs::no_typeof_undef::NeedsWithinTypeofExpr) /*join*/"###),
                                                                                                                                                        afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                        {
                                                                                                                                                            let __cloned = __v.clone();
                                                                                                                                                            match <NeedsWithinTypeofExpr>::from_ddvalue(__v) {
                                                                                                                                                                NeedsWithinTypeofExpr{file: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                                                _ => None
                                                                                                                                                            }.map(|x|(x,__cloned))
                                                                                                                                                        }
                                                                                                                                                        __f},
                                                                                                                                                        queryable: false
                                                                                                                                                    });
pub static __Arng_outputs_no_typeof_undef_WithinTypeofExpr_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                                  name: std::borrow::Cow::from(r###"(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(_0: ast::ExprId)}: outputs::no_typeof_undef::WithinTypeofExpr) /*join*/"###),
                                                                                                                                                   afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                   {
                                                                                                                                                       let __cloned = __v.clone();
                                                                                                                                                       match <WithinTypeofExpr>::from_ddvalue(__v) {
                                                                                                                                                           WithinTypeofExpr{type_of: _, expr: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                                           _ => None
                                                                                                                                                       }.map(|x|(x,__cloned))
                                                                                                                                                   }
                                                                                                                                                   __f},
                                                                                                                                                   queryable: false
                                                                                                                                               });
pub static __Arng_outputs_no_typeof_undef_WithinTypeofExpr_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                                   name: std::borrow::Cow::from(r###"(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(_0: ast::ExprId)}: outputs::no_typeof_undef::WithinTypeofExpr) /*antijoin*/"###),
                                                                                                                                                   fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                   {
                                                                                                                                                       match <WithinTypeofExpr>::from_ddvalue(__v) {
                                                                                                                                                           WithinTypeofExpr{type_of: _, expr: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                                           _ => None
                                                                                                                                                       }
                                                                                                                                                   }
                                                                                                                                                   __f},
                                                                                                                                                   distinct: true
                                                                                                                                               });
pub static __Rule_outputs_no_typeof_undef_NeedsWithinTypeofExpr_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_typeof_undef::NeedsWithinTypeofExpr[(outputs::no_typeof_undef::NeedsWithinTypeofExpr{.file=file}: outputs::no_typeof_undef::NeedsWithinTypeofExpr)] :- config::EnableNoTypeofUndef[(config::EnableNoTypeofUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoTypeofUndefConfig>)}: config::EnableNoTypeofUndef)]. */
                                                                                                                                             program::Rule::CollectionRule {
                                                                                                                                                 description: std::borrow::Cow::from("outputs::no_typeof_undef::NeedsWithinTypeofExpr(.file=file) :- config::EnableNoTypeofUndef(.file=file, .config=_)."),
                                                                                                                                                 rel: 2,
                                                                                                                                                 xform: Some(XFormCollection::FilterMap{
                                                                                                                                                                 description: std::borrow::Cow::from("head of outputs::no_typeof_undef::NeedsWithinTypeofExpr(.file=file) :- config::EnableNoTypeofUndef(.file=file, .config=_)."),
                                                                                                                                                                 fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                 {
                                                                                                                                                                     let ref file = match *<types__config::EnableNoTypeofUndef>::from_ddvalue_ref(&__v) {
                                                                                                                                                                         types__config::EnableNoTypeofUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                                         _ => return None
                                                                                                                                                                     };
                                                                                                                                                                     Some(((NeedsWithinTypeofExpr{file: (*file).clone()})).into_ddvalue())
                                                                                                                                                                 }
                                                                                                                                                                 __f},
                                                                                                                                                                 next: Box::new(None)
                                                                                                                                                             })
                                                                                                                                             });
pub static __Rule_outputs_no_typeof_undef_NeedsWithinTypeofExpr_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_typeof_undef::NeedsWithinTypeofExpr[(outputs::no_typeof_undef::NeedsWithinTypeofExpr{.file=file}: outputs::no_typeof_undef::NeedsWithinTypeofExpr)] :- config::EnableNoUndef[(config::EnableNoUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUndefConfig>)}: config::EnableNoUndef)]. */
                                                                                                                                             program::Rule::CollectionRule {
                                                                                                                                                 description: std::borrow::Cow::from("outputs::no_typeof_undef::NeedsWithinTypeofExpr(.file=file) :- config::EnableNoUndef(.file=file, .config=_)."),
                                                                                                                                                 rel: 3,
                                                                                                                                                 xform: Some(XFormCollection::FilterMap{
                                                                                                                                                                 description: std::borrow::Cow::from("head of outputs::no_typeof_undef::NeedsWithinTypeofExpr(.file=file) :- config::EnableNoUndef(.file=file, .config=_)."),
                                                                                                                                                                 fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                 {
                                                                                                                                                                     let ref file = match *<types__config::EnableNoUndef>::from_ddvalue_ref(&__v) {
                                                                                                                                                                         types__config::EnableNoUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                                         _ => return None
                                                                                                                                                                     };
                                                                                                                                                                     Some(((NeedsWithinTypeofExpr{file: (*file).clone()})).into_ddvalue())
                                                                                                                                                                 }
                                                                                                                                                                 __f},
                                                                                                                                                                 next: Box::new(None)
                                                                                                                                                             })
                                                                                                                                             });
pub static __Rule_outputs_no_typeof_undef_WithinTypeofExpr_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_typeof_undef::WithinTypeofExpr[(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=type_of, .expr=expr}: outputs::no_typeof_undef::WithinTypeofExpr)] :- outputs::no_typeof_undef::NeedsWithinTypeofExpr[(outputs::no_typeof_undef::NeedsWithinTypeofExpr{.file=(file: ast::FileId)}: outputs::no_typeof_undef::NeedsWithinTypeofExpr)], inputs::UnaryOp[(inputs::UnaryOp{.expr_id=(type_of@ (ast::ExprId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::ExprId)), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp)]. */
                                                                                                                                        program::Rule::ArrangementRule {
                                                                                                                                            description: std::borrow::Cow::from( "outputs::no_typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=expr) :- outputs::no_typeof_undef::NeedsWithinTypeofExpr(.file=file), inputs::UnaryOp(.expr_id=(type_of@ ast::ExprId{.id=_, .file=file}), .op=ddlog_std::Some{.x=ast::UnaryTypeof{}}, .expr=ddlog_std::Some{.x=expr})."),
                                                                                                                                            arr: ( 68, 0),
                                                                                                                                            xform: XFormArrangement::Join{
                                                                                                                                                       description: std::borrow::Cow::from("outputs::no_typeof_undef::NeedsWithinTypeofExpr(.file=file), inputs::UnaryOp(.expr_id=(type_of@ ast::ExprId{.id=_, .file=file}), .op=ddlog_std::Some{.x=ast::UnaryTypeof{}}, .expr=ddlog_std::Some{.x=expr})"),
                                                                                                                                                       ffun: None,
                                                                                                                                                       arrangement: (54,0),
                                                                                                                                                       jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                       {
                                                                                                                                                           let ref file = match *<NeedsWithinTypeofExpr>::from_ddvalue_ref(__v1) {
                                                                                                                                                               NeedsWithinTypeofExpr{file: ref file} => (*file).clone(),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           let (ref type_of, ref expr) = match *<types__inputs::UnaryOp>::from_ddvalue_ref(__v2) {
                                                                                                                                                               types__inputs::UnaryOp{expr_id: ref type_of, op: ddlog_std::Option::Some{x: types__ast::UnaryOperand::UnaryTypeof{}}, expr: ddlog_std::Option::Some{x: ref expr}} => match type_of {
                                                                                                                                                                                                                                                                                                                                        types__ast::ExprId{id: _, file: _} => ((*type_of).clone(), (*expr).clone()),
                                                                                                                                                                                                                                                                                                                                        _ => return None
                                                                                                                                                                                                                                                                                                                                    },
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           Some(((WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*expr).clone()})).into_ddvalue())
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(None)
                                                                                                                                                   }
                                                                                                                                        });
pub static __Rule_outputs_no_typeof_undef_WithinTypeofExpr_1 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_typeof_undef::WithinTypeofExpr[(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=type_of, .expr=grouped}: outputs::no_typeof_undef::WithinTypeofExpr)] :- outputs::no_typeof_undef::WithinTypeofExpr[(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: outputs::no_typeof_undef::WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]. */
                                                                                                                                        program::Rule::ArrangementRule {
                                                                                                                                            description: std::borrow::Cow::from( "outputs::no_typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=grouped) :- outputs::no_typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=expr), inputs::Expression(.id=expr, .kind=ast::ExprGrouping{.inner=ddlog_std::Some{.x=grouped}}, .scope=_, .span=_)."),
                                                                                                                                            arr: ( 70, 0),
                                                                                                                                            xform: XFormArrangement::Join{
                                                                                                                                                       description: std::borrow::Cow::from("outputs::no_typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=expr), inputs::Expression(.id=expr, .kind=ast::ExprGrouping{.inner=ddlog_std::Some{.x=grouped}}, .scope=_, .span=_)"),
                                                                                                                                                       ffun: None,
                                                                                                                                                       arrangement: (27,1),
                                                                                                                                                       jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                       {
                                                                                                                                                           let (ref type_of, ref expr) = match *<WithinTypeofExpr>::from_ddvalue_ref(__v1) {
                                                                                                                                                               WithinTypeofExpr{type_of: ref type_of, expr: ref expr} => ((*type_of).clone(), (*expr).clone()),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           let ref grouped = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                               types__inputs::Expression{id: _, kind: types__ast::ExprKind::ExprGrouping{inner: ddlog_std::Option::Some{x: ref grouped}}, scope: _, span: _} => (*grouped).clone(),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           Some(((WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*grouped).clone()})).into_ddvalue())
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(None)
                                                                                                                                                   }
                                                                                                                                        });
pub static __Rule_outputs_no_typeof_undef_WithinTypeofExpr_2 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_typeof_undef::WithinTypeofExpr[(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=type_of, .expr=last}: outputs::no_typeof_undef::WithinTypeofExpr)] :- outputs::no_typeof_undef::WithinTypeofExpr[(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: outputs::no_typeof_undef::WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprSequence{.exprs=(sequence: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], ((ddlog_std::Some{.x=(var last: ast::ExprId)}: ddlog_std::Option<ast::ExprId>) = ((vec::last: function(ddlog_std::Vec<ast::ExprId>):ddlog_std::Option<ast::ExprId>)(sequence))). */
                                                                                                                                        program::Rule::ArrangementRule {
                                                                                                                                            description: std::borrow::Cow::from( "outputs::no_typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=last) :- outputs::no_typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=expr), inputs::Expression(.id=expr, .kind=ast::ExprSequence{.exprs=sequence}, .scope=_, .span=_), (ddlog_std::Some{.x=var last} = (vec::last(sequence)))."),
                                                                                                                                            arr: ( 70, 0),
                                                                                                                                            xform: XFormArrangement::Join{
                                                                                                                                                       description: std::borrow::Cow::from("outputs::no_typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=expr), inputs::Expression(.id=expr, .kind=ast::ExprSequence{.exprs=sequence}, .scope=_, .span=_)"),
                                                                                                                                                       ffun: None,
                                                                                                                                                       arrangement: (27,2),
                                                                                                                                                       jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                       {
                                                                                                                                                           let (ref type_of, ref expr) = match *<WithinTypeofExpr>::from_ddvalue_ref(__v1) {
                                                                                                                                                               WithinTypeofExpr{type_of: ref type_of, expr: ref expr} => ((*type_of).clone(), (*expr).clone()),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           let ref sequence = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                               types__inputs::Expression{id: _, kind: types__ast::ExprKind::ExprSequence{exprs: ref sequence}, scope: _, span: _} => (*sequence).clone(),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           let ref last: types__ast::ExprId = match types__vec::last::<types__ast::ExprId>(sequence) {
                                                                                                                                                               ddlog_std::Option::Some{x: last} => last,
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           Some(((WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*last).clone()})).into_ddvalue())
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(None)
                                                                                                                                                   }
                                                                                                                                        });
pub static __Rule_outputs_no_typeof_undef_NoTypeofUndef_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* outputs::no_typeof_undef::NoTypeofUndef[(outputs::no_typeof_undef::NoTypeofUndef{.whole_expr=whole_expr, .undefined_expr=undefined_expr}: outputs::no_typeof_undef::NoTypeofUndef)] :- config::EnableNoTypeofUndef[(config::EnableNoTypeofUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoTypeofUndefConfig>)}: config::EnableNoTypeofUndef)], inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr@ (ast::ExprId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::ExprId)), .value=(name: internment::Intern<string>)}: inputs::NameRef)], outputs::no_typeof_undef::WithinTypeofExpr[(outputs::no_typeof_undef::WithinTypeofExpr{.type_of=(whole_expr: ast::ExprId), .expr=(undefined_expr: ast::ExprId)}: outputs::no_typeof_undef::WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not name_in_scope::NameInScope[(name_in_scope::NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope)]. */
                                                                                                                                     program::Rule::ArrangementRule {
                                                                                                                                         description: std::borrow::Cow::from( "outputs::no_typeof_undef::NoTypeofUndef(.whole_expr=whole_expr, .undefined_expr=undefined_expr) :- config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name), outputs::no_typeof_undef::WithinTypeofExpr(.type_of=whole_expr, .expr=undefined_expr), inputs::Expression(.id=undefined_expr, .kind=_, .scope=scope, .span=span), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)."),
                                                                                                                                         arr: ( 2, 0),
                                                                                                                                         xform: XFormArrangement::Join{
                                                                                                                                                    description: std::borrow::Cow::from("config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name)"),
                                                                                                                                                    ffun: None,
                                                                                                                                                    arrangement: (43,1),
                                                                                                                                                    jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                    {
                                                                                                                                                        let ref file = match *<types__config::EnableNoTypeofUndef>::from_ddvalue_ref(__v1) {
                                                                                                                                                            types__config::EnableNoTypeofUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                            _ => return None
                                                                                                                                                        };
                                                                                                                                                        let (ref undefined_expr, ref name) = match *<types__inputs::NameRef>::from_ddvalue_ref(__v2) {
                                                                                                                                                            types__inputs::NameRef{expr_id: ref undefined_expr, value: ref name} => match undefined_expr {
                                                                                                                                                                                                                                        types__ast::ExprId{id: _, file: _} => ((*undefined_expr).clone(), (*name).clone()),
                                                                                                                                                                                                                                        _ => return None
                                                                                                                                                                                                                                    },
                                                                                                                                                            _ => return None
                                                                                                                                                        };
                                                                                                                                                        Some((ddlog_std::tuple2((*undefined_expr).clone(), (*name).clone())).into_ddvalue())
                                                                                                                                                    }
                                                                                                                                                    __f},
                                                                                                                                                    next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                            description: std::borrow::Cow::from("arrange config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name) by (undefined_expr)"),
                                                                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                            {
                                                                                                                                                                                let ddlog_std::tuple2(ref undefined_expr, ref name) = *<ddlog_std::tuple2<types__ast::ExprId, internment::Intern<String>>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                Some((((*undefined_expr).clone()).into_ddvalue(), (ddlog_std::tuple2((*undefined_expr).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                                                                            }
                                                                                                                                                                            __f},
                                                                                                                                                                            next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                               description: std::borrow::Cow::from("config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name), outputs::no_typeof_undef::WithinTypeofExpr(.type_of=whole_expr, .expr=undefined_expr)"),
                                                                                                                                                                                               ffun: None,
                                                                                                                                                                                               arrangement: (70,0),
                                                                                                                                                                                               jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                               {
                                                                                                                                                                                                   let ddlog_std::tuple2(ref undefined_expr, ref name) = *<ddlog_std::tuple2<types__ast::ExprId, internment::Intern<String>>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                   let ref whole_expr = match *<WithinTypeofExpr>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                       WithinTypeofExpr{type_of: ref whole_expr, expr: _} => (*whole_expr).clone(),
                                                                                                                                                                                                       _ => return None
                                                                                                                                                                                                   };
                                                                                                                                                                                                   Some((ddlog_std::tuple3((*undefined_expr).clone(), (*name).clone(), (*whole_expr).clone())).into_ddvalue())
                                                                                                                                                                                               }
                                                                                                                                                                                               __f},
                                                                                                                                                                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                                       description: std::borrow::Cow::from("arrange config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name), outputs::no_typeof_undef::WithinTypeofExpr(.type_of=whole_expr, .expr=undefined_expr) by (undefined_expr)"),
                                                                                                                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                                       {
                                                                                                                                                                                                                           let ddlog_std::tuple3(ref undefined_expr, ref name, ref whole_expr) = *<ddlog_std::tuple3<types__ast::ExprId, internment::Intern<String>, types__ast::ExprId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                           Some((((*undefined_expr).clone()).into_ddvalue(), (ddlog_std::tuple3((*undefined_expr).clone(), (*name).clone(), (*whole_expr).clone())).into_ddvalue()))
                                                                                                                                                                                                                       }
                                                                                                                                                                                                                       __f},
                                                                                                                                                                                                                       next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                                                                          description: std::borrow::Cow::from("config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name), outputs::no_typeof_undef::WithinTypeofExpr(.type_of=whole_expr, .expr=undefined_expr), inputs::Expression(.id=undefined_expr, .kind=_, .scope=scope, .span=span)"),
                                                                                                                                                                                                                                          ffun: None,
                                                                                                                                                                                                                                          arrangement: (27,0),
                                                                                                                                                                                                                                          jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                                                                          {
                                                                                                                                                                                                                                              let ddlog_std::tuple3(ref undefined_expr, ref name, ref whole_expr) = *<ddlog_std::tuple3<types__ast::ExprId, internment::Intern<String>, types__ast::ExprId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                                                              let (ref scope, ref span) = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                                                                  types__inputs::Expression{id: _, kind: _, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                                                                                                                                                                                                                  _ => return None
                                                                                                                                                                                                                                              };
                                                                                                                                                                                                                                              Some((ddlog_std::tuple4((*undefined_expr).clone(), (*name).clone(), (*whole_expr).clone(), (*scope).clone())).into_ddvalue())
                                                                                                                                                                                                                                          }
                                                                                                                                                                                                                                          __f},
                                                                                                                                                                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                                                                                  description: std::borrow::Cow::from("arrange config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name), outputs::no_typeof_undef::WithinTypeofExpr(.type_of=whole_expr, .expr=undefined_expr), inputs::Expression(.id=undefined_expr, .kind=_, .scope=scope, .span=span) by (name, scope)"),
                                                                                                                                                                                                                                                                  afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                                                                                  {
                                                                                                                                                                                                                                                                      let ddlog_std::tuple4(ref undefined_expr, ref name, ref whole_expr, ref scope) = *<ddlog_std::tuple4<types__ast::ExprId, internment::Intern<String>, types__ast::ExprId, types__ast::ScopeId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                                                                      Some(((ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), (ddlog_std::tuple2((*undefined_expr).clone(), (*whole_expr).clone())).into_ddvalue()))
                                                                                                                                                                                                                                                                  }
                                                                                                                                                                                                                                                                  __f},
                                                                                                                                                                                                                                                                  next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                                                                                                                                     description: std::borrow::Cow::from("config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name), outputs::no_typeof_undef::WithinTypeofExpr(.type_of=whole_expr, .expr=undefined_expr), inputs::Expression(.id=undefined_expr, .kind=_, .scope=scope, .span=span), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)"),
                                                                                                                                                                                                                                                                                     ffun: None,
                                                                                                                                                                                                                                                                                     arrangement: (61,1),
                                                                                                                                                                                                                                                                                     next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                                                                                                                                             description: std::borrow::Cow::from("head of outputs::no_typeof_undef::NoTypeofUndef(.whole_expr=whole_expr, .undefined_expr=undefined_expr) :- config::EnableNoTypeofUndef(.file=file, .config=_), inputs::NameRef(.expr_id=(undefined_expr@ ast::ExprId{.id=_, .file=file}), .value=name), outputs::no_typeof_undef::WithinTypeofExpr(.type_of=whole_expr, .expr=undefined_expr), inputs::Expression(.id=undefined_expr, .kind=_, .scope=scope, .span=span), not name_in_scope::NameInScope(.name=name, .scope=scope, .declared=_)."),
                                                                                                                                                                                                                                                                                                             fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                                                                                                                                             {
                                                                                                                                                                                                                                                                                                                 let ddlog_std::tuple2(ref undefined_expr, ref whole_expr) = *<ddlog_std::tuple2<types__ast::ExprId, types__ast::ExprId>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                                                                                                                                 Some(((NoTypeofUndef{whole_expr: (*whole_expr).clone(), undefined_expr: (*undefined_expr).clone()})).into_ddvalue())
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
                                                                                                                                                }
                                                                                                                                     });