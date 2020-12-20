//! Datalog timestamps

use abomonation::Abomonation;
use differential_dataflow::lattice::Lattice;
use num::One;
use std::{
    ops::{Add, Mul},
    sync::atomic::AtomicU32,
};
use timely::{
    order::{PartialOrder, Product},
    progress::{PathSummary, Timestamp},
};

/// 16-bit timestamp.
// TODO: get rid of this and use `u16` directly when/if differential implements
// `Lattice`, `Timestamp`, `PathSummary` traits for `u16`.
#[derive(Copy, PartialOrd, PartialEq, Eq, Debug, Default, Clone, Hash, Ord)]
pub struct TS16 {
    pub x: u16,
}

impl TS16 {
    pub const fn max_value() -> TS16 {
        TS16 { x: 0xffff }
    }
}

impl Abomonation for TS16 {}

impl Mul for TS16 {
    type Output = TS16;

    fn mul(self, rhs: TS16) -> Self::Output {
        TS16 { x: self.x * rhs.x }
    }
}

impl Add for TS16 {
    type Output = TS16;

    fn add(self, rhs: TS16) -> Self::Output {
        TS16 { x: self.x + rhs.x }
    }
}

impl One for TS16 {
    fn one() -> Self {
        TS16 { x: 1 }
    }
}

impl PartialOrder for TS16 {
    fn less_equal(&self, other: &Self) -> bool {
        self.x.less_equal(&other.x)
    }

    fn less_than(&self, other: &Self) -> bool {
        self.x.less_than(&other.x)
    }
}

impl Lattice for TS16 {
    fn minimum() -> Self {
        TS16 {
            x: u16::min_value(),
        }
    }

    fn join(&self, other: &Self) -> Self {
        TS16 {
            x: std::cmp::max(self.x, other.x),
        }
    }

    fn meet(&self, other: &Self) -> Self {
        TS16 {
            x: std::cmp::min(self.x, other.x),
        }
    }
}

impl Timestamp for TS16 {
    type Summary = TS16;
}

impl PathSummary<TS16> for TS16 {
    fn results_in(&self, src: &TS16) -> Option<TS16> {
        self.x.checked_add(src.x).map(|y| TS16 { x: y })
    }

    fn followed_by(&self, other: &TS16) -> Option<TS16> {
        self.x.checked_add(other.x).map(|y| TS16 { x: y })
    }
}

impl From<TS16> for u64 {
    fn from(ts: TS16) -> Self {
        ts.x as u64
    }
}

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
pub type TSNested = TS16;

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
