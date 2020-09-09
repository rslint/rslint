//! JS Number parsing. 

use lexical::parse_radix;

pub use num_bigint::BigInt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsNum {
    Float(f64),
    BigInt(BigInt)
}

/// Parse a js number as a string into a number.  
pub(crate) fn parse_js_num(num: String) -> Option<JsNum> {
    let (radix, mut raw) = match num.get(0..2) {
        Some("0x") => (16, num.get(2..).unwrap()),
        Some("0b") => (2, num.get(2..).unwrap()),
        Some("0o") => (8, num.get(2..).unwrap()),
        _ => (10, num.as_str())
    };

    let bigint = if raw.get(raw.len() - 1..raw.len()) == Some("n") {
        raw = raw.split_at(raw.len() - 1).0;
        true
    } else {
        false
    };

    if bigint {
        Some(JsNum::BigInt(BigInt::parse_bytes(raw.as_bytes(), radix)?))
    } else {
        Some(JsNum::Float(parse_radix::<f64, _>(raw.as_bytes(), radix as u8).ok()?))
    }
}
