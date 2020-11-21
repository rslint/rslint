use crate::{
    ddval::{DDVal, DDValMethods},
    record::{IntoRecord, Mutator, Record},
};
use abomonation::Abomonation;
use serde::ser::{Serialize, Serializer};
use std::{
    any::TypeId,
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
};

/// DDValue: this type is stored in all DD collections.
/// It consists of value and associated vtable.
pub struct DDValue {
    pub(super) val: DDVal,
    pub(super) vtable: &'static DDValMethods,
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

    pub fn type_id(&self) -> TypeId {
        (self.vtable.type_id)(&self.val)
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

/// `Serialize` implementation simply forwards the `serialize` operation to the
/// inner object.
/// Note: we cannot provide a generic `Deserialize` implementation for `DDValue`,
/// as there is no object to forward `deserialize` to.  Instead, we are going
/// generate a `Deserialize` implementation for `Update<DDValue>` in the DDlog
/// compiler. This implementation will use relation id inside `Update` to figure
/// out which type to deserialize.  See `src/lib.rs` for more details.
impl Serialize for DDValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        erased_serde::serialize((self.vtable.ddval_serialize)(&self.val), serializer)
    }
}

impl Display for DDValue {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        (self.vtable.fmt_display)(&self.val, f)
    }
}

impl Debug for DDValue {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        (self.vtable.fmt_debug)(&self.val, f)
    }
}

impl PartialOrd for DDValue {
    fn partial_cmp(&self, other: &DDValue) -> Option<Ordering> {
        if (self.vtable.type_id)(&self.val) == (other.vtable.type_id)(&other.val) {
            // Safety: The types of both values are the same
            unsafe { (self.vtable.partial_cmp)(&self.val, &other.val) }
        } else {
            // TODO: Should this panic instead?
            None
        }
    }
}

impl PartialEq for DDValue {
    fn eq(&self, other: &Self) -> bool {
        if (self.vtable.type_id)(&self.val) == (other.vtable.type_id)(&other.val) {
            // Safety: The types of both values are the same
            unsafe { (self.vtable.eq)(&self.val, &other.val) }
        } else {
            // TODO: Should this panic instead?
            false
        }
    }
}

impl Eq for DDValue {}

impl Ord for DDValue {
    fn cmp(&self, other: &Self) -> Ordering {
        assert_eq!(
            (self.vtable.type_id)(&self.val),
            (other.vtable.type_id)(&other.val),
            "attempted to compare two values of different types",
        );

        // Safety: The types of both values are the same
        unsafe { (self.vtable.cmp)(&self.val, &other.val) }
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
