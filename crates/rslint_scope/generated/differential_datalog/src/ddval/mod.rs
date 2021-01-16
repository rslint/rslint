//! DDValue: Generic value type stored in all differential-dataflow relations.
//!
//! Rationale: Differential dataflow allows the user to assign an arbitrary user-defined type to
//! each collection.  It relies on Rust's static dispatch mechanism to specialize its internal
//! machinery for each user-defined type.  Unfortunately, beyond very simple programs this leads to
//! extremely long compilation times.  One workaround that we used to rely on is to declare a
//! single enum type with a variant per concrete type used in at least one relation.  This make
//! compilation feasible, but still very slow (~6 minutes for a simple DDlog program and ~10
//! minutes for complex programs).
//!
//! Another alternative we implement here is to use a fixed value type that does not depend on
//! a concrete DDlog program and rely on dynamic dispatch to forward operations that DD expects
//! all values to implement (comparison, hashing, etc.) to their concrete implementations.  This
//! way this crate (differential-datalog) can be compiled all the way to binary code separately
//! from the DDlog program using it and does not need to be re-compiled when the DDlog program
//! changes.  Thus, the only part that must be re-compiled on changes to the DDlog code is the
//! auto-generated crate that declares concrete value types and rules.  This is much faster than
//! re-compiling both crates together.
//!
//! The next design decision is how to implement dynamic dispatch.  Rust trait objects is an
//! obvious choice, with value type being declared as `Box<dyn SomeTrait>`.  However, this proved
//! suboptimal in our experiments, as this design requires a dynamic memory allocation per value,
//! no matter how small.  Furthermore, cloning a value (which DD does a lot, e.g., during
//! compaction) requires another allocation.
//!
//! We improve over this naive design in two ways.  First, we use `Arc` instead of `Box`, which
//! introduces extra space overhead to store the reference count, but avoids memory allocation due
//! to cloning and shares the same heap allocation across multiple copies of the value.  Second, we
//! store small objects <=`usize` bytes inline instead of wrapping them in an Arc to avoid dynamic
//! memory allocation for such objects altogether.  Unfortunately, Rust's dynamic dispatch
//! mechanism does not support this, so we roll our own instead, with the following `DDValue`
//! declaration:
//!
//! ```
//! use differential_datalog::ddval::*;
//! pub struct DDValue {
//!    val: DDVal,
//!    vtable: &'static DDValMethods,
//! }
//! ```
//!
//! where `DDVal` is a `usize` that stores either an `Arc<T>` or `T` (where `T` is the actual type
//! of value stored in the DDlog relation), and `DDValMethods` is a virtual table of methods that
//! must be implemented for all DD values.
//!
//! This design still requires a separate heap allocation for each value >8 bytes, which slows
//! things down quite a bit.  Nevertheless, it has the same performance as our earlier
//! implementation using static dispatch and at least in some benchmarks uses less memory.  The
//! only way to improve things further I can think of is to somehow co-design this with DD to use
//! DD's knowledge of the context where a value is being created to, e.g., allocate blocks of
//! values when possible.
//!

#[macro_use]
mod ddval_convert;
mod ddvalue;

pub use ddval_convert::DDValConvert;
pub use ddvalue::DDValue;

use crate::record::Record;
use std::{
    any::TypeId,
    cmp::Ordering,
    fmt::{Error, Formatter},
    hash::Hasher,
};

/// Type-erased representation of a value.  Can store the actual value or a pointer to it.
/// This could be just a `usize`, but we wrap it in a struct as we don't want it to implement
/// `Copy`.
pub struct DDVal {
    pub v: usize,
}

/// vtable of methods to be implemented by every value stored in DD.
pub struct DDValMethods {
    pub clone: fn(this: &DDVal) -> DDVal,
    pub into_record: fn(this: DDVal) -> Record,

    /// Safety: The types of the values contained in `this` and `other` must be the same
    pub eq: unsafe fn(this: &DDVal, other: &DDVal) -> bool,

    /// Safety: The types of the values contained in `this` and `other` must be the same
    pub partial_cmp: unsafe fn(this: &DDVal, other: &DDVal) -> Option<Ordering>,

    /// Safety: The types of the values contained in `this` and `other` must be the same
    pub cmp: unsafe fn(this: &DDVal, other: &DDVal) -> Ordering,

    pub hash: fn(this: &DDVal, state: &mut dyn Hasher),
    pub mutate: fn(this: &mut DDVal, record: &Record) -> Result<(), String>,
    pub fmt_debug: fn(this: &DDVal, f: &mut Formatter) -> Result<(), Error>,
    pub fmt_display: fn(this: &DDVal, f: &mut Formatter) -> Result<(), Error>,
    pub drop: fn(this: &mut DDVal),
    pub ddval_serialize: fn(this: &DDVal) -> &dyn erased_serde::Serialize,
    pub type_id: fn(this: &DDVal) -> TypeId,
}
