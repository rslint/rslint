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


pub mod is_exported;
pub mod name_in_scope;
pub mod outputs;
pub mod var_decls;
use internment::Intern;
use once_cell::sync::Lazy;
use types__ast::{Pattern, Span, Spanned};

/// The implicitly introduced `arguments` variable for function scopes,
/// kept in a global so we only allocate & intern it once
pub static IMPLICIT_ARGUMENTS: Lazy<Intern<Pattern>> = Lazy::new(|| {
    Intern::new(Pattern::SinglePattern {
        name: Some(Spanned {
            data: Intern::new("arguments".to_owned()),
            // TODO: Give this the span of the creating function I guess
            span: Span::new(0, 0),
        })
        .into(),
    })
});

pub static __Arng___Prefix_0_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_: ast::FileId), (_: ast::ExprId), (_: ast::ExprId), (_1: ast::ScopeId), (_: ast::Span), (_0: internment::Intern<string>)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match <ddlog_std::tuple6<types__ast::FileId, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span, internment::Intern<String>>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple6(_, _, _, ref _1, _, ref _0) => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Rule___Prefix_0_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_0[((file: ast::FileId), (expr: ast::ExprId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>))] :- config::EnableNoUseBeforeDef[(config::EnableNoUseBeforeDef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUseBeforeDefConfig>)}: config::EnableNoUseBeforeDef)], inputs::New[(inputs::New{.expr_id=(expr@ (ast::ExprId{.id=(_: bit<32>), .file=(file: ast::FileId)}: ast::ExprId)), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(object: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)]. */
                                                                                                          program::Rule::ArrangementRule {
                                                                                                              description: std::borrow::Cow::from( "__Prefix_0[(file, expr, object, used_scope, used_in, name)] :- config::EnableNoUseBeforeDef(.file=file, .config=_), inputs::New(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .args=_), inputs::Expression(.id=expr, .kind=_, .scope=used_scope, .span=used_in), inputs::NameRef(.expr_id=object, .value=name)."),
                                                                                                              arr: ( 6, 0),
                                                                                                              xform: XFormArrangement::Join{
                                                                                                                         description: std::borrow::Cow::from("config::EnableNoUseBeforeDef(.file=file, .config=_), inputs::New(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .args=_)"),
                                                                                                                         ffun: None,
                                                                                                                         arrangement: (44,0),
                                                                                                                         jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                         {
                                                                                                                             let ref file = match *<types__config::EnableNoUseBeforeDef>::from_ddvalue_ref(__v1) {
                                                                                                                                 types__config::EnableNoUseBeforeDef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             let (ref expr, ref object) = match *<types__inputs::New>::from_ddvalue_ref(__v2) {
                                                                                                                                 types__inputs::New{expr_id: ref expr, object: ddlog_std::Option::Some{x: ref object}, args: _} => match expr {
                                                                                                                                                                                                                                       types__ast::ExprId{id: _, file: _} => ((*expr).clone(), (*object).clone()),
                                                                                                                                                                                                                                       _ => return None
                                                                                                                                                                                                                                   },
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             Some((ddlog_std::tuple3((*file).clone(), (*expr).clone(), (*object).clone())).into_ddvalue())
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                 description: std::borrow::Cow::from("arrange config::EnableNoUseBeforeDef(.file=file, .config=_), inputs::New(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .args=_) by (expr)"),
                                                                                                                                                 afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                 {
                                                                                                                                                     let ddlog_std::tuple3(ref file, ref expr, ref object) = *<ddlog_std::tuple3<types__ast::FileId, types__ast::ExprId, types__ast::ExprId>>::from_ddvalue_ref( &__v );
                                                                                                                                                     Some((((*expr).clone()).into_ddvalue(), (ddlog_std::tuple3((*file).clone(), (*expr).clone(), (*object).clone())).into_ddvalue()))
                                                                                                                                                 }
                                                                                                                                                 __f},
                                                                                                                                                 next: Box::new(XFormArrangement::Join{
                                                                                                                                                                    description: std::borrow::Cow::from("config::EnableNoUseBeforeDef(.file=file, .config=_), inputs::New(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .args=_), inputs::Expression(.id=expr, .kind=_, .scope=used_scope, .span=used_in)"),
                                                                                                                                                                    ffun: None,
                                                                                                                                                                    arrangement: (27,0),
                                                                                                                                                                    jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                    {
                                                                                                                                                                        let ddlog_std::tuple3(ref file, ref expr, ref object) = *<ddlog_std::tuple3<types__ast::FileId, types__ast::ExprId, types__ast::ExprId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                        let (ref used_scope, ref used_in) = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                                            types__inputs::Expression{id: _, kind: _, scope: ref used_scope, span: ref used_in} => ((*used_scope).clone(), (*used_in).clone()),
                                                                                                                                                                            _ => return None
                                                                                                                                                                        };
                                                                                                                                                                        Some((ddlog_std::tuple5((*file).clone(), (*expr).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue())
                                                                                                                                                                    }
                                                                                                                                                                    __f},
                                                                                                                                                                    next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                            description: std::borrow::Cow::from("arrange config::EnableNoUseBeforeDef(.file=file, .config=_), inputs::New(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .args=_), inputs::Expression(.id=expr, .kind=_, .scope=used_scope, .span=used_in) by (object)"),
                                                                                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                            {
                                                                                                                                                                                                let ddlog_std::tuple5(ref file, ref expr, ref object, ref used_scope, ref used_in) = *<ddlog_std::tuple5<types__ast::FileId, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span>>::from_ddvalue_ref( &__v );
                                                                                                                                                                                                Some((((*object).clone()).into_ddvalue(), (ddlog_std::tuple5((*file).clone(), (*expr).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue()))
                                                                                                                                                                                            }
                                                                                                                                                                                            __f},
                                                                                                                                                                                            next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                                               description: std::borrow::Cow::from("config::EnableNoUseBeforeDef(.file=file, .config=_), inputs::New(.expr_id=(expr@ ast::ExprId{.id=_, .file=file}), .object=ddlog_std::Some{.x=object}, .args=_), inputs::Expression(.id=expr, .kind=_, .scope=used_scope, .span=used_in), inputs::NameRef(.expr_id=object, .value=name)"),
                                                                                                                                                                                                               ffun: None,
                                                                                                                                                                                                               arrangement: (43,0),
                                                                                                                                                                                                               jfun: {fn __f(_: &DDValue, __v1: &DDValue, __v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                                               {
                                                                                                                                                                                                                   let ddlog_std::tuple5(ref file, ref expr, ref object, ref used_scope, ref used_in) = *<ddlog_std::tuple5<types__ast::FileId, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                                                                   let ref name = match *<types__inputs::NameRef>::from_ddvalue_ref(__v2) {
                                                                                                                                                                                                                       types__inputs::NameRef{expr_id: _, value: ref name} => (*name).clone(),
                                                                                                                                                                                                                       _ => return None
                                                                                                                                                                                                                   };
                                                                                                                                                                                                                   Some((ddlog_std::tuple6((*file).clone(), (*expr).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue())
                                                                                                                                                                                                               }
                                                                                                                                                                                                               __f},
                                                                                                                                                                                                               next: Box::new(None)
                                                                                                                                                                                                           })
                                                                                                                                                                                        }))
                                                                                                                                                                })
                                                                                                                                             }))
                                                                                                                     }
                                                                                                          });