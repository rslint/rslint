#![allow(
    path_statements,
//unused_imports,
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
    clippy::match_single_binding
)]

// Required for #[derive(Serialize, Deserialize)].
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;

use crate::closure;
use crate::std_usize;
use crate::string_append;
use crate::string_append_str;

//
// use crate::ddlog_std;

use differential_datalog::record;
use differential_datalog::record::Record;
use std::cmp;
use std::fmt;
use std::hash::Hash;
use std::ops::Deref;

#[cfg(feature = "flatbuf")]
use crate::flatbuf::{FromFlatBuffer, ToFlatBuffer, ToFlatBufferTable, ToFlatBufferVectorElement};

/* `flatc`-generated declarations re-exported by `flatbuf.rs` */
#[cfg(feature = "flatbuf")]
use crate::flatbuf::fb;

/* FlatBuffers runtime */
#[cfg(feature = "flatbuf")]
use flatbuffers as fbrt;

#[derive(Default, Eq, PartialEq, Clone, Hash)]
pub struct Intern<A>
where
    A: Eq + Send + Sync + Hash + 'static,
{
    intern: arc_interner::ArcIntern<A>,
}

impl<A: Eq + Send + Sync + Hash + 'static> PartialOrd for Intern<A> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        let sptr = self.as_ref() as *const A as usize;
        let optr = other.as_ref() as *const A as usize;

        sptr.partial_cmp(&optr)
    }
}

impl<A: Eq + Send + Sync + Hash + 'static> Ord for Intern<A> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let sptr = self.as_ref() as *const A as usize;
        let optr = other.as_ref() as *const A as usize;

        sptr.cmp(&optr)
    }
}

impl<A: Eq + Send + Sync + Hash + 'static> Deref for Intern<A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        self.intern.deref()
    }
}

impl<A: Eq + Hash + Send + Sync + 'static> Intern<A> {
    pub fn new(x: A) -> Intern<A> {
        Intern {
            intern: arc_interner::ArcIntern::new(x),
        }
    }
}

impl<A> AsRef<A> for Intern<A>
where
    A: Eq + Hash + Send + Sync + 'static,
{
    fn as_ref(&self) -> &A {
        self.intern.as_ref()
    }
}

pub fn intern<A: Eq + Hash + Send + Sync + Clone + 'static>(x: &A) -> Intern<A> {
    Intern::new(x.clone())
}

pub fn ival<A: Eq + Hash + Send + Sync + Clone>(x: &Intern<A>) -> &A {
    x.intern.as_ref()
}

/*pub fn intern_istring_ord(s: &intern_istring) -> u32 {
    s.x
}*/

impl<A: fmt::Display + Eq + Hash + Send + Sync + Clone> fmt::Display for Intern<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_ref(), f)
        //record::format_ddlog_str(&intern_istring_str(self), f)
    }
}

impl<A: fmt::Debug + Eq + Hash + Send + Sync + Clone> fmt::Debug for Intern<A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_ref(), f)
        //record::format_ddlog_str(&intern_istring_str(self), f)
    }
}

impl<A: Serialize + Eq + Hash + Send + Sync + Clone> serde::Serialize for Intern<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.as_ref().serialize(serializer)
    }
}

impl<'de, A: Deserialize<'de> + Eq + Hash + Send + Sync + 'static> serde::Deserialize<'de>
    for Intern<A>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        A::deserialize(deserializer).map(Intern::new)
    }
}

impl<A: FromRecord + Eq + Hash + Send + Sync + 'static> FromRecord for Intern<A> {
    fn from_record(val: &Record) -> Result<Self, String> {
        A::from_record(val).map(Intern::new)
    }
}

impl<A: IntoRecord + Eq + Hash + Send + Sync + Clone> IntoRecord for Intern<A> {
    fn into_record(self) -> Record {
        ival(&self).clone().into_record()
    }
}

impl<A> Mutator<Intern<A>> for Record
where
    A: Clone + Eq + Send + Sync + Hash,
    Record: Mutator<A>,
{
    fn mutate(&self, x: &mut Intern<A>) -> Result<(), String> {
        let mut v = ival(x).clone();
        self.mutate(&mut v)?;
        *x = intern(&v);
        Ok(())
    }
}

#[cfg(feature = "flatbuf")]
impl<A, FB> FromFlatBuffer<FB> for Intern<A>
where
    A: Eq + Hash + Send + Sync + 'static,
    A: FromFlatBuffer<FB>,
{
    fn from_flatbuf(fb: FB) -> Result<Self, String> {
        Ok(Intern::new(A::from_flatbuf(fb)?))
    }
}

#[cfg(feature = "flatbuf")]
impl<'b, A, T> ToFlatBuffer<'b> for Intern<A>
where
    T: 'b,
    A: Eq + Send + Sync + Hash + ToFlatBuffer<'b, Target = T>,
{
    type Target = T;

    fn to_flatbuf(&self, fbb: &mut fbrt::FlatBufferBuilder<'b>) -> Self::Target {
        self.as_ref().to_flatbuf(fbb)
    }
}

/*#[cfg(feature = "flatbuf")]
impl<'a> FromFlatBuffer<fb::__String<'a>> for intern_istring {
    fn from_flatbuf(v: fb::__String<'a>) -> Response<Self> {
        Ok(intern_string_intern(&String::from_flatbuf(v)?))
    }
}*/

#[cfg(feature = "flatbuf")]
impl<'b, A, T> ToFlatBufferTable<'b> for Intern<A>
where
    T: 'b,
    A: Eq + Send + Sync + Hash + ToFlatBufferTable<'b, Target = T>,
{
    type Target = T;
    fn to_flatbuf_table(
        &self,
        fbb: &mut fbrt::FlatBufferBuilder<'b>,
    ) -> fbrt::WIPOffset<Self::Target> {
        self.as_ref().to_flatbuf_table(fbb)
    }
}

#[cfg(feature = "flatbuf")]
impl<'b, A, T> ToFlatBufferVectorElement<'b> for Intern<A>
where
    T: 'b + fbrt::Push + Copy,
    A: Eq + Send + Sync + Hash + ToFlatBufferVectorElement<'b, Target = T>,
{
    type Target = T;

    fn to_flatbuf_vector_element(&self, fbb: &mut fbrt::FlatBufferBuilder<'b>) -> Self::Target {
        self.as_ref().to_flatbuf_vector_element(fbb)
    }
}

pub fn istring_join(strings: &crate::ddlog_std::Vec<istring>, sep: &String) -> String {
    strings
        .x
        .iter()
        .map(|s| s.as_ref())
        .cloned()
        .collect::<Vec<String>>()
        .join(sep.as_str())
}

pub fn istring_split(s: &istring, sep: &String) -> crate::ddlog_std::Vec<String> {
    crate::ddlog_std::Vec {
        x: s.as_ref().split(sep).map(|x| x.to_owned()).collect(),
    }
}

pub fn istring_contains(s1: &istring, s2: &String) -> bool {
    s1.as_ref().contains(s2.as_str())
}

pub fn istring_substr(s: &istring, start: &std_usize, end: &std_usize) -> String {
    let len = s.as_ref().len();
    let from = cmp::min(*start as usize, len);
    let to = cmp::max(from, cmp::min(*end as usize, len));
    s.as_ref()[from..to].to_string()
}

pub fn istring_replace(s: &istring, from: &String, to: &String) -> String {
    s.as_ref().replace(from, to)
}

pub fn istring_starts_with(s: &istring, prefix: &String) -> bool {
    s.as_ref().starts_with(prefix)
}

pub fn istring_ends_with(s: &istring, suffix: &String) -> bool {
    s.as_ref().ends_with(suffix)
}

pub fn istring_trim(s: &istring) -> String {
    s.as_ref().trim().to_string()
}

pub fn istring_len(s: &istring) -> std_usize {
    s.as_ref().len() as std_usize
}

pub fn istring_to_bytes(s: &istring) -> crate::ddlog_std::Vec<u8> {
    crate::ddlog_std::Vec::from(s.as_ref().as_bytes())
}

pub fn istring_to_lowercase(s: &istring) -> String {
    s.as_ref().to_lowercase()
}

pub fn istring_to_uppercase(s: &istring) -> String {
    s.as_ref().to_uppercase()
}

pub fn istring_reverse(s: &istring) -> String {
    s.as_ref().chars().rev().collect()
}

pub type istring = crate::internment::Intern<String>;
/* fn intern<A: crate::Val>(s: & A) -> crate::internment::Intern<A> */
/* fn istring_contains(s1: & crate::internment::istring, s2: & String) -> bool */
/* fn istring_ends_with(s: & crate::internment::istring, suffix: & String) -> bool */
/* fn istring_join(strings: & crate::ddlog_std::Vec<crate::internment::istring>, sep: & String) -> String */
/* fn istring_len(s: & crate::internment::istring) -> u64 */
/* fn istring_replace(s: & crate::internment::istring, from: & String, to: & String) -> String */
/* fn istring_reverse(s: & crate::internment::istring) -> String */
/* fn istring_split(s: & crate::internment::istring, sep: & String) -> crate::ddlog_std::Vec<String> */
/* fn istring_starts_with(s: & crate::internment::istring, prefix: & String) -> bool */
/* fn istring_substr(s: & crate::internment::istring, start: & u64, end: & u64) -> String */
/* fn istring_to_bytes(s: & crate::internment::istring) -> crate::ddlog_std::Vec<u8> */
/* fn istring_to_lowercase(s: & crate::internment::istring) -> String */
/* fn istring_to_uppercase(s: & crate::internment::istring) -> String */
/* fn istring_trim(s: & crate::internment::istring) -> String */
/* fn ival<A: crate::Val>(s: & crate::internment::Intern<A>) -> A */
pub fn contains(s1: &crate::internment::istring, s2: &String) -> bool {
    crate::internment::istring_contains(s1, s2)
}
pub fn ends_with(s: &crate::internment::istring, suffix: &String) -> bool {
    crate::internment::istring_ends_with(s, suffix)
}
pub fn join(strings: &crate::ddlog_std::Vec<crate::internment::istring>, sep: &String) -> String {
    crate::internment::istring_join(strings, sep)
}
pub fn len(s: &crate::internment::istring) -> u64 {
    crate::internment::istring_len(s)
}
pub fn replace(s: &crate::internment::istring, from: &String, to: &String) -> String {
    crate::internment::istring_replace(s, from, to)
}
pub fn reverse(s: &crate::internment::istring) -> String {
    crate::internment::istring_reverse(s)
}
pub fn split(s: &crate::internment::istring, sep: &String) -> crate::ddlog_std::Vec<String> {
    crate::internment::istring_split(s, sep)
}
pub fn starts_with(s: &crate::internment::istring, prefix: &String) -> bool {
    crate::internment::istring_starts_with(s, prefix)
}
pub fn substr(s: &crate::internment::istring, start: &u64, end: &u64) -> String {
    crate::internment::istring_substr(s, start, end)
}
pub fn to_bytes(s: &crate::internment::istring) -> crate::ddlog_std::Vec<u8> {
    crate::internment::istring_to_bytes(s)
}
pub fn to_lowercase(s: &crate::internment::istring) -> String {
    crate::internment::istring_to_lowercase(s)
}
pub fn to_string(s: &crate::internment::istring) -> String {
    (*crate::internment::ival(s)).clone()
}
pub fn to_uppercase(s: &crate::internment::istring) -> String {
    crate::internment::istring_to_uppercase(s)
}
pub fn trim(s: &crate::internment::istring) -> String {
    crate::internment::istring_trim(s)
}
