use abomonation::Abomonation;
use serde::Deserialize;
use serde::Serialize;
use std::fmt;
use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::record::IntoRecord;
use crate::record::Mutator;
use crate::record::Record;
use crate::uint;

/// `Value` type that implements `trait DDValConvert` and is thus useful for testing Rust modules that
/// interact with the DDlog API, but do not define their own value type.

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct Empty {}
impl Abomonation for Empty {}
impl Display for Empty {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for Empty {
    fn into_record(self) -> Record {
        unimplemented!("Empty::IntoRecord");
    }
}
impl Mutator<Empty> for Record {
    fn mutate(&self, _v: &mut Empty) -> Result<(), std::string::String> {
        unimplemented!("Empty::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct Bool(pub bool);
impl Abomonation for Bool {}
impl Display for Bool {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for Bool {
    fn into_record(self) -> Record {
        unimplemented!("Bool::IntoRecord");
    }
}
impl Mutator<Bool> for Record {
    fn mutate(&self, _v: &mut Bool) -> Result<(), std::string::String> {
        unimplemented!("Bool::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct Uint(pub uint::Uint);
impl Abomonation for Uint {}
impl Display for Uint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for Uint {
    fn into_record(self) -> Record {
        unimplemented!("Uint::IntoRecord");
    }
}
impl Mutator<Uint> for Record {
    fn mutate(&self, _v: &mut Uint) -> Result<(), std::string::String> {
        unimplemented!("Uint::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct String(pub std::string::String);
impl Abomonation for String {}
impl Display for String {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for String {
    fn into_record(self) -> Record {
        unimplemented!("String::IntoRecord");
    }
}
impl Mutator<String> for Record {
    fn mutate(&self, _v: &mut String) -> Result<(), std::string::String> {
        unimplemented!("String::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct U8(pub u8);
impl Abomonation for U8 {}
impl Display for U8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for U8 {
    fn into_record(self) -> Record {
        unimplemented!("U8::IntoRecord");
    }
}
impl Mutator<U8> for Record {
    fn mutate(&self, _v: &mut U8) -> Result<(), std::string::String> {
        unimplemented!("U8::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct U16(pub u16);
impl Abomonation for U16 {}
impl Display for U16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for U16 {
    fn into_record(self) -> Record {
        unimplemented!("U16::IntoRecord");
    }
}
impl Mutator<U16> for Record {
    fn mutate(&self, _v: &mut U16) -> Result<(), std::string::String> {
        unimplemented!("U16::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct U32(pub u32);
impl Abomonation for U32 {}
impl Display for U32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for U32 {
    fn into_record(self) -> Record {
        unimplemented!("U32::IntoRecord");
    }
}
impl Mutator<U32> for Record {
    fn mutate(&self, _v: &mut U32) -> Result<(), std::string::String> {
        unimplemented!("U32::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct U64(pub u64);
impl Abomonation for U64 {}
impl Display for U64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for U64 {
    fn into_record(self) -> Record {
        unimplemented!("U64::IntoRecord");
    }
}
impl Mutator<U64> for Record {
    fn mutate(&self, _v: &mut U64) -> Result<(), std::string::String> {
        unimplemented!("U64::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct I64(pub i64);
impl Abomonation for I64 {}
impl Display for I64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for I64 {
    fn into_record(self) -> Record {
        unimplemented!("I64::IntoRecord");
    }
}
impl Mutator<I64> for Record {
    fn mutate(&self, _v: &mut I64) -> Result<(), std::string::String> {
        unimplemented!("I64::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct BoolTuple(pub (bool, bool));
impl Abomonation for BoolTuple {}
impl Display for BoolTuple {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for BoolTuple {
    fn into_record(self) -> Record {
        unimplemented!("BoolTuple::IntoRecord");
    }
}
impl Mutator<BoolTuple> for Record {
    fn mutate(&self, _v: &mut BoolTuple) -> Result<(), std::string::String> {
        unimplemented!("BoolTuple::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct Tuple2<T>(pub Box<T>, pub Box<T>);
impl<T: Abomonation> Abomonation for Tuple2<T> {}
impl<T: Debug> Display for Tuple2<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl<T> IntoRecord for Tuple2<T> {
    fn into_record(self) -> Record {
        unimplemented!("Tuple2::IntoRecord");
    }
}
impl<T> Mutator<Tuple2<T>> for Record {
    fn mutate(&self, _v: &mut Tuple2<T>) -> Result<(), std::string::String> {
        unimplemented!("Tuple2::Mutator");
    }
}

#[derive(Default, Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct Q {
    pub f1: bool,
    pub f2: String,
}
impl Abomonation for Q {}
impl Display for Q {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for Q {
    fn into_record(self) -> Record {
        unimplemented!("Q::IntoRecord");
    }
}
impl Mutator<Q> for Record {
    fn mutate(&self, _v: &mut Q) -> Result<(), std::string::String> {
        unimplemented!("Q::Mutator");
    }
}

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub enum S {
    S1 {
        f1: u32,
        f2: String,
        f3: Q,
        f4: Uint,
    },
    S2 {
        e1: bool,
    },
    S3 {
        g1: Q,
        g2: Q,
    },
}
impl Abomonation for S {}
impl S {
    pub fn f1(&mut self) -> &mut u32 {
        match self {
            S::S1 { ref mut f1, .. } => f1,
            _ => panic!(""),
        }
    }
}
impl Default for S {
    fn default() -> S {
        S::S1 {
            f1: u32::default(),
            f2: String::default(),
            f3: Q::default(),
            f4: Uint::default(),
        }
    }
}
impl Display for S {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}
impl IntoRecord for S {
    fn into_record(self) -> Record {
        unimplemented!("S::IntoRecord");
    }
}
impl Mutator<S> for Record {
    fn mutate(&self, _v: &mut S) -> Result<(), std::string::String> {
        unimplemented!("S::Mutator");
    }
}

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize, Debug)]
pub struct P {
    pub f1: Q,
    pub f2: bool,
}
impl Abomonation for P {}
