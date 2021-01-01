//! Tests for functions and macros in `record.rs`

use std::collections::{BTreeMap, BTreeSet};
use std::iter::FromIterator;
use std::vec;

use num::bigint::{ToBigInt, ToBigUint};
use num::{BigInt, BigUint};

use crate::record::*;

#[test]
fn test_u8() {
    assert_eq!(
        u8::from_record(&Record::Int(25_u8.to_bigint().unwrap())),
        Ok(25)
    );
    assert_eq!(
        u8::from_record(&Record::Int(0xab.to_bigint().unwrap())),
        Ok(0xab)
    );
    assert_eq!(
        u8::from_record(&Record::Int(0xabcd.to_bigint().unwrap())),
        Err("cannot convert 43981 to u8".to_string())
    );
    assert_eq!(u8::into_record(0x25), Record::Int(BigInt::from(0x25)));
}

#[test]
fn test_u16() {
    assert_eq!(
        u16::from_record(&Record::Int(25_u16.to_bigint().unwrap())),
        Ok(25)
    );
    assert_eq!(
        u16::from_record(&Record::Int(0xab.to_bigint().unwrap())),
        Ok(0xab)
    );
    assert_eq!(
        u16::from_record(&Record::Int(0xabcdef.to_bigint().unwrap())),
        Err("cannot convert 11259375 to u16".to_string())
    );
    assert_eq!(u16::into_record(32000), Record::Int(BigInt::from(32000)));
}

#[test]
fn test_u32() {
    assert_eq!(
        u32::from_record(&Record::Int(25_u32.to_bigint().unwrap())),
        Ok(25)
    );
    assert_eq!(
        u32::from_record(&Record::Int(0xab.to_bigint().unwrap())),
        Ok(0xab)
    );
    assert_eq!(
        u32::from_record(&Record::Int(0xabcdef.to_bigint().unwrap())),
        Ok(0xabcdef)
    );
    assert_eq!(
        u32::from_record(&Record::Int((-0xabcdef).to_bigint().unwrap())),
        Err("cannot convert -11259375 to u32".to_string())
    );
}

#[test]
fn test_u64() {
    assert_eq!(
        u64::from_record(&Record::Int(25_u64.to_bigint().unwrap())),
        Ok(25)
    );
    assert_eq!(
        u64::from_record(&Record::Int(0xab.to_bigint().unwrap())),
        Ok(0xab)
    );
    assert_eq!(
        u64::from_record(&Record::Int(0xabcdef.to_bigint().unwrap())),
        Ok(0xabcdef)
    );
    assert_eq!(
        u64::from_record(&Record::Int((-0xabcdef).to_bigint().unwrap())),
        Err("cannot convert -11259375 to u64".to_string())
    );
}

#[test]
fn test_u128() {
    assert_eq!(
        u128::from_record(&Record::Int(25_u128.to_bigint().unwrap())),
        Ok(25)
    );
    assert_eq!(
        u128::from_record(&Record::Int(
            100000000000000000000000000000000000000_u128
                .to_bigint()
                .unwrap()
        )),
        Ok(100000000000000000000000000000000000000)
    );
    assert_eq!(
        u128::from_record(&Record::Int(0xab.to_bigint().unwrap())),
        Ok(0xab)
    );
    assert_eq!(
        u128::from_record(&Record::Int(0xabcdef.to_bigint().unwrap())),
        Ok(0xabcdef)
    );
    assert_eq!(
        u128::from_record(&Record::Int((-0xabcdef).to_bigint().unwrap())),
        Err("cannot convert -11259375 to u128".to_string())
    );
}

#[test]
fn test_bigint() {
    let v = (-25_i64).to_bigint().unwrap();
    assert_eq!(BigInt::from_record(&Record::Int(v.clone())), Ok(v));
}

#[test]
fn test_biguint() {
    let vi = (25_i64).to_bigint().unwrap();
    let vu = (25_i64).to_biguint().unwrap();
    assert_eq!(BigUint::from_record(&Record::Int(vi)), Ok(vu));

    let vi = (-25_i64).to_bigint().unwrap();
    assert_eq!(
        BigUint::from_record(&Record::Int(vi)),
        Err("cannot convert -25 to BigUint".to_string())
    );
}

#[test]
fn test_bool() {
    assert_eq!(bool::from_record(&Record::Bool(true)), Ok(true));
}

#[test]
fn test_string() {
    assert_eq!(
        String::from_record(&Record::String("foo".to_string())),
        Ok("foo".to_string())
    );
    assert_eq!(
        String::from_record(&Record::Bool(true)),
        Err("not a string Bool(true)".to_string())
    );
}

#[test]
fn test_tuple() {
    assert_eq!(
        <(bool, bool)>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false)
        ])),
        Ok((true, false))
    );
    assert_eq!(
        <(bool, bool, bool)>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false)
        ])),
        Ok((true, false, false))
    );
    assert_eq!(
        <(bool, bool, bool, bool)>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true)
        ])),
        Ok((true, false, false, true))
    );
    assert_eq!(
        <(bool, bool, bool, bool, String)>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::String("foo".to_string())
        ])),
        Ok((true, false, false, true, "foo".to_string()))
    );
    assert_eq!(
        <(bool, bool, bool, bool, String, bool)>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::String("foo".to_string()),
            Record::Bool(false)
        ])),
        Ok((true, false, false, true, "foo".to_string(), false))
    );
    assert_eq!(
        <(bool, bool, bool, bool, String, bool, bool)>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::String("foo".to_string()),
            Record::Bool(false),
            Record::Bool(false)
        ])),
        Ok((true, false, false, true, "foo".to_string(), false, false))
    );
    assert_eq!(
        <(bool, bool, bool, bool, String, bool, bool, bool)>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::String("foo".to_string()),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(false)
        ])),
        Ok((
            true,
            false,
            false,
            true,
            "foo".to_string(),
            false,
            false,
            false
        ))
    );
    assert_eq!(
        <(bool, bool, bool, bool, String, bool, bool, bool, bool)>::from_record(&Record::Tuple(
            vec![
                Record::Bool(true),
                Record::Bool(false),
                Record::Bool(false),
                Record::Bool(true),
                Record::String("foo".to_string()),
                Record::Bool(false),
                Record::Bool(false),
                Record::Bool(false),
                Record::Bool(true)
            ]
        )),
        Ok((
            true,
            false,
            false,
            true,
            "foo".to_string(),
            false,
            false,
            false,
            true
        ))
    );
    assert_eq!(
        <(bool, bool, bool, bool, String, bool, bool, bool, bool, bool)>::from_record(
            &Record::Tuple(vec![
                Record::Bool(true),
                Record::Bool(false),
                Record::Bool(false),
                Record::Bool(true),
                Record::String("foo".to_string()),
                Record::Bool(false),
                Record::Bool(false),
                Record::Bool(false),
                Record::Bool(true),
                Record::Bool(true)
            ])
        ),
        Ok((
            true,
            false,
            false,
            true,
            "foo".to_string(),
            false,
            false,
            false,
            true,
            true
        ))
    );
    assert_eq!(
        <(
            bool,
            bool,
            bool,
            bool,
            String,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool
        )>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::String("foo".to_string()),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::Bool(true),
            Record::Bool(true)
        ])),
        Ok((
            true,
            false,
            false,
            true,
            "foo".to_string(),
            false,
            false,
            false,
            true,
            true,
            true
        ))
    );
    assert_eq!(
        <(
            bool,
            bool,
            bool,
            bool,
            String,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool,
            bool
        )>::from_record(&Record::Tuple(vec![
            Record::Bool(true),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::String("foo".to_string()),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(false),
            Record::Bool(true),
            Record::Bool(true),
            Record::Bool(true),
            Record::Bool(false)
        ])),
        Ok((
            true,
            false,
            false,
            true,
            "foo".to_string(),
            false,
            false,
            false,
            true,
            true,
            true,
            false
        ))
    );
}

#[test]
fn test_vec() {
    assert_eq!(
        <vec::Vec<bool>>::from_record(&Record::Array(
            CollectionKind::Unknown,
            vec![Record::Bool(true), Record::Bool(false)]
        )),
        Ok(vec![true, false])
    );
}

#[test]
fn test_array() {
    assert_eq!(
        <[bool; 2]>::from_record(&Record::Array(
            CollectionKind::Vector,
            vec![Record::Bool(true), Record::Bool(false)]
        )),
        Ok([true, false])
    );
    assert_eq!(
        <[bool; 2]>::from_record(&Record::Array(
            CollectionKind::Vector,
            vec![Record::Bool(true)]
        )),
        Err("cannot convert Array(Vector, [Bool(true)]) to array of length 2".to_owned())
    );
}

#[test]
fn test_map() {
    assert_eq!(
        <BTreeMap<u32, u32>>::from_record(&Record::Array(
            CollectionKind::Unknown,
            vec![
                Record::Tuple(vec![
                    Record::Int(BigInt::from(0)),
                    Record::Int(BigInt::from(10))
                ]),
                Record::Tuple(vec![
                    Record::Int(BigInt::from(1)),
                    Record::Int(BigInt::from(10))
                ])
            ]
        )),
        Ok(BTreeMap::from_iter(vec![(0, 10), (1, 10)]))
    );

    let mut v = <BTreeMap<u32, u32>>::from_record(&Record::Array(
        CollectionKind::Unknown,
        vec![
            Record::Tuple(vec![
                Record::Int(BigInt::from(0)),
                Record::Int(BigInt::from(10)),
            ]),
            Record::Tuple(vec![
                Record::Int(BigInt::from(1)),
                Record::Int(BigInt::from(10)),
            ]),
        ],
    ))
    .unwrap();
    Record::Array(
        CollectionKind::Unknown,
        vec![
            Record::Tuple(vec![
                Record::Int(BigInt::from(0)),
                Record::Int(BigInt::from(10)),
            ]),
            Record::Tuple(vec![
                Record::Int(BigInt::from(1)),
                Record::Int(BigInt::from(10)),
            ]),
        ],
    )
    .mutate(&mut v)
    .unwrap();
    assert_eq!(v, BTreeMap::from_iter(vec![]));

    v = <BTreeMap<u32, u32>>::from_record(&Record::Array(
        CollectionKind::Unknown,
        vec![
            Record::Tuple(vec![
                Record::Int(BigInt::from(2)),
                Record::Int(BigInt::from(20)),
            ]),
            Record::Tuple(vec![
                Record::Int(BigInt::from(1)),
                Record::Int(BigInt::from(20)),
            ]),
        ],
    ))
    .unwrap();
    Record::Array(
        CollectionKind::Unknown,
        vec![
            Record::Tuple(vec![
                Record::Int(BigInt::from(0)),
                Record::Int(BigInt::from(10)),
            ]),
            Record::Tuple(vec![
                Record::Int(BigInt::from(1)),
                Record::Int(BigInt::from(10)),
            ]),
        ],
    )
    .mutate(&mut v)
    .unwrap();
    assert_eq!(v, BTreeMap::from_iter(vec![(0, 10), (1, 10), (2, 20)]));

    v = <BTreeMap<u32, u32>>::from_record(&Record::Array(
        CollectionKind::Unknown,
        vec![
            Record::Tuple(vec![
                Record::Int(BigInt::from(2)),
                Record::Int(BigInt::from(20)),
            ]),
            Record::Tuple(vec![
                Record::Int(BigInt::from(1)),
                Record::Int(BigInt::from(10)),
            ]),
        ],
    ))
    .unwrap();
    Record::Array(
        CollectionKind::Unknown,
        vec![
            Record::Tuple(vec![
                Record::Int(BigInt::from(0)),
                Record::Int(BigInt::from(10)),
            ]),
            Record::Tuple(vec![
                Record::Int(BigInt::from(1)),
                Record::Int(BigInt::from(10)),
            ]),
        ],
    )
    .mutate(&mut v)
    .unwrap();
    assert_eq!(v, BTreeMap::from_iter(vec![(0, 10), (2, 20)]));
}

#[test]
fn test_set() {
    assert_eq!(
        <BTreeSet<u32>>::from_record(&Record::Array(
            CollectionKind::Unknown,
            vec![Record::Int(BigInt::from(0)), Record::Int(BigInt::from(1))]
        )),
        Ok(BTreeSet::from_iter(vec![0, 1]))
    );

    let mut v = <BTreeSet<u32>>::from_record(&Record::Array(
        CollectionKind::Unknown,
        vec![Record::Int(BigInt::from(0)), Record::Int(BigInt::from(2))],
    ))
    .unwrap();
    Record::Array(
        CollectionKind::Unknown,
        vec![Record::Int(BigInt::from(0)), Record::Int(BigInt::from(1))],
    )
    .mutate(&mut v)
    .unwrap();
    assert_eq!(v, BTreeSet::from_iter(vec![1, 2]));
}
