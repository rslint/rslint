//! Datalog program.
//!
//! The client constructs a `struct Program` that describes Datalog relations and rules and
//! calls `Program::run()` to instantiate the program.  The method returns an error or an
//! instance of `RunningProgram` that can be used to interact with the program at runtime.
//! Interactions include starting, committing or rolling back a transaction and modifying input
//! relations. The engine invokes user-provided callbacks as records are added or removed from
//! relations. `RunningProgram::stop()` terminates the Datalog program destroying all its state.
//! If not invoked manually (which allows for manual error handling), `RunningProgram::stop`
//! will be called when the program object leaves scope.

// TODO: namespace cleanup
// TODO: single input relation

mod arrange;
mod timestamp;
mod update;

pub use arrange::concatenate_collections;
pub use timestamp::{TSNested, TupleTS, TS, TS16};
pub use update::Update;

use crate::{ddval::*, profile::*, record::Mutator, variable::*};
use arrange::{antijoin_arranged, ArrangedCollection, Arrangements, A};
use fnv::{FnvBuildHasher, FnvHashMap, FnvHashSet};
use std::{
    collections::{hash_map, BTreeMap, BTreeSet, HashMap},
    fmt::{self, Debug, Formatter},
    iter,
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Barrier, Mutex,
    },
    thread,
    time::Duration,
};
use timestamp::{TSAtomic, ToTupleTS};

use differential_dataflow::difference::Semigroup;
use differential_dataflow::input::{Input, InputSession};
use differential_dataflow::lattice::Lattice;
use differential_dataflow::logging::DifferentialEvent;
use differential_dataflow::operators::arrange::arrangement::Arranged;
use differential_dataflow::operators::arrange::*;
use differential_dataflow::operators::*;
use differential_dataflow::trace::implementations::ord::OrdKeySpine as DefaultKeyTrace;
use differential_dataflow::trace::implementations::ord::OrdValSpine as DefaultValTrace;
use differential_dataflow::trace::wrappers::enter::TraceEnter;
use differential_dataflow::trace::{BatchReader, Cursor, TraceReader};
// use differential_dataflow::trace::cursor::CursorDebug;
use differential_dataflow::Collection;
use timely::communication::initialize::Configuration;
use timely::communication::Allocator;
use timely::dataflow::operators::*;
use timely::dataflow::scopes::*;
use timely::dataflow::ProbeHandle;
use timely::logging::TimelyEvent;
use timely::order::TotalOrder;
use timely::progress::{timestamp::Refines, Timestamp};
use timely::worker::Worker;

type ValTrace<S> = DefaultValTrace<DDValue, DDValue, <S as ScopeParent>::Timestamp, Weight, u32>;
type KeyTrace<S> = DefaultKeyTrace<DDValue, <S as ScopeParent>::Timestamp, Weight, u32>;

type TValAgent<S> = TraceAgent<ValTrace<S>>;
type TKeyAgent<S> = TraceAgent<KeyTrace<S>>;

type TValEnter<'a, P, T> = TraceEnter<TValAgent<P>, T>;
type TKeyEnter<'a, P, T> = TraceEnter<TKeyAgent<P>, T>;

/// Diff associated with records in differential dataflow
pub type Weight = i32;

/// Message buffer for profiling messages
const PROF_MSG_BUF_SIZE: usize = 10000;

/// Result type returned by this library
pub type Response<X> = Result<X, String>;

/// Unique identifier of a DDlog relation.
pub type RelId = usize;

/// Unique identifier of an index.
pub type IdxId = usize;

/// Unique identifier of an arranged relation.
/// The first element of the tuple identifies relation; the second is the index
/// of arrangement for the given relation.
pub type ArrId = (RelId, usize);

/// Function type used to map the content of a relation
/// (see `XFormCollection::Map`).
pub type MapFunc = fn(DDValue) -> DDValue;

/// (see `XFormCollection::FlatMap`).
pub type FlatMapFunc = fn(DDValue) -> Option<Box<dyn Iterator<Item = DDValue>>>;

/// Function type used to filter a relation
/// (see `XForm*::Filter`).
pub type FilterFunc = fn(&DDValue) -> bool;

/// Function type used to simultaneously filter and map a relation
/// (see `XFormCollection::FilterMap`).
pub type FilterMapFunc = fn(DDValue) -> Option<DDValue>;

/// Function type used to inspect a relation
/// (see `XFormCollection::InspectFunc`)
pub type InspectFunc = fn(&DDValue, TupleTS, Weight) -> ();

/// Function type used to arrange a relation into key-value pairs
/// (see `XFormArrangement::Join`, `XFormArrangement::Antijoin`).
pub type ArrangeFunc = fn(DDValue) -> Option<(DDValue, DDValue)>;

/// Function type used to assemble the result of a join into a value.
/// Takes join key and a pair of values from the two joined relation
/// (see `XFormArrangement::Join`).
pub type JoinFunc = fn(&DDValue, &DDValue, &DDValue) -> Option<DDValue>;

/// Function type used to assemble the result of a semijoin into a value.
/// Takes join key and value (see `XFormArrangement::Semijoin`).
pub type SemijoinFunc = fn(&DDValue, &DDValue, &()) -> Option<DDValue>;

/// Aggregation function: aggregates multiple values into a single value.
pub type AggFunc = fn(&DDValue, &[(&DDValue, Weight)]) -> Option<DDValue>;

// TODO: add validating constructor for Program:
// - relation id's are unique
// - rules only refer to previously declared relations or relations in the local scc
// - input relations do not occur in LHS of rules
// - all references to arrangements are valid
/// A Datalog program is a vector of nodes representing
/// individual non-recursive relations and strongly connected components
/// comprised of one or more mutually recursive relations.
#[derive(Clone)]
pub struct Program {
    pub nodes: Vec<ProgNode>,
    pub init_data: Vec<(RelId, DDValue)>,
}

type TransformerMap<'a> =
    FnvHashMap<RelId, Collection<Child<'a, Worker<Allocator>, TS>, DDValue, Weight>>;

/// Represents a dataflow fragment implemented outside of DDlog directly in differential-dataflow.
///
/// Takes the set of already constructed collections and modifies this
/// set, adding new collections. Note that the transformer can only be applied in the top scope
/// (`Child<'a, Worker<Allocator>, TS>`), as we currently don't have a way to ensure that the
/// transformer is monotonic and thus it may not converge if used in a nested scope.
pub type TransformerFuncRes = Box<dyn for<'a> Fn(&mut TransformerMap<'a>)>;

/// A function returning a dataflow fragment implemented in differential-dataflow
pub type TransformerFunc = fn() -> TransformerFuncRes;

/// Program node is either an individual non-recursive relation, a transformer application or
/// a vector of one or more mutually recursive relations.
#[derive(Clone)]
pub enum ProgNode {
    Rel { rel: Relation },
    Apply { tfun: TransformerFunc },
    SCC { rels: Vec<RecursiveRelation> },
}

/// Relation computed in a nested scope as a fixed point.
///
/// The `distinct` flag indicates that the `distinct` operator should be applied
/// to the relation before closing the loop to enforce convergence of the fixed
/// point computation.
#[derive(Clone)]
pub struct RecursiveRelation {
    pub rel: Relation,
    pub distinct: bool,
}

pub trait CBFn: FnMut(RelId, &DDValue, Weight) + Send {
    fn clone_boxed(&self) -> Box<dyn CBFn>;
}

impl<T> CBFn for T
where
    T: 'static + Send + Clone + FnMut(RelId, &DDValue, Weight),
{
    fn clone_boxed(&self) -> Box<dyn CBFn> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn CBFn> {
    fn clone(&self) -> Self {
        self.as_ref().clone_boxed()
    }
}

/// Caching mode for input relations only
///
/// `NoCache` - don't cache the contents of the relation.
/// `CacheSet` - cache relation as a set.  Duplicate inserts are
///     ignored (for relations without a key) or fail (for relations
///     with key).
/// `CacheMultiset` - cache relation as a generalized multiset with
///     integer weights.
#[derive(Clone)]
pub enum CachingMode {
    Stream,
    Set,
    Multiset,
}

/// Datalog relation.
///
/// defines a set of rules and a set of arrangements with which this relation is used in
/// rules.  The set of rules can be empty (if this is a ground relation); the set of arrangements
/// can also be empty if the relation is not used in the RHS of any rules.
#[derive(Clone)]
pub struct Relation {
    /// Relation name; does not have to be unique
    pub name: String,
    /// `true` if this is an input relation. Input relations are populated by the client
    /// of the library via `RunningProgram::insert()`, `RunningProgram::delete()` and `RunningProgram::apply_updates()` methods.
    pub input: bool,
    /// Apply distinct_total() to this relation after concatenating all its rules
    pub distinct: bool,
    /// Caching mode (for input relations only).
    pub caching_mode: CachingMode,
    /// If `key_func` is present, this indicates that the relation is indexed with a unique
    /// key computed by key_func
    pub key_func: Option<fn(&DDValue) -> DDValue>,
    /// Unique relation id
    pub id: RelId,
    /// Rules that define the content of the relation.
    /// Input relations cannot have rules.
    /// Rules can only refer to relations introduced earlier in the program as well as relations in the same strongly connected
    /// component.
    pub rules: Vec<Rule>,
    /// Arrangements of the relation used to compute other relations.  Index in this vector
    /// along with relation id uniquely identifies the arrangement (see `ArrId`).
    pub arrangements: Vec<Arrangement>,
    /// Callback invoked when an element is added or removed from relation.
    pub change_cb: Option<Arc<Mutex<Box<dyn CBFn>>>>,
}

/// A Datalog relation or rule can depend on other relations and their
/// arrangements.
#[derive(Copy, PartialEq, Eq, Hash, Debug, Clone)]
pub enum Dep {
    Rel(RelId),
    Arr(ArrId),
}

impl Dep {
    pub fn relid(&self) -> RelId {
        match self {
            Dep::Rel(relid) => *relid,
            Dep::Arr((relid, _)) => *relid,
        }
    }
}

/// Transformations, such as maps, flatmaps, filters, joins, etc. are the building blocks of
/// DDlog rules.
///
/// Different kinds of transformations can be applied only to flat collections,
/// only to arranged collections, or both. We therefore use separate types to represent
/// collection and arrangement transformations.
///
/// Note that differential sometimes allows the same kind of transformation to be applied to both
/// collections and arrangements; however the former is implemented on top of the latter and incurs
/// the additional cost of arranging the collection. We only support the arranged version of these
/// transformations, forcing the user to explicitly arrange the collection if necessary (or, as much
/// as possible, keep the data arranged throughout the chain of transformations).
///
/// `XFormArrangement` - arrangement transformation.
#[derive(Clone)]
pub enum XFormArrangement {
    /// FlatMap arrangement into a collection
    FlatMap {
        description: String,
        fmfun: FlatMapFunc,
        /// Transformation to apply to resulting collection.
        /// `None` terminates the chain of transformations.
        next: Box<Option<XFormCollection>>,
    },
    FilterMap {
        description: String,
        fmfun: FilterMapFunc,
        /// Transformation to apply to resulting collection.
        /// `None` terminates the chain of transformations.
        next: Box<Option<XFormCollection>>,
    },
    /// Aggregate
    Aggregate {
        description: String,
        /// Filter arrangement before grouping
        ffun: Option<FilterFunc>,
        /// Aggregation to apply to each group.
        aggfun: AggFunc,
        /// Apply transformation to the resulting collection.
        next: Box<Option<XFormCollection>>,
    },
    /// Join
    Join {
        description: String,
        /// Filter arrangement before joining
        ffun: Option<FilterFunc>,
        /// Arrangement to join with.
        arrangement: ArrId,
        /// Function used to put together ouput value.
        jfun: JoinFunc,
        /// Join returns a collection: apply `next` transformation to it.
        next: Box<Option<XFormCollection>>,
    },
    /// Semijoin
    Semijoin {
        description: String,
        /// Filter arrangement before joining
        ffun: Option<FilterFunc>,
        /// Arrangement to semijoin with.
        arrangement: ArrId,
        /// Function used to put together ouput value.
        jfun: SemijoinFunc,
        /// Join returns a collection: apply `next` transformation to it.
        next: Box<Option<XFormCollection>>,
    },
    /// Return a subset of values that correspond to keys not present in `arrangement`.
    Antijoin {
        description: String,
        /// Filter arrangement before joining
        ffun: Option<FilterFunc>,
        /// Arrangement to antijoin with
        arrangement: ArrId,
        /// Antijoin returns a collection: apply `next` transformation to it.
        next: Box<Option<XFormCollection>>,
    },
}

impl XFormArrangement {
    pub fn description(&self) -> &str {
        match self {
            XFormArrangement::FlatMap { description, .. } => &description,
            XFormArrangement::FilterMap { description, .. } => &description,
            XFormArrangement::Aggregate { description, .. } => &description,
            XFormArrangement::Join { description, .. } => &description,
            XFormArrangement::Semijoin { description, .. } => &description,
            XFormArrangement::Antijoin { description, .. } => &description,
        }
    }

    pub(super) fn dependencies(&self) -> FnvHashSet<Dep> {
        match self {
            XFormArrangement::FlatMap { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
            XFormArrangement::FilterMap { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
            XFormArrangement::Aggregate { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
            XFormArrangement::Join {
                arrangement, next, ..
            } => {
                let mut deps = match **next {
                    None => FnvHashSet::default(),
                    Some(ref n) => n.dependencies(),
                };
                deps.insert(Dep::Arr(*arrangement));
                deps
            }
            XFormArrangement::Semijoin {
                arrangement, next, ..
            } => {
                let mut deps = match **next {
                    None => FnvHashSet::default(),
                    Some(ref n) => n.dependencies(),
                };
                deps.insert(Dep::Arr(*arrangement));
                deps
            }
            XFormArrangement::Antijoin {
                arrangement, next, ..
            } => {
                let mut deps = match **next {
                    None => FnvHashSet::default(),
                    Some(ref n) => n.dependencies(),
                };
                deps.insert(Dep::Arr(*arrangement));
                deps
            }
        }
    }
}

/// `XFormCollection` - collection transformation.
#[derive(Clone)]
pub enum XFormCollection {
    /// Arrange the collection, apply `next` transformation to the resulting collection.
    Arrange {
        description: String,
        afun: ArrangeFunc,
        next: Box<XFormArrangement>,
    },
    /// Apply `mfun` to each element in the collection
    Map {
        description: String,
        mfun: MapFunc,
        next: Box<Option<XFormCollection>>,
    },
    /// FlatMap
    FlatMap {
        description: String,
        fmfun: FlatMapFunc,
        next: Box<Option<XFormCollection>>,
    },
    /// Filter collection
    Filter {
        description: String,
        ffun: FilterFunc,
        next: Box<Option<XFormCollection>>,
    },
    /// Map and filter
    FilterMap {
        description: String,
        fmfun: FilterMapFunc,
        next: Box<Option<XFormCollection>>,
    },
    /// Inspector
    Inspect {
        description: String,
        ifun: InspectFunc,
        next: Box<Option<XFormCollection>>,
    },
}

impl XFormCollection {
    pub fn description(&self) -> &str {
        match self {
            XFormCollection::Arrange { description, .. } => &description,
            XFormCollection::Map { description, .. } => &description,
            XFormCollection::FlatMap { description, .. } => &description,
            XFormCollection::Filter { description, .. } => &description,
            XFormCollection::FilterMap { description, .. } => &description,
            XFormCollection::Inspect { description, .. } => &description,
        }
    }

    pub fn dependencies(&self) -> FnvHashSet<Dep> {
        match self {
            XFormCollection::Arrange { next, .. } => next.dependencies(),
            XFormCollection::Map { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
            XFormCollection::FlatMap { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
            XFormCollection::Filter { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
            XFormCollection::FilterMap { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
            XFormCollection::Inspect { next, .. } => match **next {
                None => FnvHashSet::default(),
                Some(ref n) => n.dependencies(),
            },
        }
    }
}

/// Datalog rule (more precisely, the body of a rule) starts with a collection
/// or arrangement and applies a chain of transformations to it.
#[derive(Clone)]
pub enum Rule {
    CollectionRule {
        description: String,
        rel: RelId,
        xform: Option<XFormCollection>,
    },
    ArrangementRule {
        description: String,
        arr: ArrId,
        xform: XFormArrangement,
    },
}

impl Rule {
    pub fn description(&self) -> &str {
        match self {
            Rule::CollectionRule { description, .. } => description.as_ref(),
            Rule::ArrangementRule { description, .. } => description.as_ref(),
        }
    }

    fn dependencies(&self) -> FnvHashSet<Dep> {
        match self {
            Rule::CollectionRule { rel, xform, .. } => {
                let mut deps = match xform {
                    None => FnvHashSet::default(),
                    Some(ref x) => x.dependencies(),
                };
                deps.insert(Dep::Rel(*rel));
                deps
            }

            Rule::ArrangementRule { arr, xform, .. } => {
                let mut deps = xform.dependencies();
                deps.insert(Dep::Arr(*arr));
                deps
            }
        }
    }
}

/// Describes arrangement of a relation.
#[derive(Clone)]
pub enum Arrangement {
    /// Arrange into (key,value) pairs
    Map {
        /// Arrangement name; does not have to be unique
        name: String,
        /// Function used to produce arrangement.
        afun: ArrangeFunc,
        /// The arrangement can be queried using `RunningProgram::query_arrangement`
        /// and `RunningProgram::dump_arrangement`.
        queryable: bool,
    },
    /// Arrange into a set of values
    Set {
        /// Arrangement name; does not have to be unique
        name: String,
        /// Function used to produce arrangement.
        fmfun: FilterMapFunc,
        /// Apply distinct_total() before arranging filtered collection.
        /// This is necessary if the arrangement is to be used in an antijoin.
        distinct: bool,
    },
}

impl Arrangement {
    fn name(&self) -> String {
        match self {
            Arrangement::Map { name, .. } => name.clone(),
            Arrangement::Set { name, .. } => name.clone(),
        }
    }

    fn queryable(&self) -> bool {
        match *self {
            Arrangement::Map { queryable, .. } => queryable,
            Arrangement::Set { .. } => false,
        }
    }

    fn build_arrangement_root<S>(
        &self,
        collection: &Collection<S, DDValue, Weight>,
    ) -> ArrangedCollection<S, TValAgent<S>, TKeyAgent<S>>
    where
        S: Scope,
        Collection<S, DDValue, Weight>: ThresholdTotal<S, DDValue, Weight>,
        S::Timestamp: Lattice + Ord + TotalOrder,
    {
        match *self {
            Arrangement::Map { afun, .. } => {
                ArrangedCollection::Map(collection.flat_map(afun).arrange())
            }
            Arrangement::Set {
                fmfun, distinct, ..
            } => {
                let filtered = collection.flat_map(fmfun);
                if distinct {
                    ArrangedCollection::Set(
                        filtered
                            .threshold_total(|_, c| if c.is_zero() { 0 } else { 1 })
                            .map(|k| (k, ()))
                            .arrange(), /* arrange_by_self() */
                    )
                } else {
                    ArrangedCollection::Set(filtered.map(|k| (k, ())).arrange())
                }
            }
        }
    }

    fn build_arrangement<S>(
        &self,
        collection: &Collection<S, DDValue, Weight>,
    ) -> ArrangedCollection<S, TValAgent<S>, TKeyAgent<S>>
    where
        S: Scope,
        S::Timestamp: Lattice + Ord,
    {
        match *self {
            Arrangement::Map { afun, .. } => {
                ArrangedCollection::Map(collection.flat_map(afun).arrange())
            }
            Arrangement::Set {
                fmfun, distinct, ..
            } => {
                let filtered = collection.flat_map(fmfun);
                if distinct {
                    ArrangedCollection::Set(
                        filtered
                            .threshold(|_, c| if c.is_zero() { 0 } else { 1 })
                            .map(|k| (k, ()))
                            .arrange(),
                    )
                } else {
                    ArrangedCollection::Set(filtered.map(|k| (k, ())).arrange())
                }
            }
        }
    }
}

/// Set relation content.
pub type ValSet = FnvHashSet<DDValue>;

/// Multiset relation content.
pub type ValMSet = DeltaSet;

/// Indexed relation content.
pub type IndexedValSet = FnvHashMap<DDValue, DDValue>;

/// Relation delta
pub type DeltaSet = FnvHashMap<DDValue, isize>;

/// Runtime representation of a datalog program.
///
/// The program will be automatically stopped when the object goes out
/// of scope. Error occurring as part of that operation are silently
/// ignored. If you want to handle such errors, call `stop` manually.
pub struct RunningProgram {
    /// Producer sides of channels used to send commands to workers.
    /// We use async channels to avoid deadlocks when workers are blocked
    /// in `step_or_park`.
    senders: Vec<mpsc::Sender<Msg>>,
    /// Channels to receive replies from worker threads. We could use a single
    /// channel with multiple senders, but use many channels instead to avoid
    /// deadlocks when one of the workers has died, but `recv` blocks instead
    /// of failing, since the channel is still considered alive.
    reply_recv: Vec<mpsc::Receiver<Reply>>,
    relations: FnvHashMap<RelId, RelationInstance>,
    /// Join handle of the thread running timely computation.
    thread_handle: Option<thread::JoinHandle<Result<(), String>>>,
    /// Timely worker threads.
    worker_threads: Vec<thread::Thread>,
    transaction_in_progress: bool,
    need_to_flush: bool,
    /// CPU profiling enabled (can be expensive).
    profile_cpu: Arc<AtomicBool>,
    /// Consume timely_events and output them to CSV file. Can be expensive.
    profile_timely: Arc<AtomicBool>,
    /// Profiling thread.
    prof_thread_handle: Option<thread::JoinHandle<()>>,
    /// Profiling statistics.
    pub profile: Arc<Mutex<Profile>>,
}

// Right now this Debug implementation is more or less a short cut.
// Ideally we would want to implement Debug for `RelationInstance`, but
// that quickly gets very cumbersome.
impl Debug for RunningProgram {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunningProgram")
            .field("senders", &self.senders)
            .field("reply_recv", &self.reply_recv)
            .field(
                "relations",
                &(&self.relations as *const FnvHashMap<RelId, RelationInstance>),
            )
            .field("thread_handle", &self.thread_handle)
            .field("transaction_in_progress", &self.transaction_in_progress)
            .field("need_to_flush", &self.need_to_flush)
            .field("profile_cpu", &self.profile_cpu)
            .field("profile_timely", &self.profile_timely)
            .field("prof_thread_handle", &self.prof_thread_handle)
            .field("profile", &self.profile)
            .finish()
    }
}

/// Runtime representation of relation
enum RelationInstance {
    Stream {
        /// Changes since start of transaction.
        delta: DeltaSet,
    },
    Multiset {
        /// Multiset of all elements in the relation.
        elements: ValMSet,
        /// Changes since start of transaction.
        delta: DeltaSet,
    },
    Flat {
        /// Set of all elements in the relation. Used to enforce set semantics for input relations
        /// (repeated inserts and deletes are ignored).
        elements: ValSet,
        /// Changes since start of transaction.
        delta: DeltaSet,
    },
    Indexed {
        key_func: fn(&DDValue) -> DDValue,
        /// Set of all elements in the relation indexed by key. Used to enforce set semantics,
        /// uniqueness of keys, and to query input relations by key.
        elements: IndexedValSet,
        /// Changes since start of transaction.  Only maintained for input relations and is used to
        /// enforce set semantics.
        delta: DeltaSet,
    },
}

impl RelationInstance {
    pub fn delta(&self) -> &DeltaSet {
        match self {
            RelationInstance::Stream { delta } => delta,
            RelationInstance::Multiset { delta, .. } => delta,
            RelationInstance::Flat { delta, .. } => delta,
            RelationInstance::Indexed { delta, .. } => delta,
        }
    }

    pub fn delta_mut(&mut self) -> &mut DeltaSet {
        match self {
            RelationInstance::Stream { delta } => delta,
            RelationInstance::Multiset { delta, .. } => delta,
            RelationInstance::Flat { delta, .. } => delta,
            RelationInstance::Indexed { delta, .. } => delta,
        }
    }
}

/// Messages sent to timely worker threads.  Most of these messages can be sent
/// to worker 0 only.
#[derive(Debug, Clone)]
enum Msg {
    /// Update input relation (worker 0 only).
    Update(Vec<Update<DDValue>>),
    /// Propagate changes through the pipeline (worker 0 only).
    Flush,
    /// Query arrangement.  If the second argument is `None`, returns
    /// all values in the collection; otherwise returns values associated
    /// with the specified key.
    Query(ArrId, Option<DDValue>),
    /// Stop all workers (worker 0 only)
    Stop,
}

/// Reply messages from timely worker threads.
#[derive(Debug)]
enum Reply {
    /// Acknowledge flush completion (sent by worker 0 only).
    FlushAck,
    /// Result of a query.
    QueryRes(Option<BTreeSet<DDValue>>),
}

impl Program {
    /// Instantiate the program with `nworkers` timely threads.
    // REFACTOR: Split this up into functions or something, it's currently a 600 line function
    pub fn run(&self, nworkers: usize) -> Result<RunningProgram, String> {
        // Clone the program, so that it can be moved into the timely computation
        let prog = self.clone();

        // Setup channels to communicate with the dataflow.
        // We use async channels to avoid deadlocks when workers are parked in
        // `step_or_park`.  This has the downside of introducing an unbounded buffer
        // that is only guaranteed to be fully flushed when the transaction commits.
        let (request_send, request_recv): (Vec<_>, Vec<_>) =
            (0..nworkers).map(|_| mpsc::channel::<Msg>()).unzip();
        let request_recv: Arc<Mutex<Vec<Option<_>>>> =
            Arc::new(Mutex::new(request_recv.into_iter().map(Some).collect()));

        // Channels for responses from worker threads.
        let (reply_send, reply_recv): (Vec<_>, Vec<_>) =
            (0..nworkers).map(|_| mpsc::channel::<Reply>()).unzip();
        let reply_send: Arc<Mutex<Vec<Option<_>>>> =
            Arc::new(Mutex::new(reply_send.into_iter().map(Some).collect()));

        let (prof_send, prof_recv) = mpsc::sync_channel::<ProfMsg>(PROF_MSG_BUF_SIZE);

        // Channel used by workers 1..n to send their thread handles to worker 0.
        let (thandle_send, thandle_recv) = mpsc::sync_channel::<(usize, thread::Thread)>(0);
        let thandle_recv = Arc::new(Mutex::new(thandle_recv));

        // Channel used by the main timely thread to send worker 0 handle to the caller.
        let (wsend, wrecv) = mpsc::sync_channel::<Vec<thread::Thread>>(0);

        // Profile data structure
        let profile = Arc::new(Mutex::new(Profile::new()));
        let profile2 = profile.clone();

        let profile_cpu = Arc::new(AtomicBool::new(false));
        let profile_timely = Arc::new(AtomicBool::new(false));

        // Thread to collect profiling data
        let prof_thread = thread::spawn(move || Self::prof_thread_func(prof_recv, profile2));

        let profile_cpu2 = profile_cpu.clone();
        let profile_timely2 = profile_timely.clone();

        // Shared timestamp managed by worker 0 and read by all other workers
        let frontier_ts = TSAtomic::new(0);
        let progress_barrier = Arc::new(Barrier::new(nworkers));

        // We must run the timely computation from a separate thread, since we need this thread to
        // take the ownership of `WorkerGuards`, returned by `timely::execute`.  `WorkerGuards` cannot
        // be sent across threads (i.e., they do not implement `Send` and therefore cannot be
        // safely stored in `RunningProgram`).
        let h = thread::spawn(move ||
            // Start up timely computation.
            timely::execute(Configuration::Process(nworkers), move |worker: &mut Worker<Allocator>| -> Result<_, String> {
                let worker_index = worker.index();

                // Peer workers thread handles; only used by worker 0.
                let mut peers: FnvHashMap<usize, thread::Thread> = FnvHashMap::default();
                if worker_index != 0 {
                    // Send worker's thread handle to worker 0.
                    thandle_send.send((worker_index, thread::current())).unwrap();
                } else {
                    // Worker 0: receive `nworkers-1` handles.
                    let thandle_recv = thandle_recv.lock().unwrap();
                    for _ in 1..nworkers {
                        let (worker, thandle) = thandle_recv.recv().unwrap();
                        peers.insert(worker, thandle);
                    }
                }

                let probe = probe::Handle::new();
                {
                    let mut probe1 = probe.clone();
                    let profile_cpu3 = profile_cpu2.clone();
                    let profile_timely3 = profile_timely2.clone();

                    let prof_send1 = prof_send.clone();
                    let prof_send2 = prof_send.clone();
                    let progress_barrier = progress_barrier.clone();

                    worker.log_register().insert::<TimelyEvent,_>("timely", move |_time, data| {
                        let profcpu: &AtomicBool = &*profile_cpu3;
                        let proftimely: &AtomicBool = &*profile_timely3;

                        // Filter out events we don't care about to avoid the overhead of sending
                        // the event around just to drop it eventually.
                        let filtered: Vec<((Duration, usize, TimelyEvent), Option<String>)> = data
                            .drain(..)
                            .filter(|event| match event.2 {
                                // Always send Operates events as they're used for always-on memory profiling.
                                TimelyEvent::Operates(_) => true,
                                TimelyEvent::Schedule(_) => profcpu.load(Ordering::Acquire) | proftimely.load(Ordering::Acquire),
                                TimelyEvent::GuardedMessage(_) |
                                TimelyEvent::Messages(_) |
                                TimelyEvent::Park(_) |
                                TimelyEvent::Progress(_) |
                                TimelyEvent::PushProgress(_) => proftimely.load(Ordering::Acquire),
                                _ => false
                            })
                            .map(|(d, s, e)| match e {
                                // Only Operate events care about the context string.
                                TimelyEvent::Operates(_) => ((d, s, e), Some(get_prof_context())),
                                _ => ((d, s, e), None),
                            })
                            .collect();

                        if !filtered.is_empty() {
                            //eprintln!("timely event {:?}", filtered);
                            prof_send1
                                .send(ProfMsg::TimelyMessage(
                                    filtered,
                                    profcpu.load(Ordering::Acquire),
                                    proftimely.load(Ordering::Acquire)
                                ))
                                .unwrap();
                        }
                    });

                    worker.log_register().insert::<DifferentialEvent,_>("differential/arrange", move |_time, data| {
                        if data.is_empty() {
                            return;
                        }

                        // Send update to profiling channel
                        prof_send2
                            .send(ProfMsg::DifferentialMessage(data.drain(..).collect()))
                            .unwrap();
                    });

                    let request_recv = {
                        let mut opt_rx = None;
                        std::mem::swap(&mut request_recv.lock().unwrap()[worker_index], &mut opt_rx);
                        opt_rx.unwrap()
                    };

                    let reply_send = {
                        let mut opt_tx = None;
                        std::mem::swap(&mut reply_send.lock().unwrap()[worker_index], &mut opt_tx);
                        opt_tx.unwrap()
                    };

                    let (mut all_sessions, mut traces) = worker.dataflow::<TS,_,_>(|outer: &mut Child<Worker<Allocator>, TS>| -> Result<_, String> {
                        let mut sessions : FnvHashMap<RelId, InputSession<TS, DDValue, Weight>> = FnvHashMap::default();
                        let mut collections : FnvHashMap<RelId, Collection<Child<Worker<Allocator>, TS>,DDValue,Weight>> = FnvHashMap::default();
                        let mut arrangements = FnvHashMap::default();

                        for (nodeid, node) in prog.nodes.iter().enumerate() {
                            match node {
                                ProgNode::Rel{rel} => {
                                    // Relation may already be in the map if it was created by an `Apply` node
                                    let mut collection = collections
                                        .remove(&rel.id)
                                        .unwrap_or_else(|| {
                                            let (session, collection) = outer.new_collection::<DDValue,Weight>();
                                            sessions.insert(rel.id, session);

                                            collection
                                        });

                                    // apply rules
                                    let mut rule_collections: Vec<_> = rel
                                        .rules
                                        .iter()
                                        .map(|rule| {
                                            prog.mk_rule(
                                                rule,
                                                |rid| collections.get(&rid),
                                                Arrangements {
                                                    arrangements1: &arrangements,
                                                    arrangements2: &FnvHashMap::default(),
                                                },
                                            )
                                        })
                                        .collect();

                                    rule_collections.push(collection);
                                    collection = with_prof_context(
                                        &format!("concatenate rules for {}", rel.name),
                                        || concatenate_collections(outer, rule_collections.into_iter()),
                                    );

                                    // don't distinct input collections, as this is already done by the set_update logic
                                    if !rel.input && rel.distinct {
                                        collection = with_prof_context(
                                            &format!("{}.threshold_total", rel.name),
                                            || collection.threshold_total(|_, c| if c.is_zero() { 0 } else { 1 }),
                                        );
                                    }

                                    // create arrangements
                                    for (i,arr) in rel.arrangements.iter().enumerate() {
                                        with_prof_context(
                                            &arr.name(),
                                            || arrangements.insert(
                                                (rel.id, i),
                                                arr.build_arrangement_root(&collection),
                                            ),
                                        );
                                    }

                                    collections.insert(rel.id, collection);
                                },
                                ProgNode::Apply { tfun } => {
                                    tfun()(&mut collections);
                                },
                                ProgNode::SCC { rels } => {
                                    // create collec tions; add them to map; we will overwrite them with
                                    // updated collections returned from the inner scope.
                                    for r in rels.iter() {
                                        let (session, collection) = outer.new_collection::<DDValue,Weight>();
                                        assert!(!r.rel.input, "input relation in nested scope: {}", r.rel.name);

                                        sessions.insert(r.rel.id, session);
                                        collections.insert(r.rel.id, collection);
                                    }

                                    // create a nested scope for mutually recursive relations
                                    let new_collections = outer.scoped("recursive component", |inner| -> Result<_, String> {
                                        // create variables for relations defined in the SCC.
                                        let mut vars = HashMap::with_capacity_and_hasher(rels.len(), FnvBuildHasher::default());
                                        // arrangements created inside the nested scope
                                        let mut local_arrangements = FnvHashMap::default();
                                        // arrangements entered from global scope
                                        let mut inner_arrangements = FnvHashMap::default();

                                        // collections entered from global scope
                                        let mut inner_collections = FnvHashMap::default();
                                        for r in rels.iter() {
                                            let var = Variable::from(
                                                &collections
                                                    .get(&r.rel.id)
                                                    .ok_or_else(|| format!("failed to find collection with relation ID {}", r.rel.id))?
                                                    .enter(inner),
                                                r.distinct,
                                                &r.rel.name,
                                            );

                                            vars.insert(r.rel.id, var);
                                        }

                                        // create arrangements
                                        for rel in rels {
                                            for (i, arr) in rel.rel.arrangements.iter().enumerate() {
                                                // check if arrangement is actually used inside this node
                                                if prog.arrangement_used_by_nodes((rel.rel.id, i)).iter().any(|n| *n == nodeid) {
                                                    with_prof_context(
                                                        &format!("local {}", arr.name()),
                                                        || local_arrangements.insert(
                                                            (rel.rel.id, i),
                                                            arr.build_arrangement(vars.get(&rel.rel.id)?.deref()),
                                                        ),
                                                    );
                                                }
                                            }
                                        }

                                        let relrels: Vec<_> = rels.iter().map(|r| &r.rel).collect();
                                        for dep in Self::dependencies(relrels.as_slice()) {
                                            match dep {
                                                Dep::Rel(relid) => {
                                                    assert!(!vars.contains_key(&relid));
                                                    let collection = collections
                                                        .get(&relid)
                                                        .ok_or_else(|| format!("failed to find collection with relation ID {}", relid))?
                                                        .enter(inner);

                                                    inner_collections.insert(relid, collection);
                                                },
                                                Dep::Arr(arrid) => {
                                                    let arrangement = arrangements
                                                        .get(&arrid)
                                                        .ok_or_else(|| format!("Arr: unknown arrangement {:?}", arrid))?
                                                        .enter(inner);

                                                    inner_arrangements.insert(arrid, arrangement);
                                                }
                                            }
                                        }

                                        // apply rules to variables
                                        for rel in rels {
                                            for rule in &rel.rel.rules {
                                                let c = prog.mk_rule(
                                                    rule,
                                                    |rid| {
                                                        vars
                                                            .get(&rid)
                                                            .map(|v| &(**v))
                                                            .or_else(|| inner_collections.get(&rid))
                                                    },
                                                    Arrangements {
                                                        arrangements1: &local_arrangements,
                                                        arrangements2: &inner_arrangements,
                                                    },
                                                );

                                                vars
                                                    .get_mut(&rel.rel.id)
                                                    .ok_or_else(|| format!("no variable found for relation ID {}", rel.rel.id))?
                                                    .add(&c);
                                            }
                                        }

                                        // bring new relations back to the outer scope
                                        let mut new_collections = FnvHashMap::default();
                                        for rel in rels {
                                            let var = vars
                                                .get(&rel.rel.id)
                                                .ok_or_else(|| format!("no variable found for relation ID {}", rel.rel.id))?;

                                            let mut collection = var.leave();
                                            // var.distinct() will be called automatically by var.drop() if var has `distinct` flag set
                                            if rel.rel.distinct && !rel.distinct {
                                                collection = with_prof_context(
                                                    &format!("{}.distinct_total", rel.rel.name),
                                                    || collection.threshold_total(|_,c| if c.is_zero() { 0 } else { 1 }),
                                                );
                                            }

                                            new_collections.insert(rel.rel.id, collection);
                                        }

                                        Ok(new_collections)
                                    })?;

                                    // add new collections to the map
                                    collections.extend(new_collections);

                                    // create arrangements
                                    for rel in rels {
                                        for (i, arr) in rel.rel.arrangements.iter().enumerate() {
                                            // only if the arrangement is used outside of this node
                                            if prog.arrangement_used_by_nodes((rel.rel.id, i)).iter().any(|n|*n != nodeid) || arr.queryable() {
                                                with_prof_context(
                                                    &format!("global {}", arr.name()),
                                                    || -> Result<_, String> {
                                                        let collection = collections
                                                            .get(&rel.rel.id)
                                                            .ok_or_else(|| format!("no collection found for relation ID {}", rel.rel.id))?;

                                                        Ok(arrangements.insert((rel.rel.id, i), arr.build_arrangement(collection)))
                                                    }
                                                )?;
                                            }
                                        }
                                    }
                                }
                            }
                        };

                        for (relid, collection) in collections {
                            // notify client about changes
                            if let Some(cb) = &prog.get_relation(relid).change_cb {
                                let mut cb = cb.lock().unwrap().clone();

                                let consolidated = with_prof_context(
                                    &format!("consolidate {}", relid),
                                    || collection.consolidate(),
                                );

                                let inspected = with_prof_context(
                                    &format!("inspect {}", relid),
                                    || consolidated.inspect(move |x| {
                                        // assert!(x.2 == 1 || x.2 == -1, "x: {:?}", x);
                                        cb(relid, &x.0, x.2)
                                    }),
                                );

                                with_prof_context(
                                    &format!("probe {}", relid),
                                    || inspected.probe_with(&mut probe1),
                                );
                            }
                        }

                        // Attach probes to index arrangements, so we know when all updates
                        // for a given epoch have been added to the arrangement, and return
                        // arrangement trace.
                        let mut traces: BTreeMap<ArrId, _> = BTreeMap::new();
                        for ((relid, arrid), arr) in arrangements.into_iter() {
                            if let ArrangedCollection::Map(arranged) = arr {
                                if prog.get_relation(relid).arrangements[arrid].queryable() {
                                    arranged.as_collection(|k,_| k.clone()).probe_with(&mut probe1);
                                    traces.insert((relid, arrid), arranged.trace.clone());
                                }
                            }
                        }

                        Ok((sessions, traces))
                    })?;
                    //println!("worker {} started", worker.index());

                    let mut epoch: TS = 0;

                    // feed initial data to sessions
                    if worker_index == 0 {
                        for (relid, v) in prog.init_data.iter() {
                            all_sessions
                              .get_mut(relid)
                              .ok_or_else(|| format!("no session found for relation ID {}", relid))?
                              .update(v.clone(), 1);
                        }

                        epoch += 1;
                        Self::advance(&mut all_sessions, &mut traces, epoch);
                        Self::flush(&mut all_sessions, &probe, worker, &peers, &frontier_ts, &progress_barrier);

                        reply_send.send(Reply::FlushAck).map_err(|e| format!("failed to send ACK: {}", e))?;
                    }

                    // Close session handles for non-input sessions;
                    // close all sessions for workers other than worker 0.
                    let mut sessions: FnvHashMap<RelId, InputSession<TS, DDValue, Weight>> = all_sessions
                        .drain()
                        .filter(|(relid, _)| worker_index == 0 && prog.get_relation(*relid).input)
                        .collect();

                    // Only worker 0 receives data
                    if worker_index == 0 {
                        loop {
                            // Non-blocking receive, so that we can do some garbage collecting
                            // when there is no real work to do.
                            match request_recv.try_recv() {
                                Err(mpsc::TryRecvError::Empty) => {
                                    // Command channel empty: use idle time to work on garbage collection.
                                    // This will block when there is no more compaction left to do.
                                    // The sender must unpark worker 0 after sending to the channel.
                                    worker.step_or_park(None);
                                },
                                Err(mpsc::TryRecvError::Disconnected)  => {
                                    // Sender hung
                                    eprintln!("Sender hung");
                                    Self::stop_workers(&peers, &frontier_ts, &progress_barrier);
                                    break;
                                },
                                Ok(Msg::Update(mut updates)) => {
                                    //println!("updates: {:?}", updates);
                                    for update in updates.drain(..) {
                                        match update {
                                            Update::Insert{relid, v} => {
                                                sessions
                                                  .get_mut(&relid)
                                                  .ok_or_else(|| format!("no session found for relation ID {}", relid))?
                                                  .update(v, 1);
                                            },
                                            Update::DeleteValue{relid, v} => {
                                                sessions
                                                  .get_mut(&relid)
                                                  .ok_or_else(|| format!("no session found for relation ID {}", relid))?
                                                  .update(v, -1);
                                            },
                                            Update::InsertOrUpdate{..} => {
                                                return Err("InsertOrUpdate command received by worker thread".to_string());
                                            },
                                            Update::DeleteKey{..} => {
                                                // workers don't know about keys
                                                return Err("DeleteKey command received by worker thread".to_string());
                                            },
                                            Update::Modify{..} => {
                                                return Err("Modify command received by worker thread".to_string());
                                            }
                                        }
                                    };
                                },
                                Ok(Msg::Flush) => {
                                    //println!("flushing");
                                    epoch += 1;
                                    Self::advance(&mut sessions, &mut traces, epoch);
                                    Self::flush(&mut sessions, &probe, worker, &peers, &frontier_ts, &progress_barrier);

                                    //println!("flushed");
                                    reply_send.send(Reply::FlushAck).map_err(|e| format!("failed to send ACK: {}", e))?;
                                },
                                Ok(Msg::Query(arrid, key)) => {
                                    Self::handle_query(&mut traces, &reply_send, arrid, key)?;
                                },
                                Ok(Msg::Stop) => {
                                    Self::stop_workers(&peers, &frontier_ts, &progress_barrier);
                                    break;
                                }
                            }
                        }
                    } else /* worker_index != 0 */ {
                        loop {
                            // Differential does not require any synchronization between workers: as
                            // long as we keep calling `step_or_park`, all workers will eventually
                            // process all inputs.  Barriers in the following code are needed so that
                            // worker 0 can know exactly when all other workers have processed all data
                            // for the `frontier_ts` timestamp, so that it knows when a transaction has
                            // been fully committed and produced all its outputs.
                            progress_barrier.wait();
                            let time = frontier_ts.load(Ordering::SeqCst);

                            // TS::max_value() == 0xffffffffffffffff*
                            if time == TS::max_value() {
                                return Ok(());
                            }

                            // `sessions` is empty, but we must advance trace frontiers, so we
                            // don't hinder trace compaction.
                            Self::advance(&mut sessions, &mut traces, time);
                            while probe.less_than(&time) {
                                if !worker.step_or_park(None) {
                                    // Dataflow terminated.
                                    return Ok(());
                                }
                            }

                            progress_barrier.wait();
                            /* We're all caught up with `frontier_ts` and can now spend some time
                             * garbage collecting.  The `step_or_park` call below will block if there
                             * is no more garbage collecting left to do.  It will wake up when one of
                             * the following conditions occurs: (1) there is more garbage collecting to
                             * do as a result of other threads making progress, (2) new inputs have
                             * been received, (3) worker 0 unparked the thread, (4) main thread sent
                             * us a message and unparked the thread.  We check if the frontier has been
                             * advanced by worker 0 and, if so, go back to the barrier to synchrnonize
                             * with other workers. */
                            while frontier_ts.load(Ordering::SeqCst) == time {
                                /* Non-blocking receive, so that we can do some garbage collecting
                                 * when there is no real work to do. */
                                match request_recv.try_recv() {
                                    Ok(Msg::Query(arrid, key)) => {
                                        Self::handle_query(&mut traces, &reply_send, arrid, key)?;
                                    },
                                    Ok(msg) => {
                                        return Err(format!("Worker {} received unexpected message: {:?}", worker_index, msg));
                                    },
                                    Err(mpsc::TryRecvError::Empty) => {
                                        /* Command channel empty: use idle time to work on garbage collection. */
                                        worker.step_or_park(None);
                                    },

                                    _ => { }
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
        ).map(|g| {
            wsend.send(
                g.guards()
                    .iter()
                    .map(|wg| wg.thread().clone()).collect()
            )
            .unwrap();

            g.join();
        }));

        let worker_threads = wrecv.recv().unwrap();
        //println!("timely computation started");

        let mut rels = FnvHashMap::default();
        for relid in self.input_relations() {
            let rel = self.get_relation(relid);
            if rel.input {
                match rel.caching_mode {
                    CachingMode::Stream => {
                        rels.insert(
                            relid,
                            RelationInstance::Stream {
                                delta: FnvHashMap::default(),
                            },
                        );
                    }
                    CachingMode::Multiset => {
                        rels.insert(
                            relid,
                            RelationInstance::Multiset {
                                elements: FnvHashMap::default(),
                                delta: FnvHashMap::default(),
                            },
                        );
                    }
                    CachingMode::Set => match rel.key_func {
                        None => {
                            rels.insert(
                                relid,
                                RelationInstance::Flat {
                                    elements: FnvHashSet::default(),
                                    delta: FnvHashMap::default(),
                                },
                            );
                        }
                        Some(f) => {
                            rels.insert(
                                relid,
                                RelationInstance::Indexed {
                                    key_func: f,
                                    elements: FnvHashMap::default(),
                                    delta: FnvHashMap::default(),
                                },
                            );
                        }
                    },
                }
            }
        }

        // Wait for the initial transaction to complete
        reply_recv[0]
            .recv()
            .map_err(|e| format!("failed to receive ACK: {}", e))?;

        Ok(RunningProgram {
            senders: request_send,
            reply_recv,
            relations: rels,
            thread_handle: Some(h),
            worker_threads,
            transaction_in_progress: false,
            need_to_flush: false,
            profile_cpu,
            profile_timely,
            prof_thread_handle: Some(prof_thread),
            profile,
        })
    }

    /// This thread function is always invoked whether or not profiling is on. If it isn't, the
    /// thread will blocks on the channel read as no message will ever arrive.
    fn prof_thread_func(chan: mpsc::Receiver<ProfMsg>, profile: Arc<Mutex<Profile>>) {
        loop {
            match chan.recv() {
                Ok(msg) => {
                    profile.lock().unwrap().update(&msg);
                }
                _ => return,
            }
        }
    }

    /* Advance the epoch on all input sessions */
    fn advance<Tr>(
        sessions: &mut FnvHashMap<RelId, InputSession<TS, DDValue, Weight>>,
        traces: &mut BTreeMap<ArrId, Tr>,
        epoch: TS,
    ) where
        Tr: TraceReader<Key = DDValue, Val = DDValue, Time = TS, R = Weight>,
        Tr::Batch: BatchReader<DDValue, DDValue, TS, Weight>,
        Tr::Cursor: Cursor<DDValue, DDValue, TS, Weight>,
    {
        for (_, s) in sessions.iter_mut() {
            //print!("advance\n");
            s.advance_to(epoch);
        }
        for (_, t) in traces.iter_mut() {
            t.distinguish_since(&[epoch]);
            t.advance_by(&[epoch]);
        }
    }

    /* Propagate all changes through the pipeline */
    fn flush(
        sessions: &mut FnvHashMap<RelId, InputSession<TS, DDValue, Weight>>,
        probe: &ProbeHandle<TS>,
        worker: &mut Worker<Allocator>,
        peers: &FnvHashMap<usize, thread::Thread>,
        frontier_ts: &TSAtomic,
        progress_barrier: &Barrier,
    ) {
        for (_, r) in sessions.iter_mut() {
            //print!("flush\n");
            r.flush();
        }
        if let Some((_, session)) = sessions.iter_mut().next() {
            /* Do nothing if timestamp has not advanced since the last
             * transaction (i.e., no updates have arrived). */
            if frontier_ts.load(Ordering::SeqCst) < *session.time() {
                frontier_ts.store(*session.time(), Ordering::SeqCst);
                Self::unpark_peers(peers);
                progress_barrier.wait();
                while probe.less_than(session.time()) {
                    //println!("flush.step");
                    worker.step_or_park(None);
                }
                progress_barrier.wait();
            }
        }
    }

    fn stop_workers(
        peers: &FnvHashMap<usize, thread::Thread>,
        frontier_ts: &TSAtomic,
        progress_barrier: &Barrier,
    ) {
        frontier_ts.store(TS::max_value(), Ordering::SeqCst);
        Self::unpark_peers(peers);
        progress_barrier.wait();
    }

    fn handle_query<Tr>(
        traces: &mut BTreeMap<ArrId, Tr>,
        reply_send: &mpsc::Sender<Reply>,
        arrid: ArrId,
        key: Option<DDValue>,
    ) -> Result<(), String>
    where
        Tr: TraceReader<Key = DDValue, Val = DDValue, Time = TS, R = Weight>,
        <Tr as TraceReader>::Batch: BatchReader<DDValue, DDValue, TS, Weight>,
        <Tr as TraceReader>::Cursor: Cursor<DDValue, DDValue, TS, Weight>,
    {
        let trace = match traces.get_mut(&arrid) {
            None => {
                reply_send
                    .send(Reply::QueryRes(None))
                    .map_err(|e| format!("handle_query: failed to send error response: {}", e))?;
                return Ok(());
            }
            Some(trace) => trace,
        };

        let (mut cursor, storage) = trace.cursor();
        // for ((k, v), diffs) in cursor.to_vec(&storage).iter() {
        //     println!("{:?}:{:?}: {:?}", *k, *v, diffs);
        // }
        /* XXX: is this necessary? */
        cursor.rewind_keys(&storage);
        cursor.rewind_vals(&storage);
        let vals = match key {
            Some(k) => {
                cursor.seek_key(&storage, &k);
                if !cursor.key_valid(&storage) {
                    BTreeSet::new()
                } else {
                    let mut vals = BTreeSet::new();
                    while cursor.val_valid(&storage) && *cursor.key(&storage) == k {
                        let mut weight = 0;
                        cursor.map_times(&storage, |_, diff| weight += diff);
                        //assert!(weight >= 0);
                        // FIXME: this will add the value to the set even if `weight < 0`,
                        // i.e., positive and negative weights are treated the same way.
                        // A negative wait should only be possible if there are values with
                        // negative weights in one of the input multisets.
                        if weight != 0 {
                            vals.insert(cursor.val(&storage).clone());
                        };
                        cursor.step_val(&storage);
                    }
                    vals
                }
            }
            None => {
                let mut vals = BTreeSet::new();
                while cursor.key_valid(&storage) {
                    while cursor.val_valid(&storage) {
                        let mut weight = 0;
                        cursor.map_times(&storage, |_, diff| weight += diff);
                        //assert!(weight >= 0);
                        if weight != 0 {
                            vals.insert(cursor.val(&storage).clone());
                        };
                        cursor.step_val(&storage);
                    }
                    cursor.step_key(&storage);
                }
                vals
            }
        };
        reply_send
            .send(Reply::QueryRes(Some(vals)))
            .map_err(|e| format!("handle_query: failed to send query response: {}", e))?;
        Ok(())
    }

    fn unpark_peers(peers: &FnvHashMap<usize, thread::Thread>) {
        for (_, t) in peers.iter() {
            t.unpark();
        }
    }

    /* Lookup relation by id */
    fn get_relation(&self, relid: RelId) -> &Relation {
        for node in &self.nodes {
            match node {
                ProgNode::Rel { rel: r } => {
                    if r.id == relid {
                        return r;
                    };
                }
                ProgNode::Apply { .. } => {}
                ProgNode::SCC { rels: rs } => {
                    for r in rs {
                        if r.rel.id == relid {
                            return &r.rel;
                        };
                    }
                }
            }
        }
        panic!("get_relation({}): relation not found", relid)
    }

    /* indices of program nodes that use arrangement */
    fn arrangement_used_by_nodes(&self, arrid: ArrId) -> Vec<usize> {
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                if Self::node_uses_arrangement(n, arrid) {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn node_uses_arrangement(n: &ProgNode, arrid: ArrId) -> bool {
        match n {
            ProgNode::Rel { rel } => Self::rel_uses_arrangement(rel, arrid),
            ProgNode::Apply { .. } => false,
            ProgNode::SCC { rels } => rels
                .iter()
                .any(|rel| Self::rel_uses_arrangement(&rel.rel, arrid)),
        }
    }

    fn rel_uses_arrangement(r: &Relation, arrid: ArrId) -> bool {
        r.rules
            .iter()
            .any(|rule| Self::rule_uses_arrangement(rule, arrid))
    }

    fn rule_uses_arrangement(r: &Rule, arrid: ArrId) -> bool {
        r.dependencies().contains(&Dep::Arr(arrid))
    }

    /// Returns all input relations of the program
    fn input_relations(&self) -> Vec<RelId> {
        self.nodes
            .iter()
            .flat_map(|node| match node {
                ProgNode::Rel { rel: r } => {
                    if r.input {
                        vec![r.id]
                    } else {
                        vec![]
                    }
                }
                ProgNode::Apply { .. } => vec![],
                ProgNode::SCC { rels: rs } => {
                    for r in rs {
                        assert!(!r.rel.input, "input relation ({}) in SCC", r.rel.name);
                    }
                    vec![]
                }
            })
            .collect()
    }

    /// Return all relations required to compute rels, excluding recursive dependencies on rels
    fn dependencies(rels: &[&Relation]) -> FnvHashSet<Dep> {
        let mut result = FnvHashSet::default();
        for rel in rels {
            for rule in &rel.rules {
                result = result.union(&rule.dependencies()).cloned().collect();
            }
        }
        result
            .into_iter()
            .filter(|d| rels.iter().all(|r| r.id != d.relid()))
            .collect()
    }

    fn xform_collection<'a, 'b, P, T>(
        col: Collection<Child<'a, P, T>, DDValue, Weight>,
        xform: &Option<XFormCollection>,
        arrangements: &Arrangements<'a, 'b, P, T>,
    ) -> Collection<Child<'a, P, T>, DDValue, Weight>
    where
        P: ScopeParent,
        P::Timestamp: Lattice,
        T: Refines<P::Timestamp> + Lattice + Timestamp + Ord,
        T: ToTupleTS,
    {
        match xform {
            None => col,
            Some(ref x) => Self::xform_collection_ref(&col, x, arrangements),
        }
    }

    fn xform_collection_ref<'a, 'b, P, T>(
        col: &Collection<Child<'a, P, T>, DDValue, Weight>,
        xform: &XFormCollection,
        arrangements: &Arrangements<'a, 'b, P, T>,
    ) -> Collection<Child<'a, P, T>, DDValue, Weight>
    where
        P: ScopeParent,
        P::Timestamp: Lattice,
        T: Refines<P::Timestamp> + Lattice + Timestamp + Ord,
        T: ToTupleTS,
    {
        match *xform {
            XFormCollection::Arrange {
                ref description,
                afun,
                ref next,
            } => {
                let arr = with_prof_context(&description, || col.flat_map(afun).arrange_by_key());
                Self::xform_arrangement(&arr, &*next, arrangements)
            }
            XFormCollection::Map {
                ref description,
                mfun,
                ref next,
            } => {
                let mapped = with_prof_context(&description, || col.map(mfun));
                Self::xform_collection(mapped, &*next, arrangements)
            }
            XFormCollection::FlatMap {
                ref description,
                fmfun,
                ref next,
            } => {
                let flattened = with_prof_context(&description, || {
                    col.flat_map(move |x| fmfun(x).into_iter().flatten())
                });
                Self::xform_collection(flattened, &*next, arrangements)
            }
            XFormCollection::Filter {
                ref description,
                ffun,
                ref next,
            } => {
                let filtered = with_prof_context(&description, || col.filter(ffun));
                Self::xform_collection(filtered, &*next, arrangements)
            }
            XFormCollection::FilterMap {
                ref description,
                fmfun,
                ref next,
            } => {
                let flattened = with_prof_context(&description, || col.flat_map(fmfun));
                Self::xform_collection(flattened, &*next, arrangements)
            }
            XFormCollection::Inspect {
                ref description,
                ifun,
                ref next,
            } => {
                let inspect = with_prof_context(&description, || {
                    col.inspect(move |(v, ts, w)| ifun(v, ts.to_tuple_ts(), *w))
                });
                Self::xform_collection(inspect, &*next, arrangements)
            }
        }
    }

    fn xform_arrangement<'a, 'b, P, T, TR>(
        arr: &Arranged<Child<'a, P, T>, TR>,
        xform: &XFormArrangement,
        arrangements: &Arrangements<'a, 'b, P, T>,
    ) -> Collection<Child<'a, P, T>, DDValue, Weight>
    where
        P: ScopeParent,
        P::Timestamp: Lattice,
        T: Refines<P::Timestamp> + Lattice + Timestamp + Ord,
        T: ToTupleTS,
        TR: TraceReader<Key = DDValue, Val = DDValue, Time = T, R = Weight> + Clone + 'static,
        TR::Batch: BatchReader<DDValue, DDValue, T, Weight>,
        TR::Cursor: Cursor<DDValue, DDValue, T, Weight>,
    {
        match *xform {
            XFormArrangement::FlatMap {
                ref description,
                fmfun,
                ref next,
            } => with_prof_context(&description, || {
                Self::xform_collection(
                    arr.flat_map_ref(move |_, v| match fmfun(v.clone()) {
                        Some(iter) => iter,
                        None => Box::new(None.into_iter()),
                    }),
                    &*next,
                    arrangements,
                )
            }),
            XFormArrangement::FilterMap {
                ref description,
                fmfun,
                ref next,
            } => with_prof_context(&description, || {
                Self::xform_collection(
                    arr.flat_map_ref(move |_, v| fmfun(v.clone())),
                    &*next,
                    arrangements,
                )
            }),
            XFormArrangement::Aggregate {
                ref description,
                ffun,
                aggfun,
                ref next,
            } => {
                let col = with_prof_context(&description, || {
                    ffun.map_or_else(
                        || {
                            arr.reduce(move |key, src, dst| {
                                if let Some(x) = aggfun(key, src) {
                                    dst.push((x, 1));
                                };
                            })
                            .map(|(_, v)| v)
                        },
                        |f| {
                            arr.filter(move |_, v| f(v))
                                .reduce(move |key, src, dst| {
                                    if let Some(x) = aggfun(key, src) {
                                        dst.push((x, 1));
                                    };
                                })
                                .map(|(_, v)| v)
                        },
                    )
                });
                Self::xform_collection(col, &*next, arrangements)
            }
            XFormArrangement::Join {
                ref description,
                ffun,
                arrangement,
                jfun,
                ref next,
            } => match arrangements.lookup_arr(arrangement) {
                A::Arrangement1(ArrangedCollection::Map(arranged)) => {
                    let col = with_prof_context(&description, || {
                        ffun.map_or_else(
                            || arr.join_core(arranged, jfun),
                            |f| arr.filter(move |_, v| f(v)).join_core(arranged, jfun),
                        )
                    });
                    Self::xform_collection(col, &*next, arrangements)
                }
                A::Arrangement2(ArrangedCollection::Map(arranged)) => {
                    let col = with_prof_context(&description, || {
                        ffun.map_or_else(
                            || arr.join_core(arranged, jfun),
                            |f| arr.filter(move |_, v| f(v)).join_core(arranged, jfun),
                        )
                    });
                    Self::xform_collection(col, &*next, arrangements)
                }

                _ => panic!("Join: not a map arrangement {:?}", arrangement),
            },
            XFormArrangement::Semijoin {
                ref description,
                ffun,
                arrangement,
                jfun,
                ref next,
            } => match arrangements.lookup_arr(arrangement) {
                A::Arrangement1(ArrangedCollection::Set(arranged)) => {
                    let col = with_prof_context(&description, || {
                        ffun.map_or_else(
                            || arr.join_core(arranged, jfun),
                            |f| arr.filter(move |_, v| f(v)).join_core(arranged, jfun),
                        )
                    });
                    Self::xform_collection(col, &*next, arrangements)
                }
                A::Arrangement2(ArrangedCollection::Set(arranged)) => {
                    let col = with_prof_context(&description, || {
                        ffun.map_or_else(
                            || arr.join_core(arranged, jfun),
                            |f| arr.filter(move |_, v| f(v)).join_core(arranged, jfun),
                        )
                    });
                    Self::xform_collection(col, &*next, arrangements)
                }
                _ => panic!("Semijoin: not a set arrangement {:?}", arrangement),
            },
            XFormArrangement::Antijoin {
                ref description,
                ffun,
                arrangement,
                ref next,
            } => match arrangements.lookup_arr(arrangement) {
                A::Arrangement1(ArrangedCollection::Set(arranged)) => {
                    let col = with_prof_context(&description, || {
                        ffun.map_or_else(
                            || antijoin_arranged(&arr, arranged).map(|(_, v)| v),
                            |f| {
                                antijoin_arranged(&arr.filter(move |_, v| f(v)), arranged)
                                    .map(|(_, v)| v)
                            },
                        )
                    });
                    Self::xform_collection(col, &*next, arrangements)
                }
                A::Arrangement2(ArrangedCollection::Set(arranged)) => {
                    let col = with_prof_context(&description, || {
                        ffun.map_or_else(
                            || antijoin_arranged(&arr, arranged).map(|(_, v)| v),
                            |f| {
                                antijoin_arranged(&arr.filter(move |_, v| f(v)), arranged)
                                    .map(|(_, v)| v)
                            },
                        )
                    });
                    Self::xform_collection(col, &*next, arrangements)
                }
                _ => panic!("Antijoin: not a set arrangement {:?}", arrangement),
            },
        }
    }

    /* Compile right-hand-side of a rule to a collection */
    fn mk_rule<'a, 'b, P, T, F>(
        &self,
        rule: &Rule,
        lookup_collection: F,
        arrangements: Arrangements<'a, 'b, P, T>,
    ) -> Collection<Child<'a, P, T>, DDValue, Weight>
    where
        P: ScopeParent + 'a,
        P::Timestamp: Lattice,
        T: Refines<P::Timestamp> + Lattice + Timestamp + Ord,
        T: ToTupleTS,
        F: Fn(RelId) -> Option<&'b Collection<Child<'a, P, T>, DDValue, Weight>>,
        'a: 'b,
    {
        match rule {
            Rule::CollectionRule {
                rel, xform: None, ..
            } => {
                let collection = lookup_collection(*rel)
                    .unwrap_or_else(|| panic!("mk_rule: unknown relation {:?}", rel));
                let rel_name = &self.get_relation(*rel).name;
                with_prof_context(format!("{} clone", rel_name).as_ref(), || {
                    collection.map(|x| x)
                })
            }
            Rule::CollectionRule {
                rel,
                xform: Some(x),
                ..
            } => Self::xform_collection_ref(
                lookup_collection(*rel)
                    .unwrap_or_else(|| panic!("mk_rule: unknown relation {:?}", rel)),
                x,
                &arrangements,
            ),
            Rule::ArrangementRule { arr, xform, .. } => match arrangements.lookup_arr(*arr) {
                A::Arrangement1(ArrangedCollection::Map(arranged)) => {
                    Self::xform_arrangement(arranged, xform, &arrangements)
                }
                A::Arrangement2(ArrangedCollection::Map(arranged)) => {
                    Self::xform_arrangement(arranged, xform, &arrangements)
                }
                _ => panic!("Rule starts with a set arrangement {:?}", *arr),
            },
        }
    }
}

/// Interface to a running datalog computation
// This should not panic, so that the client has a chance to recover from failures
// TODO: error messages
impl RunningProgram {
    /// Controls forwarding of `TimelyEvent::Schedule` event to the CPU profiling thread.
    ///
    /// `enable = true`  - enables forwarding. This can be expensive in large dataflows.
    /// `enable = false` - disables forwarding.
    pub fn enable_cpu_profiling(&self, enable: bool) {
        self.profile_cpu.store(enable, Ordering::SeqCst);
    }

    pub fn enable_timely_profiling(&self, enable: bool) {
        self.profile_timely.store(enable, Ordering::SeqCst);
    }

    /// Terminate program, killing all worker threads.
    pub fn stop(&mut self) -> Response<()> {
        if let Some(thread) = self.thread_handle.take() {
            self.flush()
                .and_then(|_| self.send(0, Msg::Stop))
                .and_then(|_| match thread.join() {
                    // Note that we intentionally do not join the profiling thread if the timely
                    // thread died, because an unexpected death of the timely thread means we
                    // are in some very weird state and the profiling thread may be stuck
                    // waiting for worker thread which will not respond. So it's best to skip
                    // this join in such a case.
                    Err(_) => Err("timely thread terminated with an error".to_string()),
                    Ok(Err(errstr)) => Err(format!("timely dataflow error: {}", errstr)),
                    Ok(Ok(())) => {
                        if let Some(thread) = self.prof_thread_handle.take() {
                            thread
                                .join()
                                .map_err(|_| "profiling thread terminated with an error".to_owned())
                        } else {
                            Ok(())
                        }
                    }
                })?;
        }

        Ok(())
    }

    /// Start a transaction. Does not return a transaction handle, as there
    /// can be at most one transaction in progress at any given time. Fails
    /// if there is already a transaction in progress.
    pub fn transaction_start(&mut self) -> Response<()> {
        if self.transaction_in_progress {
            return Err("transaction already in progress".to_string());
        }

        self.transaction_in_progress = true;
        Result::Ok(())
    }

    /// Commit a transaction.
    pub fn transaction_commit(&mut self) -> Response<()> {
        if !self.transaction_in_progress {
            return Err("transaction_commit: no transaction in progress".to_string());
        }

        self.flush().and_then(|_| self.delta_cleanup()).map(|_| {
            self.transaction_in_progress = false;
        })
    }

    /// Rollback the transaction, undoing all changes.
    pub fn transaction_rollback(&mut self) -> Response<()> {
        if !self.transaction_in_progress {
            return Err("transaction_rollback: no transaction in progress".to_string());
        }

        self.flush().and_then(|_| self.delta_undo()).map(|_| {
            self.transaction_in_progress = false;
        })
    }

    /// Insert one record into input relation. Relations have set semantics, i.e.,
    /// adding an existing record is a no-op.
    pub fn insert(&mut self, relid: RelId, v: DDValue) -> Response<()> {
        self.apply_updates(iter::once(Update::Insert { relid, v }), |_| Ok(()))
    }

    /// Insert one record into input relation or replace existing record with the same key.
    pub fn insert_or_update(&mut self, relid: RelId, v: DDValue) -> Response<()> {
        self.apply_updates(iter::once(Update::InsertOrUpdate { relid, v }), |_| Ok(()))
    }

    /// Remove a record if it exists in the relation.
    pub fn delete_value(&mut self, relid: RelId, v: DDValue) -> Response<()> {
        self.apply_updates(iter::once(Update::DeleteValue { relid, v }), |_| Ok(()))
    }

    /// Remove a key if it exists in the relation.
    pub fn delete_key(&mut self, relid: RelId, k: DDValue) -> Response<()> {
        self.apply_updates(iter::once(Update::DeleteKey { relid, k }), |_| Ok(()))
    }

    /// Modify a key if it exists in the relation.
    pub fn modify_key(
        &mut self,
        relid: RelId,
        k: DDValue,
        m: Arc<dyn Mutator<DDValue> + Send + Sync>,
    ) -> Response<()> {
        self.apply_updates(iter::once(Update::Modify { relid, k, m }), |_| Ok(()))
    }

    /// Applies a single update.
    fn apply_update(
        &mut self,
        update: Update<DDValue>,
        filtered_updates: &mut Vec<Update<DDValue>>,
    ) -> Response<()> {
        let rel = self
            .relations
            .get_mut(&update.relid())
            .ok_or_else(|| format!("apply_update: unknown input relation {}", update.relid()))?;

        match rel {
            RelationInstance::Stream { delta } => {
                Self::stream_update(delta, update, filtered_updates)
            }
            RelationInstance::Multiset { elements, delta } => {
                Self::mset_update(elements, delta, update, filtered_updates)
            }
            RelationInstance::Flat { elements, delta } => {
                Self::set_update(elements, delta, update, filtered_updates)
            }
            RelationInstance::Indexed {
                key_func,
                elements,
                delta,
            } => Self::indexed_set_update(*key_func, elements, delta, update, filtered_updates),
        }
    }

    /// Apply multiple insert and delete operations in one batch.
    /// Updates can only be applied to input relations (see `struct Relation`).
    pub fn apply_updates<I, F>(&mut self, updates: I, inspect: F) -> Response<()>
    where
        I: Iterator<Item = Update<DDValue>>,
        F: Fn(&Update<DDValue>) -> Response<()>,
    {
        if !self.transaction_in_progress {
            return Err("apply_updates: no transaction in progress".to_string());
        }

        // Remove no-op updates to maintain set semantics
        let mut filtered_updates = Vec::new();
        for update in updates {
            inspect(&update)?;
            self.apply_update(update, &mut filtered_updates)?;
        }

        self.send(0, Msg::Update(filtered_updates)).map(|_| {
            self.need_to_flush = true;
        })
    }

    /// Deletes all values in an input table
    pub fn clear_relation(&mut self, relid: RelId) -> Response<()> {
        if !self.transaction_in_progress {
            return Err("clear_relation: no transaction in progress".to_string());
        }

        let updates = {
            let rel = self
                .relations
                .get_mut(&relid)
                .ok_or_else(|| format!("clear_relation: unknown input relation {}", relid))?;

            match rel {
                RelationInstance::Stream { .. } => {
                    return Err("clear_relation: operation not supported for streams".to_string())
                }
                RelationInstance::Multiset { elements, .. } => {
                    let mut updates: Vec<Update<DDValue>> = Vec::with_capacity(elements.len());
                    Self::delta_undo_updates(relid, elements, &mut updates);

                    updates
                }
                RelationInstance::Flat { elements, .. } => {
                    let mut updates: Vec<Update<DDValue>> = Vec::with_capacity(elements.len());
                    for v in elements.iter() {
                        updates.push(Update::DeleteValue {
                            relid,
                            v: v.clone(),
                        });
                    }

                    updates
                }
                RelationInstance::Indexed { elements, .. } => {
                    let mut updates: Vec<Update<DDValue>> = Vec::with_capacity(elements.len());
                    for k in elements.keys() {
                        updates.push(Update::DeleteKey {
                            relid,
                            k: k.clone(),
                        });
                    }

                    updates
                }
            }
        };

        self.apply_updates(updates.into_iter(), |_| Ok(()))
    }

    /// Returns all values in the arrangement with the specified key.
    pub fn query_arrangement(&mut self, arrid: ArrId, k: DDValue) -> Response<BTreeSet<DDValue>> {
        self._query_arrangement(arrid, Some(k))
    }

    /// Returns the entire content of an arrangement.
    pub fn dump_arrangement(&mut self, arrid: ArrId) -> Response<BTreeSet<DDValue>> {
        self._query_arrangement(arrid, None)
    }

    fn _query_arrangement(
        &mut self,
        arrid: ArrId,
        k: Option<DDValue>,
    ) -> Response<BTreeSet<DDValue>> {
        // Send query and receive replies from all workers. If a key is specified, then at most
        // one worker will send a non-empty reply.
        self.broadcast(Msg::Query(arrid, k))?;

        let mut res: BTreeSet<DDValue> = BTreeSet::new();
        let mut unknown = false;
        for (worker_index, chan) in self.reply_recv.iter().enumerate() {
            let reply = chan.recv().map_err(|e| {
                format!(
                    "query_arrangement: failed to receive reply from worker {}: {:?}",
                    worker_index, e
                )
            })?;

            match reply {
                Reply::QueryRes(Some(mut vals)) => {
                    if !vals.is_empty() {
                        if res.is_empty() {
                            std::mem::swap(&mut res, &mut vals);
                        } else {
                            res.append(&mut vals);
                        }
                    }
                }
                Reply::QueryRes(None) => {
                    unknown = true;
                }
                repl => {
                    return Err(format!(
                        "query_arrangement: unexpected reply from worker {}: {:?}",
                        worker_index, repl
                    ));
                }
            }
        }

        if unknown {
            Err(format!("query_arrangement: unknown index: {:?}", arrid))
        } else {
            Ok(res)
        }
    }

    /// increment the counter associated with value `x` in the delta-set
    /// `delta(x) == false` => remove entry (equivalent to delta(x):=0)
    /// `x not in delta => `delta(x) := true`
    /// `delta(x) == true` => error
    fn delta_inc(ds: &mut DeltaSet, x: &DDValue) {
        let entry = ds.entry(x.clone());
        match entry {
            hash_map::Entry::Occupied(mut oe) => {
                // debug_assert!(!*oe.get());
                let v = oe.get_mut();
                if *v == -1 {
                    oe.remove_entry();
                } else {
                    *v += 1;
                }
            }
            hash_map::Entry::Vacant(ve) => {
                ve.insert(1);
            }
        }
    }

    /// reverse of delta_inc
    fn delta_dec(ds: &mut DeltaSet, key: &DDValue) {
        let entry = ds.entry(key.clone());
        match entry {
            hash_map::Entry::Occupied(mut oe) => {
                //debug_assert!(*oe.get());
                let v = oe.get_mut();
                if *v == 1 {
                    oe.remove_entry();
                } else {
                    *v -= 1;
                }
            }
            hash_map::Entry::Vacant(ve) => {
                ve.insert(-1);
            }
        }
    }

    /// Update delta set of an input stream relation before performing an update.
    /// `ds` is delta since start of transaction.
    /// `x` is the value being inserted or deleted.
    /// `insert` indicates type of update (`true` for insert, `false` for delete)
    fn stream_update(
        ds: &mut DeltaSet,
        update: Update<DDValue>,
        updates: &mut Vec<Update<DDValue>>,
    ) -> Response<()> {
        match &update {
            Update::Insert { v, .. } => {
                Self::delta_inc(ds, v);
            }
            Update::DeleteValue { v, .. } => {
                Self::delta_dec(ds, v);
            }
            Update::InsertOrUpdate { relid, .. } => {
                return Err(format!(
                    "Cannot perform insert_or_update operation on relation {} that does not have a primary key",
                    relid,
                ));
            }
            Update::DeleteKey { relid, .. } => {
                return Err(format!(
                    "Cannot delete by key from relation {} that does not have a primary key",
                    relid,
                ));
            }
            Update::Modify { relid, .. } => {
                return Err(format!(
                    "Cannot modify record in relation {} that does not have a primary key",
                    relid,
                ));
            }
        };
        updates.push(update);

        Ok(())
    }

    /// Update value and delta multisets of an input multiset relation before performing an update.
    /// `s` is the current content of the relation.
    /// `ds` is delta since start of transaction.
    /// `x` is the value being inserted or deleted.
    /// `insert` indicates type of update (`true` for insert, `false` for delete).
    /// Returns `true` if the update modifies the relation, i.e., it's not a no-op.
    fn mset_update(
        s: &mut ValMSet,
        ds: &mut DeltaSet,
        upd: Update<DDValue>,
        updates: &mut Vec<Update<DDValue>>,
    ) -> Response<()> {
        match &upd {
            Update::Insert { v, .. } => {
                Self::delta_inc(s, v);
                Self::delta_inc(ds, v);
            }
            Update::DeleteValue { v, .. } => {
                Self::delta_dec(s, v);
                Self::delta_dec(ds, v);
            }
            Update::InsertOrUpdate { relid, .. } => {
                return Err(format!(
                    "Cannot perform insert_or_update operation on relation {} that does not have a primary key",
                    relid
                ));
            }
            Update::DeleteKey { relid, .. } => {
                return Err(format!(
                    "Cannot delete by key from relation {} that does not have a primary key",
                    relid
                ));
            }
            Update::Modify { relid, .. } => {
                return Err(format!(
                    "Cannot modify record in relation {} that does not have a primary key",
                    relid
                ));
            }
        };
        updates.push(upd);

        Ok(())
    }

    /// Update value set and delta set of an input relation before performing an update.
    /// `s` is the current content of the relation.
    /// `ds` is delta since start of transaction.
    /// `x` is the value being inserted or deleted.
    /// `insert` indicates type of update (`true` for insert, `false` for delete).
    /// Returns `true` if the update modifies the relation, i.e., it's not a no-op.
    fn set_update(
        s: &mut ValSet,
        ds: &mut DeltaSet,
        upd: Update<DDValue>,
        updates: &mut Vec<Update<DDValue>>,
    ) -> Response<()> {
        let ok = match &upd {
            Update::Insert { v, .. } => {
                let new = s.insert(v.clone());
                if new {
                    Self::delta_inc(ds, v);
                }

                new
            }
            Update::DeleteValue { v, .. } => {
                let present = s.remove(&v);
                if present {
                    Self::delta_dec(ds, v);
                }

                present
            }
            Update::InsertOrUpdate { relid, .. } => {
                return Err(format!(
                    "Cannot perform insert_or_update operation on relation {} that does not have a primary key",
                    relid,
                ));
            }
            Update::DeleteKey { relid, .. } => {
                return Err(format!(
                    "Cannot delete by key from relation {} that does not have a primary key",
                    relid,
                ));
            }
            Update::Modify { relid, .. } => {
                return Err(format!(
                    "Cannot modify record in relation {} that does not have a primary key",
                    relid,
                ));
            }
        };

        if ok {
            updates.push(upd);
        }

        Ok(())
    }

    /// insert:
    ///      key exists in `s`:
    ///          - error
    ///      key not in `s`:
    ///          - s.insert(x)
    ///          - ds(x)++;
    /// delete:
    ///      key not in `s`
    ///          - return error
    ///      key in `s` with value `v`:
    ///          - s.delete(key)
    ///          - ds(v)--
    fn indexed_set_update(
        key_func: fn(&DDValue) -> DDValue,
        s: &mut IndexedValSet,
        ds: &mut DeltaSet,
        upd: Update<DDValue>,
        updates: &mut Vec<Update<DDValue>>,
    ) -> Response<()> {
        match upd {
            Update::Insert { relid, v } => match s.entry(key_func(&v)) {
                hash_map::Entry::Occupied(_) => Err(format!(
                    "Insert: duplicate key {:?} in value {:?}",
                    key_func(&v),
                    v
                )),
                hash_map::Entry::Vacant(ve) => {
                    ve.insert(v.clone());
                    Self::delta_inc(ds, &v);
                    updates.push(Update::Insert { relid, v });

                    Ok(())
                }
            },

            Update::InsertOrUpdate { relid, v } => match s.entry(key_func(&v)) {
                hash_map::Entry::Occupied(mut oe) => {
                    // Delete old value.
                    let old = oe.get().clone();
                    Self::delta_dec(ds, oe.get());
                    updates.push(Update::DeleteValue { relid, v: old });

                    // Insert new value.
                    Self::delta_inc(ds, &v);
                    updates.push(Update::Insert {
                        relid,
                        v: v.clone(),
                    });

                    // Update store
                    *oe.get_mut() = v;

                    Ok(())
                }
                hash_map::Entry::Vacant(ve) => {
                    ve.insert(v.clone());
                    Self::delta_inc(ds, &v);
                    updates.push(Update::Insert { relid, v });

                    Ok(())
                }
            },

            Update::DeleteValue { relid, v } => match s.entry(key_func(&v)) {
                hash_map::Entry::Occupied(oe) => {
                    if *oe.get() != v {
                        Err(format!("DeleteValue: key exists with a different value. Value specified: {:?}; existing value: {:?}", v, oe.get()))
                    } else {
                        Self::delta_dec(ds, oe.get());
                        oe.remove_entry();
                        updates.push(Update::DeleteValue { relid, v });
                        Ok(())
                    }
                }
                hash_map::Entry::Vacant(_) => {
                    Err(format!("DeleteValue: key not found {:?}", key_func(&v)))
                }
            },

            Update::DeleteKey { relid, k } => match s.entry(k.clone()) {
                hash_map::Entry::Occupied(oe) => {
                    let old = oe.get().clone();
                    Self::delta_dec(ds, oe.get());
                    oe.remove_entry();
                    updates.push(Update::DeleteValue { relid, v: old });
                    Ok(())
                }
                hash_map::Entry::Vacant(_) => Err(format!("DeleteKey: key not found {:?}", k)),
            },

            Update::Modify { relid, k, m } => match s.entry(k.clone()) {
                hash_map::Entry::Occupied(mut oe) => {
                    let new = oe.get_mut();
                    let old: DDValue = (*new).clone();
                    m.mutate(new)?;
                    Self::delta_dec(ds, &old);
                    updates.push(Update::DeleteValue { relid, v: old });
                    Self::delta_inc(ds, &new);
                    updates.push(Update::Insert {
                        relid,
                        v: new.clone(),
                    });

                    Ok(())
                }
                hash_map::Entry::Vacant(_) => Err(format!("Modify: key not found {:?}", k)),
            },
        }
    }

    /// Returns a reference to indexed input relation content.
    /// If called in the middle of a transaction, returns state snapshot including changes
    /// made by the current transaction.
    pub fn get_input_relation_index(&self, relid: RelId) -> Response<&IndexedValSet> {
        match self.relations.get(&relid) {
            None => Err(format!("unknown relation {}", relid)),
            Some(RelationInstance::Indexed { elements, .. }) => Ok(elements),
            Some(_) => Err(format!("not an indexed relation {}", relid)),
        }
    }

    /// Returns a reference to a flat input relation content.
    /// If called in the middle of a transaction, returns state snapshot including changes
    /// made by the current transaction.
    pub fn get_input_relation_data(&self, relid: RelId) -> Response<&ValSet> {
        match self.relations.get(&relid) {
            None => Err(format!("unknown relation {}", relid)),
            Some(RelationInstance::Flat { elements, .. }) => Ok(elements),
            Some(_) => Err(format!("not a flat relation {}", relid)),
        }
    }

    /// Returns a reference to an input multiset content.
    /// If called in the middle of a transaction, returns state snapshot including changes
    /// made by the current transaction.
    pub fn get_input_multiset_data(&self, relid: RelId) -> Response<&ValMSet> {
        match self.relations.get(&relid) {
            None => Err(format!("unknown relation {}", relid)),
            Some(RelationInstance::Multiset { elements, .. }) => Ok(elements),
            Some(_) => Err(format!("not a flat relation {}", relid)),
        }
    }

    /*
    /// Returns a reference to delta accumulated by the current transaction
    pub fn relation_delta(&mut self, relid: RelId) -> Response<&DeltaSet<V>> {
        if !self.transaction_in_progress {
            return resp_from_error!("no transaction in progress");
        };

        self.flush().and_then(move |_| {
            match self.relations.get_mut(&relid) {
                None => resp_from_error!("unknown relation"),
                Some(rel) => Ok(&rel.delta)
            }
        })
    }
    */

    /// Send message to a worker thread.
    fn send(&self, worker_index: usize, msg: Msg) -> Response<()> {
        match self.senders[worker_index].send(msg) {
            Ok(()) => {
                /* Worker 0 may be blocked in `step_or_park`.  Unpark to ensure receipt of the
                 * message. */
                self.worker_threads[worker_index].unpark();
                Ok(())
            }

            Err(_) => Err("failed to communicate with timely dataflow thread".to_string()),
        }
    }

    /// Broadcast message to all worker threads.
    fn broadcast(&self, msg: Msg) -> Response<()> {
        for worker_index in 0..self.senders.len() {
            self.send(worker_index, msg.clone())?;
        }

        Ok(())
    }

    /// Clear delta sets of all input relations on transaction commit.
    fn delta_cleanup(&mut self) -> Response<()> {
        for rel in self.relations.values_mut() {
            rel.delta_mut().clear();
        }

        Ok(())
    }

    fn delta_undo_updates(relid: RelId, ds: &DeltaSet, updates: &mut Vec<Update<DDValue>>) {
        // first delete, then insert to avoid duplicate key
        // errors in `apply_updates()`
        for (k, w) in ds {
            if *w >= 0 {
                for _ in 0..*w {
                    updates.push(Update::DeleteValue {
                        relid,
                        v: k.clone(),
                    });
                }
            }
        }

        for (k, w) in ds {
            if *w < 0 {
                for _ in 0..(-*w) {
                    updates.push(Update::Insert {
                        relid,
                        v: k.clone(),
                    });
                }
            }
        }
    }

    /// Reverse all changes recorded in delta sets to rollback the transaction.
    fn delta_undo(&mut self) -> Response<()> {
        let mut updates = Vec::with_capacity(self.relations.len());
        for (relid, rel) in &self.relations {
            Self::delta_undo_updates(*relid, rel.delta(), &mut updates);
        }

        // println!("updates: {:?}", updates);
        self.apply_updates(updates.into_iter(), |_| Ok(()))
            .and_then(|_| self.flush())
            .map(|_| {
                /* validation: all deltas must be empty */
                for rel in self.relations.values() {
                    //println!("delta: {:?}", *d);
                    debug_assert!(rel.delta().is_empty());
                }
            })
    }

    /// Propagates all changes through the dataflow pipeline.
    fn flush(&mut self) -> Response<()> {
        if !self.need_to_flush {
            return Ok(());
        }

        self.send(0, Msg::Flush).and_then(|()| {
            self.need_to_flush = false;
            match self.reply_recv[0].recv() {
                Err(_) => Err(
                    "failed to receive flush ack message from timely dataflow thread".to_string(),
                ),
                Ok(Reply::FlushAck) => Ok(()),
                Ok(msg) => Err(format!(
                    "received unexpected reply to flush request: {:?}",
                    msg,
                )),
            }
        })
    }
}

impl Drop for RunningProgram {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
