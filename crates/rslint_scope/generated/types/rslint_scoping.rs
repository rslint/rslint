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


pub static __STATIC_0: ::once_cell::sync::Lazy<ddlog_std::Ref<crate::var_decls::VariableMeta>> = ::once_cell::sync::Lazy::new(|| ddlog_std::ref_new((&(crate::var_decls::VariableMeta{is_function_argument: false, implicitly_declared: false, declaration_span: (ddlog_std::Option::None{})}))));
pub static __STATIC_1: ::once_cell::sync::Lazy<ddlog_std::Ref<crate::var_decls::VariableMeta>> = ::once_cell::sync::Lazy::new(|| ddlog_std::ref_new((&(crate::var_decls::VariableMeta{is_function_argument: true, implicitly_declared: false, declaration_span: (ddlog_std::Option::None{})}))));
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

pub static __Arng___Prefix_4_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple2<types__ast::FileId, types__config::Config>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple2(ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_5_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple2<types__ast::FileId, types__config::Config>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple2(ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_6_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple2<types__ast::FileId, types__config::Config>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple2(ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_8_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple2<types__ast::FileId, types__config::Config>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple2(ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_9_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple2<types__ast::FileId, types__config::Config>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple2(ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_1_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_1: ast::FileId), (_: config::Config), (_: ast::ExprId), (_0: ast::ExprId), (_: ast::ScopeId), (_: ast::Span)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple6<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple6(ref _1, _, _, ref _0, _, _) => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_1_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config), (_: ast::ExprId), (_: ast::ExprId), (_: ast::ScopeId), (_: ast::Span)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple6<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple6(ref _0, _, _, _, _, _) => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_0_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config), (_: ast::ExprId), (_: ast::ExprId), (_2: ast::ScopeId), (_: ast::Span), (_1: internment::Intern<string>)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple7<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span, internment::Intern<String>>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple7(ref _0, _, _, _, ref _2, _, ref _1) => Some((ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_0_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_0: ast::FileId), (_: config::Config), (_: ast::ExprId), (_: ast::ExprId), (_: ast::ScopeId), (_: ast::Span), (_1: internment::Intern<string>)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple7<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span, internment::Intern<String>>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple7(ref _0, _, _, _, _, _, ref _1) => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng___Prefix_2_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"((_1: ast::FileId), (_: config::Config), (_: ast::ExprId), (_0: ast::ExprId)) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < ddlog_std::tuple4<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId>>::from_ddvalue(__v) {
                                                                                                                             ddlog_std::tuple4(ref _1, _, _, ref _0) => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Rule___Prefix_4_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_4[((file: ast::FileId), (config: config::Config))] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(config: config::Config)}: inputs::File)], ((config::no_undef_enabled(config)) or (config::no_typeof_undef_enabled(config))). */
                                                                                                          program::Rule::CollectionRule {
                                                                                                              description: std::borrow::Cow::from("__Prefix_4[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), ((config::no_undef_enabled(config)) or (config::no_typeof_undef_enabled(config)))."),
                                                                                                              rel: 29,
                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                              description: std::borrow::Cow::from("head of __Prefix_4[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), ((config::no_undef_enabled(config)) or (config::no_typeof_undef_enabled(config)))."),
                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                              {
                                                                                                                                  let (ref file, ref config) = match *<types__inputs::File>::from_ddvalue_ref(&__v) {
                                                                                                                                      types__inputs::File{id: ref file, kind: _, top_level_scope: _, config: ref config} => ((*file).clone(), (*config).clone()),
                                                                                                                                      _ => return None
                                                                                                                                  };
                                                                                                                                  if !(types__config::no_undef_enabled(config) || types__config::no_typeof_undef_enabled(config)) {return None;};
                                                                                                                                  Some((ddlog_std::tuple2((*file).clone(), (*config).clone())).into_ddvalue())
                                                                                                                              }
                                                                                                                              __f},
                                                                                                                              next: Box::new(None)
                                                                                                                          })
                                                                                                          });
pub static __Rule___Prefix_5_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_5[((file: ast::FileId), (config: config::Config))] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(config: config::Config)}: inputs::File)], (config::no_use_before_def_enabled(config)). */
                                                                                                          program::Rule::CollectionRule {
                                                                                                              description: std::borrow::Cow::from("__Prefix_5[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_use_before_def_enabled(config))."),
                                                                                                              rel: 29,
                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                              description: std::borrow::Cow::from("head of __Prefix_5[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_use_before_def_enabled(config))."),
                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                              {
                                                                                                                                  let (ref file, ref config) = match *<types__inputs::File>::from_ddvalue_ref(&__v) {
                                                                                                                                      types__inputs::File{id: ref file, kind: _, top_level_scope: _, config: ref config} => ((*file).clone(), (*config).clone()),
                                                                                                                                      _ => return None
                                                                                                                                  };
                                                                                                                                  if !types__config::no_use_before_def_enabled(config) {return None;};
                                                                                                                                  Some((ddlog_std::tuple2((*file).clone(), (*config).clone())).into_ddvalue())
                                                                                                                              }
                                                                                                                              __f},
                                                                                                                              next: Box::new(None)
                                                                                                                          })
                                                                                                          });
pub static __Rule___Prefix_6_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_6[((file: ast::FileId), (config: config::Config))] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(config: config::Config)}: inputs::File)], (config::no_unused_vars_enabled(config)). */
                                                                                                          program::Rule::CollectionRule {
                                                                                                              description: std::borrow::Cow::from("__Prefix_6[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_unused_vars_enabled(config))."),
                                                                                                              rel: 29,
                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                              description: std::borrow::Cow::from("head of __Prefix_6[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_unused_vars_enabled(config))."),
                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                              {
                                                                                                                                  let (ref file, ref config) = match *<types__inputs::File>::from_ddvalue_ref(&__v) {
                                                                                                                                      types__inputs::File{id: ref file, kind: _, top_level_scope: _, config: ref config} => ((*file).clone(), (*config).clone()),
                                                                                                                                      _ => return None
                                                                                                                                  };
                                                                                                                                  if !types__config::no_unused_vars_enabled(config) {return None;};
                                                                                                                                  Some((ddlog_std::tuple2((*file).clone(), (*config).clone())).into_ddvalue())
                                                                                                                              }
                                                                                                                              __f},
                                                                                                                              next: Box::new(None)
                                                                                                                          })
                                                                                                          });
pub static __Rule___Prefix_8_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_8[((file: ast::FileId), (config: config::Config))] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(config: config::Config)}: inputs::File)], (config::no_undef_enabled(config)). */
                                                                                                          program::Rule::CollectionRule {
                                                                                                              description: std::borrow::Cow::from("__Prefix_8[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_undef_enabled(config))."),
                                                                                                              rel: 29,
                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                              description: std::borrow::Cow::from("head of __Prefix_8[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_undef_enabled(config))."),
                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                              {
                                                                                                                                  let (ref file, ref config) = match *<types__inputs::File>::from_ddvalue_ref(&__v) {
                                                                                                                                      types__inputs::File{id: ref file, kind: _, top_level_scope: _, config: ref config} => ((*file).clone(), (*config).clone()),
                                                                                                                                      _ => return None
                                                                                                                                  };
                                                                                                                                  if !types__config::no_undef_enabled(config) {return None;};
                                                                                                                                  Some((ddlog_std::tuple2((*file).clone(), (*config).clone())).into_ddvalue())
                                                                                                                              }
                                                                                                                              __f},
                                                                                                                              next: Box::new(None)
                                                                                                                          })
                                                                                                          });
pub static __Rule___Prefix_9_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_9[((file: ast::FileId), (config: config::Config))] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(config: config::Config)}: inputs::File)], (config::no_shadow_enabled(config)). */
                                                                                                          program::Rule::CollectionRule {
                                                                                                              description: std::borrow::Cow::from("__Prefix_9[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_shadow_enabled(config))."),
                                                                                                              rel: 29,
                                                                                                              xform: Some(XFormCollection::FilterMap{
                                                                                                                              description: std::borrow::Cow::from("head of __Prefix_9[(file, config)] :- inputs::File(.id=file, .kind=_, .top_level_scope=_, .config=config), (config::no_shadow_enabled(config))."),
                                                                                                                              fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                              {
                                                                                                                                  let (ref file, ref config) = match *<types__inputs::File>::from_ddvalue_ref(&__v) {
                                                                                                                                      types__inputs::File{id: ref file, kind: _, top_level_scope: _, config: ref config} => ((*file).clone(), (*config).clone()),
                                                                                                                                      _ => return None
                                                                                                                                  };
                                                                                                                                  if !types__config::no_shadow_enabled(config) {return None;};
                                                                                                                                  Some((ddlog_std::tuple2((*file).clone(), (*config).clone())).into_ddvalue())
                                                                                                                              }
                                                                                                                              __f},
                                                                                                                              next: Box::new(None)
                                                                                                                          })
                                                                                                          });
pub static __Rule___Prefix_1_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_1[((file: ast::FileId), (config: config::Config), (expr: ast::ExprId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))] :- __Prefix_5[((file: ast::FileId), (config: config::Config))], inputs::New[(inputs::New{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)]. */
                                                                                                          program::Rule::ArrangementRule {
                                                                                                              description: std::borrow::Cow::from( "__Prefix_1[(file, config, expr, object, used_scope, used_in)] :- __Prefix_5[(file, config)], inputs::New(.expr_id=expr, .file=file, .object=ddlog_std::Some{.x=object}, .args=_), inputs::Expression(.id=expr, .file=file, .kind=_, .scope=used_scope, .span=used_in)."),
                                                                                                              arr: ( 4, 0),
                                                                                                              xform: XFormArrangement::Join{
                                                                                                                         description: std::borrow::Cow::from("__Prefix_5[(file, config)], inputs::New(.expr_id=expr, .file=file, .object=ddlog_std::Some{.x=object}, .args=_)"),
                                                                                                                         ffun: None,
                                                                                                                         arrangement: (45,0),
                                                                                                                         jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                         {
                                                                                                                             let (ref file, ref config) = match *<ddlog_std::tuple2<types__ast::FileId, types__config::Config>>::from_ddvalue_ref(__v1) {
                                                                                                                                 ddlog_std::tuple2(ref file, ref config) => ((*file).clone(), (*config).clone()),
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             let (ref expr, ref object) = match *<types__inputs::New>::from_ddvalue_ref(__v2) {
                                                                                                                                 types__inputs::New{expr_id: ref expr, file: _, object: ddlog_std::Option::Some{x: ref object}, args: _} => ((*expr).clone(), (*object).clone()),
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             Some((ddlog_std::tuple4((*file).clone(), (*config).clone(), (*expr).clone(), (*object).clone())).into_ddvalue())
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                 description: std::borrow::Cow::from("arrange __Prefix_5[(file, config)], inputs::New(.expr_id=expr, .file=file, .object=ddlog_std::Some{.x=object}, .args=_) by (expr, file)"),
                                                                                                                                                 afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                 {
                                                                                                                                                     let ddlog_std::tuple4(ref file, ref config, ref expr, ref object) = *<ddlog_std::tuple4<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId>>::from_ddvalue_ref( &__v );
                                                                                                                                                     Some(((ddlog_std::tuple2((*expr).clone(), (*file).clone())).into_ddvalue(), (ddlog_std::tuple4((*file).clone(), (*config).clone(), (*expr).clone(), (*object).clone())).into_ddvalue()))
                                                                                                                                                 }
                                                                                                                                                 __f},
                                                                                                                                                 next: Box::new(XFormArrangement::Join{
                                                                                                                                                                    description: std::borrow::Cow::from("__Prefix_5[(file, config)], inputs::New(.expr_id=expr, .file=file, .object=ddlog_std::Some{.x=object}, .args=_), inputs::Expression(.id=expr, .file=file, .kind=_, .scope=used_scope, .span=used_in)"),
                                                                                                                                                                    ffun: None,
                                                                                                                                                                    arrangement: (28,0),
                                                                                                                                                                    jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                    {
                                                                                                                                                                        let ddlog_std::tuple4(ref file, ref config, ref expr, ref object) = *<ddlog_std::tuple4<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId>>::from_ddvalue_ref( __v1 );
                                                                                                                                                                        let (ref used_scope, ref used_in) = match *<types__inputs::Expression>::from_ddvalue_ref(__v2) {
                                                                                                                                                                            types__inputs::Expression{id: _, file: _, kind: _, scope: ref used_scope, span: ref used_in} => ((*used_scope).clone(), (*used_in).clone()),
                                                                                                                                                                            _ => return None
                                                                                                                                                                        };
                                                                                                                                                                        Some((ddlog_std::tuple6((*file).clone(), (*config).clone(), (*expr).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue())
                                                                                                                                                                    }
                                                                                                                                                                    __f},
                                                                                                                                                                    next: Box::new(None)
                                                                                                                                                                })
                                                                                                                                             }))
                                                                                                                     }
                                                                                                          });
pub static __Rule___Prefix_0_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_0[((file: ast::FileId), (config: config::Config), (expr: ast::ExprId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>))] :- __Prefix_1[((file: ast::FileId), (config: config::Config), (expr: ast::ExprId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(object: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)]. */
                                                                                                          program::Rule::ArrangementRule {
                                                                                                              description: std::borrow::Cow::from( "__Prefix_0[(file, config, expr, object, used_scope, used_in, name)] :- __Prefix_1[(file, config, expr, object, used_scope, used_in)], inputs::NameRef(.expr_id=object, .file=file, .value=name)."),
                                                                                                              arr: ( 1, 0),
                                                                                                              xform: XFormArrangement::Join{
                                                                                                                         description: std::borrow::Cow::from("__Prefix_1[(file, config, expr, object, used_scope, used_in)], inputs::NameRef(.expr_id=object, .file=file, .value=name)"),
                                                                                                                         ffun: None,
                                                                                                                         arrangement: (44,0),
                                                                                                                         jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                         {
                                                                                                                             let (ref file, ref config, ref expr, ref object, ref used_scope, ref used_in) = match *<ddlog_std::tuple6<types__ast::FileId, types__config::Config, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span>>::from_ddvalue_ref(__v1) {
                                                                                                                                 ddlog_std::tuple6(ref file, ref config, ref expr, ref object, ref used_scope, ref used_in) => ((*file).clone(), (*config).clone(), (*expr).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone()),
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             let ref name = match *<types__inputs::NameRef>::from_ddvalue_ref(__v2) {
                                                                                                                                 types__inputs::NameRef{expr_id: _, file: _, value: ref name} => (*name).clone(),
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             Some((ddlog_std::tuple7((*file).clone(), (*config).clone(), (*expr).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue())
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         next: Box::new(None)
                                                                                                                     }
                                                                                                          });
pub static __Rule___Prefix_2_0 : ::once_cell::sync::Lazy<program::Rule> = ::once_cell::sync::Lazy::new(|| /* __Prefix_2[((file: ast::FileId), (config: config::Config), (type_of: ast::ExprId), (expr: ast::ExprId))] :- __Prefix_4[((file: ast::FileId), (config: config::Config))], outputs::typeof_undef::WithinTypeofExpr[(outputs::typeof_undef::WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: outputs::typeof_undef::WithinTypeofExpr)]. */
                                                                                                          program::Rule::ArrangementRule {
                                                                                                              description: std::borrow::Cow::from( "__Prefix_2[(file, config, type_of, expr)] :- __Prefix_4[(file, config)], outputs::typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=expr, .file=file)."),
                                                                                                              arr: ( 3, 0),
                                                                                                              xform: XFormArrangement::Join{
                                                                                                                         description: std::borrow::Cow::from("__Prefix_4[(file, config)], outputs::typeof_undef::WithinTypeofExpr(.type_of=type_of, .expr=expr, .file=file)"),
                                                                                                                         ffun: None,
                                                                                                                         arrangement: (75,0),
                                                                                                                         jfun: {fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                         {
                                                                                                                             let (ref file, ref config) = match *<ddlog_std::tuple2<types__ast::FileId, types__config::Config>>::from_ddvalue_ref(__v1) {
                                                                                                                                 ddlog_std::tuple2(ref file, ref config) => ((*file).clone(), (*config).clone()),
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             let (ref type_of, ref expr) = match *<crate::outputs::typeof_undef::WithinTypeofExpr>::from_ddvalue_ref(__v2) {
                                                                                                                                 crate::outputs::typeof_undef::WithinTypeofExpr{type_of: ref type_of, expr: ref expr, file: _} => ((*type_of).clone(), (*expr).clone()),
                                                                                                                                 _ => return None
                                                                                                                             };
                                                                                                                             Some((ddlog_std::tuple4((*file).clone(), (*config).clone(), (*type_of).clone(), (*expr).clone())).into_ddvalue())
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         next: Box::new(None)
                                                                                                                     }
                                                                                                          });