use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use csv::Writer;
use serde::{Deserialize, Serialize};

use fnv::FnvHashMap;
use serde::export::Formatter;
use std::fmt;
use std::fmt::Debug;
use std::fs::File;
use timely::logging::{ParkEvent, StartStop, TimelyEvent};

/// Possible events fields for CSV event_type column.
#[derive(Serialize, Deserialize, Debug, Clone)]
enum CSVEventType {
    Invalid,
    OperatorCreation,
    OperatorCall,
    Schedule,
    Progress,
    PushProgress,
    Message,
    Shutdown,
    ChannelCreation,
    Application,
    GuardedMessage,
    GuardedProgress,
    CommChannels,
    Input,
    Park,
    Text,
    Activation,
    EventCounts,
}

impl Default for CSVEventType {
    fn default() -> Self {
        CSVEventType::Invalid
    }
}

/// Map from (worker_id, op_id) to start time for timely events.
#[repr(transparent)]
struct StartTimeKeeper<K> {
    start_times: HashMap<K, Duration>,
}

impl<K> StartTimeKeeper<K>
where
    K: Hash + Eq + Debug,
{
    fn new() -> StartTimeKeeper<K> {
        StartTimeKeeper {
            start_times: HashMap::new(),
        }
    }

    fn new_start_time(&mut self, key: K, time: Duration) {
        assert!(!self.start_times.contains_key(&key));
        self.start_times.insert(key, time);
    }

    fn pop_start_time(&mut self, key: &K) -> Option<Duration> {
        self.start_times.remove(key)
    }
}

/// This struct will be serialized to create a row for our CSV format where the CSV columns equal
/// this struct's fields. Use CSVLogEvent::default() to pre-fill all fields and only assign fields
/// relevant for your logging event.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
struct CSVLogEvent {
    worker_id: usize,
    // Physical time the event took place.
    start_time: Option<u128>,
    end_time: Option<u128>,
    // Worker-unique identifier for this operator.
    operator_id: Option<usize>,
    event_type: CSVEventType,
    // Total execution time of event. Only applicable to operator activation right now.
    elapsed_time: Option<u128>,
    operator_name: Option<String>,
    // We use string for the addr instead of Vec<usize> since CSV writer attempts to turn the vec
    // into multiple columns for an entry. Not what we want.
    operator_addr: Option<String>,
    // Keep track of whether send or receive event.
    is_send: Option<bool>,
    source_worker: Option<usize>,
    // Channel id for this event.
    channel: Option<usize>,
    sequence_number: Option<usize>,
    target_worker: Option<usize>,
    // Length in bytes of message data.
    data_length: Option<usize>,
}

impl CSVLogEvent {
    /// CSV serializer will try to turn vectors into comma delimted values. Turn vector into a string
    /// with elements separated by '-'. E.g: "1-2-3" == vec_to_csv_string(vec![1, 2, 3])
    fn vec_to_csv_string(v: &[usize]) -> String {
        let mut s = String::new();
        for n in v {
            s.push_str(&n.to_string());
            s.push('-');
        }
        s
    }

    fn guarded_message_entry(
        worker_id: usize,
        start_time: &Duration,
        end_time: &Duration,
    ) -> CSVLogEvent {
        let elapsed_time = (*end_time - *start_time).as_nanos();

        CSVLogEvent {
            worker_id,
            start_time: Some(start_time.as_nanos()),
            end_time: Some(end_time.as_nanos()),
            event_type: CSVEventType::GuardedMessage,
            elapsed_time: Some(elapsed_time),
            ..CSVLogEvent::default()
        }
    }

    fn message_entry(
        worker_id: usize,
        is_send: bool,
        channel_id: usize,
        source: usize,
        target: usize,
        data_length: usize,
    ) -> CSVLogEvent {
        CSVLogEvent {
            worker_id,
            event_type: CSVEventType::Message,
            is_send: Some(is_send),
            channel: Some(channel_id),
            source_worker: Some(source),
            target_worker: Some(target),
            data_length: Some(data_length),
            ..CSVLogEvent::default()
        }
    }

    fn park_entry(worker_id: usize, start_time: &Duration, end_time: &Duration) -> CSVLogEvent {
        let elapsed_time = (*end_time - *start_time).as_nanos();

        CSVLogEvent {
            worker_id,
            start_time: Some(start_time.as_nanos()),
            end_time: Some(end_time.as_nanos()),
            event_type: CSVEventType::Park,
            elapsed_time: Some(elapsed_time),
            ..CSVLogEvent::default()
        }
    }

    // fn activation_entry(worker_id: usize, elapsed_time: &Duration) -> CSVLogEvent {
    //     CSVLogEvent {
    //         worker_id: worker_id,
    //         event_type: CSVEventType::Activation,
    //         elapsed_time: Some(elapsed_time.as_nanos()),
    //         ..CSVLogEvent::default()
    //     }
    // }

    fn schedule_entry(
        worker_id: usize,
        start_time: &Duration,
        end_time: &Duration,
        operator_id: usize,
        operator_name: String,
        operator_addr: &[usize],
    ) -> CSVLogEvent {
        let execution_time = *end_time - *start_time;

        CSVLogEvent {
            worker_id,
            start_time: Some(start_time.as_nanos()),
            end_time: Some(end_time.as_nanos()),
            operator_id: Some(operator_id),
            event_type: CSVEventType::Schedule,
            elapsed_time: Some(execution_time.as_nanos()),
            operator_name: Some(operator_name),
            operator_addr: Some(CSVLogEvent::vec_to_csv_string(operator_addr)),
            ..CSVLogEvent::default()
        }
    }

    fn progress_entry(
        worker_id: usize,
        source_worker: usize,
        operator_addr: &[usize],
        sequence_number: usize,
        is_send: bool,
        channel_id: usize,
    ) -> CSVLogEvent {
        CSVLogEvent {
            worker_id,
            source_worker: Some(source_worker),
            event_type: CSVEventType::Progress,
            operator_addr: Some(CSVLogEvent::vec_to_csv_string(operator_addr)),
            sequence_number: Some(sequence_number),
            is_send: Some(is_send),
            channel: Some(channel_id),
            ..CSVLogEvent::default()
        }
    }

    fn push_progress(worker_id: usize, operator_id: usize) -> CSVLogEvent {
        CSVLogEvent {
            worker_id,
            operator_id: Some(operator_id),
            event_type: CSVEventType::PushProgress,
            ..CSVLogEvent::default()
        }
    }
}

// Events may arrive out of order. Keep track start/stop times to coalesce two-part events, like
// Schedule into single event. Also contains stateful information about each operator; Mainly
// name and address.
pub struct Statistics {
    // Tuple of (worker_id, op_id) make for unique key.
    schedule_start_time: StartTimeKeeper<(usize, usize)>,
    park_start_time: StartTimeKeeper<usize>,
    guarded_message_start_time: StartTimeKeeper<usize>,
    csv_writer: Writer<File>,
}

/// Create Debug
impl Debug for Statistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Statistics")
    }
}

impl Statistics {
    // A new csv file is created on successful return. The file will automatically be filled by
    // calls to `handle_batch`.
    pub fn new(path: &str) -> csv::Result<Statistics> {
        let csv_writer = Writer::from_path(path)?;
        Ok(Statistics {
            schedule_start_time: StartTimeKeeper::new(),
            park_start_time: StartTimeKeeper::new(),
            guarded_message_start_time: StartTimeKeeper::new(),
            csv_writer,
        })
    }

    // Process TimelyEvents at the batch granularity. Writes events to CSV file. `self.csv_write`
    // buffers input so there is no guarantee that events will flushed to file after this function
    // returns. `self` should be dropped to ensure successful drop.
    pub fn handle_event(
        &mut self,
        timestamp: Duration,
        worker_index: usize,
        data: &TimelyEvent,
        addresses: &FnvHashMap<usize, Vec<usize>>,
        names: &FnvHashMap<usize, String>,
    ) {
        match data {
            TimelyEvent::GuardedMessage(g) => {
                if g.is_start {
                    self.guarded_message_start_time
                        .new_start_time(worker_index, timestamp);
                } else {
                    // Start time can be missing due to https://github.com/vmware/differential-datalog/issues/745
                    // In that case we simple ignore this message.
                    if let Some(start_time) = self
                        .guarded_message_start_time
                        .pop_start_time(&worker_index)
                    {
                        let e = CSVLogEvent::guarded_message_entry(
                            worker_index,
                            &start_time,
                            &timestamp,
                        );
                        self.csv_writer
                            .serialize(e)
                            .expect("unable to serialize record");
                    };
                }
            }
            TimelyEvent::Park(ParkEvent::Park(Some(_duration))) => {
                panic!("Park event with duration not handled!")
            }
            TimelyEvent::Park(ParkEvent::Park(None)) => {
                self.park_start_time.new_start_time(worker_index, timestamp);
            }
            TimelyEvent::Park(ParkEvent::Unpark) => {
                // Start time can be missing due to https://github.com/vmware/differential-datalog/issues/745
                // In that case we simple ignore this message.
                if let Some(start_time) = self.park_start_time.pop_start_time(&worker_index) {
                    self.csv_writer
                        .serialize(CSVLogEvent::park_entry(
                            worker_index,
                            &start_time,
                            &timestamp,
                        ))
                        .expect("unable to serialize record");
                }
            }
            // TimelyEvent::ActivationAdvance(e) => {
            //     self.csv_writer.serialize(CSVLogEvent::activation_entry(*worker_index, &e.elapsed)).
            //         expect("unable to serialize record");
            // }
            TimelyEvent::Schedule(s) => {
                match s.start_stop {
                    StartStop::Start => {
                        self.schedule_start_time
                            .new_start_time((worker_index, s.id), timestamp);
                    }
                    StartStop::Stop => {
                        let key = &(worker_index, s.id);
                        // Start time can be missing due to https://github.com/vmware/differential-datalog/issues/745
                        // In that case we simple ignore this message.
                        if let Some(start_time) = self.schedule_start_time.pop_start_time(key) {
                            let op_name = names.get(&s.id).expect("Name should have been in map");
                            let op_addr = addresses
                                .get(&s.id)
                                .expect("Address should have been in map");
                            let e = CSVLogEvent::schedule_entry(
                                worker_index,
                                &start_time,
                                &timestamp,
                                s.id,
                                op_name.clone(),
                                op_addr,
                            );
                            self.csv_writer
                                .serialize(e)
                                .expect("unable to serialize record");
                        }
                    }
                }
            }
            TimelyEvent::Progress(p) => {
                let e = CSVLogEvent::progress_entry(
                    worker_index,
                    p.source,
                    &p.addr,
                    p.seq_no,
                    p.is_send,
                    p.channel,
                );
                self.csv_writer
                    .serialize(e)
                    .expect("unable to serialize record");
            }
            TimelyEvent::PushProgress(p) => {
                self.csv_writer
                    .serialize(CSVLogEvent::push_progress(worker_index, p.op_id))
                    .expect("unable to serialize record");
            }
            TimelyEvent::Messages(m) => {
                let e = CSVLogEvent::message_entry(
                    worker_index,
                    m.is_send,
                    m.channel,
                    m.source,
                    m.target,
                    m.length,
                );
                self.csv_writer
                    .serialize(e)
                    .expect("unable to serialize record");
            }
            _ => {
                // Skip.
            }
        }
    }
}
