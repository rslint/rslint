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
    clippy::unknown_clippy_lints,
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

//use ::serde::de::DeserializeOwned;
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


use abomonation::Abomonation;
/// Rust implementation of DDlog standard library functions and types.
use differential_datalog::{
    decl_record_mutator_struct, decl_struct_from_record, decl_struct_into_record,
    record::{arg_extract, Record},
};
use fnv::FnvHasher;
use num::Zero;
use serde::{
    de::{DeserializeOwned, Deserializer},
    ser::{SerializeStruct, Serializer},
};
use std::{
    borrow,
    cmp::{self, Ordering},
    collections::{btree_map, btree_set, BTreeMap, BTreeSet},
    fmt::{self, Debug, Display, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    io,
    iter::FromIterator,
    mem,
    ops::{self, Add, DerefMut},
    option::Option as StdOption,
    result::Result as StdResult,
    slice,
    sync::Arc,
    vec::{self, Vec as StdVec},
};

const XX_SEED1: u64 = 0x23b691a751d0e108;
const XX_SEED2: u64 = 0x20b09801dce5ff84;

pub fn default<T: Default>() -> T {
    T::default()
}

// Result

/// Convert Rust result type to DDlog's std::Result
pub fn res2std<T, E: Display>(res: StdResult<T, E>) -> Result<T, String> {
    match res {
        Ok(res) => Result::Ok { res },
        Err(e) => Result::Err {
            err: format!("{}", e),
        },
    }
}

pub fn result_unwrap_or_default<T: Default + Clone, E>(res: &Result<T, E>) -> T {
    match res {
        Result::Ok { res } => res.clone(),
        Result::Err { err } => T::default(),
    }
}

// Ref

/// An atomically reference counted reference
#[derive(Eq, PartialOrd, PartialEq, Ord, Clone, Hash)]
pub struct Ref<T> {
    x: Arc<T>,
}

impl<T: Default> Default for Ref<T> {
    fn default() -> Self {
        Self {
            x: Arc::new(T::default()),
        }
    }
}

impl<T> Deref for Ref<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &*self.x
    }
}

impl<T> From<T> for Ref<T> {
    fn from(x: T) -> Self {
        Self { x: Arc::new(x) }
    }
}

impl<T: Abomonation> Abomonation for Ref<T> {
    unsafe fn entomb<W: io::Write>(&self, write: &mut W) -> io::Result<()> {
        self.deref().entomb(write)
    }

    unsafe fn exhume<'a, 'b>(&'a mut self, bytes: &'b mut [u8]) -> StdOption<&'b mut [u8]> {
        Arc::get_mut(&mut self.x).unwrap().exhume(bytes)
    }

    fn extent(&self) -> usize {
        self.deref().extent()
    }
}

impl<T> Ref<T> {
    pub fn get_mut(this: &mut Self) -> StdOption<&mut T> {
        Arc::get_mut(&mut this.x)
    }
}

impl<T: Display> Display for Ref<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.deref().fmt(f)
    }
}

impl<T: Debug> Debug for Ref<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.deref().fmt(f)
    }
}

impl<T: Serialize> Serialize for Ref<T> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.deref().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Ref<T> {
    fn deserialize<D>(deserializer: D) -> StdResult<Ref<T>, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(deserializer).map(Self::from)
    }
}

impl<T: FromRecord> FromRecord for Ref<T> {
    fn from_record(val: &Record) -> StdResult<Self, String> {
        T::from_record(val).map(Self::from)
    }
}

impl<T: IntoRecord + Clone> IntoRecord for Ref<T> {
    fn into_record(self) -> Record {
        (*self.x).clone().into_record()
    }
}

impl<T> Mutator<Ref<T>> for Record
where
    T: Clone,
    Record: Mutator<T>,
{
    fn mutate(&self, arc: &mut Ref<T>) -> StdResult<(), String> {
        let mut copy: T = (*arc).deref().clone();
        self.mutate(&mut copy)?;
        *arc = Ref::from(copy);

        Ok(())
    }
}

pub fn ref_new<A: Clone>(x: &A) -> Ref<A> {
    Ref::from(x.clone())
}

pub fn deref<A: Clone>(x: &Ref<A>) -> &A {
    x.deref()
}

// Arithmetic functions
pub fn u8_pow32(base: &u8, exp: &u32) -> u8 {
    base.wrapping_pow(*exp)
}
pub fn u16_pow32(base: &u16, exp: &u32) -> u16 {
    base.wrapping_pow(*exp)
}
pub fn u32_pow32(base: &u32, exp: &u32) -> u32 {
    base.wrapping_pow(*exp)
}
pub fn u64_pow32(base: &u64, exp: &u32) -> u64 {
    base.wrapping_pow(*exp)
}
pub fn u128_pow32(base: &u128, exp: &u32) -> u128 {
    base.wrapping_pow(*exp)
}
pub fn s8_pow32(base: &i8, exp: &u32) -> i8 {
    base.wrapping_pow(*exp)
}
pub fn s16_pow32(base: &i16, exp: &u32) -> i16 {
    base.wrapping_pow(*exp)
}
pub fn s32_pow32(base: &i32, exp: &u32) -> i32 {
    base.wrapping_pow(*exp)
}
pub fn s64_pow32(base: &i64, exp: &u32) -> i64 {
    base.wrapping_pow(*exp)
}
pub fn s128_pow32(base: &i128, exp: &u32) -> i128 {
    base.wrapping_pow(*exp)
}
pub fn bigint_pow32(base: &ddlog_bigint::Int, exp: &u32) -> ddlog_bigint::Int {
    num::pow::pow(base.clone(), *exp as usize)
}

// Option
pub fn option2std<T>(x: StdOption<T>) -> Option<T> {
    match x {
        StdOption::None => Option::None,
        StdOption::Some(v) => Option::Some { x: v },
    }
}

pub fn std2option<T>(x: Option<T>) -> StdOption<T> {
    match x {
        Option::None => StdOption::None,
        Option::Some { x } => StdOption::Some(x),
    }
}

impl<T> From<StdOption<T>> for Option<T> {
    fn from(x: StdOption<T>) -> Self {
        option2std(x)
    }
}

// this requires Rust 1.41+
impl<T> From<Option<T>> for StdOption<T> {
    fn from(x: Option<T>) -> Self {
        std2option(x)
    }
}

impl<T> FromRecord for Option<T>
where
    T: FromRecord + DeserializeOwned + Default,
{
    fn from_record(val: &Record) -> StdResult<Self, String> {
        match val {
            Record::PosStruct(constr, args) => match constr.as_ref() {
                "ddlog_std::None" if args.len() == 0 => Ok(Option::None {}),
                "ddlog_std::Some" if args.len() == 1 => Ok(Option::Some {
                    x: <T>::from_record(&args[0])?,
                }),
                c => StdResult::Err(format!(
                    "unknown constructor {} of type Option in {:?}",
                    c, *val
                )),
            },

            Record::NamedStruct(constr, args) => match constr.as_ref() {
                "ddlog_std::None" => Ok(Option::None {}),
                "ddlog_std::Some" => Ok(Option::Some {
                    x: arg_extract::<T>(args, "x")?,
                }),
                c => StdResult::Err(format!(
                    "unknown constructor {} of type Option in {:?}",
                    c, *val
                )),
            },

            // `Option` encoded as an array of size 0 or 1.  This is, for instance, useful when
            // interfacing with OVSDB.
            Record::Array(kind, records) => match (records.len()) {
                0 => Ok(Option::None {}),
                1 => Ok(Option::Some {
                    x: T::from_record(&records[0])?,
                }),
                n => Err(format!(
                    "cannot deserialize ddlog_std::Option from container of size {:?}",
                    n
                )),
            },

            Record::Serialized(format, s) => {
                if format == "json" {
                    serde_json::from_str(&*s).map_err(|e| format!("{}", e))
                } else {
                    StdResult::Err(format!("unsupported serialization format '{}'", format))
                }
            }

            v => {
                // Finally, assume that the record contains the inner value of a `Some`.
                // XXX: this introduces ambiguity, as an array could represent either the inner
                // value or an array encoding of `Option`.
                Ok(Option::Some {
                    x: T::from_record(&v)?,
                })
            }
        }
    }
}

pub fn option_unwrap_or_default<T: Default + Clone>(opt: &Option<T>) -> T {
    match opt {
        Option::Some { x } => x.clone(),
        Option::None => T::default(),
    }
}

/*
This function has been deprecated since its definition seems to be
buggy.  By commenting it out we will cause an error for users.

// Range
pub fn range<A: Clone + Ord + ops::Add<Output = A> + PartialOrd>(
    from: &A,
    to: &A,
    step: &A,
) -> Vec<A> {
    let mut vec = Vec::new();
    let mut x = from.clone();
    while x <= *to {
        vec.push(x.clone());
        x = x + step.clone();
    }
    vec
}
*/

// Range
pub fn range_vec<A: Clone + Ord + Add<Output = A> + PartialOrd + Zero>(
    from: &A,
    to: &A,
    step: &A,
) -> Vec<A> {
    let mut vec = Vec::new();
    let mut x = from.clone();

    if step < &A::zero() {
        while x > *to {
            vec.push(x.clone());
            x = x + step.clone();
        }
    } else if step > &A::zero() {
        while x < *to {
            vec.push(x.clone());
            x = x + step.clone();
        }
    }

    vec
}

/// A contiguous growable array type mirroring [`Vec`]
///
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default)]
pub struct Vec<T> {
    pub vec: StdVec<T>,
}

impl<T> Vec<T> {
    /// Creates a new, empty vector
    pub const fn new() -> Self {
        Vec { vec: StdVec::new() }
    }

    /// Creates a new, empty `Vec<T>` with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Vec {
            vec: StdVec::with_capacity(capacity),
        }
    }

    /// Returns an iterator over the vector
    pub fn iter<'a>(&'a self) -> VecIter<'a, T> {
        VecIter::new(self)
    }
}

impl<T: Clone> Vec<T> {
    pub fn resize(&mut self, new_len: usize, value: &T) {
        self.vec.resize_with(new_len, || value.clone());
    }
}

impl<T: Clone> From<&[T]> for Vec<T> {
    fn from(slice: &[T]) -> Self {
        Vec {
            vec: slice.to_vec(),
        }
    }
}

impl<T> FromIterator<T> for Vec<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Self {
            vec: ::std::vec::Vec::from_iter(iter),
        }
    }
}

impl<T> From<StdVec<T>> for Vec<T> {
    fn from(vec: StdVec<T>) -> Self {
        Vec { vec }
    }
}

impl<T> Deref for Vec<T> {
    type Target = StdVec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}

impl<T> AsRef<StdVec<T>> for Vec<T> {
    fn as_ref(&self) -> &StdVec<T> {
        &self.vec
    }
}

impl<T> AsMut<StdVec<T>> for Vec<T> {
    fn as_mut(&mut self) -> &mut StdVec<T> {
        &mut self.vec
    }
}

impl<T> AsRef<[T]> for Vec<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> AsMut<[T]> for Vec<T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.vec.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Vec<T> {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        StdVec::deserialize(deserializer).map(|vec| Vec { vec })
    }
}

impl<T: FromRecord> FromRecord for Vec<T> {
    fn from_record(val: &Record) -> StdResult<Self, String> {
        StdVec::from_record(val).map(|vec| Vec { vec })
    }
}

impl<T: IntoRecord> IntoRecord for Vec<T> {
    fn into_record(self) -> Record {
        self.vec.into_record()
    }
}

impl<T: FromRecord> Mutator<Vec<T>> for Record {
    fn mutate(&self, vec: &mut Vec<T>) -> StdResult<(), String> {
        self.mutate(&mut vec.vec)
    }
}

impl<T: Debug> Debug for Vec<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_list().entries(self.vec.iter()).finish()
    }
}

impl<T> IntoIterator for Vec<T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

// This is needed so we can support for-loops over `Vec`'s
pub struct VecIter<'a, X> {
    iter: slice::Iter<'a, X>,
}

impl<'a, X> VecIter<'a, X> {
    pub fn new(vec: &'a Vec<X>) -> VecIter<'a, X> {
        VecIter {
            iter: vec.vec.iter(),
        }
    }
}

impl<'a, X> Iterator for VecIter<'a, X> {
    type Item = &'a X;

    fn next(&mut self) -> StdOption<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, StdOption<usize>) {
        self.iter.size_hint()
    }
}

/// Get the length of a vec
pub fn vec_len<T>(vec: &Vec<T>) -> std_usize {
    vec.len() as std_usize
}

/// Create a new, empty vec
pub const fn vec_empty<T>() -> Vec<T> {
    Vec::new()
}

/// Create an fill a vec with `len` copies of `splat`
pub fn vec_with_length<T: Clone>(len: &std_usize, splat: &T) -> Vec<T> {
    let mut res = Vec::with_capacity(*len as usize);
    res.resize(*len as usize, splat);
    res
}

/// Create a new vec with the requested capacity
pub fn vec_with_capacity<T>(len: &std_usize) -> Vec<T> {
    Vec::with_capacity(*len as usize)
}

/// Create a new vec with a single element
pub fn vec_singleton<T: Clone>(value: &T) -> Vec<T> {
    Vec {
        vec: vec![value.clone()],
    }
}

pub fn vec_append<T: Clone>(vec: &mut Vec<T>, other: &Vec<T>) {
    vec.extend_from_slice(other.as_slice());
}

pub fn vec_push<T: Clone>(vec: &mut Vec<T>, elem: &T) {
    vec.push(elem.clone());
}

/// Pushes to a vector immutably by creating an entirely new vector and pushing the new
/// element to it
pub fn vec_push_imm<T: Clone>(immutable_vec: &Vec<T>, x: &T) -> Vec<T> {
    let mut mutable_vec = Vec::with_capacity(immutable_vec.len());
    mutable_vec.extend_from_slice(immutable_vec.as_slice());
    mutable_vec.push(x.clone());

    mutable_vec
}

pub fn vec_contains<T: PartialEq>(vec: &Vec<T>, x: &T) -> bool {
    vec.contains(x)
}

pub fn vec_is_empty<T>(vec: &Vec<T>) -> bool {
    vec.is_empty()
}

pub fn vec_nth<T: Clone>(vec: &Vec<T>, nth: &std_usize) -> Option<T> {
    vec.get(*nth as usize).cloned().into()
}

pub fn vec_to_set<T: Ord + Clone>(vec: &Vec<T>) -> Set<T> {
    Set {
        x: vec.vec.iter().cloned().collect(),
    }
}

pub fn vec_sort<T: Ord>(vec: &mut Vec<T>) {
    vec.as_mut_slice().sort();
}

pub fn vec_sort_imm<T: Ord + Clone>(vec: &Vec<T>) -> Vec<T> {
    let mut res = (*vec).clone();
    res.vec.sort();
    res
}

pub fn vec_resize<T: Clone>(vec: &mut Vec<T>, new_len: &std_usize, value: &T) {
    vec.resize(*new_len as usize, value)
}

pub fn vec_truncate<T>(vec: &mut Vec<T>, new_len: &std_usize) {
    vec.vec.truncate(*new_len as usize)
}

pub fn vec_swap_nth<T: Clone>(vec: &mut Vec<T>, idx: &std_usize, value: &mut T) -> bool {
    if (*idx as usize) < vec.len() {
        mem::swap(&mut vec[*idx as usize], value);
        true
    } else {
        false
    }
}

pub fn vec_update_nth<T: Clone>(vec: &mut Vec<T>, idx: &std_usize, value: &T) -> bool {
    if (*idx as usize) < vec.len() {
        vec[*idx as usize] = value.clone();
        true
    } else {
        false
    }
}

pub fn vec_zip<X: Clone, Y: Clone>(v1: &Vec<X>, v2: &Vec<Y>) -> Vec<tuple2<X, Y>> {
    Vec {
        vec: v1
            .iter()
            .zip(v2.iter())
            .map(|(x, y)| tuple2(x.clone(), y.clone()))
            .collect(),
    }
}

// Set

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default)]
pub struct Set<T: Ord> {
    pub x: BTreeSet<T>,
}

impl<T: Ord + Serialize> Serialize for Set<T> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.x.serialize(serializer)
    }
}

impl<'de, T: Ord + Deserialize<'de>> Deserialize<'de> for Set<T> {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BTreeSet::deserialize(deserializer).map(|x| Set { x })
    }
}

/* This is needed so we can support for-loops over `Set`'s
 */
pub struct SetIter<'a, X> {
    iter: btree_set::Iter<'a, X>,
}

impl<'a, T> SetIter<'a, T> {
    pub fn new(set: &'a Set<T>) -> SetIter<'a, T>
    where
        T: Ord,
    {
        SetIter { iter: set.x.iter() }
    }
}

impl<'a, X> Iterator for SetIter<'a, X> {
    type Item = &'a X;

    fn next(&mut self) -> StdOption<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, StdOption<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T: Ord> Set<T> {
    pub fn iter(&'a self) -> SetIter<'a, T> {
        SetIter::new(self)
    }
}

impl<T: Ord> Set<T> {
    pub fn new() -> Self {
        Set { x: BTreeSet::new() }
    }

    pub fn insert(&mut self, v: T) {
        self.x.insert(v);
    }
}

impl<T: FromRecord + Ord> FromRecord for Set<T> {
    fn from_record(val: &Record) -> StdResult<Self, String> {
        BTreeSet::from_record(val).map(|x| Set { x })
    }
}

impl<T: IntoRecord + Ord> IntoRecord for Set<T> {
    fn into_record(self) -> Record {
        self.x.into_record()
    }
}

impl<T: FromRecord + Ord> Mutator<Set<T>> for Record {
    fn mutate(&self, set: &mut Set<T>) -> StdResult<(), String> {
        self.mutate(&mut set.x)
    }
}

impl<T: Ord> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = btree_set::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.x.into_iter()
    }
}

impl<T: Ord> FromIterator<T> for Set<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Set {
            x: BTreeSet::from_iter(iter),
        }
    }
}

impl<T: Debug + Ord> Debug for Set<T> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_set().entries(self.x.iter()).finish()
    }
}

pub fn set_size<X: Ord + Clone>(s: &Set<X>) -> std_usize {
    s.x.len() as std_usize
}

pub fn set_empty<X: Clone + Ord>() -> Set<X> {
    Set::new()
}

pub fn set_singleton<X: Ord + Clone>(v: &X) -> Set<X> {
    let mut s = Set::new();
    s.insert(v.clone());
    s
}

pub fn set_insert<X: Ord + Clone>(s: &mut Set<X>, v: &X) {
    s.x.insert((*v).clone());
}

pub fn set_insert_imm<X: Ord + Clone>(s: &Set<X>, v: &X) -> Set<X> {
    let mut s2 = s.clone();
    s2.insert((*v).clone());
    s2
}

pub fn set_contains<X: Ord>(s: &Set<X>, v: &X) -> bool {
    s.x.contains(v)
}

pub fn set_is_empty<X: Ord>(s: &Set<X>) -> bool {
    s.x.is_empty()
}

pub fn set_nth<X: Ord + Clone>(s: &Set<X>, n: &std_usize) -> Option<X> {
    option2std(s.x.iter().nth(*n as usize).cloned())
}

pub fn set_to_vec<X: Ord + Clone>(set: &Set<X>) -> Vec<X> {
    Vec {
        vec: set.x.iter().cloned().collect(),
    }
}

pub fn set_union<X: Ord + Clone>(s1: &Set<X>, s2: &Set<X>) -> Set<X> {
    let mut s = s1.clone();
    s.x.append(&mut s2.x.clone());
    s
}

pub fn set_unions<X: Ord + Clone>(sets: &Vec<Set<X>>) -> Set<X> {
    let mut s = BTreeSet::new();
    for set in sets.iter() {
        s.append(&mut set.x.clone());
    }

    Set { x: s }
}

pub fn set_intersection<X: Ord + Clone>(s1: &Set<X>, s2: &Set<X>) -> Set<X> {
    Set {
        x: s1.x.intersection(&s2.x).cloned().collect(),
    }
}

pub fn set_difference<X: Ord + Clone>(s1: &Set<X>, s2: &Set<X>) -> Set<X> {
    Set {
        x: s1.x.difference(&s2.x).cloned().collect(),
    }
}

// Map

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default)]
pub struct Map<K: Ord, V> {
    pub x: BTreeMap<K, V>,
}

impl<K: Ord + Serialize, V: Serialize> Serialize for Map<K, V> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.x.serialize(serializer)
    }
}

impl<'de, K: Ord + Deserialize<'de>, V: Deserialize<'de>> Deserialize<'de> for Map<K, V> {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        BTreeMap::deserialize(deserializer).map(|x| Map { x })
    }
}

// This is needed so we can support for-loops over `Map`'s
pub struct MapIter<'a, K, V> {
    iter: btree_map::Iter<'a, K, V>,
}

impl<'a, K: Ord, V> MapIter<'a, K, V> {
    pub fn new(map: &'a Map<K, V>) -> MapIter<'a, K, V> {
        MapIter { iter: map.x.iter() }
    }
}

impl<'a, K: Clone, V: Clone> Iterator for MapIter<'a, K, V> {
    type Item = tuple2<K, V>;

    fn next(&mut self) -> StdOption<Self::Item> {
        self.iter.next().map(|(k, v)| tuple2(k.clone(), v.clone()))
    }

    fn size_hint(&self) -> (usize, StdOption<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, K: Ord, V> Map<K, V> {
    pub fn iter(&'a self) -> MapIter<'a, K, V> {
        MapIter::new(self)
    }
}

impl<K: Ord, V> Map<K, V> {
    pub fn new() -> Self {
        Map { x: BTreeMap::new() }
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.x.insert(k, v);
    }
}

impl<K: FromRecord + Ord, V: FromRecord> FromRecord for Map<K, V> {
    fn from_record(val: &Record) -> StdResult<Self, String> {
        BTreeMap::from_record(val).map(|x| Map { x })
    }
}

impl<K: IntoRecord + Ord, V: IntoRecord> IntoRecord for Map<K, V> {
    fn into_record(self) -> Record {
        self.x.into_record()
    }
}

impl<K: FromRecord + Ord, V: FromRecord + PartialEq> Mutator<Map<K, V>> for Record {
    fn mutate(&self, map: &mut Map<K, V>) -> StdResult<(), String> {
        self.mutate(&mut map.x)
    }
}

pub struct MapIntoIter<K, V> {
    iter: btree_map::IntoIter<K, V>,
}

impl<K: Ord, V> MapIntoIter<K, V> {
    pub fn new(map: Map<K, V>) -> MapIntoIter<K, V> {
        MapIntoIter {
            iter: map.x.into_iter(),
        }
    }
}

impl<K, V> Iterator for MapIntoIter<K, V> {
    type Item = tuple2<K, V>;

    fn next(&mut self) -> StdOption<Self::Item> {
        self.iter.next().map(|(k, v)| tuple2(k, v))
    }

    fn size_hint(&self) -> (usize, StdOption<usize>) {
        self.iter.size_hint()
    }
}

impl<K: Ord, V> IntoIterator for Map<K, V> {
    type Item = tuple2<K, V>;
    type IntoIter = MapIntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::new(self)
    }
}

impl<K: Ord, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Map {
            x: BTreeMap::from_iter(iter),
        }
    }
}

impl<K: Debug + Ord, V: Debug> Debug for Map<K, V> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_map().entries(self.x.iter()).finish()
    }
}

pub fn map_size<K: Ord, V>(m: &Map<K, V>) -> std_usize {
    m.x.len() as std_usize
}

pub fn map_empty<K: Ord + Clone, V: Clone>() -> Map<K, V> {
    Map::new()
}

pub fn map_singleton<K: Ord + Clone, V: Clone>(k: &K, v: &V) -> Map<K, V> {
    let mut m = Map::new();
    m.insert(k.clone(), v.clone());
    m
}

pub fn map_insert<K: Ord + Clone, V: Clone>(m: &mut Map<K, V>, k: &K, v: &V) {
    m.x.insert((*k).clone(), (*v).clone());
}

pub fn map_remove<K: Ord + Clone, V: Clone>(m: &mut Map<K, V>, k: &K) -> Option<V> {
    option2std(m.x.remove(k))
}

pub fn map_insert_imm<K: Ord + Clone, V: Clone>(m: &Map<K, V>, k: &K, v: &V) -> Map<K, V> {
    let mut m2 = m.clone();
    m2.insert((*k).clone(), (*v).clone());
    m2
}

pub fn map_get<K: Ord, V: Clone>(m: &Map<K, V>, k: &K) -> Option<V> {
    option2std(m.x.get(k).cloned())
}

pub fn map_contains_key<K: Ord, V: Clone>(s: &Map<K, V>, k: &K) -> bool {
    s.x.contains_key(k)
}

pub fn map_is_empty<K: Ord, V: Clone>(m: &Map<K, V>) -> bool {
    m.x.is_empty()
}

pub fn map_union<K: Ord + Clone, V: Clone>(m1: &Map<K, V>, m2: &Map<K, V>) -> Map<K, V> {
    let mut m = m1.clone();
    m.x.append(&mut m2.x.clone());
    m
}

pub fn map_keys<K: Ord + Clone, V>(map: &Map<K, V>) -> Vec<K> {
    Vec {
        vec: map.x.keys().cloned().collect(),
    }
}

// strings

pub fn __builtin_2string<T: Display>(x: &T) -> String {
    format!("{}", *x)
}

pub fn hex<T: fmt::LowerHex>(x: &T) -> String {
    format!("{:x}", *x)
}

pub fn parse_dec_u64(s: &String) -> Option<u64> {
    option2std(s.parse::<u64>().ok())
}

pub fn parse_dec_i64(s: &String) -> Option<i64> {
    option2std(s.parse::<i64>().ok())
}

pub fn string_join(strings: &Vec<String>, sep: &String) -> String {
    strings.join(sep.as_str())
}

pub fn string_split(string: &String, sep: &String) -> Vec<String> {
    Vec {
        vec: string.split(sep).map(|x| x.to_owned()).collect(),
    }
}

pub fn string_contains(s1: &String, s2: &String) -> bool {
    s1.contains(s2.as_str())
}

pub fn string_substr(s: &String, start: &std_usize, end: &std_usize) -> String {
    let len = s.len();
    let from = cmp::min(*start as usize, len);
    let to = cmp::max(from, cmp::min(*end as usize, len));
    s[from..to].to_string()
}

pub fn string_replace(s: &String, from: &String, to: &String) -> String {
    s.replace(from, to)
}

pub fn string_starts_with(s: &String, prefix: &String) -> bool {
    s.starts_with(prefix)
}

pub fn string_ends_with(s: &String, suffix: &String) -> bool {
    s.ends_with(suffix)
}

pub fn string_trim(s: &String) -> String {
    s.trim().to_string()
}

pub fn string_len(s: &String) -> std_usize {
    s.len() as std_usize
}

pub fn string_to_bytes(s: &String) -> Vec<u8> {
    Vec::from(s.as_bytes())
}

pub fn str_to_lower(s: &String) -> String {
    s.to_lowercase()
}

pub fn string_to_lowercase(s: &String) -> String {
    s.to_lowercase()
}

pub fn string_to_uppercase(s: &String) -> String {
    s.to_uppercase()
}

pub fn string_reverse(s: &String) -> String {
    s.chars().rev().collect()
}

// Hashing

pub fn hash64<T: Hash>(x: &T) -> u64 {
    let mut hasher = FnvHasher::with_key(XX_SEED1);
    x.hash(&mut hasher);
    hasher.finish()
}

pub fn hash128<T: Hash>(x: &T) -> u128 {
    let mut hasher = FnvHasher::with_key(XX_SEED1);
    x.hash(&mut hasher);
    let w1 = hasher.finish();

    let mut hasher = FnvHasher::with_key(XX_SEED2);
    x.hash(&mut hasher);
    let w2 = hasher.finish();

    ((w1 as u128) << 64) | (w2 as u128)
}

pub type ProjectFunc<X> = Arc<dyn Fn(&DDValue) -> X + Send + Sync>;

/*
 * Group type (returned by the `group_by` operator).
 *
 * A group captures output of the differential dataflow `reduce` operator.
 * Thus, upon creation it is backed by references to DD state.  However, we
 * would like to be able to manipulate groups as normal variables, store then
 * in relations, which requires copying the contents of a group during cloning.
 * Since we want the same code (e.g., the same aggregation functions) to work
 * on both reference-backed and value-backed groups, we represent groups as
 * an enum and provide uniform API over both variants.
 *
 * There is a problem of managing the lifetime of a group.  Since one of the
 * two variants contains references, the group type is parameterized by the
 * lifetime of these refs.  However, in order to be able to freely store and
 * pass groups to and from functions, we want `'static` lifetime.  Because
 * of the way we use groups in DDlog-generated code, we can safely transmute
 * them to the `'static` lifetime upon creation.  Here is why.  A group is
 * always created like this:
 * ```
 * let ref g = GroupEnum::ByRef{key, vals, project}
 * ```
 * where `vals` haa local lifetime `'a` that contains the lifetime
 * `'b` of the resulting reference `g`.  Since we are never going to move
 * `vals` refs out of the group (the accessor API returns them
 * by-value), it is ok to tranmute `g` from `&'b Group<'a>` to
 * `&'b Group<'static>` and have the `'b` lifetime protect access to the group.
 * The only way to use the group outside of `'b` is to clone it, which will
 * create an instance of `ByVal` that truly has `'static` lifetime.
 */

pub type Group<K, V> = GroupEnum<'static, K, V>;

fn test() {
    fn is_sync<T: Send + Sync>() {}
    is_sync::<Group<u8, u8>>(); // compiles only if true
}

pub enum GroupEnum<'a, K, V> {
    ByRef {
        key: K,
        group: &'a [(&'a DDValue, Weight)],
        project: ProjectFunc<V>,
    },
    ByVal {
        key: K,
        group: StdVec<V>,
    },
}

/* Always clone by value. */
impl<K: Clone, V: Clone> Clone for Group<K, V> {
    fn clone(&self) -> Self {
        match self {
            GroupEnum::ByRef {
                key,
                group,
                project,
            } => GroupEnum::ByVal {
                key: key.clone(),
                group: group.iter().map(|(v, _)| project(v)).collect(),
            },
            GroupEnum::ByVal { key, group } => GroupEnum::ByVal {
                key: key.clone(),
                group: group.clone(),
            },
        }
    }
}

impl<K: Default, V: Default> Default for Group<K, V> {
    fn default() -> Self {
        GroupEnum::ByVal {
            key: K::default(),
            group: vec![],
        }
    }
}

/* We compare two groups by comparing values returned by their `project()`
 * functions, not the underlying DDValue's.  DDValue's are not visible to
 * the DDlog program; hence two groups are iff they have the same
 * projections. */

impl<K: PartialEq, V: Clone + PartialEq> PartialEq for Group<K, V> {
    fn eq(&self, other: &Self) -> bool {
        (self.key_ref() == other.key_ref()) && (self.iter().eq(other.iter()))
    }
}

impl<K: PartialEq, V: Clone + PartialEq> Eq for Group<K, V> {}

impl<K: PartialOrd, V: Clone + PartialOrd> PartialOrd for Group<K, V> {
    fn partial_cmp(&self, other: &Self) -> StdOption<Ordering> {
        match self.key_ref().partial_cmp(other.key_ref()) {
            None => None,
            Some(Ordering::Equal) => self.iter().partial_cmp(other.iter()),
            ord => ord,
        }
    }
}

impl<K: Ord, V: Clone + Ord> Ord for Group<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.key_ref().cmp(other.key_ref()) {
            Ordering::Equal => self.iter().cmp(other.iter()),
            ord => ord,
        }
    }
}

/* Likewise, we hash projections, not the underlying DDValue's. */
impl<K: Hash, V: Clone + Hash> Hash for Group<K, V> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.key_ref().hash(state);
        for v in self.iter() {
            v.hash(state);
        }
    }
}

/* We rely on DDlogGroup to serialize/deserialize and print groups. */

impl<K: Clone, V: Clone> DDlogGroup<K, V> {
    pub fn from_group(g: &Group<K, V>) -> Self {
        let vals: StdVec<V> = g.iter().collect();
        DDlogGroup {
            key: g.key(),
            vals: Vec::from(vals),
        }
    }
}

impl<K, V> From<DDlogGroup<K, V>> for Group<K, V> {
    fn from(g: DDlogGroup<K, V>) -> Self {
        Group::new(g.key, g.vals.vec)
    }
}

impl<K: Debug + Clone, V: Debug + Clone> Debug for Group<K, V> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Debug::fmt(&DDlogGroup::from_group(self), f)
    }
}

impl<K: IntoRecord + Clone, V: IntoRecord + Clone> IntoRecord for Group<K, V> {
    fn into_record(self) -> Record {
        DDlogGroup::from_group(&self).into_record()
    }
}

impl<K, V> Mutator<Group<K, V>> for Record
where
    Record: Mutator<K>,
    Record: Mutator<V>,
    K: IntoRecord + FromRecord + Clone,
    V: IntoRecord + FromRecord + Clone,
{
    fn mutate(&self, grp: &mut Group<K, V>) -> StdResult<(), String> {
        let mut dgrp = DDlogGroup::from_group(grp);
        self.mutate(&mut dgrp)?;
        *grp = Group::from(dgrp);
        Ok(())
    }
}

impl<K, V> FromRecord for Group<K, V>
where
    K: Default + FromRecord + serde::de::DeserializeOwned,
    V: Default + FromRecord + serde::de::DeserializeOwned,
{
    fn from_record(rec: &Record) -> StdResult<Self, String> {
        DDlogGroup::from_record(rec).map(|g| Group::from(g))
    }
}

impl<K: Clone + Serialize, V: Clone + Serialize> Serialize for Group<K, V> {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        DDlogGroup::from_group(self).serialize(serializer)
    }
}

impl<'de, K: Deserialize<'de>, V: Deserialize<'de>> Deserialize<'de> for Group<K, V> {
    fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        DDlogGroup::deserialize(deserializer).map(|g| Group::from(g))
    }
}

/* This is needed so we can support for-loops over `Group`'s */
pub enum GroupIter<'a, V> {
    ByRef {
        iter: slice::Iter<'a, (&'static DDValue, Weight)>,
        project: ProjectFunc<V>,
    },
    ByVal {
        iter: slice::Iter<'a, V>,
    },
}

impl<'a, V> GroupIter<'a, V> {
    pub fn new<K>(grp: &'a Group<K, V>) -> GroupIter<'a, V> {
        match grp {
            GroupEnum::ByRef { group, project, .. } => GroupIter::ByRef {
                iter: group.iter(),
                project: project.clone(),
            },
            GroupEnum::ByVal { group, .. } => GroupIter::ByVal { iter: group.iter() },
        }
    }
}

impl<'a, V: Clone> Iterator for GroupIter<'a, V> {
    type Item = V;

    fn next(&mut self) -> StdOption<Self::Item> {
        match self {
            GroupIter::ByRef { iter, project } => match iter.next() {
                None => None,
                Some((x, _)) => Some(project(x)),
            },
            GroupIter::ByVal { iter } => match iter.next() {
                None => None,
                Some(x) => Some(x.clone()),
            },
        }
    }

    fn size_hint(&self) -> (usize, StdOption<usize>) {
        match self {
            GroupIter::ByRef { iter, .. } => iter.size_hint(),
            GroupIter::ByVal { iter } => iter.size_hint(),
        }
    }
}

/* This is needed so we can support FlatMap over `Group`'s */
pub enum GroupIntoIter<V> {
    ByRef {
        iter: slice::Iter<'static, (&'static DDValue, Weight)>,
        project: ProjectFunc<V>,
    },
    ByVal {
        iter: vec::IntoIter<V>,
    },
}

impl<V: Clone> GroupIntoIter<V> {
    pub fn new<K>(grp: Group<K, V>) -> GroupIntoIter<V> {
        match grp {
            GroupEnum::ByRef { group, project, .. } => GroupIntoIter::ByRef {
                iter: group.into_iter(),
                project: project.clone(),
            },
            GroupEnum::ByVal { group, .. } => GroupIntoIter::ByVal {
                iter: group.into_iter(),
            },
        }
    }
}

impl<V: Clone> Iterator for GroupIntoIter<V> {
    type Item = V;

    fn next(&mut self) -> StdOption<Self::Item> {
        match self {
            GroupIntoIter::ByRef { iter, project } => match iter.next() {
                None => None,
                Some((x, _)) => Some(project(x)),
            },
            GroupIntoIter::ByVal { iter } => match iter.next() {
                None => None,
                Some(x) => Some(x.clone()),
            },
        }
    }

    fn size_hint(&self) -> (usize, StdOption<usize>) {
        match self {
            GroupIntoIter::ByRef { iter, .. } => iter.size_hint(),
            GroupIntoIter::ByVal { iter } => iter.size_hint(),
        }
    }
}

impl<K, V> Group<K, V> {
    /* Unsafe constructor for use in auto-generated code only. */
    pub unsafe fn new_by_ref<'a>(
        key: K,
        group: &'a [(&'a DDValue, Weight)],
        project: ProjectFunc<V>,
    ) -> Group<K, V> {
        GroupEnum::ByRef {
            key,
            group: ::std::mem::transmute::<_, &'static [(&'static DDValue, Weight)]>(group),
            project,
        }
    }

    pub fn new<'a>(key: K, group: StdVec<V>) -> Group<K, V> {
        GroupEnum::ByVal { key, group }
    }

    pub fn key_ref(&self) -> &K {
        match self {
            GroupEnum::ByRef { key, .. } => key,
            GroupEnum::ByVal { key, .. } => key,
        }
    }

    fn size(&self) -> std_usize {
        match self {
            GroupEnum::ByRef { group, .. } => group.len() as std_usize,
            GroupEnum::ByVal { group, .. } => group.len() as std_usize,
        }
    }
}

impl<K: Clone, V> Group<K, V> {
    /* Extract key by value; use `key_ref` to get a reference to key. */
    pub fn key(&self) -> K {
        match self {
            GroupEnum::ByRef { key, .. } => key.clone(),
            GroupEnum::ByVal { key, .. } => key.clone(),
        }
    }
}

impl<K, V: Clone> Group<K, V> {
    fn first(&self) -> V {
        match self {
            GroupEnum::ByRef { group, project, .. } => project(group[0].0),
            GroupEnum::ByVal { group, .. } => group[0].clone(),
        }
    }

    fn nth_unchecked(&self, n: std_usize) -> V {
        match self {
            GroupEnum::ByRef { group, project, .. } => project(group[n as usize].0),
            GroupEnum::ByVal { group, .. } => group[n as usize].clone(),
        }
    }

    pub fn iter<'a>(&'a self) -> GroupIter<'a, V> {
        GroupIter::new(self)
    }

    fn nth(&self, n: std_usize) -> Option<V> {
        match self {
            GroupEnum::ByRef { group, project, .. } => {
                if self.size() > n {
                    Option::Some {
                        x: project(group[n as usize].0),
                    }
                } else {
                    Option::None
                }
            }
            GroupEnum::ByVal { group, .. } => {
                if self.size() > n {
                    Option::Some {
                        x: group[n as usize].clone(),
                    }
                } else {
                    Option::None
                }
            }
        }
    }
}

impl<K, V: Clone> IntoIterator for Group<K, V> {
    type Item = V;
    type IntoIter = GroupIntoIter<V>;

    fn into_iter(self) -> Self::IntoIter {
        GroupIntoIter::new(self)
    }
}

/*
 * DDlog-visible functions.
 */

pub fn group_key<K: Clone, V>(g: &Group<K, V>) -> K {
    g.key()
}

/* Standard aggregation functions. */
pub fn group_count<K, V>(g: &Group<K, V>) -> std_usize {
    g.size()
}

pub fn group_first<K, V: Clone>(g: &Group<K, V>) -> V {
    g.first()
}

pub fn group_nth<K, V: Clone>(g: &Group<K, V>, n: &std_usize) -> Option<V> {
    g.nth(*n)
}

pub fn group_to_set<K, V: Ord + Clone>(g: &Group<K, V>) -> Set<V> {
    let mut res = Set::new();
    for v in g.iter() {
        set_insert(&mut res, &v);
    }
    res
}

pub fn group_set_unions<K, V: Ord + Clone>(g: &Group<K, Set<V>>) -> Set<V> {
    let mut res = Set::new();
    for gr in g.iter() {
        for v in gr.iter() {
            set_insert(&mut res, v);
        }
    }
    res
}

pub fn group_setref_unions<K, V: Ord + Clone>(g: &Group<K, Ref<Set<V>>>) -> Ref<Set<V>> {
    if g.size() == 1 {
        g.first()
    } else {
        let mut res: Ref<Set<V>> = ref_new(&Set::new());
        {
            let mut rres = Ref::get_mut(&mut res).unwrap();
            for gr in g.iter() {
                for v in gr.iter() {
                    set_insert(&mut rres, &v);
                }
            }
        }

        res
    }
}

pub fn group_to_vec<K, V: Ord + Clone>(g: &Group<K, V>) -> Vec<V> {
    let mut res = Vec::with_capacity(g.size() as usize);
    for v in g.iter() {
        vec_push(&mut res, &v);
    }
    res
}

pub fn group_to_map<K1, K2: Ord + Clone, V: Clone>(g: &Group<K1, tuple2<K2, V>>) -> Map<K2, V> {
    let mut res = Map::new();
    for tuple2(k, v) in g.iter() {
        map_insert(&mut res, &k, &v);
    }
    res
}

pub fn group_to_setmap<K1, K2: Ord + Clone, V: Clone + Ord>(
    g: &Group<K1, tuple2<K2, V>>,
) -> Map<K2, Set<V>> {
    let mut res = Map::new();
    for tuple2(k, v) in g.iter() {
        match res.x.entry(k) {
            btree_map::Entry::Vacant(ve) => {
                ve.insert(set_singleton(&v));
            }
            btree_map::Entry::Occupied(mut oe) => {
                oe.get_mut().insert(v);
            }
        }
    }

    res
}

pub fn group_min<K, V: Clone + Ord>(g: &Group<K, V>) -> V {
    g.iter().min().unwrap()
}

pub fn group_max<K, V: Clone + Ord>(g: &Group<K, V>) -> V {
    g.iter().max().unwrap()
}

pub fn group_sum<K, V: Clone + ops::Add<Output = V>>(g: &Group<K, V>) -> V {
    let mut res = group_first(g);
    for v in g.iter().skip(1) {
        res = res + v;
    }

    res
}

/* Tuples */
#[derive(Copy, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct tuple0;

impl Debug for tuple0 {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.debug_tuple("").finish()
    }
}

impl From<()> for tuple0 {
    fn from(_: ()) -> Self {
        Self
    }
}

impl Into<()> for tuple0 {
    fn into(self) {}
}

impl FromRecord for tuple0 {
    fn from_record(val: &Record) -> StdResult<Self, String> {
        <()>::from_record(val).map(|_| tuple0)
    }
}

impl IntoRecord for tuple0 {
    fn into_record(self) -> Record {
        ().into_record()
    }
}

impl Abomonation for tuple0 {}

macro_rules! declare_tuples {
    (
        $(
            $tuple_name:ident<$($element:tt),* $(,)?>
        ),*
        $(,)?
    ) => {
        $(
            #[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
            pub struct $tuple_name<$($element,)*>($(pub $element,)*);

            impl<$($element),*> From<($($element,)*)> for $tuple_name<$($element,)*> {
                fn from(($($element,)*): ($($element,)*)) -> Self {
                    Self($($element),*)
                }
            }

            impl<$($element),*> Into<($($element,)*)> for $tuple_name<$($element,)*> {
                fn into(self) -> ($($element,)*) {
                    let $tuple_name($($element),*) = self;
                    ($($element,)*)
                }
            }

            impl<$($element: Debug),*> Debug for $tuple_name<$($element),*> {
                fn fmt(&self, f: &mut Formatter) -> FmtResult {
                    let $tuple_name($($element),*) = self;
                    f.debug_tuple("")
                        $(.field(&$element))*
                        .finish()
                }
            }

            impl<$($element: Copy),*> Copy for $tuple_name<$($element),*> {}

            impl<$($element),*> Abomonation for $tuple_name<$($element),*> {}

            impl<$($element: FromRecord),*> FromRecord for $tuple_name<$($element),*> {
                fn from_record(val: &Record) -> StdResult<Self, String> {
                    <($($element,)*)>::from_record(val).map(|($($element,)*)| {
                        $tuple_name($($element),*)
                    })
                }
            }

            impl<$($element: IntoRecord),*> IntoRecord for $tuple_name<$($element),*> {
                fn into_record(self) -> Record {
                    let $tuple_name($($element),*) = self;
                    Record::Tuple(vec![$($element.into_record()),*])
                }
            }

            impl<$($element: FromRecord),*> Mutator<$tuple_name<$($element),*>> for Record {
                fn mutate(&self, x: &mut $tuple_name<$($element),*>) -> StdResult<(), String> {
                    *x = <$tuple_name<$($element),*>>::from_record(self)?;
                    Ok(())
                }
            }
        )*
    };
}

declare_tuples! {
    tuple1 <T1>,
    tuple2 <T1, T2>,
    tuple3 <T1, T2, T3>,
    tuple4 <T1, T2, T3, T4>,
    tuple5 <T1, T2, T3, T4, T5>,
    tuple6 <T1, T2, T3, T4, T5, T6>,
    tuple7 <T1, T2, T3, T4, T5, T6, T7>,
    tuple8 <T1, T2, T3, T4, T5, T6, T7, T8>,
    tuple9 <T1, T2, T3, T4, T5, T6, T7, T8, T9>,
    tuple10<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>,
    tuple11<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>,
    tuple12<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>,
    tuple13<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13>,
    tuple14<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14>,
    tuple15<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>,
    tuple16<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16>,
    tuple17<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17>,
    tuple18<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18>,
    tuple19<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19>,
    tuple20<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20>,
    tuple21<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21>,
    tuple22<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22>,
    tuple23<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23>,
    tuple24<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24>,
    tuple25<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25>,
    tuple26<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26>,
    tuple27<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27>,
    tuple28<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28>,
    tuple29<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29>,
    tuple30<T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30>,
}

// Endianness
pub fn ntohl(x: &u32) -> u32 {
    u32::from_be(*x)
}

pub fn ntohs(x: &u16) -> u16 {
    u16::from_be(*x)
}

pub fn htonl(x: &u32) -> u32 {
    u32::to_be(*x)
}

pub fn htons(x: &u16) -> u16 {
    u16::to_be(*x)
}

pub type DDEpoch = u64;
pub type DDIteration = u64;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct DDNestedTS {
    pub epoch: DDEpoch,
    pub iter: DDIteration
}
impl abomonation::Abomonation for DDNestedTS{}
::differential_datalog::decl_struct_from_record!(DDNestedTS["ddlog_std::DDNestedTS"]<>, ["ddlog_std::DDNestedTS"][2]{[0]epoch["epoch"]: DDEpoch, [1]iter["iter"]: DDIteration});
::differential_datalog::decl_struct_into_record!(DDNestedTS, ["ddlog_std::DDNestedTS"]<>, epoch, iter);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(DDNestedTS, <>, epoch: DDEpoch, iter: DDIteration);
impl ::std::fmt::Display for DDNestedTS {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            DDNestedTS{epoch,iter} => {
                __formatter.write_str("ddlog_std::DDNestedTS{")?;
                ::std::fmt::Debug::fmt(epoch, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(iter, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for DDNestedTS {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub type DDWeight = s64;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct DDlogGroup<K, V> {
    pub key: K,
    pub vals: Vec<V>
}
impl <K: ::ddlog_rt::Val, V: ::ddlog_rt::Val> abomonation::Abomonation for DDlogGroup<K, V>{}
::differential_datalog::decl_struct_from_record!(DDlogGroup["ddlog_std::DDlogGroup"]<K,V>, ["ddlog_std::DDlogGroup"][2]{[0]key["key"]: K, [1]vals["vals"]: Vec<V>});
::differential_datalog::decl_struct_into_record!(DDlogGroup, ["ddlog_std::DDlogGroup"]<K,V>, key, vals);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(DDlogGroup, <K,V>, key: K, vals: Vec<V>);
impl <K: ::std::fmt::Debug, V: ::std::fmt::Debug> ::std::fmt::Display for DDlogGroup<K, V> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            DDlogGroup{key,vals} => {
                __formatter.write_str("ddlog_std::DDlogGroup{")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(vals, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl <K: ::std::fmt::Debug, V: ::std::fmt::Debug> ::std::fmt::Debug for DDlogGroup<K, V> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Either<A, B> {
    Left {
        l: A
    },
    Right {
        r: B
    }
}
impl <A: ::ddlog_rt::Val, B: ::ddlog_rt::Val> abomonation::Abomonation for Either<A, B>{}
::differential_datalog::decl_enum_from_record!(Either["ddlog_std::Either"]<A,B>, Left["ddlog_std::Left"][1]{[0]l["l"]: A}, Right["ddlog_std::Right"][1]{[0]r["r"]: B});
::differential_datalog::decl_enum_into_record!(Either<A,B>, Left["ddlog_std::Left"]{l}, Right["ddlog_std::Right"]{r});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(Either<A,B>, Left{l: A}, Right{r: B});
impl <A: ::std::fmt::Debug, B: ::std::fmt::Debug> ::std::fmt::Display for Either<A, B> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Either::Left{l} => {
                __formatter.write_str("ddlog_std::Left{")?;
                ::std::fmt::Debug::fmt(l, __formatter)?;
                __formatter.write_str("}")
            },
            Either::Right{r} => {
                __formatter.write_str("ddlog_std::Right{")?;
                ::std::fmt::Debug::fmt(r, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl <A: ::std::fmt::Debug, B: ::std::fmt::Debug> ::std::fmt::Debug for Either<A, B> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl <A: ::std::default::Default, B: ::std::default::Default> ::std::default::Default for Either<A, B> {
    fn default() -> Self {
        Either::Left{l : ::std::default::Default::default()}
    }
}
#[serde(from="::std::option::Option<A>", into="::std::option::Option<A>", bound(serialize="A: Clone+Serialize"))]
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Option<A> {
    None,
    Some {
        x: A
    }
}
impl <A: ::ddlog_rt::Val> abomonation::Abomonation for Option<A>{}
::differential_datalog::decl_enum_into_record!(Option<A>, None["ddlog_std::None"]{}, Some["ddlog_std::Some"]{x});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(Option<A>, None{}, Some{x: A});
impl <A: ::std::fmt::Debug> ::std::fmt::Display for Option<A> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Option::None{} => {
                __formatter.write_str("ddlog_std::None{")?;
                __formatter.write_str("}")
            },
            Option::Some{x} => {
                __formatter.write_str("ddlog_std::Some{")?;
                ::std::fmt::Debug::fmt(x, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl <A: ::std::fmt::Debug> ::std::fmt::Debug for Option<A> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl <A: ::std::default::Default> ::std::default::Default for Option<A> {
    fn default() -> Self {
        Option::None{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Result<V, E> {
    Ok {
        res: V
    },
    Err {
        err: E
    }
}
impl <V: ::ddlog_rt::Val, E: ::ddlog_rt::Val> abomonation::Abomonation for Result<V, E>{}
::differential_datalog::decl_enum_from_record!(Result["ddlog_std::Result"]<V,E>, Ok["ddlog_std::Ok"][1]{[0]res["res"]: V}, Err["ddlog_std::Err"][1]{[0]err["err"]: E});
::differential_datalog::decl_enum_into_record!(Result<V,E>, Ok["ddlog_std::Ok"]{res}, Err["ddlog_std::Err"]{err});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(Result<V,E>, Ok{res: V}, Err{err: E});
impl <V: ::std::fmt::Debug, E: ::std::fmt::Debug> ::std::fmt::Display for Result<V, E> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Result::Ok{res} => {
                __formatter.write_str("ddlog_std::Ok{")?;
                ::std::fmt::Debug::fmt(res, __formatter)?;
                __formatter.write_str("}")
            },
            Result::Err{err} => {
                __formatter.write_str("ddlog_std::Err{")?;
                ::std::fmt::Debug::fmt(err, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl <V: ::std::fmt::Debug, E: ::std::fmt::Debug> ::std::fmt::Debug for Result<V, E> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl <V: ::std::default::Default, E: ::std::default::Default> ::std::default::Default for Result<V, E> {
    fn default() -> Self {
        Result::Ok{res : ::std::default::Default::default()}
    }
}
pub type s128 = i128;
pub type s16 = i16;
pub type s32 = i32;
pub type s64 = i64;
pub type s8 = i8;
/* fn __builtin_2string<X: ::ddlog_rt::Val>(x: & X) -> String */
/* fn bigint_pow32(base: & ::ddlog_bigint::Int, exp: & u32) -> ::ddlog_bigint::Int */
/* fn default<T: ::ddlog_rt::Val>() -> T */
/* fn deref<A: ::ddlog_rt::Val>(x: & Ref<A>) -> A */
/* fn group_count<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> u64 */
/* fn group_first<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> V */
/* fn group_key<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> K */
/* fn group_max<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> V */
/* fn group_min<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> V */
/* fn group_nth<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>, n: & u64) -> Option<V> */
/* fn group_set_unions<K: ::ddlog_rt::Val,A: ::ddlog_rt::Val>(g: & Group<K, Set<A>>) -> Set<A> */
/* fn group_setref_unions<K: ::ddlog_rt::Val,A: ::ddlog_rt::Val>(g: & Group<K, Ref<Set<A>>>) -> Ref<Set<A>> */
/* fn group_sum<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> V */
/* fn group_to_map<K1: ::ddlog_rt::Val,K2: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K1, tuple2<K2, V>>) -> Map<K2, V> */
/* fn group_to_set<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> Set<V> */
/* fn group_to_setmap<K1: ::ddlog_rt::Val,K2: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K1, tuple2<K2, V>>) -> Map<K2, Set<V>> */
/* fn group_to_vec<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> Vec<V> */
/* fn hash128<X: ::ddlog_rt::Val>(x: & X) -> u128 */
/* fn hash64<X: ::ddlog_rt::Val>(x: & X) -> u64 */
/* fn hex<X: ::ddlog_rt::Val>(x: & X) -> String */
/* fn htonl(x: & u32) -> u32 */
/* fn htons(x: & u16) -> u16 */
/* fn map_contains_key<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>, k: & K) -> bool */
/* fn map_empty<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>() -> Map<K, V> */
/* fn map_get<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>, k: & K) -> Option<V> */
/* fn map_insert<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: &mut Map<K, V>, k: & K, v: & V) -> () */
/* fn map_insert_imm<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>, k: & K, v: & V) -> Map<K, V> */
/* fn map_is_empty<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>) -> bool */
/* fn map_keys<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>) -> Vec<K> */
/* fn map_remove<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: &mut Map<K, V>, k: & K) -> Option<V> */
/* fn map_singleton<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(k: & K, v: & V) -> Map<K, V> */
/* fn map_size<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>) -> u64 */
/* fn map_union<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m1: & Map<K, V>, m2: & Map<K, V>) -> Map<K, V> */
/* fn ntohl(x: & u32) -> u32 */
/* fn ntohs(x: & u16) -> u16 */
/* fn option_unwrap_or_default<A: ::ddlog_rt::Val>(opt: & Option<A>) -> A */
/* fn parse_dec_i64(s: & String) -> Option<i64> */
/* fn parse_dec_u64(s: & String) -> Option<u64> */
/* fn range_vec<A: ::ddlog_rt::Val>(from: & A, to: & A, step: & A) -> Vec<A> */
/* fn ref_new<A: ::ddlog_rt::Val>(x: & A) -> Ref<A> */
/* fn result_unwrap_or_default<V: ::ddlog_rt::Val,E: ::ddlog_rt::Val>(res: & Result<V, E>) -> V */
/* fn s128_pow32(base: & s128, exp: & u32) -> s128 */
/* fn s16_pow32(base: & s16, exp: & u32) -> s16 */
/* fn s32_pow32(base: & s32, exp: & u32) -> s32 */
/* fn s64_pow32(base: & s64, exp: & u32) -> s64 */
/* fn s8_pow32(base: & s8, exp: & u32) -> s8 */
/* fn set_contains<X: ::ddlog_rt::Val>(s: & Set<X>, v: & X) -> bool */
/* fn set_difference<X: ::ddlog_rt::Val>(s1: & Set<X>, s2: & Set<X>) -> Set<X> */
/* fn set_empty<X: ::ddlog_rt::Val>() -> Set<X> */
/* fn set_insert<X: ::ddlog_rt::Val>(s: &mut Set<X>, v: & X) -> () */
/* fn set_insert_imm<X: ::ddlog_rt::Val>(s: & Set<X>, v: & X) -> Set<X> */
/* fn set_intersection<X: ::ddlog_rt::Val>(s1: & Set<X>, s2: & Set<X>) -> Set<X> */
/* fn set_is_empty<X: ::ddlog_rt::Val>(s: & Set<X>) -> bool */
/* fn set_nth<X: ::ddlog_rt::Val>(s: & Set<X>, n: & u64) -> Option<X> */
/* fn set_singleton<X: ::ddlog_rt::Val>(x: & X) -> Set<X> */
/* fn set_size<X: ::ddlog_rt::Val>(s: & Set<X>) -> u64 */
/* fn set_to_vec<A: ::ddlog_rt::Val>(s: & Set<A>) -> Vec<A> */
/* fn set_union<X: ::ddlog_rt::Val>(s1: & Set<X>, s2: & Set<X>) -> Set<X> */
/* fn set_unions<X: ::ddlog_rt::Val>(sets: & Vec<Set<X>>) -> Set<X> */
/* fn str_to_lower(s: & String) -> String */
/* fn string_contains(s1: & String, s2: & String) -> bool */
/* fn string_ends_with(s: & String, suffix: & String) -> bool */
/* fn string_join(strings: & Vec<String>, sep: & String) -> String */
/* fn string_len(s: & String) -> u64 */
/* fn string_replace(s: & String, from: & String, to: & String) -> String */
/* fn string_reverse(s: & String) -> String */
/* fn string_split(s: & String, sep: & String) -> Vec<String> */
/* fn string_starts_with(s: & String, prefix: & String) -> bool */
/* fn string_substr(s: & String, start: & u64, end: & u64) -> String */
/* fn string_to_bytes(s: & String) -> Vec<u8> */
/* fn string_to_lowercase(s: & String) -> String */
/* fn string_to_uppercase(s: & String) -> String */
/* fn string_trim(s: & String) -> String */
/* fn u128_pow32(base: & u128, exp: & u32) -> u128 */
/* fn u16_pow32(base: & u16, exp: & u32) -> u16 */
/* fn u32_pow32(base: & u32, exp: & u32) -> u32 */
/* fn u64_pow32(base: & u64, exp: & u32) -> u64 */
/* fn u8_pow32(base: & u8, exp: & u32) -> u8 */
/* fn vec_append<X: ::ddlog_rt::Val>(v: &mut Vec<X>, other: & Vec<X>) -> () */
/* fn vec_contains<X: ::ddlog_rt::Val>(v: & Vec<X>, x: & X) -> bool */
/* fn vec_empty<A: ::ddlog_rt::Val>() -> Vec<A> */
/* fn vec_is_empty<X: ::ddlog_rt::Val>(v: & Vec<X>) -> bool */
/* fn vec_len<X: ::ddlog_rt::Val>(v: & Vec<X>) -> u64 */
/* fn vec_nth<X: ::ddlog_rt::Val>(v: & Vec<X>, n: & u64) -> Option<X> */
/* fn vec_push<X: ::ddlog_rt::Val>(v: &mut Vec<X>, x: & X) -> () */
/* fn vec_push_imm<X: ::ddlog_rt::Val>(v: & Vec<X>, x: & X) -> Vec<X> */
/* fn vec_resize<X: ::ddlog_rt::Val>(v: &mut Vec<X>, new_len: & u64, value: & X) -> () */
/* fn vec_singleton<X: ::ddlog_rt::Val>(x: & X) -> Vec<X> */
/* fn vec_sort<X: ::ddlog_rt::Val>(v: &mut Vec<X>) -> () */
/* fn vec_sort_imm<X: ::ddlog_rt::Val>(v: & Vec<X>) -> Vec<X> */
/* fn vec_swap_nth<X: ::ddlog_rt::Val>(v: &mut Vec<X>, idx: & u64, value: &mut X) -> bool */
/* fn vec_to_set<A: ::ddlog_rt::Val>(s: & Vec<A>) -> Set<A> */
/* fn vec_truncate<X: ::ddlog_rt::Val>(v: &mut Vec<X>, len: & u64) -> () */
/* fn vec_update_nth<X: ::ddlog_rt::Val>(v: &mut Vec<X>, idx: & u64, value: & X) -> bool */
/* fn vec_with_capacity<A: ::ddlog_rt::Val>(len: & u64) -> Vec<A> */
/* fn vec_with_length<A: ::ddlog_rt::Val>(len: & u64, x: & A) -> Vec<A> */
/* fn vec_zip<X: ::ddlog_rt::Val,Y: ::ddlog_rt::Val>(v1: & Vec<X>, v2: & Vec<Y>) -> Vec<tuple2<X, Y>> */
pub fn and_then<T: ::ddlog_rt::Val,U: ::ddlog_rt::Val>(o: & Option<T>, f: & Box<dyn ddlog_rt::Closure<*const T, Option<U>>>) -> Option<U>
{   match (*o) {
        Option::None{} => (Option::None{}),
        Option::Some{x: ref x} => f.call(x)
    }
}
pub fn append<X: ::ddlog_rt::Val>(v: &mut Vec<X>, other: & Vec<X>) -> ()
{   vec_append(v, other)
}
pub fn contains___Stringval___Stringval___Boolval(s1: & String, s2: & String) -> bool
{   string_contains(s1, s2)
}
pub fn contains_ddlog_std_Vec__X_X___Boolval<X: ::ddlog_rt::Val>(v: & Vec<X>, x: & X) -> bool
{   vec_contains(v, x)
}
pub fn contains_ddlog_std_Set__X_X___Boolval<X: ::ddlog_rt::Val>(s: & Set<X>, v: & X) -> bool
{   set_contains(s, v)
}
pub fn contains_key<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>, k: & K) -> bool
{   map_contains_key(m, k)
}
pub fn count<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> u64
{   group_count(g)
}
pub fn difference<X: ::ddlog_rt::Val>(s1: & Set<X>, s2: & Set<X>) -> Set<X>
{   set_difference(s1, s2)
}
pub fn ends_with(s: & String, suffix: & String) -> bool
{   string_ends_with(s, suffix)
}
pub fn first<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> V
{   group_first(g)
}
pub fn get<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>, k: & K) -> Option<V>
{   map_get(m, k)
}
pub fn group_unzip<K: ::ddlog_rt::Val,X: ::ddlog_rt::Val,Y: ::ddlog_rt::Val>(g: & Group<K, tuple2<X, Y>>) -> tuple2<Vec<X>, Vec<Y>>
{   let ref mut xs: Vec<X> = vec_with_capacity((&size_ddlog_std_Group__K_V___Bitval64::<K, tuple2<X, Y>>(g)));
    let ref mut ys: Vec<Y> = vec_with_capacity((&size_ddlog_std_Group__K_V___Bitval64::<K, tuple2<X, Y>>(g)));
    for ref v in g.iter() {
        {
            let tuple2(ref mut x, ref mut y): tuple2<X, Y> = (*v).clone();
            vec_push(xs, x);
            vec_push(ys, y)
        }
    };
    tuple2((*xs).clone(), (*ys).clone())
}
pub fn insert_ddlog_std_Map__K_V_K_V___Tuple0__<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: &mut Map<K, V>, k: & K, v: & V) -> ()
{   map_insert(m, k, v)
}
pub fn insert_ddlog_std_Set__X_X___Tuple0__<X: ::ddlog_rt::Val>(s: &mut Set<X>, v: & X) -> ()
{   set_insert(s, v)
}
pub fn insert_imm_ddlog_std_Map__K_V_K_V_ddlog_std_Map__K_V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>, k: & K, v: & V) -> Map<K, V>
{   map_insert_imm(m, k, v)
}
pub fn insert_imm_ddlog_std_Set__X_X_ddlog_std_Set__X<X: ::ddlog_rt::Val>(s: & Set<X>, v: & X) -> Set<X>
{   set_insert_imm(s, v)
}
pub fn intersection<X: ::ddlog_rt::Val>(s1: & Set<X>, s2: & Set<X>) -> Set<X>
{   set_intersection(s1, s2)
}
pub fn is_empty_ddlog_std_Vec__X___Boolval<X: ::ddlog_rt::Val>(v: & Vec<X>) -> bool
{   vec_is_empty(v)
}
pub fn is_empty_ddlog_std_Map__K_V___Boolval<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>) -> bool
{   map_is_empty(m)
}
pub fn is_empty_ddlog_std_Set__X___Boolval<X: ::ddlog_rt::Val>(s: & Set<X>) -> bool
{   set_is_empty(s)
}
pub fn is_err<V: ::ddlog_rt::Val,E: ::ddlog_rt::Val>(res: & Result<V, E>) -> bool
{   match (*res) {
        Result::Ok{res: _} => false,
        Result::Err{err: _} => true
    }
}
pub fn is_none<A: ::ddlog_rt::Val>(x: & Option<A>) -> bool
{   match (*x) {
        Option::None{} => true,
        _ => false
    }
}
pub fn is_ok<V: ::ddlog_rt::Val,E: ::ddlog_rt::Val>(res: & Result<V, E>) -> bool
{   match (*res) {
        Result::Ok{res: _} => true,
        Result::Err{err: _} => false
    }
}
pub fn is_some<A: ::ddlog_rt::Val>(x: & Option<A>) -> bool
{   match (*x) {
        Option::Some{x: _} => true,
        _ => false
    }
}
pub fn join(strings: & Vec<String>, sep: & String) -> String
{   string_join(strings, sep)
}
pub fn key<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> K
{   group_key(g)
}
pub fn keys<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>) -> Vec<K>
{   map_keys(m)
}
pub fn len___Stringval___Bitval64(s: & String) -> u64
{   string_len(s)
}
pub fn len_ddlog_std_Vec__X___Bitval64<X: ::ddlog_rt::Val>(v: & Vec<X>) -> u64
{   vec_len(v)
}
pub fn map_ddlog_std_Option__A___Closureimm_A_ret_B_ddlog_std_Option__B<A: ::ddlog_rt::Val,B: ::ddlog_rt::Val>(opt: & Option<A>, f: & Box<dyn ddlog_rt::Closure<*const A, B>>) -> Option<B>
{   match (*opt) {
        Option::None{} => (Option::None{}),
        Option::Some{x: ref x} => (Option::Some{x: f.call(x)})
    }
}
pub fn map_ddlog_std_Result__V1_E___Closureimm_V1_ret_V2_ddlog_std_Result__V2_E<V1: ::ddlog_rt::Val,E: ::ddlog_rt::Val,V2: ::ddlog_rt::Val>(res: & Result<V1, E>, f: & Box<dyn ddlog_rt::Closure<*const V1, V2>>) -> Result<V2, E>
{   match (*res) {
        Result::Err{err: ref e} => (Result::Err{err: (*e).clone()}),
        Result::Ok{res: ref x} => (Result::Ok{res: f.call(x)})
    }
}
pub fn map_err<V: ::ddlog_rt::Val,E1: ::ddlog_rt::Val,E2: ::ddlog_rt::Val>(res: & Result<V, E1>, f: & Box<dyn ddlog_rt::Closure<*const E1, E2>>) -> Result<V, E2>
{   match (*res) {
        Result::Err{err: ref e} => (Result::Err{err: f.call(e)}),
        Result::Ok{res: ref x} => (Result::Ok{res: (*x).clone()})
    }
}
pub fn max_A_A_A<A: ::ddlog_rt::Val>(x: & A, y: & A) -> A
{   if ((&*x) > (&*y)) {
        (*x).clone()
    } else {
        (*y).clone()
    }
}
pub fn max_ddlog_std_Group__K_V_V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> V
{   group_max(g)
}
pub fn min_A_A_A<A: ::ddlog_rt::Val>(x: & A, y: & A) -> A
{   if ((&*x) < (&*y)) {
        (*x).clone()
    } else {
        (*y).clone()
    }
}
pub fn min_ddlog_std_Group__K_V_V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> V
{   group_min(g)
}
pub fn nth_ddlog_std_Group__K_V___Bitval64_ddlog_std_Option__V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>, n: & u64) -> Option<V>
{   group_nth(g, n)
}
pub fn nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X<X: ::ddlog_rt::Val>(v: & Vec<X>, n: & u64) -> Option<X>
{   vec_nth(v, n)
}
pub fn nth_ddlog_std_Set__X___Bitval64_ddlog_std_Option__X<X: ::ddlog_rt::Val>(s: & Set<X>, n: & u64) -> Option<X>
{   set_nth(s, n)
}
pub fn ok_or<T: ::ddlog_rt::Val,E: ::ddlog_rt::Val>(o: & Option<T>, e: & E) -> Result<T, E>
{   match (*o) {
        Option::Some{x: ref x} => (Result::Ok{res: (*x).clone()}),
        Option::None{} => (Result::Err{err: (*e).clone()})
    }
}
pub fn ok_or_else<T: ::ddlog_rt::Val,E: ::ddlog_rt::Val>(o: & Option<T>, e: & Box<dyn ddlog_rt::Closure<(), E>>) -> Result<T, E>
{   match (*o) {
        Option::Some{x: ref x} => (Result::Ok{res: (*x).clone()}),
        Option::None{} => (Result::Err{err: e.call(())})
    }
}
pub fn pow32___Bitval8___Bitval32___Bitval8(base: & u8, exp: & u32) -> u8
{   u8_pow32(base, exp)
}
pub fn pow32___Bitval16___Bitval32___Bitval16(base: & u16, exp: & u32) -> u16
{   u16_pow32(base, exp)
}
pub fn pow32___Bitval32___Bitval32___Bitval32(base: & u32, exp: & u32) -> u32
{   u32_pow32(base, exp)
}
pub fn pow32___Bitval64___Bitval32___Bitval64(base: & u64, exp: & u32) -> u64
{   u64_pow32(base, exp)
}
pub fn pow32___Bitval128___Bitval32___Bitval128(base: & u128, exp: & u32) -> u128
{   u128_pow32(base, exp)
}
pub fn pow32___Signedval8___Bitval32___Signedval8(base: & s8, exp: & u32) -> s8
{   s8_pow32(base, exp)
}
pub fn pow32___Signedval16___Bitval32___Signedval16(base: & s16, exp: & u32) -> s16
{   s16_pow32(base, exp)
}
pub fn pow32___Signedval32___Bitval32___Signedval32(base: & s32, exp: & u32) -> s32
{   s32_pow32(base, exp)
}
pub fn pow32___Signedval64___Bitval32___Signedval64(base: & s64, exp: & u32) -> s64
{   s64_pow32(base, exp)
}
pub fn pow32___Signedval128___Bitval32___Signedval128(base: & s128, exp: & u32) -> s128
{   s128_pow32(base, exp)
}
pub fn pow32___Intval___Bitval32___Intval(base: & ::ddlog_bigint::Int, exp: & u32) -> ::ddlog_bigint::Int
{   bigint_pow32(base, exp)
}
pub fn push<X: ::ddlog_rt::Val>(v: &mut Vec<X>, x: & X) -> ()
{   vec_push(v, x)
}
pub fn push_imm<X: ::ddlog_rt::Val>(v: & Vec<X>, x: & X) -> Vec<X>
{   vec_push_imm(v, x)
}
pub fn remove<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: &mut Map<K, V>, k: & K) -> Option<V>
{   map_remove(m, k)
}
pub fn replace(s: & String, from: & String, to: & String) -> String
{   string_replace(s, from, to)
}
pub fn resize<X: ::ddlog_rt::Val>(v: &mut Vec<X>, new_len: & u64, value: & X) -> ()
{   vec_resize(v, new_len, value)
}
pub fn reverse(s: & String) -> String
{   string_reverse(s)
}
pub fn setref_unions<K: ::ddlog_rt::Val,A: ::ddlog_rt::Val>(g: & Group<K, Ref<Set<A>>>) -> Ref<Set<A>>
{   group_setref_unions(g)
}
pub fn size_ddlog_std_Group__K_V___Bitval64<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> u64
{   group_count(g)
}
pub fn size_ddlog_std_Map__K_V___Bitval64<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m: & Map<K, V>) -> u64
{   map_size(m)
}
pub fn size_ddlog_std_Set__X___Bitval64<X: ::ddlog_rt::Val>(s: & Set<X>) -> u64
{   set_size(s)
}
pub fn sort<X: ::ddlog_rt::Val>(v: &mut Vec<X>) -> ()
{   vec_sort(v)
}
pub fn sort_imm<X: ::ddlog_rt::Val>(v: & Vec<X>) -> Vec<X>
{   vec_sort_imm(v)
}
pub fn split(s: & String, sep: & String) -> Vec<String>
{   string_split(s, sep)
}
pub fn starts_with(s: & String, prefix: & String) -> bool
{   string_starts_with(s, prefix)
}
pub fn substr(s: & String, start: & u64, end: & u64) -> String
{   string_substr(s, start, end)
}
pub fn swap_nth<X: ::ddlog_rt::Val>(v: &mut Vec<X>, idx: & u64, value: &mut X) -> bool
{   vec_swap_nth(v, idx, value)
}
pub fn to_bytes(s: & String) -> Vec<u8>
{   string_to_bytes(s)
}
pub fn to_lowercase(s: & String) -> String
{   string_to_lowercase(s)
}
pub fn to_map_ddlog_std_Group__K1___Tuple2__K2_V_ddlog_std_Map__K2_V<K1: ::ddlog_rt::Val,K2: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K1, tuple2<K2, V>>) -> Map<K2, V>
{   group_to_map(g)
}
pub fn to_map_ddlog_std_Vec____Tuple2__K_V_ddlog_std_Map__K_V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(v: & Vec<tuple2<K, V>>) -> Map<K, V>
{   let ref mut res: Map<K, V> = map_empty();
    for kv in v.iter() {
        {
            insert_ddlog_std_Map__K_V_K_V___Tuple0__::<K, V>(res, (&(kv.0)), (&(kv.1)));
            ()
        }
    };
    (*res).clone()
}
pub fn to_set_ddlog_std_Option__X_ddlog_std_Set__X<X: ::ddlog_rt::Val>(o: & Option<X>) -> Set<X>
{   match (*o) {
        Option::Some{x: ref x} => set_singleton(x),
        Option::None{} => set_empty()
    }
}
pub fn to_set_ddlog_std_Group__K_V_ddlog_std_Set__V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> Set<V>
{   group_to_set(g)
}
pub fn to_set_ddlog_std_Vec__A_ddlog_std_Set__A<A: ::ddlog_rt::Val>(s: & Vec<A>) -> Set<A>
{   vec_to_set(s)
}
pub fn to_setmap<K1: ::ddlog_rt::Val,K2: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K1, tuple2<K2, V>>) -> Map<K2, Set<V>>
{   group_to_setmap(g)
}
pub fn to_string_ddlog_std_DDNestedTS___Stringval(ts: & DDNestedTS) -> String
{   ::ddlog_rt::string_append_str(::ddlog_rt::string_append(::ddlog_rt::string_append_str(::ddlog_rt::string_append(String::from(r###"("###), (&__builtin_2string((&ts.epoch)))), r###","###), (&__builtin_2string((&ts.iter)))), r###")"###)
}
pub fn to_string___Boolval___Stringval(x: & bool) -> String
{   __builtin_2string(x)
}
pub fn to_string___Intval___Stringval(x: & ::ddlog_bigint::Int) -> String
{   __builtin_2string(x)
}
pub fn to_string___Floatval___Stringval(x: & ::ordered_float::OrderedFloat<f32>) -> String
{   __builtin_2string(x)
}
pub fn to_string___Doubleval___Stringval(x: & ::ordered_float::OrderedFloat<f64>) -> String
{   __builtin_2string(x)
}
pub fn to_string___Signedval8___Stringval(x: & s8) -> String
{   __builtin_2string(x)
}
pub fn to_string___Signedval16___Stringval(x: & s16) -> String
{   __builtin_2string(x)
}
pub fn to_string___Signedval32___Stringval(x: & s32) -> String
{   __builtin_2string(x)
}
pub fn to_string___Signedval64___Stringval(x: & s64) -> String
{   __builtin_2string(x)
}
pub fn to_string___Signedval128___Stringval(x: & s128) -> String
{   __builtin_2string(x)
}
pub fn to_string___Bitval8___Stringval(x: & u8) -> String
{   __builtin_2string(x)
}
pub fn to_string___Bitval16___Stringval(x: & u16) -> String
{   __builtin_2string(x)
}
pub fn to_string___Bitval32___Stringval(x: & u32) -> String
{   __builtin_2string(x)
}
pub fn to_string___Bitval64___Stringval(x: & u64) -> String
{   __builtin_2string(x)
}
pub fn to_string___Bitval128___Stringval(x: & u128) -> String
{   __builtin_2string(x)
}
pub fn to_string___Stringval___Stringval(x: & String) -> String
{   __builtin_2string(x)
}
pub fn to_uppercase(s: & String) -> String
{   string_to_uppercase(s)
}
pub fn to_vec_ddlog_std_Option__X_ddlog_std_Vec__X<X: ::ddlog_rt::Val>(o: & Option<X>) -> Vec<X>
{   match (*o) {
        Option::Some{x: ref x} => vec_singleton(x),
        Option::None{} => vec_empty()
    }
}
pub fn to_vec_ddlog_std_Group__K_V_ddlog_std_Vec__V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(g: & Group<K, V>) -> Vec<V>
{   group_to_vec(g)
}
pub fn to_vec_ddlog_std_Set__A_ddlog_std_Vec__A<A: ::ddlog_rt::Val>(s: & Set<A>) -> Vec<A>
{   set_to_vec(s)
}
pub fn trim(s: & String) -> String
{   string_trim(s)
}
pub fn truncate<X: ::ddlog_rt::Val>(v: &mut Vec<X>, len: & u64) -> ()
{   vec_truncate(v, len)
}
pub fn union_ddlog_std_Group__K_ddlog_std_Set__A_ddlog_std_Set__A<K: ::ddlog_rt::Val,A: ::ddlog_rt::Val>(g: & Group<K, Set<A>>) -> Set<A>
{   group_set_unions(g)
}
pub fn union_ddlog_std_Group__K_ddlog_std_Ref__ddlog_std_Set__A_ddlog_std_Ref__ddlog_std_Set__A<K: ::ddlog_rt::Val,A: ::ddlog_rt::Val>(g: & Group<K, Ref<Set<A>>>) -> Ref<Set<A>>
{   group_setref_unions(g)
}
pub fn union_ddlog_std_Map__K_V_ddlog_std_Map__K_V_ddlog_std_Map__K_V<K: ::ddlog_rt::Val,V: ::ddlog_rt::Val>(m1: & Map<K, V>, m2: & Map<K, V>) -> Map<K, V>
{   map_union(m1, m2)
}
pub fn union_ddlog_std_Set__X_ddlog_std_Set__X_ddlog_std_Set__X<X: ::ddlog_rt::Val>(s1: & Set<X>, s2: & Set<X>) -> Set<X>
{   set_union(s1, s2)
}
pub fn union_ddlog_std_Vec__ddlog_std_Set__X_ddlog_std_Set__X<X: ::ddlog_rt::Val>(sets: & Vec<Set<X>>) -> Set<X>
{   set_unions(sets)
}
pub fn unions<X: ::ddlog_rt::Val>(sets: & Vec<Set<X>>) -> Set<X>
{   set_unions(sets)
}
pub fn unwrap_or_ddlog_std_Option__A_A_A<A: ::ddlog_rt::Val>(x: & Option<A>, def: & A) -> A
{   match (*x) {
        Option::Some{x: ref v} => (*v).clone(),
        Option::None{} => (*def).clone()
    }
}
pub fn unwrap_or_ddlog_std_Result__V_E_V_V<V: ::ddlog_rt::Val,E: ::ddlog_rt::Val>(res: & Result<V, E>, def: & V) -> V
{   match (*res) {
        Result::Ok{res: ref v} => (*v).clone(),
        Result::Err{err: _} => (*def).clone()
    }
}
pub fn unwrap_or_default_ddlog_std_Option__A_A<A: ::ddlog_rt::Val>(opt: & Option<A>) -> A
{   option_unwrap_or_default(opt)
}
pub fn unwrap_or_default_ddlog_std_Result__V_E_V<V: ::ddlog_rt::Val,E: ::ddlog_rt::Val>(res: & Result<V, E>) -> V
{   result_unwrap_or_default(res)
}
pub fn unzip<X: ::ddlog_rt::Val,Y: ::ddlog_rt::Val>(v: & Vec<tuple2<X, Y>>) -> tuple2<Vec<X>, Vec<Y>>
{   let ref mut v1: Vec<X> = vec_with_capacity((&len_ddlog_std_Vec__X___Bitval64::<tuple2<X, Y>>(v)));
    let ref mut v2: Vec<Y> = vec_with_capacity((&len_ddlog_std_Vec__X___Bitval64::<tuple2<X, Y>>(v)));
    for val in v.iter() {
        {
            push::<X>(v1, (&(val.0)));
            push::<Y>(v2, (&(val.1)));
            ()
        }
    };
    tuple2((*v1).clone(), (*v2).clone())
}
pub fn update_nth<X: ::ddlog_rt::Val>(v: &mut Vec<X>, idx: & u64, value: & X) -> bool
{   vec_update_nth(v, idx, value)
}
pub fn zip<X: ::ddlog_rt::Val,Y: ::ddlog_rt::Val>(v1: & Vec<X>, v2: & Vec<Y>) -> Vec<tuple2<X, Y>>
{   vec_zip(v1, v2)
}