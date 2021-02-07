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
use types__intern::GLOBAL_STRING_INTERNER;

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
use abomonation_derive::Abomonation;
use ddlog_std::{Either, Option as DDlogOption, Ref, Vec as DDlogVec};
use differential_dataflow::{
    collection::Collection,
    difference::{Abelian, Semigroup},
    lattice::Lattice,
    operators::{
        arrange::{Arrange, ArrangeByKey, ArrangeBySelf, Arranged, TraceAgent},
        Consolidate, Iterate, Join, JoinCore, Threshold,
    },
    trace::{
        implementations::ord::{OrdKeySpine, OrdValSpine},
        BatchReader, Cursor, TraceReader,
    },
    AsCollection, Data, ExchangeData, Hashable,
};
use internment::Intern;
use std::{
    fmt::{self, Debug, Display},
    hash::Hash,
    iter, mem,
    num::NonZeroU32,
    ops::Mul,
};
use timely::dataflow::{
    channels::pact::{Exchange, Pipeline},
    operators::Operator,
    Scope, ScopeParent, Stream,
};
use types__ast::{ExportKind, ExprId, FileId, Name, ScopeId};
use types__inputs::{Assign, Expression, FileExport, InputScope, NameRef};

#[allow(clippy::too_many_arguments, non_snake_case)]
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
        .map_named("Map: FilesToResolve", move |file| {
            convert_files_to_resolve(file).file
        })
        .arrange_by_self_exchange_named("ArrangeByKeyExchange: FilesToResolve", |file| {
            file.id as u64
        });

    // Expressions for enabled files, arranged by their expression ids
    let expressions = {
        let exprs_by_file = expressions
            .map_named("Map: Expression", move |file| {
                let expr = convert_expressions(file);
                (expr.id.file, expr)
            })
            // Note: making this into a `.arrange_by_key_exchange()` while
            //       making the post-semijoin arrange pipelined causes
            //       fairly severe of performance loss, although I'm unsure
            //       as to why
            .arrange_by_key_named("ArrangeByKeyExchange: Expression");

        // Only select expressions for which symbol resolution is enabled
        semijoin_arrangements(&exprs_by_file, &files_to_resolve)
            // Key expressions by their `ExprId` and arrange them
            .map(|(_, expr)| (expr.id, expr))
            .arrange_by_key_exchange_named("ArrangeByKeyExchange: Expressions", |file, _| {
                file.id as u64
            })
    };

    let variable_declarations = {
        let variable_declarations = variable_declarations
            .map_named("Map: VariableDeclarations", move |decl| {
                let decl = convert_variable_declarations(decl);
                let file = match decl.scope {
                    DeclarationScope::Unhoistable { scope } => scope.file,
                    DeclarationScope::Hoistable { hoisted, .. } => hoisted.file,
                };

                (file, decl)
            })
            .arrange_by_key_exchange_named(
                "ArrangeByKeyExchange: VariableDeclarations",
                |file, _| file.id as u64,
            );

        semijoin_arrangements(&variable_declarations, &files_to_resolve)
    };

    let input_scopes = {
        let input_scopes = input_scopes
            .map_named("Map: InputScope", move |scope| {
                let scope = convert_input_scopes(scope);
                (scope.parent.file, scope)
            })
            .arrange_by_key_exchange_named("ArrangeByKeyExchange: InputScope", |file, _| {
                file.id as u64
            });

        semijoin_arrangements(&input_scopes, &files_to_resolve)
            .flat_map(|(_, scope)| {
                if scope.parent != scope.child {
                    Some(scope)
                } else {
                    None
                }
            })
            .distinct_pipelined_named("DistinctPipelined: InputScopes")
    };

    let input_scopes_by_parent = input_scopes
        .map(|scope| (scope.parent, scope.child))
        .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: InputScopes by parent");
    let input_scopes_by_child = input_scopes
        .map(|scope| (scope.child, scope.parent))
        .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: InputScopes by child");

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
    .arrange_by_self_pipelined_named("ArrangeBySelfPipelined: Symbol occurrences");

    let scope_of_decl_name = variable_declarations.map(|(_, decl)| {
        let scope = match decl.scope {
            DeclarationScope::Unhoistable { scope } => scope,
            DeclarationScope::Hoistable { hoisted, .. } => hoisted,
        };

        (IStr::new(&*decl.name), scope, decl.declared_in)
    });

    let decl_names_by_name_and_scope = scope_of_decl_name
        .map(|(name, scope, _)| (name, scope))
        .arrange_by_self_pipelined_named("ArrangeBySelfPipelined: Declarations by name and scope");

    let name_in_scope = {
        let concrete_declarations = variable_declarations
            .map(|(_, decl)| {
                let hoisted_scope = match decl.scope {
                    DeclarationScope::Unhoistable { scope } => scope,
                    DeclarationScope::Hoistable { hoisted, .. } => hoisted,
                };

                ((IStr::new(&*decl.name), hoisted_scope), decl.declared_in)
            })
            .arrange_by_key_pipelined_named("ArrangeByKeyPipelined Concrete declarations");

        semijoin_arrangements(&concrete_declarations, &symbol_occurrences)
            .iterate(|names| {
                let blanket_propagations = names
                    .map(|((name, parent), decl)| (parent, (name, decl)))
                    .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Mapped names")
                    .join_core(
                        &input_scopes_by_parent.enter(&names.scope()),
                        |_parent, &(name, decl), &child| iter::once(((name, child), decl)),
                    )
                    .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Blanket propagations");

                let used_propagations = antijoin_arranged(
                    &blanket_propagations,
                    &decl_names_by_name_and_scope.enter(&names.scope()),
                )
                .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Used propagations");

                semijoin_arrangements(
                    &used_propagations,
                    &symbol_occurrences.enter(&names.scope()),
                )
                .concat(&names)
                .distinct_pipelined_named("DistinctPipelined: Iterative names in scope")
            })
            .map(move |((name, scope), declared)| {
                let name = NameInScope {
                    name: Intern::new(name.to_string()),
                    scope,
                    declared,
                };

                convert_name_in_scope(name)
            })
    };

    (
        name_in_scope,
        scope_of_decl_name.map(move |(name, scope, declared)| {
            let decl = ScopeOfDeclName {
                name: Intern::new(name.to_string()),
                scope,
                declared,
            };
            convert_scope_of_decl_name(decl)
        }),
    )
}

/// Collects all usages of symbols within files that need symbol resolution
#[allow(clippy::clippy::too_many_arguments)]
fn collect_name_occurrences<S, D, R, A1, A2, A3, Names, Assigns, Exports>(
    files: &Arranged<S, A1>,
    exprs: &Arranged<S, A2>,
    input_scopes_by_child: &Arranged<S, A3>,

    name_refs: &Collection<S, D, R>,
    convert_name_refs: Names,

    assignments: &Collection<S, D, R>,
    convert_assignments: Assigns,

    file_exports: &Collection<S, D, R>,
    convert_file_exports: Exports,
) -> Collection<S, (IStr, ScopeId), R>
where
    S: Scope,
    S::Timestamp: Lattice,
    R: Semigroup + Abelian + ExchangeData + Mul<Output = R> + From<i8>,
    D: Data,
    A1: TraceReader<Key = FileId, Val = (), Time = S::Timestamp, R = R> + Clone + 'static,
    A2: TraceReader<Key = ExprId, Val = Expression, Time = S::Timestamp, R = R> + Clone + 'static,
    A3: TraceReader<Key = ScopeId, Val = ScopeId, Time = S::Timestamp, R = R> + Clone + 'static,
    Names: Fn(D) -> NameRef + 'static,
    Assigns: Fn(D) -> Assign + 'static,
    Exports: Fn(D) -> FileExport + 'static,
{
    // Only the name references for which symbol resolution is required
    let name_refs = name_refs
        .map_named("Map: NameRef", move |name| {
            let name = convert_name_refs(name);
            (name.expr_id, IStr::new(&*name.value))
        })
        .arrange_by_key_exchange_named("ArrangeByKeyExchange: NameRefs", |expr, _| {
            expr.file.id as u64
        })
        // Join all name references to their corresponding expressions
        // `expr` has already been filtered here to only contain the expressions
        // for which symbol resolution is enabled, so this does double duty
        .join_core(&exprs, |expr_id, &name, expr| {
            iter::once((expr.scope, name))
        });

    let assignments = assignments
        .map_named("Map: Assign", move |assign| {
            let assign = convert_assignments(assign);
            (assign.expr_id, assign)
        })
        .arrange_by_key_exchange_named("ArrangeByKeyExchange: Assignments", |expr, _| expr.file.id as u64)
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
                .map(move |name| (scope, IStr::new(&*name.data)))
        });

    let file_exports = {
        let file_exports = file_exports
            .map_named("Map: FileExport", move |export| {
                let export = convert_file_exports(export);
                (export.scope.file, (export.export, export.scope))
            })
            .arrange_by_key_exchange_named("ArrangeByKeyExchange: FileExport", |file, _| {
                file.id as u64
            });

        semijoin_arrangements(&file_exports, files).flat_map(|(_, (export, scope))| {
            if let ExportKind::NamedExport { name, alias } = export {
                ddlog_std::std2option(alias)
                    .or_else(|| name.into())
                    .map(|name| (scope, IStr::new(&*name.data)))
            } else {
                None
            }
        })
    };

    let symbols = name_refs.concat(&assignments).concat(&file_exports);
    let symbols_arranged = symbols.arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Symbols");

    let propagated_symbols = symbols
        .map(|(child, _)| (child, child))
        .iterate(|transitive_parents| {
            let arranged_parents = transitive_parents
                .map(|(child, parent)| (parent, child))
                .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Arranged parents");

            arranged_parents
                .join_core(
                    &input_scopes_by_child.enter(&transitive_parents.scope()),
                    |_parent, &child, &grandparent| iter::once((child, grandparent)),
                )
                .concat(transitive_parents)
                .distinct_pipelined_named("DistinctPipelined: Iterative propagated symbols")
        })
        .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Propagated symbols");

    symbols_arranged.join_core(&propagated_symbols, |_child, &name, &parent| {
        iter::once((name, parent))
    })
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

fn semijoin_arrangements<S, K, V, R1, R2, A1, A2>(
    values: &Arranged<S, A1>,
    keys: &Arranged<S, A2>,
) -> Collection<S, (K, V), <R1 as Mul<R2>>::Output>
where
    S: Scope,
    S::Timestamp: Lattice,
    K: ExchangeData + Hashable,
    V: ExchangeData,
    R1: ExchangeData + Semigroup + Mul<R2>,
    R2: Semigroup,
    <R1 as Mul<R2>>::Output: Semigroup,
    A1: TraceReader<Key = K, Val = V, Time = S::Timestamp, R = R1> + Clone + 'static,
    A2: TraceReader<Key = K, Val = (), Time = S::Timestamp, R = R2> + Clone + 'static,
{
    values.join_core(keys, |key, value, _| Some((key.clone(), value.clone())))
}

fn semijoin_arranged_pipelined<S, K, V, R, R2, A>(
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
    let arranged_values = values.arrange_by_key_pipelined();
    arranged_values.join_core(keys, |key, value, _| Some((key.clone(), value.clone())))
}

fn semijoin_arranged_exchange<S, K, V, R, R2, A, F>(
    values: &Collection<S, (K, V), R>,
    route: F,
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
    F: Fn(&K, &V) -> u64 + 'static,
{
    let arranged_values = values.arrange_by_key_exchange(route);
    arranged_values.join_core(keys, |key, value, _| Some((key.clone(), value.clone())))
}

fn antijoin_arranged<S, K, V, R, A1, A2>(
    values: &Arranged<S, A1>,
    keys: &Arranged<S, A2>,
) -> Collection<S, (K, V), R>
where
    S: Scope,
    S::Timestamp: Lattice,
    R: Semigroup + Abelian + ExchangeData + Mul<Output = R>,
    K: ExchangeData,
    V: ExchangeData,
    A1: TraceReader<Key = K, Val = V, Time = S::Timestamp, R = R> + Clone + 'static,
    A2: TraceReader<Key = K, Val = (), Time = S::Timestamp, R = R> + Clone + 'static,
{
    let semijoin = values
        .join_core(keys, |key, value, _| Some((key.clone(), value.clone())))
        .negate();

    values
        .as_collection(|key, val| (key.clone(), val.clone()))
        .concat(&semijoin)
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
        self.unary(Pipeline, name.as_ref(), move |_capability, _info| {
            let mut buffer = Vec::new();

            move |input, output| {
                input.for_each(|time, data| {
                    data.swap(&mut buffer);
                    output
                        .session(&time)
                        .give_iterator(buffer.drain(..).map(|x| logic(x)));
                });
            }
        })
    }
}

pub trait ArrangeByKeyExt<K, V> {
    type Output;

    fn arrange_by_key_exchange<F>(&self, route: F) -> Self::Output
    where
        F: Fn(&K, &V) -> u64 + 'static,
    {
        self.arrange_by_key_exchange_named("ArrangeByKeyExchange", route)
    }

    fn arrange_by_key_exchange_named<F>(&self, name: &str, route: F) -> Self::Output
    where
        F: Fn(&K, &V) -> u64 + 'static;

    fn arrange_by_key_pipelined(&self) -> Self::Output {
        self.arrange_by_key_pipelined_named("ArrangeByKeyPipelined")
    }

    fn arrange_by_key_pipelined_named(&self, name: &str) -> Self::Output;
}

impl<S, K, V, R> ArrangeByKeyExt<K, V> for Collection<S, (K, V), R>
where
    S: Scope,
    S::Timestamp: Lattice,
    K: ExchangeData + Hashable,
    V: ExchangeData,
    R: Semigroup + ExchangeData,
{
    #[allow(clippy::type_complexity)]
    type Output = Arranged<S, TraceAgent<OrdValSpine<K, V, S::Timestamp, R>>>;

    fn arrange_by_key_exchange_named<F>(&self, name: &str, route: F) -> Self::Output
    where
        F: Fn(&K, &V) -> u64 + 'static,
    {
        let exchange = Exchange::new(move |((key, value), _time, _diff)| route(key, value));
        self.arrange_core(exchange, name)
    }

    fn arrange_by_key_pipelined_named(&self, name: &str) -> Self::Output {
        self.arrange_core(Pipeline, name)
    }
}

pub trait ArrangeBySelfExt<K> {
    type Output;

    fn arrange_by_self_exchange<F>(&self, route: F) -> Self::Output
    where
        F: Fn(&K) -> u64 + 'static,
    {
        self.arrange_by_self_exchange_named("ArrangeBySelfExchange", route)
    }

    fn arrange_by_self_exchange_named<F>(&self, name: &str, route: F) -> Self::Output
    where
        F: Fn(&K) -> u64 + 'static;

    fn arrange_by_self_pipelined(&self) -> Self::Output {
        self.arrange_by_self_pipelined_named("ArrangeBySelfPipelined")
    }

    fn arrange_by_self_pipelined_named(&self, name: &str) -> Self::Output;
}

impl<S, K, R> ArrangeBySelfExt<K> for Collection<S, K, R>
where
    S: Scope,
    S::Timestamp: Lattice,
    K: ExchangeData + Hashable,
    R: Semigroup + ExchangeData,
{
    type Output = Arranged<S, TraceAgent<OrdKeySpine<K, S::Timestamp, R>>>;

    fn arrange_by_self_exchange_named<F>(&self, name: &str, route: F) -> Self::Output
    where
        F: Fn(&K) -> u64 + 'static,
    {
        let exchange = Exchange::new(move |((key, ()), _time, _diff)| route(key));
        self.map(|key| (key, ())).arrange_core(exchange, name)
    }

    fn arrange_by_self_pipelined_named(&self, name: &str) -> Self::Output {
        self.map(|key| (key, ())).arrange_core(Pipeline, name)
    }
}

pub trait DistinctExt<D, R1, R2 = R1> {
    type Output;

    fn distinct_exchange<F>(&self, route: F) -> Self::Output
    where
        F: Fn(&D) -> u64 + 'static,
    {
        self.distinct_exchange_named("DistinctExchange", route)
    }

    fn distinct_exchange_named<F>(&self, name: &str, route: F) -> Self::Output
    where
        F: Fn(&D) -> u64 + 'static;

    fn distinct_pipelined(&self) -> Self::Output {
        self.distinct_pipelined_named("DistinctPipelined")
    }

    fn distinct_pipelined_named(&self, name: &str) -> Self::Output;
}

impl<S, D, R1, R2> DistinctExt<D, R1, R2> for Collection<S, D, R1>
where
    S: Scope,
    S::Timestamp: Lattice,
    D: ExchangeData + Hashable,
    R1: Semigroup + ExchangeData,
    R2: Semigroup + Abelian + From<i8>,
{
    type Output = Collection<S, D, R2>;

    fn distinct_exchange_named<F>(&self, name: &str, route: F) -> Self::Output
    where
        F: Fn(&D) -> u64 + 'static,
    {
        self.arrange_by_self_exchange(route)
            .threshold_named(name, |_, _| R2::from(1))
    }

    fn distinct_pipelined_named(&self, name: &str) -> Self::Output {
        self.arrange_by_self_pipelined()
            .threshold_named(name, |_, _| R2::from(1))
    }
}

impl<S, K, R1, R2, A> DistinctExt<K, R1, R2> for Arranged<S, A>
where
    S: Scope,
    S::Timestamp: Lattice,
    K: ExchangeData + Hashable,
    R1: Semigroup + ExchangeData,
    R2: Semigroup + Abelian + From<i8>,
    A: TraceReader<Key = K, Val = (), Time = S::Timestamp, R = R1> + Clone + 'static,
    A::Batch: BatchReader<K, (), S::Timestamp, R1>,
    A::Cursor: Cursor<K, (), S::Timestamp, R1>,
{
    type Output = Collection<S, K, R2>;

    fn distinct_exchange_named<F>(&self, name: &str, route: F) -> Self::Output
    where
        F: Fn(&K) -> u64 + 'static,
    {
        self.threshold_named(name, |_, _| R2::from(1))
    }

    fn distinct_pipelined_named(&self, name: &str) -> Self::Output {
        self.threshold_named(name, |_, _| R2::from(1))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Abomonation)]
pub struct IStr(NonZeroU32);

impl IStr {
    pub fn new(string: &str) -> Self {
        let key = unsafe {
            mem::transmute::<_, NonZeroU32>(GLOBAL_STRING_INTERNER.get_or_intern(string))
        };

        Self(key)
    }
}

impl Display for IStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let key = unsafe { mem::transmute::<NonZeroU32, _>(self.0) };

        f.write_str(GLOBAL_STRING_INTERNER.resolve(&key))
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
