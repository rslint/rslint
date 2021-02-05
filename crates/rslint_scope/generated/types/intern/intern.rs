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


use differential_datalog::record::{self, Record};
use fxhash::FxBuildHasher;
use lasso::{Capacity, Key, Spur, ThreadedRodeo};
use once_cell::sync::Lazy;
use serde::{de::Deserializer, ser::Serializer};
use std::{
    fmt::{self, Debug, Display, Formatter},
    marker::PhantomData,
    num::NonZeroUsize,
};

pub static GLOBAL_STRING_INTERNER: Lazy<ThreadedRodeo<Spur, FxBuildHasher>> = Lazy::new(|| {
    // Safety: `NonZeroUsize::new_unchecked()` must be called with a non-zero
    //         integer, and 4096 is not zero
    ThreadedRodeo::with_capacity_and_hasher(
        Capacity::new(512, unsafe { NonZeroUsize::new_unchecked(4096) }),
        FxBuildHasher::default(),
    )
});

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IObj<T> {
    key: Spur,
    __type: PhantomData<T>,
}

impl<T> IObj<T> {
    pub const fn new(key: Spur) -> Self {
        Self {
            key,
            __type: PhantomData,
        }
    }
}

impl Default for IString {
    fn default() -> Self {
        IObj::new(GLOBAL_STRING_INTERNER.get_or_intern_static(""))
    }
}

pub fn string_intern(string: &String) -> IString {
    IObj::new(GLOBAL_STRING_INTERNER.get_or_intern(string))
}

pub fn istring_str(string: &IString) -> String {
    GLOBAL_STRING_INTERNER.resolve(&string.key).to_owned()
}

pub fn istring_ord(string: &IString) -> u32 {
    string.key.into_usize() as u32
}

impl Display for IString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let string = GLOBAL_STRING_INTERNER.resolve(&self.key);
        record::format_ddlog_str(string, f)
    }
}

impl Debug for IString {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let string = GLOBAL_STRING_INTERNER.resolve(&self.key);
        record::format_ddlog_str(string, f)
    }
}

impl Serialize for IString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let string = GLOBAL_STRING_INTERNER.resolve(&self.key);
        string.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| string_intern(&s))
    }
}

impl FromRecord for IString {
    fn from_record(val: &Record) -> Result<Self, String> {
        String::from_record(val).map(|s| string_intern(&s))
    }
}

impl IntoRecord for IString {
    fn into_record(self) -> Record {
        istring_str(&self).into_record()
    }
}

impl Mutator<IString> for Record {
    fn mutate(&self, s: &mut IString) -> Result<(), String> {
        let mut string = istring_str(s);
        self.mutate(&mut string)?;
        *s = string_intern(&string);

        Ok(())
    }
}

pub type IString = IObj<String>;
/* fn istring_ord(s: & IString) -> u32 */
/* fn istring_str(s: & IString) -> String */
/* fn string_intern(s: & String) -> IString */