use crate::{
    ddval::DDValue,
    profile::{get_prof_context, with_prof_context, ProfMsg},
    program::{
        arrange::{ArrangedCollection, Arrangements},
        timestamp::TSAtomic,
        ArrId, Dep, Msg, ProgNode, Program, Reply, Update, TS,
    },
    program::{RelId, Weight},
    variable::Variable,
};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use differential_dataflow::{
    input::{Input, InputSession},
    logging::DifferentialEvent,
    operators::{arrange::TraceAgent, Consolidate, ThresholdTotal},
    trace::{
        implementations::{ord::OrdValBatch, spine_fueled::Spine},
        BatchReader, Cursor, TraceReader,
    },
    Collection,
};
use fnv::{FnvBuildHasher, FnvHashMap};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    mem,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Barrier,
    },
    thread::{self, Thread},
    time::Duration,
};
use timely::{
    communication::Allocator,
    dataflow::{operators::probe::Handle as ProbeHandle, scopes::Child, Scope},
    logging::TimelyEvent,
    progress::frontier::AntichainRef,
    worker::Worker,
};

type SessionData = (
    FnvHashMap<RelId, InputSession<TS, DDValue, Weight>>,
    BTreeMap<
        ArrId,
        TraceAgent<
            Spine<DDValue, DDValue, u32, i32, Rc<OrdValBatch<DDValue, DDValue, u32, i32, u32>>>,
        >,
    >,
);

/// A DDlog timely worker
pub struct DDlogWorker<'a> {
    /// The timely worker instance
    worker: &'a mut Worker<Allocator>,
    /// The program this worker is executing
    program: Arc<Program>,
    /// The atomically synchronized timestamp for the transaction
    /// frontier
    frontier_timestamp: &'a TSAtomic,
    /// Peer workers' thread handles, only used by worker 0.
    peers: FnvHashMap<usize, Thread>,
    /// The progress barrier used for transactions
    progress_barrier: Arc<Barrier>,
    /// Information on which metrics are enabled and a
    /// channel for sending profiling data
    profiling: ProfilingData,
    /// The current worker's receiver for receiving messages
    request_receiver: Receiver<Msg>,
    /// The current worker's sender for sending messages
    reply_sender: Sender<Reply>,
}

impl<'a> DDlogWorker<'a> {
    /// Create a new ddlog timely worker
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        worker: &'a mut Worker<Allocator>,
        program: Arc<Program>,
        frontier_timestamp: &'a TSAtomic,
        num_workers: usize,
        progress_barrier: Arc<Barrier>,
        profiling: ProfilingData,
        request_receivers: Arc<[Receiver<Msg>]>,
        reply_senders: Arc<[Sender<Reply>]>,
        thread_handle_sender: Sender<(usize, Thread)>,
        thread_handle_receiver: Arc<Receiver<(usize, Thread)>>,
    ) -> Self {
        let worker_index = worker.index();

        // The hashmap that will contain all worker thread handles
        let mut peers: FnvHashMap<usize, Thread> = HashMap::with_capacity_and_hasher(
            // Only worker zero will populate this, so all others can create an empty map
            if worker_index == 0 { num_workers } else { 0 },
            FnvBuildHasher::default(),
        );

        // If this is worker zero, receive all other worker's thread handles and
        // populate the `peers` map with them
        if worker_index == 0 {
            for _ in 0..num_workers - 1 {
                let (worker, thread_handle) = thread_handle_receiver
                    .recv()
                    .expect("failed to receive thread handle from timely worker");

                peers.insert(worker, thread_handle);
            }

        // Send other all worker's thread handles to worker 0.
        } else {
            thread_handle_sender
                .send((worker_index, thread::current()))
                .expect("failed to send thread handle for a timely worker");
        }

        Self {
            worker,
            program,
            frontier_timestamp,
            peers,
            progress_barrier,
            profiling,
            request_receiver: request_receivers[worker_index].clone(),
            reply_sender: reply_senders[worker_index].clone(),
        }
    }

    /// Returns whether or not the current worker is the leader, worker 0
    pub fn is_leader(&self) -> bool {
        self.worker_index() == 0
    }

    /// Get the index of the current worker
    pub fn worker_index(&self) -> usize {
        self.worker.index()
    }

    /// Set the current transaction frontier's timestamp
    fn set_frontier_timestamp(&self, timestamp: TS) {
        self.frontier_timestamp.store(timestamp, Ordering::Relaxed);
    }

    /// Get the current transaction frontier's timestamp
    fn frontier_timestamp(&self) -> TS {
        self.frontier_timestamp.load(Ordering::Relaxed)
    }

    /// Unpark every other worker thread
    fn unpark_peers(&self) {
        for thread in self.peers.values() {
            thread.unpark();
        }
    }

    pub fn run(mut self) -> Result<(), String> {
        // Initialize profiling
        self.init_profiling();

        let probe = ProbeHandle::new();
        let (mut all_sessions, mut traces) = self.session_dataflow(probe.clone())?;

        let mut epoch: TS = 0;

        // feed initial data to sessions
        if self.is_leader() {
            for (relid, v) in self.program.init_data.iter() {
                all_sessions
                    .get_mut(relid)
                    .ok_or_else(|| format!("no session found for relation ID {}", relid))?
                    .update(v.clone(), 1);
            }

            epoch += 1;
            self.advance(&mut all_sessions, &mut traces, epoch);
            self.flush(&mut all_sessions, &probe);

            self.reply_sender
                .send(Reply::FlushAck)
                .map_err(|e| format!("failed to send ACK: {}", e))?;
        }

        // Close session handles for non-input sessions;
        // close all sessions for workers other than worker 0.
        let mut sessions: FnvHashMap<RelId, InputSession<TS, DDValue, Weight>> = all_sessions
            .drain()
            .filter(|&(relid, _)| self.is_leader() && self.program.get_relation(relid).input)
            .collect();

        // Only worker 0 receives data
        if self.is_leader() {
            loop {
                // Non-blocking receive, so that we can do some garbage collecting
                // when there is no real work to do.
                match self.request_receiver.try_recv() {
                    Ok(Msg::Update(mut updates)) => {
                        //println!("updates: {:?}", updates);
                        for update in updates.drain(..) {
                            match update {
                                Update::Insert { relid, v } => {
                                    sessions
                                        .get_mut(&relid)
                                        .ok_or_else(|| {
                                            format!("no session found for relation ID {}", relid)
                                        })?
                                        .update(v, 1);
                                }
                                Update::DeleteValue { relid, v } => {
                                    sessions
                                        .get_mut(&relid)
                                        .ok_or_else(|| {
                                            format!("no session found for relation ID {}", relid)
                                        })?
                                        .update(v, -1);
                                }
                                Update::InsertOrUpdate { .. } => {
                                    return Err("InsertOrUpdate command received by worker thread"
                                        .to_string());
                                }
                                Update::DeleteKey { .. } => {
                                    // workers don't know about keys
                                    return Err(
                                        "DeleteKey command received by worker thread".to_string()
                                    );
                                }
                                Update::Modify { .. } => {
                                    return Err(
                                        "Modify command received by worker thread".to_string()
                                    );
                                }
                            }
                        }
                    }

                    Ok(Msg::Flush) => {
                        //println!("flushing");
                        epoch += 1;
                        self.advance(&mut sessions, &mut traces, epoch);
                        self.flush(&mut sessions, &probe);

                        //println!("flushed");
                        self.reply_sender
                            .send(Reply::FlushAck)
                            .map_err(|e| format!("failed to send ACK: {}", e))?;
                    }

                    Ok(Msg::Query(arrid, key)) => {
                        self.handle_query(&mut traces, arrid, key)?;
                    }

                    Ok(Msg::Stop) => {
                        self.stop_all_workers();
                        break;
                    }

                    Err(TryRecvError::Empty) => {
                        // Command channel empty: use idle time to work on garbage collection.
                        // This will block when there is no more compaction left to do.
                        // The sender must unpark worker 0 after sending to the channel.
                        self.worker.step_or_park(None);
                    }

                    Err(TryRecvError::Disconnected) => {
                        eprintln!("sender disconnected");
                        self.stop_all_workers();

                        break;
                    }
                }
            }

        // worker_index != 0
        } else {
            loop {
                // Differential does not require any synchronization between workers: as
                // long as we keep calling `step_or_park`, all workers will eventually
                // process all inputs.  Barriers in the following code are needed so that
                // worker 0 can know exactly when all other workers have processed all data
                // for the `frontier_ts` timestamp, so that it knows when a transaction has
                // been fully committed and produced all its outputs.
                self.progress_barrier.wait();
                let time = self.frontier_timestamp.load(Ordering::SeqCst);

                // TS::max_value() == 0xffffffffffffffff*
                if time == TS::max_value() {
                    return Ok(());
                }

                // `sessions` is empty, but we must advance trace frontiers, so we
                // don't hinder trace compaction.
                self.advance(&mut sessions, &mut traces, time);
                while probe.less_than(&time) {
                    if !self.worker.step_or_park(None) {
                        // Dataflow terminated.
                        return Ok(());
                    }
                }

                self.progress_barrier.wait();
                // We're all caught up with `frontier_ts` and can now spend some time
                // garbage collecting.  The `step_or_park` call below will block if there
                // is no more garbage collecting left to do.  It will wake up when one of
                // the following conditions occurs: (1) there is more garbage collecting to
                // do as a result of other threads making progress, (2) new inputs have
                // been received, (3) worker 0 unparked the thread, (4) main thread sent
                // us a message and unparked the thread.  We check if the frontier has been
                // advanced by worker 0 and, if so, go back to the barrier to synchronize
                // with other workers.
                while self.frontier_timestamp.load(Ordering::SeqCst) == time {
                    // Non-blocking receive, so that we can do some garbage collecting
                    // when there is no real work to do.
                    match self.request_receiver.try_recv() {
                        Ok(Msg::Query(arrid, key)) => {
                            self.handle_query(&mut traces, arrid, key)?;
                        }

                        Ok(msg) => {
                            return Err(format!(
                                "Worker {} received unexpected message: {:?}",
                                self.worker_index(),
                                msg,
                            ));
                        }

                        Err(TryRecvError::Empty) => {
                            // Command channel empty: use idle time to work on garbage collection.
                            self.worker.step_or_park(None);
                        }

                        // The sender disconnected, so we can gracefully exit
                        Err(TryRecvError::Disconnected) => break,
                    }
                }
            }
        }

        Ok(())
    }

    /// Advance the epoch on all input sessions
    fn advance<Trace>(
        &self,
        sessions: &mut FnvHashMap<RelId, InputSession<TS, DDValue, Weight>>,
        traces: &mut BTreeMap<ArrId, Trace>,
        epoch: TS,
    ) where
        Trace: TraceReader<Key = DDValue, Val = DDValue, Time = TS, R = Weight>,
        Trace::Batch: BatchReader<DDValue, DDValue, TS, Weight>,
        Trace::Cursor: Cursor<DDValue, DDValue, TS, Weight>,
    {
        for (_, session_input) in sessions.iter_mut() {
            session_input.advance_to(epoch);
        }

        for (_, trace) in traces.iter_mut() {
            let e = [epoch];
            let ac = AntichainRef::new(&e);
            trace.distinguish_since(ac);
            trace.advance_by(ac);
        }
    }

    /// Propagate all changes through the pipeline
    fn flush(
        &mut self,
        sessions: &mut FnvHashMap<RelId, InputSession<TS, DDValue, Weight>>,
        probe: &ProbeHandle<TS>,
    ) {
        for (_, relation_input) in sessions.iter_mut() {
            relation_input.flush();
        }

        if let Some((_, session)) = sessions.iter_mut().next() {
            // Do nothing if timestamp has not advanced since the last
            // transaction (i.e., no updates have arrived).
            if self.frontier_timestamp() < *session.time() {
                self.set_frontier_timestamp(*session.time());
                self.unpark_peers();

                self.progress_barrier.wait();
                while probe.less_than(session.time()) {
                    self.worker.step_or_park(None);
                }

                self.progress_barrier.wait();
            }
        }
    }

    /// Stop all worker threads by setting the frontier timestamp
    /// to the maximum and unparking all other workers
    fn stop_all_workers(&self) {
        self.set_frontier_timestamp(TS::max_value());
        self.unpark_peers();
        self.progress_barrier.wait();
    }

    /// Handle a query
    fn handle_query<Trace>(
        &self,
        traces: &mut BTreeMap<ArrId, Trace>,
        arrid: ArrId,
        key: Option<DDValue>,
    ) -> Result<(), String>
    where
        Trace: TraceReader<Key = DDValue, Val = DDValue, Time = TS, R = Weight>,
        <Trace as TraceReader>::Batch: BatchReader<DDValue, DDValue, TS, Weight>,
        <Trace as TraceReader>::Cursor: Cursor<DDValue, DDValue, TS, Weight>,
    {
        let trace = match traces.get_mut(&arrid) {
            Some(trace) => trace,
            None => {
                self.reply_sender
                    .send(Reply::QueryRes(None))
                    .map_err(|e| format!("handle_query: failed to send error response: {}", e))?;

                return Ok(());
            }
        };

        let (mut cursor, storage) = trace.cursor();
        // for ((k, v), diffs) in cursor.to_vec(&storage).iter() {
        //     println!("{:?}:{:?}: {:?}", *k, *v, diffs);
        // }

        /* XXX: is this necessary? */
        cursor.rewind_keys(&storage);
        cursor.rewind_vals(&storage);

        let values = match key {
            Some(k) => {
                cursor.seek_key(&storage, &k);
                if !cursor.key_valid(&storage) {
                    BTreeSet::new()
                } else {
                    let mut values = BTreeSet::new();
                    while cursor.val_valid(&storage) && *cursor.key(&storage) == k {
                        let mut weight = 0;
                        cursor.map_times(&storage, |_, &diff| weight += diff);

                        //assert!(weight >= 0);
                        // FIXME: this will add the value to the set even if `weight < 0`,
                        // i.e., positive and negative weights are treated the same way.
                        // A negative wait should only be possible if there are values with
                        // negative weights in one of the input multisets.
                        if weight != 0 {
                            values.insert(cursor.val(&storage).clone());
                        }

                        cursor.step_val(&storage);
                    }

                    values
                }
            }

            None => {
                let mut values = BTreeSet::new();
                while cursor.key_valid(&storage) {
                    while cursor.val_valid(&storage) {
                        let mut weight = 0;
                        cursor.map_times(&storage, |_, &diff| weight += diff);

                        //assert!(weight >= 0);
                        if weight != 0 {
                            values.insert(cursor.val(&storage).clone());
                        }

                        cursor.step_val(&storage);
                    }

                    cursor.step_key(&storage);
                }

                values
            }
        };

        self.reply_sender
            .send(Reply::QueryRes(Some(values)))
            .map_err(|e| format!("handle_query: failed to send query response: {}", e))?;

        Ok(())
    }

    /// Initialize timely and differential profiling logging hooks
    fn init_profiling(&self) {
        let profiling = self.profiling.clone();
        self.worker
            .log_register()
            .insert::<TimelyEvent, _>("timely", move |_time, data| {
                let profile_cpu = profiling.is_cpu_enabled();
                let profile_timely = profiling.is_timely_enabled();

                // Filter out events we don't care about to avoid the overhead of sending
                // the event around just to drop it eventually.
                let filtered: Vec<((Duration, usize, TimelyEvent), Option<String>)> = data
                    .drain(..)
                    .filter(|event| {
                        match event.2 {
                            // Always send Operates events as they're used for always-on memory profiling.
                            TimelyEvent::Operates(_) => true,

                            // Send scheduling events if profiling is enabled
                            TimelyEvent::Schedule(_) => profile_cpu || profile_timely,

                            // Send timely events if timely profiling is enabled
                            TimelyEvent::GuardedMessage(_)
                            | TimelyEvent::Messages(_)
                            | TimelyEvent::Park(_)
                            | TimelyEvent::Progress(_)
                            | TimelyEvent::PushProgress(_) => profile_timely,

                            _ => false,
                        }
                    })
                    .map(|(d, s, e)| match e {
                        // Only Operate events care about the context string.
                        TimelyEvent::Operates(_) => ((d, s, e), Some(get_prof_context())),
                        _ => ((d, s, e), None),
                    })
                    .collect();

                // If there are any profiling events, record them
                if !filtered.is_empty() {
                    profiling.record(ProfMsg::TimelyMessage(
                        filtered,
                        profile_cpu,
                        profile_timely,
                    ));
                }
            });

        let profiling = self.profiling.clone();
        self.worker.log_register().insert::<DifferentialEvent, _>(
            "differential/arrange",
            move |_time, data| {
                // If there are events, send them through the profiling channel
                if !data.is_empty() {
                    profiling.record(ProfMsg::DifferentialMessage(mem::take(data)));
                }
            },
        );
    }

    fn session_dataflow(&mut self, mut probe: ProbeHandle<TS>) -> Result<SessionData, String> {
        let program = self.program.clone();

        self.worker.dataflow::<TS, _, _>(|outer: &mut Child<Worker<Allocator>, TS>| -> Result<_, String> {
            let mut sessions : FnvHashMap<RelId, InputSession<TS, DDValue, Weight>> = FnvHashMap::default();
            let mut collections : FnvHashMap<RelId, Collection<Child<Worker<Allocator>, TS>, DDValue, Weight>> =
                    HashMap::with_capacity_and_hasher(program.nodes.len(), FnvBuildHasher::default());
            let mut arrangements = FnvHashMap::default();

            for (nodeid, node) in program.nodes.iter().enumerate() {
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
                        let rule_collections = rel
                            .rules
                            .iter()
                            .map(|rule| {
                                program.mk_rule(
                                    rule,
                                    |rid| collections.get(&rid),
                                    Arrangements {
                                        arrangements1: &arrangements,
                                        arrangements2: &FnvHashMap::default(),
                                    },
                                )
                            });

                        collection = with_prof_context(
                            &format!("concatenate rules for {}", rel.name),
                            || collection.concatenate(rule_collections),
                        );

                        // don't distinct input collections, as this is already done by the set_update logic
                        if !rel.input && rel.distinct {
                            collection = with_prof_context(
                                &format!("{}.threshold_total", rel.name),
                                || collection.threshold_total(|_, c| if *c == 0 { 0 } else { 1 }),
                            );
                        }

                        // create arrangements
                        for (i,arr) in rel.arrangements.iter().enumerate() {
                            with_prof_context(
                                arr.name(),
                                || arrangements.insert(
                                    (rel.id, i),
                                    arr.build_arrangement_root(&collection),
                                ),
                            );
                        }

                        collections.insert(rel.id, collection);
                    },
                    &ProgNode::Apply { tfun } => {
                        tfun()(&mut collections);
                    },
                    ProgNode::SCC { rels } => {
                        // Preallocate the memory required to store the new relations
                        sessions.reserve(rels.len());
                        collections.reserve(rels.len());

                        // create collections; add them to map; we will overwrite them with
                        // updated collections returned from the inner scope.
                        for r in rels.iter() {
                            let (session, collection) = outer.new_collection::<DDValue,Weight>();
                            //assert!(!r.rel.input, "input relation in nested scope: {}", r.rel.name);
                            if r.rel.input {
                                return Err(format!("input relation in nested scope: {}", r.rel.name));
                            }

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
                                    if program.arrangement_used_by_nodes((rel.rel.id, i)).any(|n| n == nodeid) {
                                        with_prof_context(
                                            &format!("local {}", arr.name()),
                                            || local_arrangements.insert(
                                                (rel.rel.id, i),
                                                arr.build_arrangement(&*vars.get(&rel.rel.id)?),
                                            ),
                                        );
                                    }
                                }
                            }

                            let dependencies = Program::dependencies(rels.iter().map(|relation| &relation.rel));

                            // collections entered from global scope
                            let mut inner_collections = HashMap::with_capacity_and_hasher(dependencies.len(), FnvBuildHasher::default());

                            for dep in dependencies {
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
                                    let c = program.mk_rule(
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
                            let mut new_collections = HashMap::with_capacity_and_hasher(rels.len(), FnvBuildHasher::default());
                            for rel in rels {
                                let var = vars
                                    .get(&rel.rel.id)
                                    .ok_or_else(|| format!("no variable found for relation ID {}", rel.rel.id))?;

                                let mut collection = var.leave();
                                // var.distinct() will be called automatically by var.drop() if var has `distinct` flag set
                                if rel.rel.distinct && !rel.distinct {
                                    collection = with_prof_context(
                                        &format!("{}.distinct_total", rel.rel.name),
                                        || collection.threshold_total(|_,c| if *c == 0 { 0 } else { 1 }),
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
                                if arr.queryable() || program.arrangement_used_by_nodes((rel.rel.id, i)).any(|n| n != nodeid) {
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
                if let Some(relation_callback) = &program.get_relation(relid).change_cb {
                    let relation_callback = relation_callback.clone();

                    let consolidated = with_prof_context(
                        &format!("consolidate {}", relid),
                        || collection.consolidate(),
                    );

                    let inspected = with_prof_context(
                        &format!("inspect {}", relid),
                        || consolidated.inspect(move |x| {
                            // assert!(x.2 == 1 || x.2 == -1, "x: {:?}", x);
                            (relation_callback)(relid, &x.0, x.2)
                        }),
                    );

                    with_prof_context(
                        &format!("probe {}", relid),
                        || inspected.probe_with(&mut probe),
                    );
                }
            }

            // Attach probes to index arrangements, so we know when all updates
            // for a given epoch have been added to the arrangement, and return
            // arrangement trace.
            let mut traces: BTreeMap<ArrId, _> = BTreeMap::new();
            for ((relid, arrid), arr) in arrangements.into_iter() {
                if let ArrangedCollection::Map(arranged) = arr {
                    if program.get_relation(relid).arrangements[arrid].queryable() {
                        arranged.as_collection(|k,_| k.clone()).probe_with(&mut probe);
                        traces.insert((relid, arrid), arranged.trace.clone());
                    }
                }
            }

            Ok((sessions, traces))
        })
    }
}

#[derive(Clone)]
pub struct ProfilingData {
    /// Whether CPU profiling is enabled
    cpu_enabled: Arc<AtomicBool>,
    /// Whether timely profiling is enabled
    timely_enabled: Arc<AtomicBool>,
    /// The channel used to send profiling data to the profiling thread
    data_channel: Sender<ProfMsg>,
}

impl ProfilingData {
    /// Create a new profiling instance
    pub const fn new(
        cpu_enabled: Arc<AtomicBool>,
        timely_enabled: Arc<AtomicBool>,
        data_channel: Sender<ProfMsg>,
    ) -> Self {
        Self {
            cpu_enabled,
            timely_enabled,
            data_channel,
        }
    }

    /// Whether CPU profiling is enabled
    pub fn is_cpu_enabled(&self) -> bool {
        self.cpu_enabled.load(Ordering::Relaxed)
    }

    /// Whether timely profiling is enabled
    pub fn is_timely_enabled(&self) -> bool {
        self.timely_enabled.load(Ordering::Relaxed)
    }

    /// Record a profiling message
    pub fn record(&self, event: ProfMsg) {
        let _ = self.data_channel.send(event);
    }
}
