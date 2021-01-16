//! Datalog timestamps

use std::sync::atomic::AtomicU32;
use timely::order::Product;

/// Outer timestamp
pub type TS = u32;
pub(crate) type TSAtomic = AtomicU32;

/// Timestamp for the nested scope
/// Use 16-bit timestamps for inner scopes to save memory
#[cfg(feature = "nested_ts_32")]
pub type TSNested = u32;

/// Timestamp for the nested scope
/// Use 16-bit timestamps for inner scopes to save memory
#[cfg(not(feature = "nested_ts_32"))]
pub type TSNested = u16;

/// `Inspect` operator expects the timestampt to be a tuple.
pub type TupleTS = (TS, TSNested);

pub(crate) trait ToTupleTS {
    fn to_tuple_ts(&self) -> TupleTS;
}

/// 0-extend top-level timestamp to a tuple.
impl ToTupleTS for TS {
    fn to_tuple_ts(&self) -> TupleTS {
        (*self, TSNested::default())
    }
}

impl ToTupleTS for Product<TS, TSNested> {
    fn to_tuple_ts(&self) -> TupleTS {
        (self.outer, self.inner)
    }
}
