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

        semijoin_arranged_exchange(
            &variable_declarations,
            |file, _| file.id as u64,
            &files_to_resolve,
        )
    };

    let input_scopes = {
        let input_scopes = input_scopes.map_named("Marshal InputScopes", move |scope| {
            let scope = convert_input_scopes(scope);
            (scope.parent.file, scope)
        });

        semijoin_arranged(&input_scopes, &files_to_resolve)
            .map(|(_, scope)| scope)
            .filter(|scope| scope.parent != scope.child)
            .distinct_core()
    };

    let input_scopes_by_parent = input_scopes
        .map(|scope| (scope.parent, scope.child))
        .arrange_by_key_exchange(|scope, _| scope.file.id as u64);
    let input_scopes_by_child = input_scopes
        .map(|scope| (scope.child, scope.parent))
        .arrange_by_key_exchange(|scope, _| scope.file.id as u64);

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
    .arrange_by_self_pipelined();

    let scope_of_decl_name = variable_declarations.map(|(_, decl)| {
        let scope = match decl.scope {
            DeclarationScope::Unhoistable { scope } => scope,
            DeclarationScope::Hoistable { hoisted, .. } => hoisted,
        };

        (IStr::new(&*decl.name), scope, decl.declared_in)
    });

    let decl_names_by_name_and_scope = scope_of_decl_name
        .map(|(name, scope, _)| (name, scope))
        .arrange_by_self_exchange(|(_name, scope)| scope.file.id as u64);

    let name_in_scope = {
        let concrete_declarations = variable_declarations.map(|(_, decl)| {
            let hoisted_scope = match decl.scope {
                DeclarationScope::Unhoistable { scope } => scope,
                DeclarationScope::Hoistable { hoisted, .. } => hoisted,
            };

            ((IStr::new(&*decl.name), hoisted_scope), decl.declared_in)
        });

        semijoin_arranged_exchange(
            &concrete_declarations,
            |(_, scope), _| scope.file.id as u64,
            &symbol_occurrences,
        )
        .iterate(|names| {
            let parent_propagations = antijoin_arranged(
                &names
                    .map(|((name, parent), decl)| (parent, (name, decl)))
                    .join_core(
                        &input_scopes_by_parent.enter(&names.scope()),
                        |_parent, &(name, decl), &child| iter::once(((name, child), decl)),
                    )
                    .arrange_by_key_pipelined(),
                &decl_names_by_name_and_scope.enter(&names.scope()),
            );

            semijoin_arranged_pipelined(
                &parent_propagations,
                &symbol_occurrences.enter(&names.scope()),
            )
            .concat(&names)
            .distinct_pipelined()
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
        .map_named("Marshal & Key NameRefs by ExprId", move |name| {
            let name = convert_name_refs(name);
            (name.expr_id, IStr::new(&*name.value))
        })
        // Join all name references to their corresponding expressions
        // `expr` has already been filtered here to only contain the expressions
        // for which symbol resolution is enabled, so this does double duty
        .join_core(&exprs, |expr_id, &name, expr| {
            iter::once((expr.scope, name))
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
                .map(move |name| (scope, IStr::new(&*name.data)))
        });

    let file_exports = {
        let file_exports =
            file_exports.map_named("Marshal & Key FileExports by FileId", move |export| {
                let export = convert_file_exports(export);
                (export.scope.file, (export.export, export.scope))
            });

        semijoin_arranged_exchange(&file_exports, |file, _| file.id as u64, files).flat_map(
            |(_, (export, scope))| {
                if let ExportKind::NamedExport { name, alias } = export {
                    ddlog_std::std2option(alias)
                        .or_else(|| name.into())
                        .map(|name| (scope, IStr::new(&*name.data)))
                } else {
                    None
                }
            },
        )
    };

    let symbols = name_refs.concat(&assignments).concat(&file_exports);
    let symbols_arranged = symbols.arrange_by_key_exchange(|scope, _| scope.file.id as u64);

    let propagated_symbols = symbols
        .map(|(child, _)| (child, child))
        .iterate(|transitive_parents| {
            let arranged_parents = transitive_parents
                .map(|(child, parent)| (parent, child))
                .arrange_by_key_exchange(|scope, _| scope.file.id as u64);

            arranged_parents
                .join_core(
                    &input_scopes_by_child.enter(&transitive_parents.scope()),
                    |_parent, &child, &grandparent| iter::once((child, grandparent)),
                )
                .concat(transitive_parents)
                .distinct_pipelined()
        })
        .arrange_by_key_pipelined();

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
