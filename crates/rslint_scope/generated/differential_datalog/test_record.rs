//! Tests for functions and macros in `record.rs`

use std::borrow::Cow;
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

#[derive(Eq, PartialEq, Debug, Clone, Default)]
struct Foo<T> {
    f1: T,
}

impl<T: FromRecord + Default> FromRecord for Foo<T> {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::PosStruct(constr, args) => match constr.as_ref() {
                "Foo" if args.len() == 1 => Ok(Foo {
                    f1: T::from_record(&args[0])?,
                }),
                c => Result::Err(format!(
                    "unknown constructor {} of type Foo in {:?}",
                    c, *val
                )),
            },
            Record::NamedStruct(constr, args) => match constr.as_ref() {
                "Foo" => Ok(Foo {
                    f1: arg_extract(args, "f1")?,
                }),
                c => Result::Err(format!(
                    "unknown constructor {} of type Foo in {:?}",
                    c, *val
                )),
            },
            v => Result::Err(format!("not a struct {:?}", *v)),
        }
    }
}

pub struct NestedStruct<T> {
    x: bool,
    y: Foo<T>,
}
pub struct StructWithNoFields;

decl_struct_into_record!(Foo, ["Foo"] <T>, f1);
decl_record_mutator_struct!(Foo, <T>, f1: T);

decl_struct_into_record!(NestedStruct, ["Foo"] <T>, x,y);
decl_record_mutator_struct!(NestedStruct, <T>, x: bool, y: Foo<T>);

decl_struct_into_record!(StructWithNoFields, ["StructWithNoFields"] <>,);
decl_record_mutator_struct!(StructWithNoFields, <>, );

#[test]
fn test_struct() {
    let foo1: Foo<BTreeMap<u32, String>> = Foo {
        f1: BTreeMap::from_iter(vec![(5, "five".to_owned()), (6, "six".to_owned())]),
    };
    let foo2 = <Foo<BTreeMap<u32, String>>>::from_record(&foo1.clone().into_record()).unwrap();

    assert_eq!(foo1, foo2);

    let upd = Record::NamedStruct(
        Cow::from("Foo"),
        vec![(
            Cow::from("f1"),
            Record::Array(
                CollectionKind::Unknown,
                vec![Record::Tuple(vec![
                    Record::Int(BigInt::from(5)),
                    Record::String("5".to_owned()),
                ])],
            ),
        )],
    );
    let mut foo_mod = foo1;
    upd.mutate(&mut foo_mod).unwrap();
    let foo_expected = Foo {
        f1: BTreeMap::from_iter(vec![(5, "5".to_owned()), (6, "six".to_owned())]),
    };
    assert_eq!(foo_mod, foo_expected);
}

type Bbool = bool;

#[derive(Eq, PartialEq, Debug, Clone)]
enum DummyEnum<T> {
    Constr1 { f1: Bbool, f2: String },
    Constr2 { f1: T, f2: BigInt, f3: Foo<T> },
    Constr3 { f1: (bool, bool) },
}

impl<T: FromRecord + Default> FromRecord for DummyEnum<T> {
    fn from_record(val: &Record) -> Result<Self, String> {
        match val {
            Record::PosStruct(constr, args) => match constr.as_ref() {
                "Constr1" if args.len() == 2 => Ok(DummyEnum::Constr1 {
                    f1: <Bbool>::from_record(&args[0])?,
                    f2: String::from_record(&args[1])?,
                }),
                "Constr2" if args.len() == 3 => Ok(DummyEnum::Constr2 {
                    f1: <T>::from_record(&args[0])?,
                    f2: <BigInt>::from_record(&args[1])?,
                    f3: <Foo<T>>::from_record(&args[2])?,
                }),
                "Constr3" if args.len() == 1 => Ok(DummyEnum::Constr3 {
                    f1: <(bool, bool)>::from_record(&args[0])?,
                }),
                c => Result::Err(format!(
                    "unknown constructor {} of type DummyEnum in {:?}",
                    c, *val
                )),
            },
            Record::NamedStruct(constr, args) => match constr.as_ref() {
                "Constr1" if args.len() == 2 => Ok(DummyEnum::Constr1 {
                    f1: arg_extract::<Bbool>(args, "f1")?,
                    f2: arg_extract::<String>(args, "f2")?,
                }),
                "Constr2" if args.len() == 3 => Ok(DummyEnum::Constr2 {
                    f1: arg_extract::<T>(args, "f1")?,
                    f2: arg_extract::<BigInt>(args, "f2")?,
                    f3: arg_extract::<Foo<T>>(args, "f3")?,
                }),
                "Constr3" if args.len() == 1 => Ok(DummyEnum::Constr3 {
                    f1: arg_extract::<(bool, bool)>(args, "f1")?,
                }),
                c => Result::Err(format!(
                    "unknown constructor {} of type DummyEnum in {:?}",
                    c, *val
                )),
            },
            v => Result::Err(format!("not a struct {:?}", *v)),
        }
    }
}

decl_enum_into_record!(DummyEnum<T>,Constr1["Constr1"]{f1,f2},Constr2["Constr2"]{f1,f2,f3},Constr3["Constr3"]{f1});
decl_record_mutator_enum!(DummyEnum<T>,Constr1{f1:Bbool ,f2: String},Constr2{f1: T, f2: BigInt, f3: Foo<T>},Constr3{f1: (bool, bool)});

#[test]
fn test_enum() {
    assert_eq!(
        DummyEnum::from_record(&Record::PosStruct(
            Cow::from("Constr1"),
            vec![Record::Bool(true), Record::String("foo".to_string())]
        )),
        Ok(DummyEnum::Constr1::<bool> {
            f1: true,
            f2: "foo".to_string()
        })
    );
    assert_eq!(
        DummyEnum::from_record(&Record::NamedStruct(
            Cow::from("Constr1"),
            vec![
                (Cow::from("f1"), Record::Bool(true)),
                (Cow::from("f2"), Record::String("foo".to_string()))
            ]
        )),
        Ok(DummyEnum::Constr1::<bool> {
            f1: true,
            f2: "foo".to_string()
        })
    );
    assert_eq!(
        DummyEnum::from_record(&Record::PosStruct(
            Cow::from("Constr2"),
            vec![
                Record::Int((5_i64).to_bigint().unwrap()),
                Record::Int((25_i64).to_bigint().unwrap()),
                Record::PosStruct(
                    Cow::from("Foo"),
                    vec![Record::Int((0_i64).to_bigint().unwrap())]
                )
            ]
        )),
        Ok(DummyEnum::Constr2::<u16> {
            f1: 5,
            f2: (25_i64).to_bigint().unwrap(),
            f3: Foo { f1: 0 }
        })
    );
    assert_eq!(
        DummyEnum::from_record(&Record::NamedStruct(
            Cow::from("Constr2"),
            vec![
                (Cow::from("f1"), Record::Int((5_i64).to_bigint().unwrap())),
                (Cow::from("f2"), Record::Int((25_i64).to_bigint().unwrap())),
                (
                    Cow::from("f3"),
                    Record::NamedStruct(
                        Cow::from("Foo"),
                        vec![(Cow::from("f1"), Record::Int((0_i64).to_bigint().unwrap()))]
                    )
                )
            ]
        )),
        Ok(DummyEnum::Constr2::<u16> {
            f1: 5,
            f2: (25_i64).to_bigint().unwrap(),
            f3: Foo { f1: 0 }
        })
    );

    let enm = DummyEnum::Constr2::<u16> {
        f1: 5,
        f2: (25_i64).to_bigint().unwrap(),
        f3: Foo { f1: 0 },
    };
    assert_eq!(DummyEnum::from_record(&enm.clone().into_record()), Ok(enm));
}
