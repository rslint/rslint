use crate::{
    ddval::{DDVal, DDValMethods, DDValue},
    record::{IntoRecord, Mutator, Record},
};
use std::{
    any::{Any, TypeId},
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    mem::{self, align_of, size_of, ManuallyDrop},
    sync::Arc,
};

/// Trait to convert `DDVal` into concrete value type and back.
pub trait DDValConvert: Sized {
    /// Extract reference to concrete type from `&DDVal`.
    ///
    /// # Safety
    ///
    /// `value` **must** be the same type as the type the `DDVal` was created with
    ///
    unsafe fn from_ddval_ref(value: &DDVal) -> &Self;

    /// Converts an `&DDValue` into a reference of the given type
    ///
    /// Returns `None` if the type given is not the same as the type the `DDValue`
    /// was created with
    ///
    fn try_from_ddvalue_ref(value: &DDValue) -> Option<&Self>
    where
        Self: 'static,
    {
        let value_type = (value.vtable.type_id)(&value.val);
        if value_type == TypeId::of::<Self>() {
            // Safety: The type we're turning the value into is the same as the one
            //         it was created with
            Some(unsafe { Self::from_ddval_ref(&value.val) })
        } else {
            None
        }
    }

    /// Converts an `&DDValue` into a reference of the given type
    ///
    /// # Panics
    ///
    /// Panics if the type given is not the same as the type the `DDValue`
    /// was created with
    ///
    fn from_ddvalue_ref(value: &DDValue) -> &Self
    where
        Self: 'static,
    {
        Self::try_from_ddvalue_ref(value)
            .expect("attempted to convert a DDValue into the incorrect type")
    }

    /// Extracts concrete value contained in `value`.
    ///
    /// # Safety
    ///
    /// `value` **must** be the same type as the type the `DDValue` was created with
    ///
    unsafe fn from_ddval(value: DDVal) -> Self;

    /// Converts a `DDValue` into a the given type
    ///
    /// Returns `None` if the type given is not the same as the type the `DDValue`
    /// was created with
    ///
    fn try_from_ddvalue(value: DDValue) -> Option<Self>
    where
        Self: 'static,
    {
        let value_type = (value.vtable.type_id)(&value.val);
        if value_type == TypeId::of::<Self>() {
            // Safety: The type we're turning the value into is the same as the one
            //         it was created with
            Some(unsafe { Self::from_ddval(value.into_ddval()) })
        } else {
            None
        }
    }

    /// Converts a `DDValue` into the given type
    ///
    /// # Panics
    ///
    /// Panics if the type given is not the same as the type the `DDValue`
    /// was created with
    ///
    fn from_ddvalue(value: DDValue) -> Self
    where
        Self: 'static,
    {
        Self::try_from_ddvalue(value)
            .expect("attempted to convert a DDValue into the incorrect type")
    }

    /// Convert a value to a `DDVal`, erasing its original type.
    ///
    /// This is a safe conversion that cannot fail.
    fn into_ddval(self) -> DDVal;

    /// Creates a `DDValue` from the current value
    fn ddvalue(&self) -> DDValue;

    /// Converts the current value into a `DDValue`
    fn into_ddvalue(self) -> DDValue;

    /// The vtable containing all `DDValue` methods for the current type
    const VTABLE: DDValMethods;
}

/// Implement `DDValConvert` for all types that satisfy its type constraints
impl<T> DDValConvert for T
where
    T: Any
        + Clone
        + Debug
        + IntoRecord
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + Hash
        + Send
        + Sync
        + erased_serde::Serialize
        + 'static,
    Record: Mutator<T>,
{
    unsafe fn from_ddval_ref(value: &DDVal) -> &Self {
        let fits_in_usize =
            size_of::<Self>() <= size_of::<usize>() && align_of::<Self>() <= align_of::<usize>();

        if fits_in_usize {
            &*<*const usize>::cast::<Self>(&value.v)
        } else {
            &*(value.v as *const Self)
        }
    }

    unsafe fn from_ddval(value: DDVal) -> Self {
        let fits_in_usize =
            size_of::<Self>() <= size_of::<usize>() && align_of::<Self>() <= align_of::<usize>();

        if fits_in_usize {
            <*const usize>::cast::<Self>(&value.v).read()
        } else {
            let arc = Arc::from_raw(value.v as *const Self);
            Arc::try_unwrap(arc).unwrap_or_else(|a| (*a).clone())
        }
    }

    fn into_ddval(self) -> DDVal {
        let fits_in_usize =
            size_of::<Self>() <= size_of::<usize>() && align_of::<Self>() <= align_of::<usize>();

        // The size and alignment of the `T` must be less than or equal to a
        // `usize`'s, otherwise we store it within an `Arc`
        if fits_in_usize {
            let mut v: usize = 0;
            unsafe { <*mut usize>::cast::<Self>(&mut v).write(self) };

            DDVal { v }
        } else {
            DDVal {
                v: Arc::into_raw(Arc::new(self)) as usize,
            }
        }
    }

    fn ddvalue(&self) -> DDValue {
        DDValConvert::into_ddvalue(self.clone())
    }

    fn into_ddvalue(self) -> DDValue {
        DDValue::new(self.into_ddval(), &Self::VTABLE)
    }

    const VTABLE: DDValMethods = {
        let clone = |this: &DDVal| -> DDVal {
            let fits_in_usize = size_of::<Self>() <= size_of::<usize>()
                && align_of::<Self>() <= align_of::<usize>();

            if fits_in_usize {
                unsafe { <Self>::from_ddval_ref(this) }.clone().into_ddval()
            } else {
                let arc = unsafe { ManuallyDrop::new(Arc::from_raw(this.v as *const Self)) };

                DDVal {
                    v: Arc::into_raw(Arc::clone(&arc)) as usize,
                }
            }
        };

        let into_record =
            |this: DDVal| -> Record { unsafe { <Self>::from_ddval(this) }.into_record() };

        let eq: unsafe fn(&DDVal, &DDVal) -> bool =
            |this, other| unsafe { <Self>::from_ddval_ref(this).eq(<Self>::from_ddval_ref(other)) };

        let partial_cmp: unsafe fn(&DDVal, &DDVal) -> Option<Ordering> = |this, other| unsafe {
            <Self>::from_ddval_ref(this).partial_cmp(<Self>::from_ddval_ref(other))
        };

        let cmp: unsafe fn(&DDVal, &DDVal) -> Ordering = |this, other| unsafe {
            <Self>::from_ddval_ref(this).cmp(<Self>::from_ddval_ref(other))
        };

        let hash = |this: &DDVal, mut state: &mut dyn Hasher| {
            Hash::hash(unsafe { <Self>::from_ddval_ref(this) }, &mut state);
        };

        let mutate = |this: &mut DDVal, record: &Record| -> Result<(), String> {
            let mut clone = unsafe { <Self>::from_ddval_ref(this) }.clone();
            Mutator::mutate(record, &mut clone)?;
            *this = clone.into_ddval();

            Ok(())
        };

        let fmt_debug = |this: &DDVal, f: &mut Formatter| -> Result<(), fmt::Error> {
            Debug::fmt(unsafe { <Self>::from_ddval_ref(this) }, f)
        };

        let fmt_display = |this: &DDVal, f: &mut Formatter| -> Result<(), fmt::Error> {
            Display::fmt(
                &unsafe { <Self>::from_ddval_ref(this) }
                    .clone()
                    .into_record(),
                f,
            )
        };

        let drop = |this: &mut DDVal| {
            let fits_in_usize = size_of::<Self>() <= size_of::<usize>()
                && align_of::<Self>() <= align_of::<usize>();

            if fits_in_usize {
                // Allow the inner value's Drop impl to run automatically
                let _val = unsafe { <*const usize>::cast::<Self>(&this.v).read() };
            } else {
                let arc = unsafe { Arc::from_raw(this.v as *const Self) };
                mem::drop(arc);
            }
        };

        let ddval_serialize: fn(&DDVal) -> &dyn erased_serde::Serialize =
            |this| unsafe { <Self>::from_ddval_ref(this) as &dyn erased_serde::Serialize };

        let type_id = |_this: &DDVal| -> TypeId { TypeId::of::<Self>() };

        DDValMethods {
            clone,
            into_record,
            eq,
            partial_cmp,
            cmp,
            hash,
            mutate,
            fmt_debug,
            fmt_display,
            drop,
            ddval_serialize,
            type_id,
        }
    };
}
