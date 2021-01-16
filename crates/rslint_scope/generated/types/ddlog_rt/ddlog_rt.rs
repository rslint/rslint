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
use ::differential_datalog::ddval::DDValConvert;
use ::differential_datalog::ddval::DDValue;
use ::differential_datalog::program;
use ::differential_datalog::program::TupleTS;
use ::differential_datalog::program::Weight;
use ::differential_datalog::program::XFormArrangement;
use ::differential_datalog::program::XFormCollection;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;

// `usize` and `isize` are builtin Rust types; we therefore declare an alias to DDlog's `usize` and
// `isize`.
pub type std_usize = u64;
pub type std_isize = i64;

use std::cmp::Ordering;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::hash::Hash;
use std::hash::Hasher;
use std::result::Result;

use serde::de::Error;
use serde::Deserializer;
use serde::Serializer;

/* This module is designed to be imported both as a standard DDlog library and as a normal Rust
 * module, e.g., from `differential_datalog_test`.  We therefore need to import thit trait
 * so that it is available in the latter case and rename it so that it doesn't cause duplicate
 * import error in the former case. */
use differential_datalog::record::IntoRecord as IntoRec;
use differential_datalog::record::Record;
use ordered_float::OrderedFloat;

use abomonation::Abomonation;

/// All DDlog types are expected to implement this trait.  In particular, it is used as a type
/// bound on all type variables.
pub trait Val:
    Default
    + Eq
    + Ord
    + Clone
    + Hash
    + PartialEq
    + PartialOrd
    + serde::Serialize
    + ::serde::de::DeserializeOwned
    + 'static
{
}

impl<T> Val for T where
    T: Default
        + Eq
        + Ord
        + Clone
        + Hash
        + PartialEq
        + PartialOrd
        + serde::Serialize
        + ::serde::de::DeserializeOwned
        + 'static
{
}

/// Use in generated Rust code to implement string concatenation (`++`)
pub fn string_append_str(mut s1: String, s2: &str) -> String {
    s1.push_str(s2);
    s1
}

/// Use in generated Rust code to implement string concatenation (`++`)
#[allow(clippy::ptr_arg)]
pub fn string_append(mut s1: String, s2: &String) -> String {
    s1.push_str(s2.as_str());
    s1
}

/// Used to implement fields with `deserialize_from_array` attribute.
/// Generates a module with `serialize` and `deserialize` methods.
/// Takes the name of the module to generate, key type (`ktype`),
/// value type (`vtype`), and a function that extracts key from array
/// element of type `vtype`.
///
/// Example:
/// ```
/// ddlog_rt::deserialize_map_from_array!(__serdejson_test_StructWithMap_f,u64,StructWithKey,key_structWithKey);
/// ````
#[macro_export]
macro_rules! deserialize_map_from_array {
    ( $modname:ident, $ktype:ty, $vtype:ty, $kfunc:path ) => {
        mod $modname {
            use super::*;
            use serde::de::{Deserialize, Deserializer};
            use serde::ser::Serializer;
            use std::collections::BTreeMap;

            pub fn serialize<S>(
                map: &ddlog_std::Map<$ktype, $vtype>,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.collect_seq(map.x.values())
            }

            pub fn deserialize<'de, D>(
                deserializer: D,
            ) -> Result<ddlog_std::Map<$ktype, $vtype>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let v = Vec::<$vtype>::deserialize(deserializer)?;
                Ok(v.into_iter().map(|item| ($kfunc(&item), item)).collect())
            }
        }
    };
}

/* Runtime support for DDlog closures. */

/* DDlog's equivalent of Rust's `Fn` trait.  This is necessary, as Rust does not allow manual
 * implementations of `Fn` trait (until `unboxed_closures` and `fn_traits` features are
 * stabilized).  Otherwise, we would just derive `Fn` and add methods for comparison and hashing.
 */
pub trait Closure<Args, Output>: Send + Sync {
    fn call(&self, args: Args) -> Output;
    /* Returns pointers to function and captured arguments, for use in comparison methods. */
    fn internals(&self) -> (usize, usize);
    fn clone_dyn(&self) -> Box<dyn Closure<Args, Output>>;
    fn eq_dyn(&self, other: &dyn Closure<Args, Output>) -> bool;
    fn cmp_dyn(&self, other: &dyn Closure<Args, Output>) -> Ordering;
    fn hash_dyn(&self, state: &mut dyn Hasher);
    fn into_record_dyn(&self) -> Record;
    fn fmt_debug_dyn(&self, f: &mut Formatter) -> std::fmt::Result;
    fn fmt_display_dyn(&self, f: &mut Formatter) -> std::fmt::Result;
    fn serialize_dyn(&self) -> &dyn erased_serde::Serialize;
}

#[derive(Clone)]
pub struct ClosureImpl<Args, Output, Captured: Val> {
    pub description: &'static str,
    pub captured: Captured,
    pub f: fn(args: Args, captured: &Captured) -> Output,
}

impl<Args, Output, Captured: Debug + Val> serde::Serialize for ClosureImpl<Args, Output, Captured> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }
}

/* Rust forces 'static trait bound on `Args` and `Output`, as the borrow checker is not smart
 * enough to realize that they are only used as arguments to `f`.
 */
impl<Args: Clone + 'static, Output: Clone + 'static, Captured: Debug + Val + Send + Sync>
    Closure<Args, Output> for ClosureImpl<Args, Output, Captured>
{
    fn call(&self, args: Args) -> Output {
        (self.f)(args, &self.captured)
    }

    fn clone_dyn(&self) -> Box<dyn Closure<Args, Output>> {
        Box::new((*self).clone()) as Box<dyn Closure<Args, Output>>
    }

    fn internals(&self) -> (usize, usize) {
        (
            self.f as *const (fn(Args, &Captured) -> Output) as usize,
            &self.captured as *const Captured as usize,
        )
    }

    fn eq_dyn(&self, other: &dyn Closure<Args, Output>) -> bool {
        /* Compare function pointers.  If equal, it is safe to compare captured variables. */
        let (other_f, other_captured) = other.internals();
        if (other_f == (self.f as *const (fn(Args, &Captured) -> Output) as usize)) {
            unsafe { *(other_captured as *const Captured) == self.captured }
        } else {
            false
        }
    }

    fn cmp_dyn(&self, other: &dyn Closure<Args, Output>) -> Ordering {
        let (other_f, other_captured) = other.internals();
        match ((self.f as *const (fn(Args, &Captured) -> Output) as usize).cmp(&other_f)) {
            Ordering::Equal => self
                .captured
                .cmp(unsafe { &*(other_captured as *const Captured) }),
            ord => ord,
        }
    }

    fn hash_dyn(&self, mut state: &mut dyn Hasher) {
        self.captured.hash(&mut state);
        (self.f as *const (fn(Args, &Captured) -> Output) as usize).hash(&mut state);
    }

    fn into_record_dyn(&self) -> Record {
        Record::String(format!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }

    fn fmt_debug_dyn(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }

    fn fmt_display_dyn(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "<closure: {}, captured_args: {:?}>",
            self.description, self.captured
        ))
    }

    fn serialize_dyn(&self) -> &dyn erased_serde::Serialize {
        self as &dyn erased_serde::Serialize
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> Display for Box<dyn Closure<Args, Output>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_display_dyn(f)
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> Debug for Box<dyn Closure<Args, Output>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_debug_dyn(f)
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> PartialEq<&Self>
    for Box<dyn Closure<Args, Output>>
{
    fn eq(&self, other: &&Self) -> bool {
        self.eq_dyn(&***other)
    }
}

/* This extra impl is a workaround for compiler bug that fails to derive `PartialEq` for
 * structs that contain fields of type `Box<dyn Closure<>>`. See:
 * https://github.com/rust-lang/rust/issues/31740#issuecomment-700950186 */
impl<Args: Clone + 'static, Output: Clone + 'static> PartialEq for Box<dyn Closure<Args, Output>> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_dyn(&**other)
    }
}
impl<Args: Clone + 'static, Output: Clone + 'static> Eq for Box<dyn Closure<Args, Output>> {}

impl<Args: Clone + 'static, Output: Clone + 'static> PartialOrd for Box<dyn Closure<Args, Output>> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_dyn(&**other))
    }
}
impl<Args: Clone + 'static, Output: Clone + 'static> Ord for Box<dyn Closure<Args, Output>> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_dyn(&**other)
    }
}

impl<Args: Clone + 'static, Output: Clone + 'static> Clone for Box<dyn Closure<Args, Output>> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone + Default> Default
    for Box<dyn Closure<Args, Output>>
{
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn default() -> Self {
        Box::new(ClosureImpl {
            description: "default closure",
            captured: (),
            f: {
                fn __f<A, O: Default>(args: A, captured: &()) -> O {
                    O::default()
                };
                __f
            },
        })
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> Hash for Box<dyn Closure<Args, Output>> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.hash_dyn(state);
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> serde::Serialize
    for Box<dyn Closure<Args, Output>>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        erased_serde::serialize((self.serialize_dyn()), serializer)
    }
}

impl<'de, Args: 'static + Clone, Output: 'static + Clone> serde::Deserialize<'de>
    for Box<dyn Closure<Args, Output>>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(D::Error::custom(
            "Deserialization of closures is not implemented.",
        ))
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone>
    differential_datalog::record::Mutator<Box<dyn Closure<Args, Output>>> for Record
{
    fn mutate(&self, x: &mut Box<dyn Closure<Args, Output>>) -> Result<(), String> {
        Err("'mutate' not implemented for closures.".to_string())
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> differential_datalog::record::IntoRecord
    for Box<dyn Closure<Args, Output>>
{
    fn into_record(self) -> Record {
        self.into_record_dyn()
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> differential_datalog::record::FromRecord
    for Box<dyn Closure<Args, Output>>
{
    fn from_record(val: &Record) -> Result<Self, String> {
        Err("'from_record' not implemented for closures.".to_string())
    }
}

impl<Args: 'static + Clone, Output: 'static + Clone> Abomonation
    for Box<dyn Closure<Args, Output>>
{
    unsafe fn entomb<W: std::io::Write>(&self, _write: &mut W) -> std::io::Result<()> {
        panic!("Closure::entomb: not implemented")
    }
    unsafe fn exhume<'a, 'b>(&'a mut self, _bytes: &'b mut [u8]) -> Option<&'b mut [u8]> {
        panic!("Closure::exhume: not implemented")
    }
    fn extent(&self) -> usize {
        panic!("Closure::extent: not implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::Closure;
    use super::ClosureImpl;
    use serde::Deserialize;
    use serde::Serialize;

    #[test]
    fn closure_test() {
        let closure1: ClosureImpl<(*const String, *const u32), Vec<String>, Vec<u64>> =
            ClosureImpl {
                description: "test closure 1",
                captured: vec![0, 1, 2, 3],
                f: {
                    fn __f(args: (*const String, *const u32), captured: &Vec<u64>) -> Vec<String> {
                        captured
                            .iter()
                            .map(|x| {
                                format!(
                                    "x: {}, arg0: {}, arg1: {}",
                                    x,
                                    unsafe { &*args.0 },
                                    unsafe { &*args.1 }
                                )
                            })
                            .collect()
                    };
                    __f
                },
            };

        let closure2: ClosureImpl<(*const String, *const u32), Vec<String>, String> = ClosureImpl {
            description: "test closure 1",
            captured: "Bar".to_string(),
            f: {
                fn __f(args: (*const String, *const u32), captured: &String) -> Vec<String> {
                    vec![format!(
                        "captured: {}, arg0: {}, arg1: {}",
                        captured,
                        unsafe { &*args.0 },
                        unsafe { &*args.1 }
                    )]
                };
                __f
            },
        };

        let ref arg1 = "bar".to_string();
        let ref arg2: u32 = 100;
        assert_eq!(
            closure1.call((arg1, arg2)),
            vec![
                "x: 0, arg0: bar, arg1: 100",
                "x: 1, arg0: bar, arg1: 100",
                "x: 2, arg0: bar, arg1: 100",
                "x: 3, arg0: bar, arg1: 100"
            ]
        );
        assert!(closure1.eq_dyn(&*closure1.clone_dyn()));
        assert!(closure2.eq_dyn(&*closure2.clone_dyn()));
        assert_eq!(closure1.eq_dyn(&closure2), false);
    }

    /* Make sure that auto-derives work for closures. */

    #[derive(Eq, PartialEq, Ord, Clone, Hash, PartialOrd, Default, Serialize, Deserialize)]
    pub struct IntClosure {
        pub f: Box<dyn Closure<*const i64, i64>>,
    }

    #[derive(Eq, PartialEq, Ord, Clone, Hash, PartialOrd, Serialize, Deserialize)]
    pub enum ClosureEnum {
        Enum1 {
            f: Box<dyn Closure<*const i64, i64>>,
        },
        Enum2 {
            f: Box<dyn Closure<(*mut Vec<String>, *const IntClosure), ()>>,
        },
    }
}
