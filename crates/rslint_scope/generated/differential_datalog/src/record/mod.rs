//! An untyped representation of DDlog values and database update commands.

mod arrays;
pub mod c_api;
mod tuples;

use num::{BigInt, BigUint, ToPrimitive};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::{btree_map, BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write;
use std::iter::FromIterator;
use std::vec;

pub type Name = Cow<'static, str>;

/* Rust's implementation of `Debug::fmt` for `str` incorrectly escapes
 * single quotes, e.g., "isn't" becomes "isn\'t".  To get around this,
 * I copied Rust's implementation and modified it to handle single quotes
 * as a special case. */
pub fn format_ddlog_str(s: &str, f: &mut fmt::Formatter) -> fmt::Result {
    //write!(f, "{:?}", s),
    f.write_char('"')?;
    let mut from = 0;
    for (i, c) in s.char_indices() {
        let esc = c.escape_debug();
        if esc.len() != 1 && c != '\'' {
            f.write_str(&s[from..i])?;
            for c in esc {
                f.write_char(c)?;
            }
            from = i + c.len_utf8();
        }
    }
    f.write_str(&s[from..])?;
    f.write_char('"')
}

/// `enum Record` represents an arbitrary DDlog value.
///
/// It relies on strings to store constructor and field names.  When manufacturing an instance of
/// `Record` from a typed DDlog value, strings can be cheap `&'static str`'s.  When manufacturing an
/// instance from some external representation, e.g., JSON, one needs to use `String`'s instead.  To
/// accommodate both options, `Record` uses `Cow` to store names.
///
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Record {
    Bool(bool),
    Int(BigInt),
    Float(OrderedFloat<f32>),
    Double(OrderedFloat<f64>),
    String(String),
    /// Value serialized in a string.  The first field stores the name of the
    /// serialization format, e.g., "json".
    Serialized(Name, String),
    Tuple(Vec<Record>),
    Array(CollectionKind, Vec<Record>),
    PosStruct(Name, Vec<Record>),
    NamedStruct(Name, Vec<(Name, Record)>),
}

impl Record {
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn is_double(&self) -> bool {
        matches!(self, Self::Double(_))
    }

    pub fn is_struct(&self) -> bool {
        matches!(self, Self::PosStruct(_, _) | Self::NamedStruct(_, _))
    }

    pub fn is_named_struct(&self) -> bool {
        matches!(self, Self::NamedStruct(_, _))
    }

    pub fn as_int(&self) -> Option<&BigInt> {
        match self {
            Self::Int(int) => Some(int),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<OrderedFloat<f32>> {
        match *self {
            Self::Float(float) => Some(float),
            _ => None,
        }
    }

    pub fn as_double(&self) -> Option<OrderedFloat<f64>> {
        match *self {
            Self::Double(double) => Some(double),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(boolean) => Some(boolean),
            _ => None,
        }
    }

    pub fn as_tuple(&self) -> Option<&[Self]> {
        match self {
            Self::Tuple(elements) => Some(elements),
            _ => None,
        }
    }

    pub fn as_vector(&self) -> Option<&[Self]> {
        match self {
            Self::Array(CollectionKind::Vector, elements) => Some(elements),
            _ => None,
        }
    }

    pub fn as_set(&self) -> Option<&[Self]> {
        match self {
            Self::Array(CollectionKind::Set, elements) => Some(elements),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&[Self]> {
        match self {
            Self::Array(CollectionKind::Map, elements) => Some(elements),
            _ => None,
        }
    }

    pub fn nth_struct_field(&self, idx: usize) -> Option<&Self> {
        match self {
            Self::PosStruct(_, fields) => fields.get(idx),
            Self::NamedStruct(_, fields) => fields.get(idx).map(|(_name, value)| value),
            _ => None,
        }
    }

    pub fn named_struct_fields(&self) -> Option<&[(Name, Self)]> {
        match self {
            Self::NamedStruct(_, fields) => Some(fields),
            _ => None,
        }
    }

    pub fn struct_constructor(&self) -> Option<&Name> {
        match self {
            Self::PosStruct(constructor, _) | Self::NamedStruct(constructor, _) => {
                Some(constructor)
            }
            _ => None,
        }
    }
}

impl fmt::Display for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Record::Bool(true) => write!(f, "true"),
            Record::Bool(false) => write!(f, "false"),
            Record::Int(i) => i.fmt(f),
            Record::Float(d) => d.fmt(f),
            Record::Double(d) => d.fmt(f),
            Record::String(s) => format_ddlog_str(s.as_ref(), f),
            Record::Serialized(n, s) => {
                write!(f, "#{}", n)?;
                format_ddlog_str(s.as_ref(), f)
            }
            Record::Tuple(recs) => {
                write!(f, "(")?;
                let len = recs.len();
                for (i, r) in recs.iter().enumerate() {
                    if i == len - 1 {
                        write!(f, "{}", r)?;
                    } else {
                        write!(f, "{}, ", r)?;
                    }
                }
                write!(f, ")")
            }
            Record::Array(_, recs) => {
                write!(f, "[")?;
                let len = recs.len();
                for (i, r) in recs.iter().enumerate() {
                    if i == len - 1 {
                        write!(f, "{}", r)?;
                    } else {
                        write!(f, "{}, ", r)?;
                    }
                }
                write!(f, "]")
            }
            Record::PosStruct(n, recs) => {
                write!(f, "{}{{", n)?;
                let len = recs.len();
                for (i, r) in recs.iter().enumerate() {
                    if i == len - 1 {
                        write!(f, "{}", r)?;
                    } else {
                        write!(f, "{}, ", r)?;
                    }
                }
                write!(f, "}}")
            }
            Record::NamedStruct(n, recs) => {
                write!(f, "{}{{", n)?;
                let len = recs.len();
                for (i, (fname, v)) in recs.iter().enumerate() {
                    if i == len - 1 {
                        write!(f, ".{} = {}", fname, v)?;
                    } else {
                        write!(f, ".{} = {}, ", fname, v)?;
                    }
                }
                write!(f, "}}")
            }
        }
    }
}

#[derive(Copy, Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum CollectionKind {
    Unknown,
    Vector,
    Set,
    Map,
}

/// Relation can be identified by name (e.g., when parsing JSON or text)
/// or ID, which is more efficient if the caller bothered to convert
/// relation name to ID.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum RelIdentifier {
    RelName(Name),
    RelId(usize),
}

impl fmt::Display for RelIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RelIdentifier::RelName(rname) => write!(f, "{}", rname),
            RelIdentifier::RelId(rid) => write!(f, "{}", rid),
        }
    }
}

/// Four types of DDlog relation update commands that match the `Update` enum in `program.rs`
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UpdCmd {
    Insert(RelIdentifier, Record),
    InsertOrUpdate(RelIdentifier, Record),
    Delete(RelIdentifier, Record),
    DeleteKey(RelIdentifier, Record),
    Modify(RelIdentifier, Record, Record),
}

/*
 * Traits for converting ddlog `Values` to/from Record's.
 */

/// `Mutator` trait represents an object that can be used to mutate a value (e.g., change some of
/// its fields).
pub trait Mutator<V>: fmt::Display {
    /// Consumes a value and returns an updated value.
    fn mutate(&self, v: &mut V) -> Result<(), String>;
}

/// `FromRecord` trait.  For types that can be converted from cmd_parser::Record type
pub trait FromRecord: Sized {
    fn from_record(val: &Record) -> Result<Self, String>;
}

pub trait IntoRecord {
    fn into_record(self) -> Record;
}

/*
 * Trait implementations for built-in types.
 */

impl FromRecord for u8 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_u8() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to u8", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl Mutator<u8> for Record {
    fn mutate(&self, v: &mut u8) -> Result<(), String> {
        *v = u8::from_record(self)?;
        Ok(())
    }
}

impl IntoRecord for u8 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl FromRecord for u16 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_u16() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to u16", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for u16 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<u16> for Record {
    fn mutate(&self, v: &mut u16) -> Result<(), String> {
        *v = u16::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for u32 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_u32() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to u32", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for u32 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<u32> for Record {
    fn mutate(&self, v: &mut u32) -> Result<(), String> {
        *v = u32::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for u64 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_u64() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to u64", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for u64 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<u64> for Record {
    fn mutate(&self, v: &mut u64) -> Result<(), String> {
        *v = u64::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for OrderedFloat<f32> {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Float(i) => Ok(*i),
            // Floating point values parsed from a command file are always stored as doubles.
            Record::Double(i) => Ok(OrderedFloat::<f32>::from(**i as f32)),
            Record::Int(i) => i
                .to_f32()
                .map(OrderedFloat)
                .ok_or_else(|| format!("Cannot convert {} to float", *i)),
            v => Err(format!("not a float {:?}", *v)),
        }
    }
}

impl IntoRecord for OrderedFloat<f32> {
    fn into_record(self) -> Record {
        Record::Float(self)
    }
}

impl Mutator<OrderedFloat<f32>> for Record {
    fn mutate(&self, v: &mut OrderedFloat<f32>) -> Result<(), String> {
        *v = OrderedFloat::<f32>::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for OrderedFloat<f64> {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Double(i) => Ok(*i),
            Record::Int(i) => i
                .to_f64()
                .map(OrderedFloat)
                .ok_or_else(|| format!("Cannot convert {} to double", *i)),
            v => Err(format!("not a double {:?}", *v)),
        }
    }
}

impl IntoRecord for OrderedFloat<f64> {
    fn into_record(self) -> Record {
        Record::Double(self)
    }
}

impl Mutator<OrderedFloat<f64>> for Record {
    fn mutate(&self, v: &mut OrderedFloat<f64>) -> Result<(), String> {
        *v = OrderedFloat::<f64>::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for u128 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_u128() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to u128", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for u128 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<u128> for Record {
    fn mutate(&self, v: &mut u128) -> Result<(), String> {
        *v = u128::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for i8 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_i8() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to i8", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl Mutator<i8> for Record {
    fn mutate(&self, v: &mut i8) -> Result<(), String> {
        *v = i8::from_record(self)?;
        Ok(())
    }
}

impl IntoRecord for i8 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl FromRecord for i16 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_i16() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to i16", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for i16 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<i16> for Record {
    fn mutate(&self, v: &mut i16) -> Result<(), String> {
        *v = i16::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for i32 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_i32() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to i32", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for i32 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<i32> for Record {
    fn mutate(&self, v: &mut i32) -> Result<(), String> {
        *v = i32::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for i64 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_i64() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to i64", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for i64 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<i64> for Record {
    fn mutate(&self, v: &mut i64) -> Result<(), String> {
        *v = i64::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for i128 {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_i128() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to i128", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for i128 {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<i128> for Record {
    fn mutate(&self, v: &mut i128) -> Result<(), String> {
        *v = i128::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for BigInt {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => Ok(i.clone()),
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for BigInt {
    fn into_record(self) -> Record {
        Record::Int(self)
    }
}

impl Mutator<BigInt> for Record {
    fn mutate(&self, v: &mut BigInt) -> Result<(), String> {
        *v = BigInt::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for BigUint {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Int(i) => match i.to_biguint() {
                Some(x) => Ok(x),
                None => Err(format!("cannot convert {} to BigUint", i)),
            },
            v => Err(format!("not an int {:?}", *v)),
        }
    }
}

impl IntoRecord for BigUint {
    fn into_record(self) -> Record {
        Record::Int(BigInt::from(self))
    }
}

impl Mutator<BigUint> for Record {
    fn mutate(&self, v: &mut BigUint) -> Result<(), String> {
        *v = BigUint::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for bool {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Bool(b) => Ok(*b),
            v => Err(format!("not a bool {:?}", *v)),
        }
    }
}

impl IntoRecord for bool {
    fn into_record(self) -> Record {
        Record::Bool(self)
    }
}

impl Mutator<bool> for Record {
    fn mutate(&self, v: &mut bool) -> Result<(), String> {
        *v = bool::from_record(self)?;
        Ok(())
    }
}

impl FromRecord for String {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::String(s) => Ok(s.clone()),
            v => Err(format!("not a string {:?}", *v)),
        }
    }
}

impl IntoRecord for String {
    fn into_record(self) -> Record {
        Record::String(self)
    }
}

impl Mutator<String> for Record {
    fn mutate(&self, v: &mut String) -> Result<(), String> {
        *v = String::from_record(self)?;
        Ok(())
    }
}

impl<T: FromRecord> FromRecord for vec::Vec<T> {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::Array(_, args) => Result::from_iter(args.iter().map(|x| T::from_record(x))),
            v => {
                T::from_record(v).map(|x| vec![x])
                //Err(format!("not an array {:?}", *v))
            }
        }
    }
}

impl<T: IntoRecord> IntoRecord for vec::Vec<T> {
    fn into_record(self) -> Record {
        Record::Array(
            CollectionKind::Vector,
            self.into_iter().map(IntoRecord::into_record).collect(),
        )
    }
}

impl<T: FromRecord> Mutator<vec::Vec<T>> for Record {
    fn mutate(&self, v: &mut vec::Vec<T>) -> Result<(), String> {
        *v = <vec::Vec<T>>::from_record(self)?;
        Ok(())
    }
}

impl<K: FromRecord + Ord, V: FromRecord> FromRecord for BTreeMap<K, V> {
    fn from_record(val: &Record) -> Result<Self, String> {
        vec::Vec::from_record(val).map(BTreeMap::from_iter)
    }
}

impl<K: IntoRecord + Ord, V: IntoRecord> IntoRecord for BTreeMap<K, V> {
    fn into_record(self) -> Record {
        Record::Array(
            CollectionKind::Map,
            self.into_iter().map(IntoRecord::into_record).collect(),
        )
    }
}

/// Map update semantics is that the update contains keys that are in one of the maps but not the
/// other, plus keys that are in both maps but with different values.
impl<K: FromRecord + Ord, V: FromRecord + PartialEq> Mutator<BTreeMap<K, V>> for Record {
    fn mutate(&self, map: &mut BTreeMap<K, V>) -> Result<(), String> {
        let upd = <BTreeMap<K, V>>::from_record(self)?;
        for (k, v) in upd.into_iter() {
            match map.entry(k) {
                btree_map::Entry::Vacant(ve) => {
                    /* key not in map -- insert */
                    ve.insert(v);
                }
                btree_map::Entry::Occupied(mut oe) => {
                    if *oe.get() == v {
                        /* key in map with the same value -- delete */
                        oe.remove_entry();
                    } else {
                        /* key in map, different value -- set new value */
                        oe.insert(v);
                    }
                }
            }
        }
        Ok(())
    }
}

impl<T: FromRecord + Ord> FromRecord for BTreeSet<T> {
    fn from_record(val: &Record) -> Result<Self, String> {
        vec::Vec::from_record(val).map(BTreeSet::from_iter)
    }
}

impl<T: IntoRecord + Ord> IntoRecord for BTreeSet<T> {
    fn into_record(self) -> Record {
        Record::Array(
            CollectionKind::Set,
            self.into_iter().map(IntoRecord::into_record).collect(),
        )
    }
}

/* Set update semantics: update contains values that are in one of the sets but not the
 * other. */
impl<T: FromRecord + Ord> Mutator<BTreeSet<T>> for Record {
    fn mutate(&self, set: &mut BTreeSet<T>) -> Result<(), String> {
        let upd = <BTreeSet<T>>::from_record(self)?;
        for v in upd.into_iter() {
            if !set.remove(&v) {
                set.insert(v);
            }
        }
        Ok(())
    }
}

/*
 * Use the following macros to generate `IntoRecord` and `Mutator` trait implementations for
 * user-defined structs and enums.
 */

pub fn arg_extract<T: FromRecord + Default>(
    args: &[(Name, Record)],
    argname: &str,
) -> Result<T, String> {
    args.iter()
        .find(|(n, _)| *n == argname)
        .map_or_else(|| Ok(Default::default()), |(_, v)| T::from_record(v))
}

pub fn arg_find<'a>(args: &'a [(Name, Record)], argname: &str) -> Option<&'a Record> {
    args.iter().find(|(n, _)| *n == argname).map(|(_, v)| v)
}

#[macro_export]
macro_rules! decl_struct_into_record {
    ( $n:ident, [ $nstr:expr ] <$( $targ:ident),*>, $( $arg:ident ),* ) => {
        impl <$($targ: $crate::record::IntoRecord),*> $crate::record::IntoRecord for $n<$($targ),*> {
            fn into_record(self) -> $crate::record::Record {
                $crate::record::Record::NamedStruct(::std::borrow::Cow::from($nstr),vec![$((::std::borrow::Cow::from(stringify!($arg)), self.$arg.into_record())),*])
            }
        }
    };

    ( $n:ident, <$( $targ:ident),*>, $( $arg:ident ),* ) => {
        #[automatically_derived]
        impl <$($targ: $crate::record::IntoRecord),*> $crate::record::IntoRecord for $n<$($targ),*> {
            fn into_record(self) -> $crate::record::Record {
                $crate::record::Record::NamedStruct(::std::borrow::Cow::from(stringify!($n)),vec![$((::std::borrow::Cow::from(stringify!($arg)), self.$arg.into_record())),*])
            }
        }
    };
}

#[macro_export]
macro_rules! decl_struct_from_record {
    ( $n:ident [$full_name:expr] <$( $targ:ident),*>, [$constructor_name:expr][$nargs:expr]{$( [$idx:expr] $arg:ident [$alt_arg:expr]: $type:ty),*} ) => {
        #[automatically_derived]
        impl <$($targ: $crate::record::FromRecord + serde::de::DeserializeOwned + ::std::default::Default),*> $crate::record::FromRecord for $n<$($targ),*> {
            fn from_record(val: &$crate::record::Record) -> ::std::result::Result<Self, String> {
                match val {
                    $crate::record::Record::PosStruct(constr, _args) => {
                        match constr.as_ref() {
                            $constructor_name if _args.len() == $nargs => {
                                  Ok($n{ $($arg : <$type>::from_record(&_args[$idx])?,)* })
                            },
                            c => ::std::result::Result::Err(format!("unknown constructor {} of type '{}' in {:?}", c, $full_name, *val))
                        }
                    },
                    $crate::record::Record::NamedStruct(constr, _args) => {
                        match constr.as_ref() {
                            $constructor_name => {
                                Ok($n{ $($arg : $crate::record::arg_extract::<$type>(_args, $alt_arg)?,)* })
                            },
                            c => ::std::result::Result::Err(format!("unknown constructor {} of type '{}' in {:?}", c, $full_name, *val))
                        }
                    },
                    $crate::record::Record::Serialized(format, s) => {
                        if format == "json" {
                            serde_json::from_str(&*s).map_err(|e|format!("{}", e))
                        } else {
                            ::std::result::Result::Err(format!("unsupported serialization format '{}'", format))
                        }
                    },
                    v => {
                        ::std::result::Result::Err(format!("not a struct {:?}", *v))
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! decl_enum_from_record {
    ( $n:ident [$full_name:expr] <$( $targ:ident),*>, $($cons:ident [$cons_name:expr][$nargs:expr]{$( [$idx:expr] $arg:ident [$alt_arg:expr]: $type:ty),*}),* ) => {
        #[automatically_derived]
        impl <$($targ: $crate::record::FromRecord + serde::de::DeserializeOwned + ::std::default::Default),*> $crate::record::FromRecord for $n<$($targ),*> {
            fn from_record(val: &$crate::record::Record) -> ::std::result::Result<Self, String> {
                match val {
                    $crate::record::Record::PosStruct(constr, _args) => {
                        match constr.as_ref() {
                            $($cons_name if _args.len() == $nargs => {
                                  Ok($n::$cons{ $($arg : <$type>::from_record(&_args[$idx])?,)* })
                            },)*
                            c => ::std::result::Result::Err(format!("unknown constructor {} of type '{}' in {:?}", c, $full_name, *val))
                        }
                    },
                    $crate::record::Record::NamedStruct(constr, _args) => {
                        match constr.as_ref() {
                            $($cons_name => {
                                Ok($n::$cons{ $($arg : $crate::record::arg_extract::<$type>(_args, $alt_arg)?,)* })
                            },)*
                            c => ::std::result::Result::Err(format!("unknown constructor {} of type '{}' in {:?}", c, $full_name, *val))
                        }
                    },
                    $crate::record::Record::Serialized(format, s) => {
                        if format == "json" {
                            serde_json::from_str(&*s).map_err(|e|format!("{}", e))
                        } else {
                            ::std::result::Result::Err(format!("unsupported serialization format '{}'", format))
                        }
                    },
                    v => {
                        ::std::result::Result::Err(format!("not a struct {:?}", *v))
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! decl_record_mutator_struct {
    ( $n:ident, <$( $targ:ident),*>, $( $arg:ident : $type:ty),* ) => {
        impl<$($targ),*> $crate::record::Mutator<$n<$($targ),*>> for $crate::record::Record
            where $($crate::record::Record: $crate::record::Mutator<$targ>, $targ: $crate::record::FromRecord),*
        {
            fn mutate(&self, _x: &mut $n<$($targ),*>) -> ::std::result::Result<(), String> {
                match self {
                    $crate::record::Record::PosStruct(_, _args) => {
                        #[allow(unused_mut)]
                        let mut index = 0;

                        $(
                            if index == _args.len() {
                                return ::std::result::Result::Err(format!("Positional struct mutator does not contain all elements"));
                            };
                            let arg_upd = &_args[index];
                            index += 1;
                            <dyn $crate::record::Mutator<$type>>::mutate(arg_upd, &mut _x.$arg)?;
                        )*
                        if index != _args.len() {
                            return ::std::result::Result::Err(format!("Positional struct mutator has too many elements"));
                        }
                    },
                    $crate::record::Record::NamedStruct(_, _args) => {
                        $(if let Some(arg_upd) = $crate::record::arg_find(_args, stringify!($arg)) {
                            <dyn $crate::record::Mutator<$type>>::mutate(arg_upd, &mut _x.$arg)?;
                          };)*
                    },
                    _ => {
                        return ::std::result::Result::Err(format!("not a struct {:?}", *self));
                    }
                };
                ::std::result::Result::Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! decl_record_mutator_enum {
    ( $n:ident<$( $targ:ident),*>, $($cons:ident {$( $arg:ident : $type:ty),*}),* ) => {
        impl<$($targ: $crate::record::FromRecord+serde::de::DeserializeOwned+::std::default::Default),*> $crate::record::Mutator<$n<$($targ),*>> for $crate::record::Record
            where $($crate::record::Record: $crate::record::Mutator<$targ>),*
        {
            fn mutate(&self, x: &mut $n<$($targ),*>) -> ::std::result::Result<(), String> {
                match self {
                    $crate::record::Record::PosStruct(constr, _args) => {
                        match (x, constr.as_ref()) {
                            $(
                                ($n::$cons{$($arg),*}, stringify!($cons)) => {
                                    let mut index = 0;
                                    $(
                                        if index == _args.len() {
                                            return ::std::result::Result::Err(format!("Positional struct mutator does not contain all elements"));
                                        };
                                        let arg_upd = &_args[index];
                                        index += 1;
                                        <dyn $crate::record::Mutator<$type>>::mutate(arg_upd, $arg)?;
                                    )*
                                    if index != _args.len() {
                                        return ::std::result::Result::Err(format!("Positional struct mutator has too many elements"));
                                    }
                                },
                            )*
                            (x, _) => {
                                *x = <$n<$($targ),*>>::from_record(self)?;
                            }
                        }
                    },
                    $crate::record::Record::NamedStruct(constr, args) => {
                        match (x, constr.as_ref()) {
                            $(
                                ($n::$cons{$($arg),*}, stringify!($cons)) => {
                                    $(
                                        if let Some(arg_upd) = $crate::record::arg_find(args, stringify!($arg)) {
                                            <dyn $crate::record::Mutator<$type>>::mutate(arg_upd, $arg)?;
                                        };
                                     )*
                                },
                            )*
                            (x, _) => {
                                *x = <$n<$($targ),*>>::from_record(self)?;
                            }
                        }
                    },
                    _ => {
                        return ::std::result::Result::Err(format!("not a struct {:?}", *self));
                    }
                };
                ::std::result::Result::Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! decl_enum_into_record {
    ( $n:ident<$( $targ:ident),*>, $($cons:ident [$consn:expr] {$($arg:ident),*} ),* ) => {
        impl <$($targ: $crate::record::IntoRecord),*> $crate::record::IntoRecord for $n<$($targ),*> {
            fn into_record(self) -> $crate::record::Record {
                match self {
                    $($n::$cons{$($arg),*} => $crate::record::Record::NamedStruct(::std::borrow::Cow::from($consn), vec![$((::std::borrow::Cow::from(stringify!($arg)), $arg.into_record())),*])),*
                }
            }
        }
    };

    ( $n:ident<$( $targ:ident),*>, $($cons:ident [$consn:expr] ($($arg:ident),*) ),* ) => {
        #[automatically_derived]
        impl <$($targ: $crate::record::IntoRecord),*> $crate::record::IntoRecord for $n<$($targ),*> {
            fn into_record(self) -> $crate::record::Record {
                match self {
                    $($n::$cons($($arg),*) => $crate::record::Record::NamedStruct(::std::borrow::Cow::from($consn), vec![$((::std::borrow::Cow::from(stringify!($arg)), $arg.into_record())),*])),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! decl_val_enum_into_record {
    ( $n:ident<$( $targ:ident),*>, $($cons:ident {$arg:ident} ),* ) => {
        #[automatically_derived]
        impl <$($targ: $crate::record::IntoRecord),*> $crate::record::IntoRecord for $n<$($targ),*> {
            fn into_record(self) -> $crate::record::Record {
                match self {
                    $($n::$cons{$arg} => $arg.into_record()),*
                }
            }
        }
    };

    ( $n:ident<$( $targ:ident),*>, $($cons:ident ($arg:ident) ),* ) => {
        #[automatically_derived]
        impl <$($targ: $crate::record::IntoRecord),*> $crate::record::IntoRecord for $n<$($targ),*> {
            fn into_record(self) -> $crate::record::Record {
                match self {
                    $($n::$cons($arg) => $arg.into_record()),*
                }
            }
        }
    };
}
