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


use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::string::ToString;

pub fn debug_event<T1: ToString, A1: Clone + IntoRecord, A2: Clone + IntoRecord>(
    operator_id: &ddlog_std::tuple3<u32, u32, u32>,
    w: &ddlog_std::DDWeight,
    ts: &T1,
    operator_type: &String,
    input1: &A1,
    out: &A2,
) {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("debug.log".to_string())
        .unwrap();

    let _ = writeln!(
        &file,
        "({},{},{}), {}, {}, {}, {}, {}",
        &operator_id.0,
        &operator_id.1,
        &operator_id.2,
        &w.to_string(),
        &ts.to_string(),
        &operator_type,
        &input1.clone().into_record(),
        &out.clone().into_record()
    );
}

pub fn debug_event_join<
    T1: ToString,
    A1: Clone + IntoRecord,
    A2: Clone + IntoRecord,
    A3: Clone + IntoRecord,
>(
    operator_id: &ddlog_std::tuple3<u32, u32, u32>,
    w: &ddlog_std::DDWeight,
    ts: &T1,
    input1: &A1,
    input2: &A2,
    out: &A3,
) {
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("debug.log".to_string())
        .unwrap();

    let _ = writeln!(
        &file,
        "({},{},{}), {}, {}, Join, {}, {}, {}",
        &operator_id.0,
        &operator_id.1,
        &operator_id.2,
        &w.to_string(),
        &ts.to_string(),
        &input1.clone().into_record(),
        &input2.clone().into_record(),
        &out.clone().into_record()
    );
}

pub fn debug_split_group<K: Clone, I: 'static + Clone, V: Clone + 'static>(
    g: &ddlog_std::Group<K, ddlog_std::tuple2<I, V>>,
) -> ddlog_std::tuple2<ddlog_std::Vec<I>, ddlog_std::Group<K, V>> {
    let mut inputs = ddlog_std::Vec::with_capacity(ddlog_std::group_count(g) as usize);
    let mut vals = ::std::vec::Vec::with_capacity(ddlog_std::group_count(g) as usize);
    for ddlog_std::tuple2(i, v) in g.iter() {
        inputs.push(i);
        vals.push(v);
    }

    ddlog_std::tuple2(inputs, ddlog_std::Group::new(g.key(), vals))
}

pub type DDlogOpId = ddlog_std::tuple3<u32, u32, u32>;
/* fn debug_event<T1: ::ddlog_rt::Val,A1: ::ddlog_rt::Val,A2: ::ddlog_rt::Val>(operator_id: & DDlogOpId, w: & ddlog_std::DDWeight, ts: & T1, operator_type: & String, input1: & A1, out: & A2) -> () */
/* fn debug_event_join<T1: ::ddlog_rt::Val,A1: ::ddlog_rt::Val,A2: ::ddlog_rt::Val,A3: ::ddlog_rt::Val>(operator_id: & DDlogOpId, w: & ddlog_std::DDWeight, ts: & T1, input1: & A1, input2: & A2, out: & A3) -> () */
/* fn debug_split_group<K: ::ddlog_rt::Val,I: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & ddlog_std::Group<K, ddlog_std::tuple2<I, V>>) -> ddlog_std::tuple2<ddlog_std::Vec<I>, ddlog_std::Group<K, V>> */