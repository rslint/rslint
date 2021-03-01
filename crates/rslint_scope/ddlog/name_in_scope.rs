
use crate::var_decls::{DeclarationScope, VariableDeclarations};
use abomonation_derive::Abomonation;
use ddlog_std::{Either, Option as DDlogOption, Ref, Vec as DDlogVec};
use differential_dataflow::{
    collection::Collection,
    difference::{Abelian, Semigroup},
    input::Input,
    lattice::Lattice,
    operators::{
        arrange::{Arrange, ArrangeByKey, ArrangeBySelf, Arranged, TraceAgent},
        iterate::Variable,
        reduce::ReduceCore,
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
    convert::TryInto,
    fmt::{self, Debug, Display},
    hash::Hash,
    iter, mem,
    num::NonZeroU32,
    ops::Mul,
};
use timely::{
    dataflow::{
        channels::{
            pact::{Exchange, Pipeline},
            pushers::{buffer::Session, Tee},
        },
        operators::{
            generic::{builder_rc::OperatorBuilder, OutputHandle},
            Operator,
        },
        Scope, ScopeParent, Stream,
    },
    order::Product,
};
use types__ast::{AnyId, ExportKind, ExprId, FileId, Name, ScopeId};
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
    KnownDecls,
    LegalGlobals,
    IllegalGlobals,
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

    convert_known_declarations: KnownDecls,
    convert_legal_implicit_globals: LegalGlobals,
    convert_illegal_implicit_globals: IllegalGlobals,
) -> (Collection<S, D, Weight>, Collection<S, D, Weight>, Collection<S, D, Weight>)
where
    S: Scope + Input,
    S::Timestamp: Lattice,
    D: Data,
    Files: Fn(D) -> NeedsSymbolResolution + 'static,
    Vars: Fn(D) -> VariableDeclarations + 'static,
    Scopes: Fn(D) -> InputScope + 'static,
    Exprs: Fn(D) -> Expression + 'static,
    Names: Fn(D) -> NameRef + 'static,
    Assigns: Fn(D) -> Assign + 'static,
    Exports: Fn(D) -> FileExport + 'static,
    KnownDecls: Fn(ResolvedName) -> D + 'static,
    LegalGlobals: Fn(ResolvedName) -> D + 'static,
    IllegalGlobals: Fn(ResolvedName) -> D + 'static,
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
            .arrange_by_key_exchange_named("ArrangeByKeyExchange: Expressions", |file, _| {
                file.id as u64
            });

        // Only select expressions for which symbol resolution is enabled
        semijoin_arrangements(&exprs_by_file, &files_to_resolve)
            // Key expressions by their `ExprId` and arrange them
            .map(|(_, expr)| (expr.id, expr))
            .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Expressions")
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
    let child_input_scopes = input_scopes
        .map(|scope| scope.child)
        .arrange_by_self_pipelined_named("ArrangeByKeyPipelined: InputScope children");

    let symbol_usages = collect_name_usages(
        &files_to_resolve,
        &expressions,
        &input_scopes_by_child,
        name_refs,
        convert_name_refs,
        assignments,
        convert_assignments,
        file_exports,
        convert_file_exports,
    );

    let scope_of_decl_name = variable_declarations.map(|(_, decl)| {
        let scope = match decl.scope {
            DeclarationScope::Unhoistable { scope } => scope,
            DeclarationScope::Hoistable { hoisted, .. } => hoisted,
        };

        ((IStr::new(&*decl.name), scope), decl.declared_in)
    });

    let decl_names_by_name_and_scope_with_declarations = scope_of_decl_name
        .map(|((name, scope), declaration)| ((name, scope), declaration))
        .arrange_by_key_pipelined_named(
            "ArrangeByKeyPipelined: Declarations by name and scope with declarations",
        );

    // let (_, use_strict) = symbol_usages.scope().new_collection();
    let use_strict_arranged =
        symbol_usages.arrange_by_self_pipelined_named("ArrangeBySelfPipelined: Use strict scopes");

    let (known_declarations, legal_implicit_globals, illegal_implicit_globals) = symbol_usages
        .scope()
        .scoped::<Product<_, i32>, _, _>("Resolve variable usages", |subgraph| {
            let usages = Variable::new_from(
                symbol_usages
                    .map(|(name, scope)| ResolvedName {
                        name,
                        scope_of_usage: scope,
                        declaration: DeclarationKind::Unknown(scope),
                    })
                    .enter(subgraph),
                Product::new(Default::default(), 1)
            );
            let decl_names_by_name_and_scope = decl_names_by_name_and_scope_with_declarations.enter(subgraph);
            let input_scopes_by_child = input_scopes_by_child.enter(subgraph);
            let child_input_scopes = child_input_scopes.enter(subgraph);

            // Partition by the # of variants in `DeclarationKind`
            let [unknown, unknown_strict, legal_implicit_globals, illegal_implicit_globals, explicit]: [Collection<_, _, _>; 5] = usages.partition(5, |usage| {
                let route = match usage.declaration {
                    DeclarationKind::Unknown(_) => 0,
                    DeclarationKind::UnknownInStrict(_) => 1,
                    DeclarationKind::LegalImplicitGlobal => 2,
                    DeclarationKind::IllegalImplicitGlobal => 3,
                    DeclarationKind::Explicit { .. } => 4,
                };

                (route, usage)
            })
            .try_into()
            .unwrap_or_else(|_| unreachable!("incorrect partition number"));

            let (known_declarations_via_unknown, propagated_unknowns, legal_implicit_globals) = {
                let unknown = unknown
                    .map(|usage| {
                        ((usage.name, usage.declaration.as_unknown().unwrap()), usage)
                    });

                let unknown_arranged = unknown
                    .arrange_by_key_pipelined_named("ArrangeBySelfPipelined: Unknown symbol usages");

                let known_declarations = &unknown_arranged
                    .join_core(&decl_names_by_name_and_scope, |&(name, declaration_scope), &usage, &declaration_id| {
                        let key = (name, declaration_scope);
                        let resolved = ResolvedName {
                            declaration: DeclarationKind::Explicit {
                                item_id: declaration_id,
                                scope: declaration_scope,
                            },
                            ..usage
                        };

                        iter::once((key, resolved))
                    });

                // Antijoin to collect the usages that we couldn't find declarations for
                let still_unknown = antijoin_arranged(
                    &unknown_arranged,
                    &known_declarations
                        .map(|(key, _)| key)
                        .arrange_by_self_pipelined_named("ArrangeBySelfPipelined: Known declarations"),
                );

                let to_be_propagated_unknowns = still_unknown
                    .map(|((name, scope), usage)| (scope, usage));
                let to_be_propagated_unknowns_arranged = to_be_propagated_unknowns
                    .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Propagated unknowns");

                let propagated_unknowns = to_be_propagated_unknowns_arranged
                    .join_core(&input_scopes_by_child, |_child, &usage, &parent| {
                        let resolved = ResolvedName {
                            declaration:  DeclarationKind::Unknown(parent),
                            ..usage
                        };

                        iter::once(resolved)
                    });

                let implicit_globals = antijoin_arranged(
                    &to_be_propagated_unknowns_arranged,
                    &child_input_scopes,
                )
                .map(|(scope, usage)| {
                    let implicit = ResolvedName {
                        declaration: DeclarationKind::LegalImplicitGlobal,
                        ..usage
                    };

                    ((usage.name, scope), usage)
                });

                (
                    known_declarations
                        .map(|(_, usage)| usage),
                    propagated_unknowns,
                    implicit_globals,
                )
            };
            
            let (known_declarations_via_unknown_strict, propagated_strict_unknowns, illegal_implicit_globals) = {
                let strict_unknown = unknown
                    .map(|usage| {
                        ((usage.name, usage.declaration.as_unknown().unwrap()), usage)
                    });

                let strict_unknown_arranged = strict_unknown
                    .arrange_by_key_pipelined_named("ArrangeBySelfPipelined: Strict unknown symbol usages");

                let known_declarations = &strict_unknown_arranged
                    .join_core(&decl_names_by_name_and_scope, |&(name, declaration_scope), &usage, &declaration_id| {
                        let key = (name, declaration_scope);
                        let resolved = ResolvedName {
                            declaration: DeclarationKind::Explicit {
                                item_id: declaration_id,
                                scope: declaration_scope,
                            },
                            ..usage
                        };

                        iter::once((key, resolved))
                    });

                // Antijoin to collect the usages that we couldn't find declarations for
                let still_strict_unknown = antijoin_arranged(
                    &strict_unknown_arranged,
                    &known_declarations
                        .map(|(key, _)| key)
                        .arrange_by_self_pipelined_named("ArrangeBySelfPipelined: Strict known declarations"),
                );

                let to_be_propagated_strict_unknowns = still_strict_unknown
                    .map(|((name, scope), usage)| (scope, usage));
                let to_be_propagated_strict_unknowns_arranged = to_be_propagated_strict_unknowns
                    .arrange_by_key_pipelined_named("ArrangeByKeyPipelined: Propagated strict unknowns");

                let propagated_strict_unknowns = to_be_propagated_strict_unknowns_arranged
                    .join_core(&input_scopes_by_child, |_child, &usage, &parent| {
                        let resolved = ResolvedName {
                            declaration:  DeclarationKind::UnknownInStrict(parent),
                            ..usage
                        };

                        iter::once(resolved)
                    });

                let illegal_implicit_globals = antijoin_arranged(
                    &to_be_propagated_strict_unknowns_arranged,
                    &child_input_scopes,
                )
                .map(|(scope, usage)| {
                    let implicit = ResolvedName {
                        declaration: DeclarationKind::IllegalImplicitGlobal,
                        ..usage
                    };

                    ((usage.name, scope), usage)
                });

                (
                    known_declarations
                        .map(|(_, usage)| usage),
                    propagated_strict_unknowns,
                    illegal_implicit_globals,
                )
            };

            usages.set(&differential_dataflow::collection::concatenate(subgraph, vec![
                propagated_unknowns,
                propagated_strict_unknowns,
            ]));

            (
                differential_dataflow::collection::concatenate(subgraph, vec![
                    known_declarations_via_unknown,
                    known_declarations_via_unknown_strict,
                ])
                .leave(),
                legal_implicit_globals.leave(),
                illegal_implicit_globals.leave(),
            )
        });

    (
        known_declarations.map(convert_known_declarations),
        legal_implicit_globals.map(move |(_, legal)| convert_legal_implicit_globals(legal)),
        illegal_implicit_globals.map(move |(_, illegal)| convert_illegal_implicit_globals(illegal)),
    ) 
}

/// Collects all usages of symbols within files that need symbol resolution
#[allow(clippy::clippy::too_many_arguments)]
fn collect_name_usages<S, D, R, A1, A2, A3, Names, Assigns, Exports>(
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
            iter::once((name, expr.scope))
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
                .map(move |name| (IStr::new(&*name.data),scope, ))
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
                    .map(|name| (IStr::new(&*name.data), scope))
            } else {
                None
            }
        })
    };

     name_refs.concat(&assignments).concat(&file_exports)
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Abomonation)]
enum DeclarationKind {
    Unknown(ScopeId),
    UnknownInStrict(ScopeId),
    LegalImplicitGlobal,
    IllegalImplicitGlobal,
    Explicit { item_id: AnyId, scope: ScopeId },
}

impl DeclarationKind {
    pub const fn as_unknown(self) -> Option<ScopeId> {
        if let DeclarationKind::Unknown(scope) = self {
            Some(scope)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Abomonation)]
struct ResolvedName {
    name: IStr,
    scope_of_usage: ScopeId,
    declaration: DeclarationKind,
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
        self.arrange_by_self_exchange_named(&format!("ArrangeBySelfExchange: {}", name), route)
            .threshold_named(name, |_, _| R2::from(1))
    }

    fn distinct_pipelined_named(&self, name: &str) -> Self::Output {
        self.arrange_by_self_pipelined_named(&format!("ArrangeBySelfPipelined: {}", name))
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

pub trait PartitionExt<D1, D2 = D1> {
    type Output;

    fn partition<F>(&self, parts: usize, route: F) -> Vec<Self::Output>
    where
        F: Fn(D1) -> (usize, D2) + 'static,
    {
        self.partition_named("Partition", parts, route)
    }

    fn partition_named<F>(&self, name: &str, parts: usize, route: F) -> Vec<Self::Output>
    where
        F: Fn(D1) -> (usize, D2) + 'static;
}

type Bundle<S, D, R> = (D, <S as ScopeParent>::Timestamp, R);
type ActivatedOut<'a, S, D, R> = OutputHandle<
    'a,
    <S as ScopeParent>::Timestamp,
    Bundle<S, D, R>,
    Tee<<S as ScopeParent>::Timestamp, Bundle<S, D, R>>,
>;
type SessionOut<'a, S, D, R> = Session<
    'a,
    <S as ScopeParent>::Timestamp,
    Bundle<S, D, R>,
    Tee<<S as ScopeParent>::Timestamp, Bundle<S, D, R>>,
>;

impl<S, D1, D2, R> PartitionExt<D1, D2> for Collection<S, D1, R>
where
    S: Scope,
    D1: Data,
    D2: Data,
    R: Semigroup,
{
    type Output = Collection<S, D2, R>;

    fn partition_named<F>(&self, name: &str, parts: usize, route: F) -> Vec<Self::Output>
    where
        F: Fn(D1) -> (usize, D2) + 'static,
    {
        let mut builder = OperatorBuilder::new(name.to_owned(), self.scope());
        let mut input = builder.new_input(&self.inner, Pipeline);

        let (mut outputs, mut streams) = (Vec::new(), Vec::new());

        for _ in 0..parts {
            let (output, stream) = builder.new_output();
            outputs.push(output);
            streams.push(Collection::new(stream));
        }

        builder.build(move |_| {
            let mut vector = Vec::new();
            move |_frontiers| {
                let (mut handles, mut sessions) = (
                    Vec::<ActivatedOut<S, D2, R>>::with_capacity(outputs.len()),
                    Vec::<SessionOut<S, D2, R>>::with_capacity(outputs.len()),
                );

                for handle in outputs.iter_mut() {
                    handles.push(handle.activate());
                }

                input.for_each(|time, data| {
                    data.swap(&mut vector);
                    sessions.extend(
                        handles
                            .iter_mut()
                            // Safety: This allows us to reuse the `sessions` vector for each input batch,
                            //         it's alright because we clear the sessions buffer at the end of each
                            //         input batch
                            .map(|handle| unsafe { mem::transmute(handle.session(&time)) }),
                    );

                    for (data, time, diff) in vector.drain(..) {
                        let (part, data) = route(data);
                        sessions[part as usize].give((data, time, diff));
                    }

                    sessions.clear();
                });
            }
        });

        streams
    }
}
