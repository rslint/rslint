#![allow(non_snake_case)]

use super::int;
use super::record::{FromRecord, IntoRecord, Mutator, Record};
use abomonation::Abomonation;
use num::bigint::ToBigInt;
use num::bigint::{BigInt, BigUint};
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
pub struct Uint {
    x: BigUint,
}

impl Default for Uint {
    fn default() -> Uint {
        Uint {
            x: BigUint::default(),
        }
    }
}

impl Abomonation for Uint {}

decl_ddval_convert! {Uint}

impl Uint {
    pub fn from_biguint(v: BigUint) -> Uint {
        Uint { x: v }
    }
    pub fn from_bigint(v: BigInt) -> Uint {
        Uint {
            x: v.to_biguint().unwrap(),
        }
    }
    pub fn from_Int(v: int::Int) -> Uint {
        v.to_Uint().unwrap()
    }
    pub fn from_u8(v: u8) -> Uint {
        Uint {
            x: BigUint::from(v),
        }
    }
    pub fn from_u16(v: u16) -> Uint {
        Uint {
            x: BigUint::from(v),
        }
    }
    pub fn from_u32(v: u32) -> Uint {
        Uint {
            x: BigUint::from(v),
        }
    }
    pub fn from_u64(v: u64) -> Uint {
        Uint {
            x: BigUint::from(v),
        }
    }
    pub fn from_u128(v: u128) -> Uint {
        Uint {
            x: BigUint::from(v),
        }
    }
    pub fn from_bytes_be(bytes: &[u8]) -> Uint {
        Uint {
            x: BigUint::from_bytes_be(bytes),
        }
    }
    pub fn to_bytes_be(&self) -> Vec<u8> {
        self.x.to_bytes_be()
    }
    pub fn to_u8(&self) -> Option<u8> {
        self.x.to_u8()
    }
    pub fn to_u16(&self) -> Option<u16> {
        self.x.to_u16()
    }
    pub fn to_u32(&self) -> Option<u32> {
        self.x.to_u32()
    }
    pub fn to_u64(&self) -> Option<u64> {
        self.x.to_u64()
    }
    pub fn to_u128(&self) -> Option<u128> {
        self.x.to_u128()
    }
    pub fn to_Int(&self) -> Option<int::Int> {
        self.x.to_bigint().map(int::Int::from_bigint)
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
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Uint {
        Uint {
            x: BigUint::parse_bytes(buf, radix).unwrap(),
        }
    }
}

#[no_mangle]
pub extern "C" fn uint_from_u64(v: u64) -> *mut Uint {
    Box::into_raw(Box::new(Uint::from_u64(v)))
}

#[no_mangle]
pub unsafe extern "C" fn uint_from_str(s: *const c_char, radix: u32) -> *mut Uint {
    let c_str = CStr::from_ptr(s);
    Box::into_raw(Box::new(Uint::parse_bytes(c_str.to_bytes(), radix)))
}

#[no_mangle]
pub unsafe extern "C" fn uint_free(x: *mut Uint) {
    if x.is_null() {
        return;
    }
    Box::from_raw(x);
}

impl fmt::Display for Uint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.x)
    }
}

impl fmt::LowerHex for Uint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.x)
    }
}

impl fmt::Debug for Uint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl Serialize for Uint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.x.to_str_radix(10))
    }
}

impl<'de> Deserialize<'de> for Uint {
    fn deserialize<D>(deserializer: D) -> Result<Uint, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer) {
            Ok(s) => match BigUint::from_str(&s) {
                Ok(i) => Ok(Uint { x: i }),
                Err(_) => Err(D::Error::custom(format!("invalid integer value: {}", s))),
            },
            Err(e) => Err(e),
        }
    }
}

impl FromRecord for Uint {
    fn from_record(val: &Record) -> Result<Self, String> {
        Ok(Uint::from_biguint(BigUint::from_record(val)?))
    }
}

impl IntoRecord for Uint {
    fn into_record(self) -> Record {
        self.x.into_record()
    }
}

impl Mutator<Uint> for Record {
    fn mutate(&self, i: &mut Uint) -> Result<(), String> {
        self.mutate(&mut i.x)
    }
}

#[test]
fn test_fromrecord() {
    let v = (25_u64).to_bigint().unwrap();
    assert_eq!(
        Uint::from_record(&Record::Int(v.clone())),
        Ok(Uint::from_bigint(v))
    );
}

/*
impl Uint {
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Uint {
        Uint{x: BigUint::parse_bytes(buf, radix).unwrap()}
    }
}
*/

/* DDlog supports 32-bit shifts */
impl Shr<u32> for Uint {
    type Output = Uint;

    #[inline]
    fn shr(self, rhs: u32) -> Uint {
        Uint {
            x: self.x.shr(rhs as usize),
        }
    }
}

impl Shl<u32> for Uint {
    type Output = Uint;

    #[inline]
    fn shl(self, rhs: u32) -> Uint {
        Uint {
            x: self.x.shl(rhs as usize),
        }
    }
}

macro_rules! forward_binop {
    (impl $imp:ident for $res:ty, $method:ident) => {
        impl $imp<$res> for $res {
            type Output = $res;

            #[inline]
            fn $method(self, other: $res) -> $res {
                // forward to val-ref
                Uint {
                    x: $imp::$method(self.x, other.x),
                }
            }
        }
    };
}

forward_binop!(impl Add for Uint, add);
forward_binop!(impl Sub for Uint, sub);
forward_binop!(impl Div for Uint, div);
forward_binop!(impl Rem for Uint, rem);
forward_binop!(impl Mul for Uint, mul);
forward_binop!(impl BitAnd for Uint, bitand);
forward_binop!(impl BitOr for Uint, bitor);

impl num::One for Uint {
    fn one() -> Uint {
        Uint { x: BigUint::one() }
    }
}

impl num::Zero for Uint {
    fn zero() -> Uint {
        Uint { x: BigUint::zero() }
    }

    fn is_zero(&self) -> bool {
        self.x == BigUint::zero()
    }
}
