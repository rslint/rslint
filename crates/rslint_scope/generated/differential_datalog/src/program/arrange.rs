//! Relation arrangements and transformations

use crate::{
    ddval::DDValue,
    program::{ArrId, TKeyAgent, TKeyEnter, TSNested, TValAgent, TValEnter, Weight},
};
use differential_dataflow::{
    difference::{Diff, Monoid},
    hashable::Hashable,
    lattice::Lattice,
    operators::{arrange::arrangement::Arranged, JoinCore},
    trace::{BatchReader, Cursor, TraceReader},
    AsCollection, Collection, Data,
};
use fnv::FnvHashMap;
use std::ops::Mul;
use timely::{
    dataflow::{
        operators::Concatenate,
        scopes::{Child, Scope, ScopeParent},
    },
    order::Product,
    progress::{timestamp::Refines, Timestamp},
};

pub(super) enum ArrangedCollection<S, T1, T2>
where
    S: Scope,
    S::Timestamp: Lattice + Ord,
    T1: TraceReader<Key = DDValue, Val = DDValue, Time = S::Timestamp, R = Weight> + Clone,
    T1::Batch: BatchReader<DDValue, DDValue, S::Timestamp, Weight>,
    T1::Cursor: Cursor<DDValue, DDValue, S::Timestamp, Weight>,
    T2: TraceReader<Key = DDValue, Val = (), Time = S::Timestamp, R = Weight> + Clone,
    T2::Batch: BatchReader<DDValue, (), S::Timestamp, Weight>,
    T2::Cursor: Cursor<DDValue, (), S::Timestamp, Weight>,
{
    Map(Arranged<S, T1>),
    Set(Arranged<S, T2>),
}

impl<S> ArrangedCollection<S, TValAgent<S>, TKeyAgent<S>>
where
    S: Scope,
    S::Timestamp: Lattice + Ord,
{
    pub(super) fn enter<'a>(
        &self,
        inner: &Child<'a, S, Product<S::Timestamp, TSNested>>,
    ) -> ArrangedCollection<
        Child<'a, S, Product<S::Timestamp, TSNested>>,
        TValEnter<S, Product<S::Timestamp, TSNested>>,
        TKeyEnter<S, Product<S::Timestamp, TSNested>>,
    > {
        match self {
            ArrangedCollection::Map(arr) => ArrangedCollection::Map(arr.enter(inner)),
            ArrangedCollection::Set(arr) => ArrangedCollection::Set(arr.enter(inner)),
        }
    }
}

/// Helper type that represents an arranged collection of one of two
/// types (e.g., an arrangement created in a local scope or entered from
/// the parent scope)
pub(super) enum A<'a, 'b, P, T>
where
    P: ScopeParent,
    P::Timestamp: Lattice + Ord,
    T: Refines<P::Timestamp> + Lattice + Timestamp + Ord,
    'a: 'b,
{
    Arrangement1(
        &'b ArrangedCollection<
            Child<'a, P, T>,
            TValAgent<Child<'a, P, T>>,
            TKeyAgent<Child<'a, P, T>>,
        >,
    ),
    Arrangement2(&'b ArrangedCollection<Child<'a, P, T>, TValEnter<'a, P, T>, TKeyEnter<'a, P, T>>),
}

pub(super) struct Arrangements<'a, 'b, P, T>
where
    P: ScopeParent,
    P::Timestamp: Lattice + Ord,
    T: Refines<P::Timestamp> + Lattice + Timestamp + Ord,
    'a: 'b,
{
    pub(super) arrangements1: &'b FnvHashMap<
        ArrId,
        ArrangedCollection<Child<'a, P, T>, TValAgent<Child<'a, P, T>>, TKeyAgent<Child<'a, P, T>>>,
    >,
    pub(super) arrangements2: &'b FnvHashMap<
        ArrId,
        ArrangedCollection<Child<'a, P, T>, TValEnter<'a, P, T>, TKeyEnter<'a, P, T>>,
    >,
}

impl<'a, 'b, P, T> Arrangements<'a, 'b, P, T>
where
    P: ScopeParent,
    P::Timestamp: Lattice + Ord,
    T: Refines<P::Timestamp> + Lattice + Timestamp + Ord,
    'a: 'b,
{
    pub(super) fn lookup_arr(&self, arrid: ArrId) -> A<'a, 'b, P, T> {
        self.arrangements1.get(&arrid).map_or_else(
            || {
                self.arrangements2
                    .get(&arrid)
                    .map(|arr| A::Arrangement2(arr))
                    .unwrap_or_else(|| panic!("mk_rule: unknown arrangement {:?}", arrid))
            },
            |arr| A::Arrangement1(arr),
        )
    }
}

// Versions of semijoin and antijoin operators that take arrangement instead of collection.
fn semijoin_arranged<G, K, V, R1, R2, T1, T2>(
    arranged: &Arranged<G, T1>,
    other: &Arranged<G, T2>,
) -> Collection<G, (K, V), <R1 as Mul<R2>>::Output>
where
    G: Scope,
    G::Timestamp: Lattice + Ord,
    T1: TraceReader<Key = K, Val = V, Time = G::Timestamp, R = R1> + Clone + 'static,
    T1::Batch: BatchReader<K, V, G::Timestamp, R1>,
    T1::Cursor: Cursor<K, V, G::Timestamp, R1>,
    T2: TraceReader<Key = K, Val = (), Time = G::Timestamp, R = R2> + Clone + 'static,
    T2::Batch: BatchReader<K, (), G::Timestamp, R2>,
    T2::Cursor: Cursor<K, (), G::Timestamp, R2>,
    K: Data + Hashable,
    V: Data,
    R2: Diff,
    R1: Diff + Mul<R2>,
    <R1 as Mul<R2>>::Output: Diff,
{
    arranged.join_core(other, |k, v, _| Some((k.clone(), v.clone())))
}

pub(super) fn antijoin_arranged<G, K, V, R1, R2, T1, T2>(
    arranged: &Arranged<G, T1>,
    other: &Arranged<G, T2>,
) -> Collection<G, (K, V), R1>
where
    G: Scope,
    G::Timestamp: Lattice + Ord,
    T1: TraceReader<Key = K, Val = V, Time = G::Timestamp, R = R1> + Clone + 'static,
    T1::Batch: BatchReader<K, V, G::Timestamp, R1>,
    T1::Cursor: Cursor<K, V, G::Timestamp, R1>,
    T2: TraceReader<Key = K, Val = (), Time = G::Timestamp, R = R2> + Clone + 'static,
    T2::Batch: BatchReader<K, (), G::Timestamp, R2>,
    T2::Cursor: Cursor<K, (), G::Timestamp, R2>,
    K: Data + Hashable,
    V: Data,
    R2: Diff,
    R1: Diff + Mul<R2, Output = R1>,
{
    arranged
        .as_collection(|k, v| (k.clone(), v.clone()))
        .concat(&semijoin_arranged(arranged, other).negate())
}

// TODO: remove when `fn concatenate()` in `collection.rs` makes it to a released version of DD
pub fn concatenate_collections<G, D, R, I>(scope: &mut G, iterator: I) -> Collection<G, D, R>
where
    G: Scope,
    D: Data,
    R: Monoid,
    I: IntoIterator<Item = Collection<G, D, R>>,
{
    scope
        .concatenate(iterator.into_iter().map(|x| x.inner))
        .as_collection()
}
