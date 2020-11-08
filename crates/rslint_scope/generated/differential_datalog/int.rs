#![allow(non_snake_case)]

use super::record::{FromRecord, IntoRecord, Mutator, Record};
use super::uint;
use abomonation::Abomonation;
use num::bigint::BigInt;
pub use num::bigint::Sign;
#[cfg(test)]
use num::bigint::ToBigInt;
use num::ToPrimitive;
use ordered_float::OrderedFloat;
use serde::de::Error;
use serde::de::*;
use serde::ser::*;
use std::ffi::CStr;
use std::fmt;
use std::ops::*;
use std::os::raw::c_char;
use std::str::FromStr;

#[derive(Eq, PartialOrd, PartialEq, Ord, Clone, Hash)]
pub struct Int {
    x: BigInt,
}

impl Default for Int {
    fn default() -> Int {
        Int {
            x: BigInt::default(),
        }
    }
}

decl_ddval_convert! {Int}

impl Abomonation for Int {}

impl Int {
    pub fn from_bigint(v: BigInt) -> Int {
        Int { x: v }
    }
    pub fn from_u8(v: u8) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_i8(v: i8) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_u16(v: u16) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_i16(v: i16) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_u32(v: u32) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_i32(v: i32) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_u64(v: u64) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_i64(v: i64) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_u128(v: u128) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_i128(v: i128) -> Int {
        Int { x: BigInt::from(v) }
    }
    pub fn from_Uint(v: uint::Uint) -> Int {
        v.to_Int().unwrap()
    }
    pub fn from_bytes_be(sign: bool, bytes: &[u8]) -> Int {
        Int {
            x: BigInt::from_bytes_be(if sign { Sign::Plus } else { Sign::Minus }, bytes),
        }
    }
    pub fn to_bytes_be(&self) -> (Sign, Vec<u8>) {
        self.x.to_bytes_be()
    }
    pub fn to_i8(&self) -> Option<i8> {
        self.x.to_i8()
    }
    pub fn to_u8(&self) -> Option<u8> {
        self.x.to_u8()
    }
    /* Extract 8 low-order bits and convert to u8 */
    pub fn truncate_to_u8(&self) -> u8 {
        (&self.x & &BigInt::from(0xffu8)).to_u8().unwrap()
    }
    pub fn to_i16(&self) -> Option<i16> {
        self.x.to_i16()
    }
    pub fn to_u16(&self) -> Option<u16> {
        self.x.to_u16()
    }
    /* Extract 16 low-order bits and convert to u16 */
    pub fn truncate_to_u16(&self) -> u16 {
        (&self.x & &BigInt::from(0xffffu16)).to_u16().unwrap()
    }
    pub fn to_i32(&self) -> Option<i32> {
        self.x.to_i32()
    }
    pub fn to_u32(&self) -> Option<u32> {
        self.x.to_u32()
    }
    /* Extract 32 low-order bits and convert to u32 */
    pub fn truncate_to_u32(&self) -> u32 {
        (&self.x & &BigInt::from(0xffff_ffffu32)).to_u32().unwrap()
    }
    pub fn to_i64(&self) -> Option<i64> {
        self.x.to_i64()
    }
    pub fn to_u64(&self) -> Option<u64> {
        self.x.to_u64()
    }
    /* Extract 64 low-order bits and convert to u64 */
    pub fn truncate_to_u64(&self) -> u64 {
        (&self.x & &BigInt::from(0xffff_ffff_ffff_ffffu64))
            .to_u64()
            .unwrap()
    }
    pub fn to_i128(&self) -> Option<i128> {
        self.x.to_i128()
    }
    pub fn to_u128(&self) -> Option<u128> {
        self.x.to_u128()
    }
    /* Extract 128 low-order bits and convert to u128 */
    pub fn truncate_to_u128(&self) -> u128 {
        (&self.x & &BigInt::from(0xffff_ffff_ffff_ffff_ffff_ffff_ffff_ffffu128))
            .to_u128()
            .unwrap()
    }
    pub fn to_float(&self) -> OrderedFloat<f32> {
        match self.x.to_f32() {
            None => OrderedFloat::<f32>(std::f32::NAN),
            Some(x) => OrderedFloat::<f32>(x),
        }
    }
    pub fn to_double(&self) -> OrderedFloat<f64> {
        match self.x.to_f64() {
            None => OrderedFloat::<f64>(std::f64::NAN),
            Some(x) => OrderedFloat::<f64>(x),
        }
    }
    pub fn to_Uint(&self) -> Option<uint::Uint> {
        self.x.to_biguint().map(uint::Uint::from_biguint)
    }
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Int {
        Int {
            x: BigInt::parse_bytes(buf, radix).unwrap(),
        }
    }
}

#[no_mangle]
pub extern "C" fn int_from_i64(v: i64) -> *mut Int {
    Box::into_raw(Box::new(Int::from_i64(v)))
}

#[no_mangle]
pub extern "C" fn int_from_u64(v: u64) -> *mut Int {
    Box::into_raw(Box::new(Int::from_u64(v)))
}

#[no_mangle]
pub unsafe extern "C" fn int_from_str(s: *const c_char, radix: u32) -> *mut Int {
    let c_str = CStr::from_ptr(s);
    Box::into_raw(Box::new(Int::parse_bytes(c_str.to_bytes(), radix)))
}

#[no_mangle]
pub unsafe extern "C" fn int_free(x: *mut Int) {
    if x.is_null() {
        return;
    }
    Box::from_raw(x);
}

impl fmt::Display for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.x)
    }
}

impl fmt::LowerHex for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.x)
    }
}

impl fmt::Debug for Int {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl Serialize for Int {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.x.to_str_radix(10))
    }
}

impl<'de> Deserialize<'de> for Int {
    fn deserialize<D>(deserializer: D) -> Result<Int, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(s) => match BigInt::from_str(&s) {
                Ok(i) => Ok(Int { x: i }),
                Err(_) => Err(D::Error::custom(format!("invalid integer value: {}", s))),
            },
            Err(e) => Err(e),
        }
    }
}

impl FromRecord for Int {
    fn from_record(val: &Record) -> Result<Self, String> {
        Ok(Int::from_bigint(BigInt::from_record(val)?))
    }
}

impl IntoRecord for Int {
    fn into_record(self) -> Record {
        self.x.into_record()
    }
}

impl Mutator<Int> for Record {
    fn mutate(&self, i: &mut Int) -> Result<(), String> {
        self.mutate(&mut i.x)
    }
}

#[test]
fn test_fromrecord() {
    let v = (-25_i64).to_bigint().unwrap();
    assert_eq!(
        Int::from_record(&Record::Int(v.clone())),
        Ok(Int::from_bigint(v))
    );
}

impl Shr<u32> for Int {
    type Output = Int;

    #[inline]
    fn shr(self, rhs: u32) -> Int {
        Int {
            x: self.x.shr(rhs as usize),
        }
    }
}

impl Shl<u32> for Int {
    type Output = Int;

    #[inline]
    fn shl(self, rhs: u32) -> Int {
        Int {
            x: self.x.shl(rhs as usize),
        }
    }
}

impl Neg for Int {
    type Output = Int;

    #[inline]
    fn neg(self) -> Self::Output {
        Int { x: self.x.neg() }
    }
}

macro_rules! forward_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                // forward to val-ref
                Int {
                    x: $imp::$method(self.x, other.x),
                }
            }
        }
    };
}

forward_binop!(impl Add for Int, add);
forward_binop!(impl Sub for Int, sub);
forward_binop!(impl Div for Int, div);
forward_binop!(impl Rem for Int, rem);
forward_binop!(impl Mul for Int, mul);
forward_binop!(impl BitAnd for Int, bitand);
forward_binop!(impl BitOr for Int, bitor);

impl num::One for Int {
    fn one() -> Int {
        Int { x: BigInt::one() }
    }
}

impl num::Zero for Int {
    fn zero() -> Int {
        Int { x: BigInt::zero() }
    }

    fn is_zero(&self) -> bool {
        self.x == BigInt::zero()
    }
}
