//! Memory profile of a DDlog program.

use crate::profile_statistics::Statistics;
use differential_dataflow::logging::DifferentialEvent;
use fnv::FnvHashMap;
use sequence_trie::SequenceTrie;
use std::cell::RefCell;
use std::cmp::max;
use std::fmt;
use std::time::Duration;
use timely::logging::{OperatesEvent, ScheduleEvent, StartStop, TimelyEvent};

thread_local! {
    pub static PROF_CONTEXT: RefCell<String> = RefCell::new("".to_string());
}

pub fn set_prof_context(s: &str) {
    PROF_CONTEXT.with(|ctx| *ctx.borrow_mut() = s.to_string());
}

pub fn get_prof_context() -> String {
    PROF_CONTEXT.with(|ctx| ctx.borrow().to_string())
}

pub fn with_prof_context<T, F: FnOnce() -> T>(s: &str, f: F) -> T {
    set_prof_context(s);
    let res = f();
    set_prof_context("");
    res
}

/* Profiling information message sent by worker to profiling thread
 */
#[derive(Debug)]
pub enum ProfMsg {
    /// Send message batch as well as who the message is for (_, profile_cpu, profile_timely).
    TimelyMessage(
        Vec<((Duration, usize, TimelyEvent), Option<String>)>,
        bool,
        bool,
    ),
    DifferentialMessage(Vec<(Duration, usize, DifferentialEvent)>),
}

#[derive(Debug)]
pub struct Profile {
    addresses: SequenceTrie<usize, usize>,
    op_address: FnvHashMap<usize, Vec<usize>>,
    /// Full name of operator including context for mapping to ddlog.
    names: FnvHashMap<usize, String>,
    /// Short name of the op only.
    short_names: FnvHashMap<usize, String>,
    sizes: FnvHashMap<usize, isize>,
    peak_sizes: FnvHashMap<usize, isize>,
    starts: FnvHashMap<(usize, usize), Duration>,
    durations: FnvHashMap<usize, (Duration, usize)>,
    // Initialization creates a file
    timely_stats: Option<Statistics>,
    // Keep track of whether we already tried initializing timely_stats, this avoids us
    // repeatedly trying to initialize it on every event batch. If we failed once we give
    // up.
    stats_init: bool,
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "\nArrangement size profile\n")?;
        self.fmt_sizes(&self.sizes, f)?;

        write!(f, "\nArrangement peak sizes\n")?;
        self.fmt_sizes(&self.peak_sizes, f)?;

        write!(f, "\nCPU profile\n")?;
        self.fmt_durations(0, &self.addresses, f)?;

        Ok(())
    }
}

impl Profile {
    pub fn new() -> Profile {
        Profile {
            addresses: SequenceTrie::new(),
            op_address: FnvHashMap::default(),
            names: FnvHashMap::default(),
            short_names: FnvHashMap::default(),
            sizes: FnvHashMap::default(),
            peak_sizes: FnvHashMap::default(),
            starts: FnvHashMap::default(),
            durations: FnvHashMap::default(),
            timely_stats: None,
            stats_init: false,
        }
    }

    pub fn fmt_sizes(
        &self,
        sizes: &FnvHashMap<usize, isize>,
        f: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error> {
        let mut size_vec: Vec<(usize, isize)> = sizes.clone().into_iter().collect();
        size_vec.sort_by(|a, b| a.1.cmp(&b.1).reverse());
        size_vec
            .iter()
            .map(|(operator, size)| {
                let name = self.names.get(operator).map(AsRef::as_ref).unwrap_or("???");
                let msg = format!("{} {}", name, operator);
                writeln!(f, "{}      {}", size, msg)
            })
            .collect()
    }

    pub fn fmt_durations(
        &self,
        depth: usize,
        addrs: &SequenceTrie<usize, usize>,
        f: &mut fmt::Formatter,
    ) -> Result<(), fmt::Error> {
        /* Sort children in the order of decreasing duration */
        let mut children = addrs.children();
        children.sort_by(|child1, child2| {
            let dur1 = child1
                .value()
                .map(|opid| self.durations.get(opid).cloned().unwrap_or_default().0)
                .unwrap_or_default();
            let dur2 = child2
                .value()
                .map(|opid| self.durations.get(opid).cloned().unwrap_or_default().0)
                .unwrap_or_default();
            dur1.cmp(&dur2).reverse()
        });

        for child in children.iter() {
            /* Print the duration of the child before calling the function recursively on it */
            match child.value() {
                None => {
                    writeln!(f, "Unknown operator")?;
                }
                Some(opid) => {
                    let name = self.names.get(opid).map(AsRef::as_ref).unwrap_or("???");
                    let duration = self.durations.get(opid).cloned().unwrap_or_default();
                    let msg = format!("{} {}", name, opid);
                    let offset = (0..depth * 2).map(|_| " ").collect::<String>();
                    writeln!(
                        f,
                        "{}{: >6}s{:0>6}us ({: >9}calls)     {}",
                        offset,
                        duration.0.as_secs(),
                        duration.0.subsec_micros(),
                        duration.1,
                        msg
                    )?;
                }
            }
            self.fmt_durations(depth + 1, child, f)?;
        }
        Ok(())
    }

    pub fn update(&mut self, msg: &ProfMsg) {
        match msg {
            ProfMsg::TimelyMessage(events, profile_cpu, profile_timely) => {
                // Init stats struct for timely events. The profile_timely bool can become true
                // at any time, so we check it on every message batch arrival.
                if !self.stats_init && *profile_timely {
                    self.stats_init = true;

                    match Statistics::new("stats.csv") {
                        Ok(init_stats) => {
                            self.timely_stats = Some(init_stats);
                        }
                        Err(e) => {
                            eprintln!("Warning: Unable to create stats.csv for program profiling.");
                            eprintln!("Reason {}", e);
                            // stats stays None.
                        }
                    }
                }
                for ((duration, id, event), context) in events.iter() {
                    match event {
                        TimelyEvent::Operates(o) => {
                            let context = context.as_ref().expect(
                                "Operates events should always have valid context attached",
                            );
                            self.handle_operates(&o, context);
                        }
                        event => {
                            if *profile_timely {
                                // In the None case it is totally fine to do nothing. This just means that
                                // profiling timely was on but we were unable to initialize the file.
                                if let Some(stats) = self.timely_stats.as_mut() {
                                    stats.handle_event(
                                        *duration,
                                        *id,
                                        &event,
                                        &self.op_address,
                                        &self.short_names,
                                    );
                                }
                            }
                            if *profile_cpu {
                                self.handle_cpu_profiling(duration, *id, event);
                            }
                        }
                    }
                }
            }

            ProfMsg::DifferentialMessage(msg) => self.handle_differential(msg),
        }
    }

    /// Add events into relevant maps. This must always be done as we might need this information
    /// later if CPU profile or timely profile is turned on. If we don't always record it, it might
    /// be too late later.
    fn handle_operates(&mut self, OperatesEvent { id, addr, name }: &OperatesEvent, context: &str) {
        self.addresses.insert(addr, *id);
        self.op_address.insert(*id, addr.clone());

        self.short_names.insert(*id, name.clone());
        self.names.insert(*id, {
            /* Remove redundant spaces. */
            let frags: Vec<String> = (name.clone() + ": " + &context.replace('\n', " "))
                .split_whitespace()
                .map(|x| x.to_string())
                .collect();
            frags.join(" ")
        });
    }

    // We always want to handle TimelyEvent::Operates as they are used for more than just
    // CPU profiling. Other events are only handled when profile_cpu is true.
    fn handle_cpu_profiling(&mut self, ts: &Duration, worker_id: usize, event: &TimelyEvent) {
        if let TimelyEvent::Schedule(ScheduleEvent { id, start_stop }) = event {
            match start_stop {
                StartStop::Start => {
                    self.starts.insert((*id, worker_id), *ts);
                }
                StartStop::Stop => {
                    let (total, ncalls) = self
                        .durations
                        .entry(*id)
                        .or_insert((Duration::new(0, 0), 0));
                    let start = self
                        .starts
                        .get(&(*id, worker_id))
                        .cloned()
                        .unwrap_or_else(|| {
                            eprintln!(
                                "TimelyEvent::Stop without a start for operator {}, worker {}",
                                *id, worker_id
                            );
                            Duration::new(0, 0)
                        });
                    *total += *ts - start;
                    *ncalls += 1;
                }
            }
        }
    }

    fn handle_differential(&mut self, msg: &[(Duration, usize, DifferentialEvent)]) {
        //eprintln!("profiling message: {:?}", msg);
        for (_, _, event) in msg.iter() {
            match event {
                DifferentialEvent::Batch(x) => {
                    let size = self.sizes.entry(x.operator).or_insert(0);
                    *size += x.length as isize;
                    let peak = self.peak_sizes.entry(x.operator).or_insert(0);
                    *peak = max(*peak, *size);
                }
                DifferentialEvent::Merge(m) => {
                    if let Some(complete) = m.complete {
                        let size = self.sizes.entry(m.operator).or_insert(0);
                        *size += (complete as isize) - (m.length1 + m.length2) as isize;
                        let peak = self.peak_sizes.entry(m.operator).or_insert(0);
                        *peak = max(*peak, *size);
                    }
                }
                _ => (),
            }
        }
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self::new()
    }
}
