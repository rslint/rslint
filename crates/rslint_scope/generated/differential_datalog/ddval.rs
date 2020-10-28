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

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;

use serde::ser::Serialize;
use serde::ser::Serializer;

use abomonation::Abomonation;

use crate::record::IntoRecord;
use crate::record::Mutator;
use crate::record::Record;

use ordered_float::OrderedFloat;

/// Type-erased representation of a value.  Can store the actual value or a pointer to it.
/// This could be just a `usize`, but we wrap it in a struct as we don't want it to implement
/// `Copy`.
pub struct DDVal {
    pub v: usize,
}

/// DDValue: this type is stored in all DD collections.
/// It consists of value and associated vtable.
pub struct DDValue {
    val: DDVal,
    vtable: &'static DDValMethods,
}

/// vtable of methods to be implemented by every value stored in DD.
pub struct DDValMethods {
    pub clone: fn(this: &DDVal) -> DDVal,
    pub into_record: fn(this: DDVal) -> Record,
    pub eq: fn(this: &DDVal, other: &DDVal) -> bool,
    pub partial_cmp: fn(this: &DDVal, other: &DDVal) -> Option<std::cmp::Ordering>,
    pub cmp: fn(this: &DDVal, other: &DDVal) -> std::cmp::Ordering,
    pub hash: fn(this: &DDVal, state: &mut dyn Hasher),
    pub mutate: fn(this: &mut DDVal, record: &Record) -> Result<(), String>,
    pub fmt_debug: fn(this: &DDVal, f: &mut Formatter) -> Result<(), std::fmt::Error>,
    pub fmt_display: fn(this: &DDVal, f: &mut Formatter) -> Result<(), std::fmt::Error>,
    pub drop: fn(this: &mut DDVal),
    pub ddval_serialize: fn(this: &DDVal) -> &dyn erased_serde::Serialize,
}

impl Drop for DDValue {
    fn drop(&mut self) {
        (self.vtable.drop)(&mut self.val);
    }
}

impl DDValue {
    pub fn new(val: DDVal, vtable: &'static DDValMethods) -> DDValue {
        DDValue { val, vtable }
    }

    pub fn into_ddval(self) -> DDVal {
        let res = DDVal { v: self.val.v };
        std::mem::forget(self);
        res
    }
}

impl Mutator<DDValue> for Record {
    fn mutate(&self, x: &mut DDValue) -> Result<(), String> {
        (x.vtable.mutate)(&mut x.val, self)
    }
}

impl IntoRecord for DDValue {
    fn into_record(self) -> Record {
        (self.vtable.into_record)(self.into_ddval())
    }
}

impl Abomonation for DDValue {
    unsafe fn entomb<W: std::io::Write>(&self, _write: &mut W) -> std::io::Result<()> {
        panic!("DDValue::entomb: not implemented")
    }
    unsafe fn exhume<'a, 'b>(&'a mut self, _bytes: &'b mut [u8]) -> Option<&'b mut [u8]> {
        panic!("DDValue::exhume: not implemented")
    }
    fn extent(&self) -> usize {
        panic!("DDValue::extent: not implemented")
    }
}

/* `Serialize` implementation simply forwards the `serialize` operation to the
 * inner object.
 * Note: we cannot provide a generic `Deserialize` implementation for `DDValue`,
 * as there is no object to forward `deserialize` to.  Instead, we are going
 * generate a `Deserialize` implementation for `Update<DDValue>` in the DDlog
 * compiler. This implementation will use relation id inside `Update` to figure
 * out which type to deserialize.  See `src/lib.rs` for more details.
 */
impl Serialize for DDValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        erased_serde::serialize((self.vtable.ddval_serialize)(&self.val), serializer)
    }
}

impl Display for DDValue {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        (self.vtable.fmt_display)(&self.val, f)
    }
}

impl Debug for DDValue {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        (self.vtable.fmt_debug)(&self.val, f)
    }
}

impl PartialOrd for DDValue {
    fn partial_cmp(&self, other: &DDValue) -> Option<std::cmp::Ordering> {
        (self.vtable.partial_cmp)(&self.val, &other.val)
    }
}

impl PartialEq for DDValue {
    fn eq(&self, other: &Self) -> bool {
        (self.vtable.eq)(&self.val, &other.val)
    }
}

impl Eq for DDValue {}

impl Ord for DDValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.vtable.cmp)(&self.val, &other.val)
    }
}

impl Clone for DDValue {
    fn clone(&self) -> Self {
        DDValue {
            val: (self.vtable.clone)(&self.val),
            vtable: self.vtable,
        }
    }
}

impl Hash for DDValue {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        (self.vtable.hash)(&self.val, state)
    }
}

/// Trait to convert `DDVal` into concrete value type and back.
pub trait DDValConvert: Sized {
    /// Extract reference to concrete type from `&DDVal`.  This causes undefined behavior
    /// if `v` does not contain a value of type `Self`.
    unsafe fn from_ddval_ref(v: &DDVal) -> &Self;

    unsafe fn from_ddvalue_ref(v: &DDValue) -> &Self {
        Self::from_ddval_ref(&v.val)
    }

    /// Extracts concrete value contained in `v`.  Panics if `v` does not contain a
    /// value of type `Self`.
    unsafe fn from_ddval(v: DDVal) -> Self;

    unsafe fn from_ddvalue(v: DDValue) -> Self {
        Self::from_ddval(v.into_ddval())
    }

    /// Convert a value to a `DDVal`, erasing its original type.  This is a safe conversion
    /// that cannot fail.
    fn into_ddval(self) -> DDVal;

    fn ddvalue(&self) -> DDValue;
    fn into_ddvalue(self) -> DDValue;
}

/// Macro to implement `DDValConvert` for type `t` that satisfies the following type bounds:
///
/// t: Eq + Ord + Clone + Send + Debug + Sync + Hash + PartialOrd + IntoRecord + 'static,
/// Record: Mutator<t>
///
#[macro_export]
macro_rules! decl_ddval_convert {
    ( $t:ty ) => {
        impl $crate::ddval::DDValConvert for $t {
            unsafe fn from_ddval_ref(v: &$crate::ddval::DDVal) -> &Self {
                if ::std::mem::size_of::<Self>() <= ::std::mem::size_of::<usize>() {
                    &*(&v.v as *const usize as *const Self)
                } else {
                    &*(v.v as *const Self)
                }
            }

            unsafe fn from_ddval(v: $crate::ddval::DDVal) -> Self {
                if ::std::mem::size_of::<Self>() <= ::std::mem::size_of::<usize>() {
                    let res: Self =
                        ::std::mem::transmute::<[u8; ::std::mem::size_of::<Self>()], Self>(
                            *(&v.v as *const usize as *const [u8; ::std::mem::size_of::<Self>()]),
                        );
                    ::std::mem::forget(v);
                    res
                } else {
                    let arc = ::std::sync::Arc::from_raw(v.v as *const Self);
                    ::std::sync::Arc::try_unwrap(arc).unwrap_or_else(|a| (*a).clone())
                }
            }

            fn into_ddval(self) -> $crate::ddval::DDVal {
                if ::std::mem::size_of::<Self>() <= ::std::mem::size_of::<usize>() {
                    let mut v: usize = 0;
                    unsafe {
                        *(&mut v as *mut usize as *mut [u8; ::std::mem::size_of::<Self>()]) =
                            ::std::mem::transmute::<Self, [u8; ::std::mem::size_of::<Self>()]>(
                                self,
                            );
                    };
                    $crate::ddval::DDVal { v }
                } else {
                    $crate::ddval::DDVal {
                        v: ::std::sync::Arc::into_raw(::std::sync::Arc::new(self)) as usize,
                    }
                }
            }

            fn ddvalue(&self) -> $crate::ddval::DDValue {
                $crate::ddval::DDValConvert::into_ddvalue(self.clone())
            }

            fn into_ddvalue(self) -> $crate::ddval::DDValue {
                const VTABLE: $crate::ddval::DDValMethods = $crate::ddval::DDValMethods {
                    clone: {
                        fn __f(this: &$crate::ddval::DDVal) -> $crate::ddval::DDVal {
                            if ::std::mem::size_of::<$t>() <= ::std::mem::size_of::<usize>() {
                                unsafe { <$t>::from_ddval_ref(this) }.clone().into_ddval()
                            } else {
                                let arc =
                                    unsafe { ::std::sync::Arc::from_raw(this.v as *const $t) };
                                let res = $crate::ddval::DDVal {
                                    v: ::std::sync::Arc::into_raw(arc.clone()) as usize,
                                };
                                ::std::sync::Arc::into_raw(arc);
                                res
                            }
                        };
                        __f
                    },
                    into_record: {
                        fn __f(this: $crate::ddval::DDVal) -> $crate::record::Record {
                            unsafe { <$t>::from_ddval(this) }.into_record()
                        };
                        __f
                    },
                    eq: {
                        fn __f(this: &$crate::ddval::DDVal, other: &$crate::ddval::DDVal) -> bool {
                            unsafe { <$t>::from_ddval_ref(this).eq(<$t>::from_ddval_ref(other)) }
                        };
                        __f
                    },
                    partial_cmp: {
                        fn __f(
                            this: &$crate::ddval::DDVal,
                            other: &$crate::ddval::DDVal,
                        ) -> Option<::std::cmp::Ordering> {
                            unsafe {
                                <$t>::from_ddval_ref(this).partial_cmp(<$t>::from_ddval_ref(other))
                            }
                        };
                        __f
                    },
                    cmp: {
                        fn __f(
                            this: &$crate::ddval::DDVal,
                            other: &$crate::ddval::DDVal,
                        ) -> ::std::cmp::Ordering {
                            unsafe { <$t>::from_ddval_ref(this).cmp(<$t>::from_ddval_ref(other)) }
                        };
                        __f
                    },
                    hash: {
                        fn __f(
                            this: &$crate::ddval::DDVal,
                            mut state: &mut dyn ::std::hash::Hasher,
                        ) {
                            ::std::hash::Hash::hash(
                                unsafe { <$t>::from_ddval_ref(this) },
                                &mut state,
                            );
                        };
                        __f
                    },
                    mutate: {
                        fn __f(
                            this: &mut $crate::ddval::DDVal,
                            record: &$crate::record::Record,
                        ) -> Result<(), ::std::string::String> {
                            let mut clone = unsafe { <$t>::from_ddval_ref(this) }.clone();
                            $crate::record::Mutator::mutate(record, &mut clone)?;
                            *this = clone.into_ddval();
                            Ok(())
                        };
                        __f
                    },
                    fmt_debug: {
                        fn __f(
                            this: &$crate::ddval::DDVal,
                            f: &mut ::std::fmt::Formatter,
                        ) -> Result<(), ::std::fmt::Error> {
                            ::std::fmt::Debug::fmt(unsafe { <$t>::from_ddval_ref(this) }, f)
                        };
                        __f
                    },
                    fmt_display: {
                        fn __f(
                            this: &$crate::ddval::DDVal,
                            f: &mut ::std::fmt::Formatter,
                        ) -> Result<(), ::std::fmt::Error> {
                            ::std::fmt::Display::fmt(
                                &unsafe { <$t>::from_ddval_ref(this) }.clone().into_record(),
                                f,
                            )
                        };
                        __f
                    },
                    drop: {
                        fn __f(this: &mut $crate::ddval::DDVal) {
                            if ::std::mem::size_of::<$t>() <= ::std::mem::size_of::<usize>() {
                                unsafe {
                                    let _v: $t = ::std::mem::transmute::<
                                        [u8; ::std::mem::size_of::<$t>()],
                                        $t,
                                    >(
                                        *(&this.v as *const usize
                                            as *const [u8; ::std::mem::size_of::<$t>()]),
                                    );
                                };
                            // v's destructor will do the rest.
                            } else {
                                let _arc =
                                    unsafe { ::std::sync::Arc::from_raw(this.v as *const $t) };
                                // arc's destructor will do the rest.
                            }
                        };
                        __f
                    },
                    ddval_serialize: {
                        fn __f(this: &$crate::ddval::DDVal) -> &dyn erased_serde::Serialize {
                            (unsafe { <$t>::from_ddval_ref(this) }) as &dyn erased_serde::Serialize
                        };
                        __f
                    },
                };
                $crate::ddval::DDValue::new(self.into_ddval(), &VTABLE)
            }
        }
    };
}

/* Implement `DDValConvert` for builtin types. */

decl_ddval_convert! {()}
decl_ddval_convert! {u8}
decl_ddval_convert! {u16}
decl_ddval_convert! {u32}
decl_ddval_convert! {u64}
decl_ddval_convert! {u128}
decl_ddval_convert! {i8}
decl_ddval_convert! {i16}
decl_ddval_convert! {i32}
decl_ddval_convert! {i64}
decl_ddval_convert! {i128}
decl_ddval_convert! {String}
decl_ddval_convert! {bool}
decl_ddval_convert! {OrderedFloat<f32>}
decl_ddval_convert! {OrderedFloat<f64>}
