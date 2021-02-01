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
use ::differential_datalog::ddval::DDValConvert;
use ::differential_datalog::ddval::DDValue;
use ::differential_datalog::program;
use ::differential_datalog::program::TupleTS;
use ::differential_datalog::program::Weight;
use ::differential_datalog::program::XFormArrangement;
use ::differential_datalog::program::XFormCollection;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;

// `usize` and `isize` are builtin Rust types; we therefore declare an alias to DDlog's `usize` and
// `isize`.
pub type std_usize = u64;
pub type std_isize = i64;

use crate::var_decls::{DeclarationScope, VariableDeclarations};
use ddlog_std::{Either, Option as DDlogOption, Vec as DDlogVec};
use differential_dataflow::{
    collection::Collection,
    difference::Semigroup,
    lattice::Lattice,
    operators::{
        arrange::{ArrangeByKey, ArrangeBySelf, Arranged, TraceAgent},
        Iterate, Join, JoinCore, Threshold,
    },
    trace::{
        implementations::ord::{OrdKeySpine, OrdValSpine},
        TraceReader,
    },
    AsCollection, Data, ExchangeData, Hashable,
};
use internment::Intern;
use std::{fmt::Debug, hash::Hash, iter, ops::Mul};
use timely::dataflow::{channels::pact::Pipeline, operators::Operator, Scope, ScopeParent, Stream};
use types__ast::{ExportKind, ExprId, FileId, Name, ScopeId};
use types__inputs::{Assign, Expression, FileExport, InputScope, NameRef};

#[allow(clippy::too_many_arguments)]
pub fn ResolveSymbols<
    S,
    D,
    Files,
    Vars,
    Scopes,
    Exprs,
    Names,
    Assigns,
    Exports,
    ScopeName,
    DeclName,
>(
    files_to_resolve: &Collection<S, D, Weight>,
    convert_files_to_resolve: Files,

    variable_declarations: &Collection<S, D, Weight>,
    convert_variable_declarations: Vars,

    input_scopes: &Collection<S, D, Weight>,
    convert_input_scopes: Scopes,

    expressions: &Collection<S, D, Weight>,
    convert_expressions: Exprs,

    name_refs: &Collection<S, D, Weight>,
    convert_name_refs: Names,

    assignments: &Collection<S, D, Weight>,
    convert_assignments: Assigns,

    file_exports: &Collection<S, D, Weight>,
    convert_file_exports: Exports,

    convert_name_in_scope: ScopeName,
    convert_scope_of_decl_name: DeclName,
) -> (Collection<S, D, Weight>, Collection<S, D, Weight>)
where
    S: Scope,
    S::Timestamp: Lattice,
    D: Data,
    Files: Fn(D) -> NeedsSymbolResolution + 'static,
    Vars: Fn(D) -> VariableDeclarations + 'static,
    Scopes: Fn(D) -> InputScope + 'static,
    Exprs: Fn(D) -> Expression + 'static,
    Names: Fn(D) -> NameRef + 'static,
    Assigns: Fn(D) -> Assign + 'static,
    Exports: Fn(D) -> FileExport + 'static,
    ScopeName: Fn(NameInScope) -> D + 'static,
    DeclName: Fn(ScopeOfDeclName) -> D + 'static,
{
    // Files that require name resolution stored as a keyed arrangement of `FileId`s
    let files_to_resolve = files_to_resolve
        .map_named("Marshal & Key FilesToResolve by FileId", move |file| {
            convert_files_to_resolve(file).file
        })
        .arrange_by_self_named("Arrange FilesToResolve");

    // Expressions for enabled files, arranged by their expression ids
    let expressions = {
        let exprs_by_file =
            expressions.map_named("Marshal & Key Expressions by ExprId", move |file| {
                let expr = convert_expressions(file);
                (expr.id.file, expr)
            });

        // Only select expressions for which symbol resolution is enabled
        semijoin_arranged(&exprs_by_file, &files_to_resolve)
            // Key expressions by their `ExprId` and arrange them
            .map(|(_, expr)| (expr.id, expr))
            .arrange_by_key_named("Arrange Expressions")
    };

    let variable_declarations = {
        let variable_declarations = variable_declarations.map_named(
            "Marshal & Key VariableDeclarations by FileId",
            move |decl| {
                let decl = convert_variable_declarations(decl);
                let file = match decl.scope {
                    DeclarationScope::Unhoistable { scope } => scope.file,
                    DeclarationScope::Hoistable { hoisted, .. } => hoisted.file,
                };

                (file, decl)
            },
        );

        semijoin_arranged(&variable_declarations, &files_to_resolve)
    };

    let input_scopes = {
        let input_scopes =
            convert_collection("Marshal InputScopes", input_scopes, convert_input_scopes)
                .map(|scope| (scope.parent.file, scope));

        semijoin_arranged(&input_scopes, &files_to_resolve).map(|(_, scope)| scope)
    };

    let input_scopes_by_parent = input_scopes
        .map(|scope| (scope.parent, scope.child))
        .arrange_by_key();
    let input_scopes_by_child = input_scopes
        .map(|scope| (scope.child, scope.parent))
        .arrange_by_key();

    let symbol_occurrences = collect_name_occurrences(
        &files_to_resolve,
        &expressions,
        &input_scopes_by_child,
        name_refs,
        convert_name_refs,
        assignments,
        convert_assignments,
        file_exports,
        convert_file_exports,
    )
    .arrange_by_self();

    let scope_of_decl_name = variable_declarations.map(|(_, decl)| {
        let scope = match decl.scope {
            DeclarationScope::Unhoistable { scope } => scope,
            DeclarationScope::Hoistable { hoisted, .. } => hoisted,
        };

        ScopeOfDeclName {
            name: decl.name,
            scope,
            declared: decl.declared_in,
        }
    });

    let decl_names_by_name_and_scope =
        scope_of_decl_name.map(|decl| (decl.name.to_string(), decl.scope));

    let name_in_scope = {
        let concrete_declarations = variable_declarations.map(|(_, decl)| {
            let hoisted_scope = match decl.scope {
                DeclarationScope::Unhoistable { scope } => scope,
                DeclarationScope::Hoistable { hoisted, .. } => hoisted,
            };

            ((decl.name.to_string(), hoisted_scope), decl.declared_in)
        });

        semijoin_arranged(&concrete_declarations, &symbol_occurrences)
            .iterate(|names| {
                let parent_propagations = names
                    .map(|((name, parent), decl)| (parent, (name, decl)))
                    .join_core(
                        &input_scopes_by_parent.enter(&names.scope()),
                        |_parent, &(ref name, decl), &child| {
                            iter::once(((name.clone(), child), decl))
                        },
                    )
                    .antijoin(&decl_names_by_name_and_scope.enter(&names.scope()));

                semijoin_arranged(
                    &parent_propagations,
                    &symbol_occurrences.enter(&names.scope()),
                )
                .concat(&names)
                .distinct_core()
            })
            .map(move |((name, scope), declared)| {
                let name = NameInScope {
                    name: Intern::new(name),
                    scope,
                    declared,
                };

                convert_name_in_scope(name)
            })
    };

    (
        name_in_scope,
        scope_of_decl_name.map(convert_scope_of_decl_name),
    )
}

type FilesTrace<S> =
    Arranged<S, TraceAgent<OrdKeySpine<FileId, <S as ScopeParent>::Timestamp, Weight>>>;

type ExprsTrace<S> =
    Arranged<S, TraceAgent<OrdValSpine<ExprId, Expression, <S as ScopeParent>::Timestamp, Weight>>>;

type InputsTrace<S> =
    Arranged<S, TraceAgent<OrdValSpine<ScopeId, ScopeId, <S as ScopeParent>::Timestamp, Weight>>>;

/// Collects all usages of symbols within files that need symbol resolution
#[allow(clippy::clippy::too_many_arguments)]
fn collect_name_occurrences<S, D, Names, Assigns, Exports>(
    files: &FilesTrace<S>,
    exprs: &ExprsTrace<S>,
    input_scopes_by_child: &InputsTrace<S>,

    name_refs: &Collection<S, D, Weight>,
    convert_name_refs: Names,

    assignments: &Collection<S, D, Weight>,
    convert_assignments: Assigns,

    file_exports: &Collection<S, D, Weight>,
    convert_file_exports: Exports,
) -> Collection<S, (String, ScopeId), Weight>
where
    S: Scope,
    S::Timestamp: Lattice,
    D: Data,
    Names: Fn(D) -> NameRef + 'static,
    Assigns: Fn(D) -> Assign + 'static,
    Exports: Fn(D) -> FileExport + 'static,
{
    // Only the name references for which symbol resolution is required
    let name_refs = name_refs
        .map_named("Marshal & Key NameRefs by ExprId", move |name| {
            let name = convert_name_refs(name);
            (name.expr_id, name)
        })
        // Join all name references to their corresponding expressions
        // `expr` has already been filtered here to only contain the expressions
        // for which symbol resolution is enabled, so this does double duty
        .join_core(&exprs, |expr_id, name_ref, expr| {
            iter::once((expr.scope, name_ref.value.to_string()))
        });

    let assignments = assignments
        .map_named("Marshal & Key Assignments by ExprId", move |assign| {
            let assign = convert_assignments(assign);
            (assign.expr_id, assign)
        })
        // Join assignments onto their corresponding expressions and extract all
        // variables they bind to, doing double duty to only process
        .join_core(&exprs, |expr_id, assign, expr| {
            let bound_variables = if let DDlogOption::Some {
                x: Either::Left { l: pattern },
            } = &assign.lhs
            {
                types__ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pattern)
            } else {
                DDlogVec::new()
            };

            let scope = expr.scope;
            bound_variables
                .into_iter()
                .map(move |name| (scope, name.data.to_string()))
        });

    let file_exports = {
        let file_exports =
            file_exports.map_named("Marshal & Key FileExports by FileId", move |export| {
                let export = convert_file_exports(export);
                (export.scope.file, export)
            });

        semijoin_arranged(&file_exports, files).flat_map(|(_, export)| {
            let scope = export.scope;

            if let ExportKind::NamedExport { name, alias } = export.export {
                ddlog_std::std2option(alias)
                    .or_else(|| name.into())
                    .map(|name| (scope, name.data.to_string()))
            } else {
                None
            }
        })
    };

    name_refs
        .concat(&assignments)
        .concat(&file_exports)
        .iterate(|symbols| {
            symbols
                .arrange_by_key()
                .join_core(
                    &input_scopes_by_child.enter(&symbols.scope()),
                    |_child, name, &parent| iter::once((parent, name.clone())),
                )
                .concat(symbols)
                .distinct_core()
        })
        .map(|(scope, name)| (name, scope))
}

fn convert_collection<S, D, R, T, F, N>(
    name: N,
    collection: &Collection<S, D, R>,
    convert: F,
) -> Collection<S, T, R>
where
    S: Scope,
    D: Data,
    R: Semigroup,
    T: Data,
    F: Fn(D) -> T + 'static,
    N: AsRef<str>,
{
    collection.map_named(name.as_ref(), convert)
}

fn semijoin_arranged<S, K, V, R, R2, A>(
    values: &Collection<S, (K, V), R>,
    keys: &Arranged<S, A>,
) -> Collection<S, (K, V), <R as Mul<R2>>::Output>
where
    S: Scope,
    S::Timestamp: Lattice,
    K: ExchangeData + Hashable,
    V: ExchangeData,
    R: ExchangeData + Semigroup + Mul<R2>,
    R2: Semigroup,
    <R as Mul<R2>>::Output: Semigroup,
    A: TraceReader<Key = K, Val = (), Time = S::Timestamp, R = R2> + Clone + 'static,
{
    let arranged_values = values.arrange_by_key();
    arranged_values.join_core(keys, |key, value, _| Some((key.clone(), value.clone())))
}

pub trait MapExt<S, D, D2> {
    type Output;

    /// An extension of [`Map`] that allows naming the operator
    ///
    /// [`Map`]: timely::dataflow::operators::map::Map
    fn map_named<N, L>(&self, name: N, logic: L) -> Self::Output
    where
        N: AsRef<str>,
        L: FnMut(D) -> D2 + 'static;
}

impl<S, D, D2, R> MapExt<S, D, D2> for Collection<S, D, R>
where
    S: Scope,
    D: Data,
    D2: Data,
    R: Semigroup,
{
    type Output = Collection<S, D2, R>;

    fn map_named<N, L>(&self, name: N, mut logic: L) -> Self::Output
    where
        N: AsRef<str>,
        L: FnMut(D) -> D2 + 'static,
    {
        self.inner
            .map_named(name, move |(data, time, delta)| (logic(data), time, delta))
            .as_collection()
    }
}

impl<S, D, D2> MapExt<S, D, D2> for Stream<S, D>
where
    S: Scope,
    D: Data,
    D2: Data,
{
    type Output = Stream<S, D2>;

    fn map_named<N, L>(&self, name: N, mut logic: L) -> Self::Output
    where
        N: AsRef<str>,
        L: FnMut(D) -> D2 + 'static,
    {
        let mut vector = Vec::new();

        self.unary(Pipeline, name.as_ref(), move |_, _| {
            move |input, output| {
                input.for_each(|time, data| {
                    data.swap(&mut vector);
                    output
                        .session(&time)
                        .give_iterator(vector.drain(..).map(|x| logic(x)));
                });
            }
        })
    }
}

#[derive(
    Eq,
    Ord,
    Clone,
    Hash,
    PartialEq,
    PartialOrd,
    IntoRecord,
    Mutator,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
)]
#[ddlog(rename = "name_in_scope::NameInScope")]
pub struct NameInScope {
    pub name: types__ast::Name,
    pub scope: types__ast::ScopeId,
    pub declared: types__ast::AnyId,
}
impl abomonation::Abomonation for NameInScope {}
impl ::std::fmt::Display for NameInScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameInScope {
                name,
                scope,
                declared,
            } => {
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
#[derive(
    Eq,
    Ord,
    Clone,
    Hash,
    PartialEq,
    PartialOrd,
    IntoRecord,
    Mutator,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
)]
#[ddlog(rename = "name_in_scope::NeedsSymbolResolution")]
pub struct NeedsSymbolResolution {
    pub file: types__ast::FileId,
}
impl abomonation::Abomonation for NeedsSymbolResolution {}
impl ::std::fmt::Display for NeedsSymbolResolution {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NeedsSymbolResolution { file } => {
                __formatter.write_str("name_in_scope::NeedsSymbolResolution{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NeedsSymbolResolution {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(
    Eq,
    Ord,
    Clone,
    Hash,
    PartialEq,
    PartialOrd,
    IntoRecord,
    Mutator,
    Default,
    Serialize,
    Deserialize,
    FromRecord,
)]
#[ddlog(rename = "name_in_scope::ScopeOfDeclName")]
pub struct ScopeOfDeclName {
    pub name: types__ast::Name,
    pub scope: types__ast::ScopeId,
    pub declared: types__ast::AnyId,
}
impl abomonation::Abomonation for ScopeOfDeclName {}
impl ::std::fmt::Display for ScopeOfDeclName {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ScopeOfDeclName {
                name,
                scope,
                declared,
            } => {
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
pub static __Arng_name_in_scope_NameInScope_0: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Map {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###,
        ),
        afun: {
            fn __f(__v: DDValue) -> Option<(DDValue, DDValue)> {
                let __cloned = __v.clone();
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _0,
                        scope: ref _1,
                        declared: _,
                    } => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                    _ => None,
                }
                .map(|x| (x, __cloned))
            }
            __f
        },
        queryable: false,
    });
pub static __Arng_name_in_scope_NameInScope_1: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Set {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###,
        ),
        fmfun: {
            fn __f(__v: DDValue) -> Option<DDValue> {
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _0,
                        scope: ref _1,
                        declared: _,
                    } => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                    _ => None,
                }
            }
            __f
        },
        distinct: true,
    });
pub static __Arng_name_in_scope_NameInScope_2: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Map {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(ast::AnyIdStmt{.stmt=(_: ast::StmtId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###,
        ),
        afun: {
            fn __f(__v: DDValue) -> Option<(DDValue, DDValue)> {
                let __cloned = __v.clone();
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _0,
                        scope: ref _1,
                        declared: types__ast::AnyId::AnyIdStmt { stmt: _ },
                    } => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                    _ => None,
                }
                .map(|x| (x, __cloned))
            }
            __f
        },
        queryable: false,
    });
pub static __Arng_name_in_scope_NameInScope_3: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Map {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(ast::AnyIdClass{.class=(_: ast::ClassId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###,
        ),
        afun: {
            fn __f(__v: DDValue) -> Option<(DDValue, DDValue)> {
                let __cloned = __v.clone();
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _0,
                        scope: ref _1,
                        declared: types__ast::AnyId::AnyIdClass { class: _ },
                    } => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                    _ => None,
                }
                .map(|x| (x, __cloned))
            }
            __f
        },
        queryable: false,
    });
pub static __Arng_name_in_scope_NameInScope_4: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Map {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(ast::AnyIdFunc{.func=(_: ast::FuncId)}: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###,
        ),
        afun: {
            fn __f(__v: DDValue) -> Option<(DDValue, DDValue)> {
                let __cloned = __v.clone();
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _0,
                        scope: ref _1,
                        declared: types__ast::AnyId::AnyIdFunc { func: _ },
                    } => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                    _ => None,
                }
                .map(|x| (x, __cloned))
            }
            __f
        },
        queryable: false,
    });
pub static __Arng_name_in_scope_NameInScope_5: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Set {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=_1, .declared=(_2: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###,
        ),
        fmfun: {
            fn __f(__v: DDValue) -> Option<DDValue> {
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _0,
                        scope: ref _1,
                        declared: ref _2,
                    } => Some(
                        (ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone()))
                            .into_ddvalue(),
                    ),
                    _ => None,
                }
            }
            __f
        },
        distinct: true,
    });
pub static __Arng_name_in_scope_NameInScope_6: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Set {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::ScopeId), .declared=(_2: ast::AnyId)}: name_in_scope::NameInScope) /*antijoin*/"###,
        ),
        fmfun: {
            fn __f(__v: DDValue) -> Option<DDValue> {
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _0,
                        scope: ref _1,
                        declared: ref _2,
                    } => Some(
                        (ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone()))
                            .into_ddvalue(),
                    ),
                    _ => None,
                }
            }
            __f
        },
        distinct: true,
    });
pub static __Arng_name_in_scope_NameInScope_7: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Map {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=_1, .scope=_0, .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###,
        ),
        afun: {
            fn __f(__v: DDValue) -> Option<(DDValue, DDValue)> {
                let __cloned = __v.clone();
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: ref _1,
                        scope: ref _0,
                        declared: _,
                    } => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                    _ => None,
                }
                .map(|x| (x, __cloned))
            }
            __f
        },
        queryable: true,
    });
pub static __Arng_name_in_scope_NameInScope_8: ::once_cell::sync::Lazy<program::Arrangement> =
    ::once_cell::sync::Lazy::new(|| program::Arrangement::Map {
        name: std::borrow::Cow::from(
            r###"(name_in_scope::NameInScope{.name=(_: internment::Intern<string>), .scope=_0, .declared=(_: ast::AnyId)}: name_in_scope::NameInScope) /*join*/"###,
        ),
        afun: {
            fn __f(__v: DDValue) -> Option<(DDValue, DDValue)> {
                let __cloned = __v.clone();
                match <NameInScope>::from_ddvalue(__v) {
                    NameInScope {
                        name: _,
                        scope: ref _0,
                        declared: _,
                    } => Some(((*_0).clone()).into_ddvalue()),
                    _ => None,
                }
                .map(|x| (x, __cloned))
            }
            __f
        },
        queryable: true,
    });
pub static __Rule_name_in_scope_NeedsSymbolResolution_0: ::once_cell::sync::Lazy<program::Rule> =
    ::once_cell::sync::Lazy::new(
        || /* name_in_scope::NeedsSymbolResolution[(name_in_scope::NeedsSymbolResolution{.file=file}: name_in_scope::NeedsSymbolResolution)] :- config::EnableNoTypeofUndef[(config::EnableNoTypeofUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoTypeofUndefConfig>)}: config::EnableNoTypeofUndef)]. */
                                                                                                                                   program::Rule::CollectionRule {
                                                                                                                                       description: std::borrow::Cow::from("name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoTypeofUndef(.file=file, .config=_)."),
                                                                                                                                       rel: 2,
                                                                                                                                       xform: Some(XFormCollection::FilterMap{
                                                                                                                                                       description: std::borrow::Cow::from("head of name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoTypeofUndef(.file=file, .config=_)."),
                                                                                                                                                       fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                       {
                                                                                                                                                           let ref file = match *<types__config::EnableNoTypeofUndef>::from_ddvalue_ref(&__v) {
                                                                                                                                                               types__config::EnableNoTypeofUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           Some(((NeedsSymbolResolution{file: (*file).clone()})).into_ddvalue())
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(None)
                                                                                                                                                   })
                                                                                                                                   },
    );
pub static __Rule_name_in_scope_NeedsSymbolResolution_1: ::once_cell::sync::Lazy<program::Rule> =
    ::once_cell::sync::Lazy::new(
        || /* name_in_scope::NeedsSymbolResolution[(name_in_scope::NeedsSymbolResolution{.file=file}: name_in_scope::NeedsSymbolResolution)] :- config::EnableNoUndef[(config::EnableNoUndef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUndefConfig>)}: config::EnableNoUndef)]. */
                                                                                                                                   program::Rule::CollectionRule {
                                                                                                                                       description: std::borrow::Cow::from("name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoUndef(.file=file, .config=_)."),
                                                                                                                                       rel: 3,
                                                                                                                                       xform: Some(XFormCollection::FilterMap{
                                                                                                                                                       description: std::borrow::Cow::from("head of name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoUndef(.file=file, .config=_)."),
                                                                                                                                                       fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                       {
                                                                                                                                                           let ref file = match *<types__config::EnableNoUndef>::from_ddvalue_ref(&__v) {
                                                                                                                                                               types__config::EnableNoUndef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           Some(((NeedsSymbolResolution{file: (*file).clone()})).into_ddvalue())
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(None)
                                                                                                                                                   })
                                                                                                                                   },
    );
pub static __Rule_name_in_scope_NeedsSymbolResolution_2: ::once_cell::sync::Lazy<program::Rule> =
    ::once_cell::sync::Lazy::new(
        || /* name_in_scope::NeedsSymbolResolution[(name_in_scope::NeedsSymbolResolution{.file=file}: name_in_scope::NeedsSymbolResolution)] :- config::EnableNoUseBeforeDef[(config::EnableNoUseBeforeDef{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUseBeforeDefConfig>)}: config::EnableNoUseBeforeDef)]. */
                                                                                                                                   program::Rule::CollectionRule {
                                                                                                                                       description: std::borrow::Cow::from("name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoUseBeforeDef(.file=file, .config=_)."),
                                                                                                                                       rel: 6,
                                                                                                                                       xform: Some(XFormCollection::FilterMap{
                                                                                                                                                       description: std::borrow::Cow::from("head of name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoUseBeforeDef(.file=file, .config=_)."),
                                                                                                                                                       fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                       {
                                                                                                                                                           let ref file = match *<types__config::EnableNoUseBeforeDef>::from_ddvalue_ref(&__v) {
                                                                                                                                                               types__config::EnableNoUseBeforeDef{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           Some(((NeedsSymbolResolution{file: (*file).clone()})).into_ddvalue())
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(None)
                                                                                                                                                   })
                                                                                                                                   },
    );
pub static __Rule_name_in_scope_NeedsSymbolResolution_3: ::once_cell::sync::Lazy<program::Rule> =
    ::once_cell::sync::Lazy::new(
        || /* name_in_scope::NeedsSymbolResolution[(name_in_scope::NeedsSymbolResolution{.file=file}: name_in_scope::NeedsSymbolResolution)] :- config::EnableNoUnusedVars[(config::EnableNoUnusedVars{.file=(file: ast::FileId), .config=(_: ddlog_std::Ref<config::NoUnusedVarsConfig>)}: config::EnableNoUnusedVars)]. */
                                                                                                                                   program::Rule::CollectionRule {
                                                                                                                                       description: std::borrow::Cow::from("name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoUnusedVars(.file=file, .config=_)."),
                                                                                                                                       rel: 5,
                                                                                                                                       xform: Some(XFormCollection::FilterMap{
                                                                                                                                                       description: std::borrow::Cow::from("head of name_in_scope::NeedsSymbolResolution(.file=file) :- config::EnableNoUnusedVars(.file=file, .config=_)."),
                                                                                                                                                       fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                       {
                                                                                                                                                           let ref file = match *<types__config::EnableNoUnusedVars>::from_ddvalue_ref(&__v) {
                                                                                                                                                               types__config::EnableNoUnusedVars{file: ref file, config: _} => (*file).clone(),
                                                                                                                                                               _ => return None
                                                                                                                                                           };
                                                                                                                                                           Some(((NeedsSymbolResolution{file: (*file).clone()})).into_ddvalue())
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(None)
                                                                                                                                                   })
                                                                                                                                   },
    );
pub fn __apply_80() -> Box<
    dyn for<'a> Fn(
        &mut ::fnv::FnvHashMap<
            program::RelId,
            collection::Collection<
                scopes::Child<'a, worker::Worker<communication::Allocator>, program::TS>,
                DDValue,
                Weight,
            >,
        >,
    ),
> {
    Box::new(|collections| {
        let (name_in_scope_NameInScope, name_in_scope_ScopeOfDeclName) = ResolveSymbols(
            collections.get(&(62)).unwrap(),
            (|__v: DDValue| <NeedsSymbolResolution>::from_ddvalue(__v)),
            collections.get(&(86)).unwrap(),
            (|__v: DDValue| <crate::var_decls::VariableDeclarations>::from_ddvalue(__v)),
            collections.get(&(40)).unwrap(),
            (|__v: DDValue| <types__inputs::InputScope>::from_ddvalue(__v)),
            collections.get(&(27)).unwrap(),
            (|__v: DDValue| <types__inputs::Expression>::from_ddvalue(__v)),
            collections.get(&(43)).unwrap(),
            (|__v: DDValue| <types__inputs::NameRef>::from_ddvalue(__v)),
            collections.get(&(10)).unwrap(),
            (|__v: DDValue| <types__inputs::Assign>::from_ddvalue(__v)),
            collections.get(&(29)).unwrap(),
            (|__v: DDValue| <types__inputs::FileExport>::from_ddvalue(__v)),
            (|v| v.into_ddvalue()),
            (|v| v.into_ddvalue()),
        );
        collections.insert(61, name_in_scope_NameInScope);
        collections.insert(63, name_in_scope_ScopeOfDeclName);
    })
}
