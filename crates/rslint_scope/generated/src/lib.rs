#![allow(
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
    clippy::toplevel_ref_arg
)]

use num::bigint::BigInt;
use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::Deref;
use std::ptr;
use std::result;
use std::sync;

use ordered_float::*;

use differential_dataflow::collection;
use timely::communication;
use timely::dataflow::scopes;
use timely::worker;

use differential_datalog::ddval::*;
use differential_datalog::int::*;
use differential_datalog::program::*;
use differential_datalog::record;
use differential_datalog::record::FromRecord;
use differential_datalog::record::IntoRecord;
use differential_datalog::record::RelIdentifier;
use differential_datalog::record::UpdCmd;
use differential_datalog::uint::*;
use differential_datalog::DDlogConvert;
use num_traits::cast::FromPrimitive;
use num_traits::identities::One;
use once_cell::sync::Lazy;

use fnv::FnvHashMap;

pub mod api;
pub mod ovsdb_api;
pub mod update_handler;

use crate::api::updcmd2upd;
use ::types::closure;
use ::types::string_append;
use ::types::string_append_str;

use serde::ser::SerializeTuple;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;

/// A default implementation of `DDlogConvert` that just forwards calls
/// to generated functions of equal name.
#[derive(Debug)]
pub struct DDlogConverter {}

impl DDlogConvert for DDlogConverter {
    fn relid2name(relId: RelId) -> Option<&'static str> {
        relid2name(relId)
    }

    fn indexid2name(idxId: IdxId) -> Option<&'static str> {
        indexid2name(idxId)
    }

    fn updcmd2upd(upd_cmd: &UpdCmd) -> ::std::result::Result<Update<DDValue>, String> {
        updcmd2upd(upd_cmd)
    }
}

/* Wrapper around `Update<DDValue>` type that implements `Serialize` and `Deserialize`
 * traits.  It is currently only used by the distributed_ddlog crate in order to
 * serialize updates before sending them over the network and deserializing them on the
 * way back.  In other scenarios, the user either creates a `Update<DDValue>` type
 * themselves (when using the strongly typed DDlog API) or deserializes `Update<DDValue>`
 * from `Record` using `DDlogConvert::updcmd2upd()`.
 *
 * Why use a wrapper instead of implementing the traits for `Update<DDValue>` directly?
 * `Update<>` and `DDValue` types are both declared in the `differential_datalog` crate,
 * whereas the `Deserialize` implementation is program-specific and must be in one of the
 * generated crates, so we need a wrapper to avoid creating an orphan `impl`.
 *
 * Serialized representation: we currently only serialize `Insert` and `DeleteValue`
 * commands, represented in serialized form as (polarity, relid, value) tuple.  This way
 * the deserializer first reads relid and uses it to decide which value to deserialize
 * next.
 *
 * `impl Serialize` - serializes the value by forwarding `serialize` call to the `DDValue`
 * object (in fact, it is generic and could be in the `differential_datalog` crate, but we
 * keep it here to make it easier to keep it in sync with `Deserialize`).
 *
 * `impl Deserialize` - gets generated in `Compile.hs` using the macro below.  The macro
 * takes a list of `(relid, type)` and generates a match statement that uses type-specific
 * `Deserialize` for each `relid`.
 */
#[derive(Debug)]
pub struct UpdateSerializer(Update<DDValue>);

impl From<Update<DDValue>> for UpdateSerializer {
    fn from(u: Update<DDValue>) -> Self {
        UpdateSerializer(u)
    }
}
impl From<UpdateSerializer> for Update<DDValue> {
    fn from(u: UpdateSerializer) -> Self {
        u.0
    }
}

impl Serialize for UpdateSerializer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tup = serializer.serialize_tuple(3)?;
        match &self.0 {
            Update::Insert { relid, v } => {
                tup.serialize_element(&true)?;
                tup.serialize_element(relid)?;
                tup.serialize_element(v)?;
            }
            Update::DeleteValue { relid, v } => {
                tup.serialize_element(&false)?;
                tup.serialize_element(relid)?;
                tup.serialize_element(v)?;
            }
            _ => panic!("Cannot serialize InsertOrUpdate/Modify/DeleteKey update"),
        };
        tup.end()
    }
}

#[macro_export]
macro_rules! decl_update_deserializer {
    ( $n:ty, $(($rel:expr, $typ:ty)),* ) => {
        impl<'de> ::serde::Deserialize<'de> for $n {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> ::std::result::Result<Self, D::Error> {

                struct UpdateVisitor;

                impl<'de> ::serde::de::Visitor<'de> for UpdateVisitor {
                    type Value = $n;

                    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                        formatter.write_str("(polarity, relid, value) tuple")
                    }

                    fn visit_seq<A>(self, mut seq: A) -> ::std::result::Result<Self::Value, A::Error>
                    where A: ::serde::de::SeqAccess<'de> {
                        let polarity = seq.next_element::<bool>()?.ok_or_else(|| <A::Error as ::serde::de::Error>::custom("Missing polarity"))?;
                        let relid = seq.next_element::<RelId>()?.ok_or_else(|| <A::Error as ::serde::de::Error>::custom("Missing relation id"))?;
                        match relid {
                            $(
                                $rel => {
                                    let v = seq.next_element::<$typ>()?.ok_or_else(|| <A::Error as ::serde::de::Error>::custom("Missing value"))?.into_ddvalue();
                                    if polarity {
                                        Ok(UpdateSerializer(Update::Insert{relid, v}))
                                    } else {
                                        Ok(UpdateSerializer(Update::DeleteValue{relid, v}))
                                    }
                                },
                            )*
                            _ => {
                                ::std::result::Result::Err(<A::Error as ::serde::de::Error>::custom(format!("Unknown input relation id {}", relid)))
                            }
                        }
                    }
                }

                deserializer.deserialize_tuple(3, UpdateVisitor)
            }
        }
    };
}

/* FlatBuffers bindings generated by `ddlog` */
#[cfg(feature = "flatbuf")]
pub mod flatbuf;

impl TryFrom<&RelIdentifier> for Relations {
    type Error = ();

    fn try_from(rel_id: &RelIdentifier) -> ::std::result::Result<Self, ()> {
        match rel_id {
            RelIdentifier::RelName(rname) => Relations::try_from(rname.as_ref()),
            RelIdentifier::RelId(id) => Relations::try_from(*id),
        }
    }
}

decl_update_deserializer!(
    UpdateSerializer,
    (0, ::types::ChildScope),
    (1, ::types::ConstDecl),
    (2, ::types::ExprBigInt),
    (3, ::types::ExprBool),
    (4, ::types::ExprNameRef),
    (5, ::types::ExprNumber),
    (6, ::types::ExprString),
    (7, ::types::Expression),
    (8, ::types::Function),
    (9, ::types::FunctionArg),
    (10, ::types::ConstDecl),
    (11, ::types::ExprBigInt),
    (12, ::types::ExprBool),
    (13, ::types::ExprNameRef),
    (14, ::types::ExprNumber),
    (15, ::types::ExprString),
    (16, ::types::Expression),
    (17, ::types::Function),
    (18, ::types::FunctionArg),
    (19, ::types::InputScope),
    (20, ::types::LetDecl),
    (21, ::types::Return),
    (22, ::types::Statement),
    (23, ::types::VarDecl),
    (24, ::types::InputScope),
    (25, ::types::InvalidNameUse),
    (26, ::types::LetDecl),
    (27, ::types::NameInScope),
    (28, ::types::Return),
    (29, ::types::Statement),
    (30, ::types::VarDecl)
);
impl TryFrom<&str> for Relations {
    type Error = ();
    fn try_from(rname: &str) -> ::std::result::Result<Self, ()> {
        match rname {
            "ChildScope" => Ok(Relations::ChildScope),
            "ConstDecl" => Ok(Relations::ConstDecl),
            "ExprBigInt" => Ok(Relations::ExprBigInt),
            "ExprBool" => Ok(Relations::ExprBool),
            "ExprNameRef" => Ok(Relations::ExprNameRef),
            "ExprNumber" => Ok(Relations::ExprNumber),
            "ExprString" => Ok(Relations::ExprString),
            "Expression" => Ok(Relations::Expression),
            "Function" => Ok(Relations::Function),
            "FunctionArg" => Ok(Relations::FunctionArg),
            "INPUT_ConstDecl" => Ok(Relations::INPUT_ConstDecl),
            "INPUT_ExprBigInt" => Ok(Relations::INPUT_ExprBigInt),
            "INPUT_ExprBool" => Ok(Relations::INPUT_ExprBool),
            "INPUT_ExprNameRef" => Ok(Relations::INPUT_ExprNameRef),
            "INPUT_ExprNumber" => Ok(Relations::INPUT_ExprNumber),
            "INPUT_ExprString" => Ok(Relations::INPUT_ExprString),
            "INPUT_Expression" => Ok(Relations::INPUT_Expression),
            "INPUT_Function" => Ok(Relations::INPUT_Function),
            "INPUT_FunctionArg" => Ok(Relations::INPUT_FunctionArg),
            "INPUT_InputScope" => Ok(Relations::INPUT_InputScope),
            "INPUT_LetDecl" => Ok(Relations::INPUT_LetDecl),
            "INPUT_Return" => Ok(Relations::INPUT_Return),
            "INPUT_Statement" => Ok(Relations::INPUT_Statement),
            "INPUT_VarDecl" => Ok(Relations::INPUT_VarDecl),
            "InputScope" => Ok(Relations::InputScope),
            "InvalidNameUse" => Ok(Relations::InvalidNameUse),
            "LetDecl" => Ok(Relations::LetDecl),
            "NameInScope" => Ok(Relations::NameInScope),
            "Return" => Ok(Relations::Return),
            "Statement" => Ok(Relations::Statement),
            "VarDecl" => Ok(Relations::VarDecl),
            "__Null" => Ok(Relations::__Null),
            _ => Err(()),
        }
    }
}
impl Relations {
    pub fn is_output(&self) -> bool {
        match self {
            Relations::ChildScope => true,
            Relations::INPUT_ConstDecl => true,
            Relations::INPUT_ExprBigInt => true,
            Relations::INPUT_ExprBool => true,
            Relations::INPUT_ExprNameRef => true,
            Relations::INPUT_ExprNumber => true,
            Relations::INPUT_ExprString => true,
            Relations::INPUT_Expression => true,
            Relations::INPUT_Function => true,
            Relations::INPUT_FunctionArg => true,
            Relations::INPUT_InputScope => true,
            Relations::INPUT_LetDecl => true,
            Relations::INPUT_Return => true,
            Relations::INPUT_Statement => true,
            Relations::INPUT_VarDecl => true,
            Relations::InvalidNameUse => true,
            Relations::NameInScope => true,
            _ => false,
        }
    }
}
impl Relations {
    pub fn is_input(&self) -> bool {
        match self {
            Relations::ConstDecl => true,
            Relations::ExprBigInt => true,
            Relations::ExprBool => true,
            Relations::ExprNameRef => true,
            Relations::ExprNumber => true,
            Relations::ExprString => true,
            Relations::Expression => true,
            Relations::Function => true,
            Relations::FunctionArg => true,
            Relations::InputScope => true,
            Relations::LetDecl => true,
            Relations::Return => true,
            Relations::Statement => true,
            Relations::VarDecl => true,
            _ => false,
        }
    }
}
impl TryFrom<RelId> for Relations {
    type Error = ();
    fn try_from(rid: RelId) -> ::std::result::Result<Self, ()> {
        match rid {
            0 => Ok(Relations::ChildScope),
            1 => Ok(Relations::ConstDecl),
            2 => Ok(Relations::ExprBigInt),
            3 => Ok(Relations::ExprBool),
            4 => Ok(Relations::ExprNameRef),
            5 => Ok(Relations::ExprNumber),
            6 => Ok(Relations::ExprString),
            7 => Ok(Relations::Expression),
            8 => Ok(Relations::Function),
            9 => Ok(Relations::FunctionArg),
            10 => Ok(Relations::INPUT_ConstDecl),
            11 => Ok(Relations::INPUT_ExprBigInt),
            12 => Ok(Relations::INPUT_ExprBool),
            13 => Ok(Relations::INPUT_ExprNameRef),
            14 => Ok(Relations::INPUT_ExprNumber),
            15 => Ok(Relations::INPUT_ExprString),
            16 => Ok(Relations::INPUT_Expression),
            17 => Ok(Relations::INPUT_Function),
            18 => Ok(Relations::INPUT_FunctionArg),
            19 => Ok(Relations::INPUT_InputScope),
            20 => Ok(Relations::INPUT_LetDecl),
            21 => Ok(Relations::INPUT_Return),
            22 => Ok(Relations::INPUT_Statement),
            23 => Ok(Relations::INPUT_VarDecl),
            24 => Ok(Relations::InputScope),
            25 => Ok(Relations::InvalidNameUse),
            26 => Ok(Relations::LetDecl),
            27 => Ok(Relations::NameInScope),
            28 => Ok(Relations::Return),
            29 => Ok(Relations::Statement),
            30 => Ok(Relations::VarDecl),
            31 => Ok(Relations::__Null),
            _ => Err(()),
        }
    }
}
pub fn relid2name(rid: RelId) -> Option<&'static str> {
    match rid {
        0 => Some(&"ChildScope"),
        1 => Some(&"ConstDecl"),
        2 => Some(&"ExprBigInt"),
        3 => Some(&"ExprBool"),
        4 => Some(&"ExprNameRef"),
        5 => Some(&"ExprNumber"),
        6 => Some(&"ExprString"),
        7 => Some(&"Expression"),
        8 => Some(&"Function"),
        9 => Some(&"FunctionArg"),
        10 => Some(&"INPUT_ConstDecl"),
        11 => Some(&"INPUT_ExprBigInt"),
        12 => Some(&"INPUT_ExprBool"),
        13 => Some(&"INPUT_ExprNameRef"),
        14 => Some(&"INPUT_ExprNumber"),
        15 => Some(&"INPUT_ExprString"),
        16 => Some(&"INPUT_Expression"),
        17 => Some(&"INPUT_Function"),
        18 => Some(&"INPUT_FunctionArg"),
        19 => Some(&"INPUT_InputScope"),
        20 => Some(&"INPUT_LetDecl"),
        21 => Some(&"INPUT_Return"),
        22 => Some(&"INPUT_Statement"),
        23 => Some(&"INPUT_VarDecl"),
        24 => Some(&"InputScope"),
        25 => Some(&"InvalidNameUse"),
        26 => Some(&"LetDecl"),
        27 => Some(&"NameInScope"),
        28 => Some(&"Return"),
        29 => Some(&"Statement"),
        30 => Some(&"VarDecl"),
        31 => Some(&"__Null"),
        _ => None,
    }
}
pub fn relid2cname(rid: RelId) -> Option<&'static ::std::ffi::CStr> {
    RELIDMAPC.get(&rid).copied()
}
/// A map of `RelId`s to their name as an `&'static str`
pub static RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map =
            ::fnv::FnvHashMap::with_capacity_and_hasher(32, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::ConstDecl, "ConstDecl");
        map.insert(Relations::ExprBigInt, "ExprBigInt");
        map.insert(Relations::ExprBool, "ExprBool");
        map.insert(Relations::ExprNameRef, "ExprNameRef");
        map.insert(Relations::ExprNumber, "ExprNumber");
        map.insert(Relations::ExprString, "ExprString");
        map.insert(Relations::Expression, "Expression");
        map.insert(Relations::Function, "Function");
        map.insert(Relations::FunctionArg, "FunctionArg");
        map.insert(Relations::INPUT_ConstDecl, "INPUT_ConstDecl");
        map.insert(Relations::INPUT_ExprBigInt, "INPUT_ExprBigInt");
        map.insert(Relations::INPUT_ExprBool, "INPUT_ExprBool");
        map.insert(Relations::INPUT_ExprNameRef, "INPUT_ExprNameRef");
        map.insert(Relations::INPUT_ExprNumber, "INPUT_ExprNumber");
        map.insert(Relations::INPUT_ExprString, "INPUT_ExprString");
        map.insert(Relations::INPUT_Expression, "INPUT_Expression");
        map.insert(Relations::INPUT_Function, "INPUT_Function");
        map.insert(Relations::INPUT_FunctionArg, "INPUT_FunctionArg");
        map.insert(Relations::INPUT_InputScope, "INPUT_InputScope");
        map.insert(Relations::INPUT_LetDecl, "INPUT_LetDecl");
        map.insert(Relations::INPUT_Return, "INPUT_Return");
        map.insert(Relations::INPUT_Statement, "INPUT_Statement");
        map.insert(Relations::INPUT_VarDecl, "INPUT_VarDecl");
        map.insert(Relations::InputScope, "InputScope");
        map.insert(Relations::InvalidNameUse, "InvalidNameUse");
        map.insert(Relations::LetDecl, "LetDecl");
        map.insert(Relations::NameInScope, "NameInScope");
        map.insert(Relations::Return, "Return");
        map.insert(Relations::Statement, "Statement");
        map.insert(Relations::VarDecl, "VarDecl");
        map.insert(Relations::__Null, "__Null");
        map
    });
/// A map of `RelId`s to their name as an `&'static CStr`
pub static RELIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<RelId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map =
            ::fnv::FnvHashMap::with_capacity_and_hasher(32, ::fnv::FnvBuildHasher::default());
        map.insert(
            0,
            ::std::ffi::CStr::from_bytes_with_nul(b"ChildScope\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            1,
            ::std::ffi::CStr::from_bytes_with_nul(b"ConstDecl\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            2,
            ::std::ffi::CStr::from_bytes_with_nul(b"ExprBigInt\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            3,
            ::std::ffi::CStr::from_bytes_with_nul(b"ExprBool\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            4,
            ::std::ffi::CStr::from_bytes_with_nul(b"ExprNameRef\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            5,
            ::std::ffi::CStr::from_bytes_with_nul(b"ExprNumber\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            6,
            ::std::ffi::CStr::from_bytes_with_nul(b"ExprString\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            7,
            ::std::ffi::CStr::from_bytes_with_nul(b"Expression\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            8,
            ::std::ffi::CStr::from_bytes_with_nul(b"Function\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            9,
            ::std::ffi::CStr::from_bytes_with_nul(b"FunctionArg\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            10,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ConstDecl\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            11,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprBigInt\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            12,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprBool\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            13,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprNameRef\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            14,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprNumber\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            15,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprString\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            16,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Expression\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            17,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Function\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            18,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_FunctionArg\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            19,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_InputScope\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            20,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_LetDecl\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            21,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Return\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            22,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Statement\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            23,
            ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_VarDecl\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            24,
            ::std::ffi::CStr::from_bytes_with_nul(b"InputScope\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            25,
            ::std::ffi::CStr::from_bytes_with_nul(b"InvalidNameUse\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            26,
            ::std::ffi::CStr::from_bytes_with_nul(b"LetDecl\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            27,
            ::std::ffi::CStr::from_bytes_with_nul(b"NameInScope\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            28,
            ::std::ffi::CStr::from_bytes_with_nul(b"Return\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            29,
            ::std::ffi::CStr::from_bytes_with_nul(b"Statement\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            30,
            ::std::ffi::CStr::from_bytes_with_nul(b"VarDecl\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map.insert(
            31,
            ::std::ffi::CStr::from_bytes_with_nul(b"__Null\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map
    });
/// A map of input `Relations`s to their name as an `&'static str`
pub static INPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map =
            ::fnv::FnvHashMap::with_capacity_and_hasher(14, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ConstDecl, "ConstDecl");
        map.insert(Relations::ExprBigInt, "ExprBigInt");
        map.insert(Relations::ExprBool, "ExprBool");
        map.insert(Relations::ExprNameRef, "ExprNameRef");
        map.insert(Relations::ExprNumber, "ExprNumber");
        map.insert(Relations::ExprString, "ExprString");
        map.insert(Relations::Expression, "Expression");
        map.insert(Relations::Function, "Function");
        map.insert(Relations::FunctionArg, "FunctionArg");
        map.insert(Relations::InputScope, "InputScope");
        map.insert(Relations::LetDecl, "LetDecl");
        map.insert(Relations::Return, "Return");
        map.insert(Relations::Statement, "Statement");
        map.insert(Relations::VarDecl, "VarDecl");
        map
    });
/// A map of output `Relations`s to their name as an `&'static str`
pub static OUTPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map =
            ::fnv::FnvHashMap::with_capacity_and_hasher(17, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::INPUT_ConstDecl, "INPUT_ConstDecl");
        map.insert(Relations::INPUT_ExprBigInt, "INPUT_ExprBigInt");
        map.insert(Relations::INPUT_ExprBool, "INPUT_ExprBool");
        map.insert(Relations::INPUT_ExprNameRef, "INPUT_ExprNameRef");
        map.insert(Relations::INPUT_ExprNumber, "INPUT_ExprNumber");
        map.insert(Relations::INPUT_ExprString, "INPUT_ExprString");
        map.insert(Relations::INPUT_Expression, "INPUT_Expression");
        map.insert(Relations::INPUT_Function, "INPUT_Function");
        map.insert(Relations::INPUT_FunctionArg, "INPUT_FunctionArg");
        map.insert(Relations::INPUT_InputScope, "INPUT_InputScope");
        map.insert(Relations::INPUT_LetDecl, "INPUT_LetDecl");
        map.insert(Relations::INPUT_Return, "INPUT_Return");
        map.insert(Relations::INPUT_Statement, "INPUT_Statement");
        map.insert(Relations::INPUT_VarDecl, "INPUT_VarDecl");
        map.insert(Relations::InvalidNameUse, "InvalidNameUse");
        map.insert(Relations::NameInScope, "NameInScope");
        map
    });
impl TryFrom<&str> for Indexes {
    type Error = ();
    fn try_from(iname: &str) -> ::std::result::Result<Self, ()> {
        match iname {
            "__Null_by_none" => Ok(Indexes::__Null_by_none),
            _ => Err(()),
        }
    }
}
impl TryFrom<IdxId> for Indexes {
    type Error = ();
    fn try_from(iid: IdxId) -> ::core::result::Result<Self, ()> {
        match iid {
            0 => Ok(Indexes::__Null_by_none),
            _ => Err(()),
        }
    }
}
pub fn indexid2name(iid: IdxId) -> Option<&'static str> {
    match iid {
        0 => Some(&"__Null_by_none"),
        _ => None,
    }
}
pub fn indexid2cname(iid: IdxId) -> Option<&'static ::std::ffi::CStr> {
    IDXIDMAPC.get(&iid).copied()
}
/// A map of `Indexes` to their name as an `&'static str`
pub static IDXIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Indexes, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map =
            ::fnv::FnvHashMap::with_capacity_and_hasher(1, ::fnv::FnvBuildHasher::default());
        map.insert(Indexes::__Null_by_none, "__Null_by_none");
        map
    });
/// A map of `IdxId`s to their name as an `&'static CStr`
pub static IDXIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<IdxId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map =
            ::fnv::FnvHashMap::with_capacity_and_hasher(1, ::fnv::FnvBuildHasher::default());
        map.insert(
            0,
            ::std::ffi::CStr::from_bytes_with_nul(b"__Null_by_none\0")
                .expect("Unreachable: A null byte was specifically inserted"),
        );
        map
    });
pub fn relval_from_record(
    rel: Relations,
    _rec: &differential_datalog::record::Record,
) -> ::std::result::Result<DDValue, String> {
    match rel {
        Relations::ChildScope => Ok(<::types::ChildScope>::from_record(_rec)?.into_ddvalue()),
        Relations::ConstDecl => Ok(<::types::ConstDecl>::from_record(_rec)?.into_ddvalue()),
        Relations::ExprBigInt => Ok(<::types::ExprBigInt>::from_record(_rec)?.into_ddvalue()),
        Relations::ExprBool => Ok(<::types::ExprBool>::from_record(_rec)?.into_ddvalue()),
        Relations::ExprNameRef => Ok(<::types::ExprNameRef>::from_record(_rec)?.into_ddvalue()),
        Relations::ExprNumber => Ok(<::types::ExprNumber>::from_record(_rec)?.into_ddvalue()),
        Relations::ExprString => Ok(<::types::ExprString>::from_record(_rec)?.into_ddvalue()),
        Relations::Expression => Ok(<::types::Expression>::from_record(_rec)?.into_ddvalue()),
        Relations::Function => Ok(<::types::Function>::from_record(_rec)?.into_ddvalue()),
        Relations::FunctionArg => Ok(<::types::FunctionArg>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_ConstDecl => Ok(<::types::ConstDecl>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_ExprBigInt => Ok(<::types::ExprBigInt>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_ExprBool => Ok(<::types::ExprBool>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_ExprNameRef => {
            Ok(<::types::ExprNameRef>::from_record(_rec)?.into_ddvalue())
        }
        Relations::INPUT_ExprNumber => Ok(<::types::ExprNumber>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_ExprString => Ok(<::types::ExprString>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_Expression => Ok(<::types::Expression>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_Function => Ok(<::types::Function>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_FunctionArg => {
            Ok(<::types::FunctionArg>::from_record(_rec)?.into_ddvalue())
        }
        Relations::INPUT_InputScope => Ok(<::types::InputScope>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_LetDecl => Ok(<::types::LetDecl>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_Return => Ok(<::types::Return>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_Statement => Ok(<::types::Statement>::from_record(_rec)?.into_ddvalue()),
        Relations::INPUT_VarDecl => Ok(<::types::VarDecl>::from_record(_rec)?.into_ddvalue()),
        Relations::InputScope => Ok(<::types::InputScope>::from_record(_rec)?.into_ddvalue()),
        Relations::InvalidNameUse => {
            Ok(<::types::InvalidNameUse>::from_record(_rec)?.into_ddvalue())
        }
        Relations::LetDecl => Ok(<::types::LetDecl>::from_record(_rec)?.into_ddvalue()),
        Relations::NameInScope => Ok(<::types::NameInScope>::from_record(_rec)?.into_ddvalue()),
        Relations::Return => Ok(<::types::Return>::from_record(_rec)?.into_ddvalue()),
        Relations::Statement => Ok(<::types::Statement>::from_record(_rec)?.into_ddvalue()),
        Relations::VarDecl => Ok(<::types::VarDecl>::from_record(_rec)?.into_ddvalue()),
        Relations::__Null => Ok(<()>::from_record(_rec)?.into_ddvalue()),
    }
}
pub fn relkey_from_record(
    rel: Relations,
    _rec: &differential_datalog::record::Record,
) -> ::std::result::Result<DDValue, String> {
    match rel {
        _ => Err(format!("relation {:?} does not have a primary key", rel)),
    }
}
pub fn idxkey_from_record(
    idx: Indexes,
    _rec: &differential_datalog::record::Record,
) -> ::std::result::Result<DDValue, String> {
    match idx {
        Indexes::__Null_by_none => Ok(<()>::from_record(_rec)?.into_ddvalue()),
    }
}
pub fn indexes2arrid(idx: Indexes) -> ArrId {
    match idx {
        Indexes::__Null_by_none => (31, 0),
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Relations {
    ChildScope = 0,
    ConstDecl = 1,
    ExprBigInt = 2,
    ExprBool = 3,
    ExprNameRef = 4,
    ExprNumber = 5,
    ExprString = 6,
    Expression = 7,
    Function = 8,
    FunctionArg = 9,
    INPUT_ConstDecl = 10,
    INPUT_ExprBigInt = 11,
    INPUT_ExprBool = 12,
    INPUT_ExprNameRef = 13,
    INPUT_ExprNumber = 14,
    INPUT_ExprString = 15,
    INPUT_Expression = 16,
    INPUT_Function = 17,
    INPUT_FunctionArg = 18,
    INPUT_InputScope = 19,
    INPUT_LetDecl = 20,
    INPUT_Return = 21,
    INPUT_Statement = 22,
    INPUT_VarDecl = 23,
    InputScope = 24,
    InvalidNameUse = 25,
    LetDecl = 26,
    NameInScope = 27,
    Return = 28,
    Statement = 29,
    VarDecl = 30,
    __Null = 31,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Indexes {
    __Null_by_none = 0,
}
pub fn prog(__update_cb: Box<dyn CBFn>) -> Program {
    let ConstDecl = Relation {
        name: "ConstDecl".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::ConstDecl as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_ConstDecl = Relation {
        name: "INPUT_ConstDecl".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_ConstDecl as RelId,
        rules: vec![
            /* INPUT_ConstDecl[x] :- ConstDecl[(x: ConstDecl)]. */
            Rule::CollectionRule {
                description: "INPUT_ConstDecl[x] :- ConstDecl[(x: ConstDecl)].".to_string(),
                rel: Relations::ConstDecl as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_ConstDecl[x] :- ConstDecl[(x: ConstDecl)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::ConstDecl>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let ExprBigInt = Relation {
        name: "ExprBigInt".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::ExprBigInt as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_ExprBigInt = Relation {
        name: "INPUT_ExprBigInt".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_ExprBigInt as RelId,
        rules: vec![
            /* INPUT_ExprBigInt[x] :- ExprBigInt[(x: ExprBigInt)]. */
            Rule::CollectionRule {
                description: "INPUT_ExprBigInt[x] :- ExprBigInt[(x: ExprBigInt)].".to_string(),
                rel: Relations::ExprBigInt as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_ExprBigInt[x] :- ExprBigInt[(x: ExprBigInt)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::ExprBigInt>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let ExprBool = Relation {
        name: "ExprBool".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::ExprBool as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_ExprBool = Relation {
        name: "INPUT_ExprBool".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_ExprBool as RelId,
        rules: vec![
            /* INPUT_ExprBool[x] :- ExprBool[(x: ExprBool)]. */
            Rule::CollectionRule {
                description: "INPUT_ExprBool[x] :- ExprBool[(x: ExprBool)].".to_string(),
                rel: Relations::ExprBool as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_ExprBool[x] :- ExprBool[(x: ExprBool)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::ExprBool>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let ExprNameRef = Relation {
                          name:         "ExprNameRef".to_string(),
                          input:        true,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::ExprNameRef as RelId,
                          rules:        vec![
                              ],
                          arrangements: vec![
                              Arrangement::Map{
                                 name: r###"(ExprNameRef{.id=(_0: bit<32>), .value=(_: internment::Intern<string>)}: ExprNameRef) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::ExprNameRef>::from_ddvalue(__v) } {
                                          ::types::ExprNameRef{id: ref _0, value: _} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              }],
                          change_cb:    None
                      };
    let INPUT_ExprNameRef = Relation {
        name: "INPUT_ExprNameRef".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_ExprNameRef as RelId,
        rules: vec![
            /* INPUT_ExprNameRef[x] :- ExprNameRef[(x: ExprNameRef)]. */
            Rule::CollectionRule {
                description: "INPUT_ExprNameRef[x] :- ExprNameRef[(x: ExprNameRef)].".to_string(),
                rel: Relations::ExprNameRef as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_ExprNameRef[x] :- ExprNameRef[(x: ExprNameRef)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::ExprNameRef>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let ExprNumber = Relation {
        name: "ExprNumber".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::ExprNumber as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_ExprNumber = Relation {
        name: "INPUT_ExprNumber".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_ExprNumber as RelId,
        rules: vec![
            /* INPUT_ExprNumber[x] :- ExprNumber[(x: ExprNumber)]. */
            Rule::CollectionRule {
                description: "INPUT_ExprNumber[x] :- ExprNumber[(x: ExprNumber)].".to_string(),
                rel: Relations::ExprNumber as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_ExprNumber[x] :- ExprNumber[(x: ExprNumber)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::ExprNumber>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let ExprString = Relation {
        name: "ExprString".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::ExprString as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_ExprString = Relation {
        name: "INPUT_ExprString".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_ExprString as RelId,
        rules: vec![
            /* INPUT_ExprString[x] :- ExprString[(x: ExprString)]. */
            Rule::CollectionRule {
                description: "INPUT_ExprString[x] :- ExprString[(x: ExprString)].".to_string(),
                rel: Relations::ExprString as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_ExprString[x] :- ExprString[(x: ExprString)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::ExprString>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let Expression = Relation {
                         name:         "Expression".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::Expression as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"(Expression{.id=(_0: bit<32>), .kind=(NameRef{}: ExprKind), .scope=(_: bit<32>), .span=(_: Span)}: Expression) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::Expression>::from_ddvalue(__v) } {
                                         ::types::Expression{id: ref _0, kind: ::types::ExprKind::NameRef{}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let INPUT_Expression = Relation {
        name: "INPUT_Expression".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_Expression as RelId,
        rules: vec![
            /* INPUT_Expression[x] :- Expression[(x: Expression)]. */
            Rule::CollectionRule {
                description: "INPUT_Expression[x] :- Expression[(x: Expression)].".to_string(),
                rel: Relations::Expression as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_Expression[x] :- Expression[(x: Expression)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::Expression>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let Function = Relation {
                       name:         "Function".to_string(),
                       input:        true,
                       distinct:     false,
                       caching_mode: CachingMode::Set,
                       key_func:     None,
                       id:           Relations::Function as RelId,
                       rules:        vec![
                           ],
                       arrangements: vec![
                           Arrangement::Map{
                              name: r###"(Function{.id=(_0: bit<32>), .name=(_: ddlog_std::Option<Name>), .scope=(_: bit<32>)}: Function) /*join*/"###.to_string(),
                               afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                               {
                                   let __cloned = __v.clone();
                                   match unsafe {< ::types::Function>::from_ddvalue(__v) } {
                                       ::types::Function{id: ref _0, name: _, scope: _} => Some(((*_0).clone()).into_ddvalue()),
                                       _ => None
                                   }.map(|x|(x,__cloned))
                               }
                               __f},
                               queryable: false
                           }],
                       change_cb:    None
                   };
    let INPUT_Function = Relation {
        name: "INPUT_Function".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_Function as RelId,
        rules: vec![
            /* INPUT_Function[x] :- Function[(x: Function)]. */
            Rule::CollectionRule {
                description: "INPUT_Function[x] :- Function[(x: Function)].".to_string(),
                rel: Relations::Function as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_Function[x] :- Function[(x: Function)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::Function>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let FunctionArg = Relation {
        name: "FunctionArg".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::FunctionArg as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_FunctionArg = Relation {
        name: "INPUT_FunctionArg".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_FunctionArg as RelId,
        rules: vec![
            /* INPUT_FunctionArg[x] :- FunctionArg[(x: FunctionArg)]. */
            Rule::CollectionRule {
                description: "INPUT_FunctionArg[x] :- FunctionArg[(x: FunctionArg)].".to_string(),
                rel: Relations::FunctionArg as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_FunctionArg[x] :- FunctionArg[(x: FunctionArg)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::FunctionArg>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let InputScope = Relation {
                         name:         "InputScope".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::InputScope as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"(InputScope{.parent=(_: bit<32>), .child=(_0: bit<32>)}: InputScope) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::InputScope>::from_ddvalue(__v) } {
                                         ::types::InputScope{parent: _, child: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             },
                             Arrangement::Map{
                                name: r###"(InputScope{.parent=(_0: bit<32>), .child=(_: bit<32>)}: InputScope) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::InputScope>::from_ddvalue(__v) } {
                                         ::types::InputScope{parent: ref _0, child: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let ChildScope = Relation {
                         name:         "ChildScope".to_string(),
                         input:        false,
                         distinct:     true,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::ChildScope as RelId,
                         rules:        vec![
                             /* ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: bit<32>), .child=(child: bit<32>)}: InputScope)]. */
                             Rule::CollectionRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: bit<32>), .child=(child: bit<32>)}: InputScope)].".to_string(),
                                 rel: Relations::InputScope as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: bit<32>), .child=(child: bit<32>)}: InputScope)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref parent, ref child) = match *unsafe {<::types::InputScope>::from_ddvalue_ref(&__v) } {
                                                         ::types::InputScope{parent: ref parent, child: ref child} => ((*parent).clone(), (*child).clone()),
                                                         _ => return None
                                                     };
                                                     Some(((::types::ChildScope{parent: (*parent).clone(), child: (*child).clone()})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: bit<32>), .child=(interum: bit<32>)}: InputScope)], InputScope[(InputScope{.parent=(interum: bit<32>), .child=(child: bit<32>)}: InputScope)]. */
                             Rule::ArrangementRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: bit<32>), .child=(interum: bit<32>)}: InputScope)], InputScope[(InputScope{.parent=(interum: bit<32>), .child=(child: bit<32>)}: InputScope)].".to_string(),
                                 arr: ( Relations::InputScope as RelId, 0),
                                 xform: XFormArrangement::Join{
                                            description: "InputScope[(InputScope{.parent=(parent: bit<32>), .child=(interum: bit<32>)}: InputScope)], InputScope[(InputScope{.parent=(interum: bit<32>), .child=(child: bit<32>)}: InputScope)]".to_string(),
                                            ffun: None,
                                            arrangement: (Relations::InputScope as RelId,1),
                                            jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                            {
                                                let (ref parent, ref interum) = match *unsafe {<::types::InputScope>::from_ddvalue_ref(__v1) } {
                                                    ::types::InputScope{parent: ref parent, child: ref interum} => ((*parent).clone(), (*interum).clone()),
                                                    _ => return None
                                                };
                                                let ref child = match *unsafe {<::types::InputScope>::from_ddvalue_ref(__v2) } {
                                                    ::types::InputScope{parent: _, child: ref child} => (*child).clone(),
                                                    _ => return None
                                                };
                                                Some(((::types::ChildScope{parent: (*parent).clone(), child: (*child).clone()})).into_ddvalue())
                                            }
                                            __f},
                                            next: Box::new(None)
                                        }
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"(ChildScope{.parent=(_0: bit<32>), .child=(_: bit<32>)}: ChildScope) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ChildScope>::from_ddvalue(__v) } {
                                         ::types::ChildScope{parent: ref _0, child: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                     };
    let INPUT_InputScope = Relation {
        name: "INPUT_InputScope".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_InputScope as RelId,
        rules: vec![
            /* INPUT_InputScope[x] :- InputScope[(x: InputScope)]. */
            Rule::CollectionRule {
                description: "INPUT_InputScope[x] :- InputScope[(x: InputScope)].".to_string(),
                rel: Relations::InputScope as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_InputScope[x] :- InputScope[(x: InputScope)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::InputScope>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let LetDecl = Relation {
        name: "LetDecl".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::LetDecl as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_LetDecl = Relation {
        name: "INPUT_LetDecl".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_LetDecl as RelId,
        rules: vec![
            /* INPUT_LetDecl[x] :- LetDecl[(x: LetDecl)]. */
            Rule::CollectionRule {
                description: "INPUT_LetDecl[x] :- LetDecl[(x: LetDecl)].".to_string(),
                rel: Relations::LetDecl as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_LetDecl[x] :- LetDecl[(x: LetDecl)].".to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x = match *unsafe { <::types::LetDecl>::from_ddvalue_ref(&__v) }
                            {
                                ref x => (*x).clone(),
                                _ => return None,
                            };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let Return = Relation {
        name: "Return".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::Return as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let INPUT_Return = Relation {
        name: "INPUT_Return".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_Return as RelId,
        rules: vec![
            /* INPUT_Return[x] :- Return[(x: Return)]. */
            Rule::CollectionRule {
                description: "INPUT_Return[x] :- Return[(x: Return)].".to_string(),
                rel: Relations::Return as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_Return[x] :- Return[(x: Return)].".to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x = match *unsafe { <::types::Return>::from_ddvalue_ref(&__v) }
                            {
                                ref x => (*x).clone(),
                                _ => return None,
                            };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let Statement = Relation {
                        name:         "Statement".to_string(),
                        input:        true,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::Statement as RelId,
                        rules:        vec![
                            ],
                        arrangements: vec![
                            Arrangement::Map{
                               name: r###"(Statement{.id=(_0: bit<32>), .kind=(_: StmtKind), .scope=(_: bit<32>), .span=(_: Span)}: Statement) /*join*/"###.to_string(),
                                afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                {
                                    let __cloned = __v.clone();
                                    match unsafe {< ::types::Statement>::from_ddvalue(__v) } {
                                        ::types::Statement{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                        _ => None
                                    }.map(|x|(x,__cloned))
                                }
                                __f},
                                queryable: false
                            },
                            Arrangement::Set{
                                name: r###"(Statement{.id=(_0: bit<32>), .kind=(_: StmtKind), .scope=(_1: bit<32>), .span=(_: Span)}: Statement) /*semijoin*/"###.to_string(),
                                fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                {
                                    match unsafe {< ::types::Statement>::from_ddvalue(__v) } {
                                        ::types::Statement{id: ref _0, kind: _, scope: ref _1, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                        _ => None
                                    }
                                }
                                __f},
                                distinct: false
                            }],
                        change_cb:    None
                    };
    let INPUT_Statement = Relation {
        name: "INPUT_Statement".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_Statement as RelId,
        rules: vec![
            /* INPUT_Statement[x] :- Statement[(x: Statement)]. */
            Rule::CollectionRule {
                description: "INPUT_Statement[x] :- Statement[(x: Statement)].".to_string(),
                rel: Relations::Statement as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_Statement[x] :- Statement[(x: Statement)]."
                        .to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x =
                                match *unsafe { <::types::Statement>::from_ddvalue_ref(&__v) } {
                                    ref x => (*x).clone(),
                                    _ => return None,
                                };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let VarDecl = Relation {
        name: "VarDecl".to_string(),
        input: true,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::VarDecl as RelId,
        rules: vec![],
        arrangements: vec![],
        change_cb: None,
    };
    let NameInScope = Relation {
                          name:         "NameInScope".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::NameInScope as RelId,
                          rules:        vec![
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Right{.r=stmt}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- LetDecl[(LetDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Right{.r=stmt}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- LetDecl[(LetDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)].".to_string(),
                                  rel: Relations::LetDecl as RelId,
                                  xform: Some(XFormCollection::Arrange {
                                                  description: "arrange LetDecl[(LetDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)] by (stmt)" .to_string(),
                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                  {
                                                      let (ref stmt, ref pat) = match *unsafe {<::types::LetDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::LetDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let ref name: ::types::internment::Intern<String> = match (*::types::internment::ival(pat)).clone() {
                                                          ::types::Pattern{name: name} => name,
                                                          _ => return None
                                                      };
                                                      Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue()))
                                                  }
                                                  __f},
                                                  next: Box::new(XFormArrangement::Join{
                                                                     description: "LetDecl[(LetDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)]".to_string(),
                                                                     ffun: None,
                                                                     arrangement: (Relations::Statement as RelId,0),
                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                     {
                                                                         let ::types::ddlog_std::tuple2(ref stmt, ref name) = *unsafe {<::types::ddlog_std::tuple2<u32, ::types::internment::Intern<String>>>::from_ddvalue_ref( __v1 ) };
                                                                         let ref scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                                             ::types::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                             _ => return None
                                                                         };
                                                                         Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::ddlog_std::Either::Right{r: (*stmt).clone()})})).into_ddvalue())
                                                                     }
                                                                     __f},
                                                                     next: Box::new(None)
                                                                 })
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Right{.r=stmt}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- ConstDecl[(ConstDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Right{.r=stmt}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- ConstDecl[(ConstDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)].".to_string(),
                                  rel: Relations::ConstDecl as RelId,
                                  xform: Some(XFormCollection::Arrange {
                                                  description: "arrange ConstDecl[(ConstDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)] by (stmt)" .to_string(),
                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                  {
                                                      let (ref stmt, ref pat) = match *unsafe {<::types::ConstDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::ConstDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let ref name: ::types::internment::Intern<String> = match (*::types::internment::ival(pat)).clone() {
                                                          ::types::Pattern{name: name} => name,
                                                          _ => return None
                                                      };
                                                      Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*stmt).clone(), (*name).clone())).into_ddvalue()))
                                                  }
                                                  __f},
                                                  next: Box::new(XFormArrangement::Join{
                                                                     description: "ConstDecl[(ConstDecl{.stmt_id=(stmt: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)]".to_string(),
                                                                     ffun: None,
                                                                     arrangement: (Relations::Statement as RelId,0),
                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                     {
                                                                         let ::types::ddlog_std::tuple2(ref stmt, ref name) = *unsafe {<::types::ddlog_std::tuple2<u32, ::types::internment::Intern<String>>>::from_ddvalue_ref( __v1 ) };
                                                                         let ref scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                                             ::types::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                             _ => return None
                                                                         };
                                                                         Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::ddlog_std::Either::Right{r: (*stmt).clone()})})).into_ddvalue())
                                                                     }
                                                                     __f},
                                                                     next: Box::new(None)
                                                                 })
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Right{.r=stmt}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- VarDecl[(VarDecl{.stmt_id=(stmt: bit<32>), .effective_scope=(scope: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Right{.r=stmt}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- VarDecl[(VarDecl{.stmt_id=(stmt: bit<32>), .effective_scope=(scope: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)].".to_string(),
                                  rel: Relations::VarDecl as RelId,
                                  xform: Some(XFormCollection::Arrange {
                                                  description: "arrange VarDecl[(VarDecl{.stmt_id=(stmt: bit<32>), .effective_scope=(scope: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)] by (stmt, scope)" .to_string(),
                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                  {
                                                      let (ref stmt, ref scope, ref pat) = match *unsafe {<::types::VarDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::VarDecl{stmt_id: ref stmt, effective_scope: ref scope, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*scope).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let ref name: ::types::internment::Intern<String> = match (*::types::internment::ival(pat)).clone() {
                                                          ::types::Pattern{name: name} => name,
                                                          _ => return None
                                                      };
                                                      Some(((::types::ddlog_std::tuple2((*stmt).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*stmt).clone(), (*scope).clone(), (*name).clone())).into_ddvalue()))
                                                  }
                                                  __f},
                                                  next: Box::new(XFormArrangement::Semijoin{
                                                                     description: "VarDecl[(VarDecl{.stmt_id=(stmt: bit<32>), .effective_scope=(scope: bit<32>), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Statement[(Statement{.id=(stmt: bit<32>), .kind=(_: StmtKind), .scope=(scope: bit<32>), .span=(_: Span)}: Statement)]".to_string(),
                                                                     ffun: None,
                                                                     arrangement: (Relations::Statement as RelId,1),
                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref stmt, ref scope, ref name) = *unsafe {<::types::ddlog_std::tuple3<u32, u32, ::types::internment::Intern<String>>>::from_ddvalue_ref( __v1 ) };
                                                                         Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::ddlog_std::Either::Right{r: (*stmt).clone()})})).into_ddvalue())
                                                                     }
                                                                     __f},
                                                                     next: Box::new(None)
                                                                 })
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Left{.l=func}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- FunctionArg[(FunctionArg{.parent_func=(func: bit<32>), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Function[(Function{.id=(func: bit<32>), .name=(_: ddlog_std::Option<Name>), .scope=(scope: bit<32>)}: Function)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(ddlog_std::Left{.l=func}: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)] :- FunctionArg[(FunctionArg{.parent_func=(func: bit<32>), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Function[(Function{.id=(func: bit<32>), .name=(_: ddlog_std::Option<Name>), .scope=(scope: bit<32>)}: Function)].".to_string(),
                                  rel: Relations::FunctionArg as RelId,
                                  xform: Some(XFormCollection::Arrange {
                                                  description: "arrange FunctionArg[(FunctionArg{.parent_func=(func: bit<32>), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)] by (func)" .to_string(),
                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                  {
                                                      let (ref func, ref pat) = match *unsafe {<::types::FunctionArg>::from_ddvalue_ref(&__v) } {
                                                          ::types::FunctionArg{parent_func: ref func, pattern: ref pat} => ((*func).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let ref name: ::types::internment::Intern<String> = match (*::types::internment::ival(pat)).clone() {
                                                          ::types::Pattern{name: name} => name,
                                                          _ => return None
                                                      };
                                                      Some((((*func).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*func).clone(), (*name).clone())).into_ddvalue()))
                                                  }
                                                  __f},
                                                  next: Box::new(XFormArrangement::Join{
                                                                     description: "FunctionArg[(FunctionArg{.parent_func=(func: bit<32>), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], ((SinglePattern{.name=(var name: internment::Intern<string>)}: Pattern) = ((internment::ival: function(internment::Intern<Pattern>):Pattern)(pat))), Function[(Function{.id=(func: bit<32>), .name=(_: ddlog_std::Option<Name>), .scope=(scope: bit<32>)}: Function)]".to_string(),
                                                                     ffun: None,
                                                                     arrangement: (Relations::Function as RelId,0),
                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                     {
                                                                         let ::types::ddlog_std::tuple2(ref func, ref name) = *unsafe {<::types::ddlog_std::tuple2<u32, ::types::internment::Intern<String>>>::from_ddvalue_ref( __v1 ) };
                                                                         let ref scope = match *unsafe {<::types::Function>::from_ddvalue_ref(__v2) } {
                                                                             ::types::Function{id: _, name: _, scope: ref scope} => (*scope).clone(),
                                                                             _ => return None
                                                                         };
                                                                         Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::ddlog_std::Either::Left{l: (*func).clone()})})).into_ddvalue())
                                                                     }
                                                                     __f},
                                                                     next: Box::new(None)
                                                                 })
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=declared_in}: NameInScope)] :- NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: bit<32>), .declared_in=(declared_in: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: bit<32>), .child=(scope: bit<32>)}: ChildScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=declared_in}: NameInScope)] :- NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: bit<32>), .declared_in=(declared_in: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: bit<32>), .child=(scope: bit<32>)}: ChildScope)].".to_string(),
                                  arr: ( Relations::NameInScope as RelId, 1),
                                  xform: XFormArrangement::Join{
                                             description: "NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: bit<32>), .declared_in=(declared_in: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: bit<32>), .child=(scope: bit<32>)}: ChildScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::ChildScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref interum, ref declared_in) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v1) } {
                                                     ::types::NameInScope{name: ref name, scope: ref interum, declared_in: ref declared_in} => ((*name).clone(), (*interum).clone(), (*declared_in).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::ChildScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::ChildScope{parent: _, child: ref scope} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (*declared_in).clone()})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              }],
                          arrangements: vec![
                              Arrangement::Set{
                                  name: r###"(NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: bit<32>), .declared_in=(_: ddlog_std::Either<StmtId,FuncId>)}: NameInScope) /*antijoin*/"###.to_string(),
                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                  {
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: ref _0, scope: ref _1, declared_in: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }
                                  }
                                  __f},
                                  distinct: true
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=(_: internment::Intern<string>), .scope=(_0: bit<32>), .declared_in=(_: ddlog_std::Either<StmtId,FuncId>)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: _, scope: ref _0, declared_in: _} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              }],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let InvalidNameUse = Relation {
                             name:         "InvalidNameUse".to_string(),
                             input:        false,
                             distinct:     true,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::InvalidNameUse as RelId,
                             rules:        vec![
                                 /* InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- ExprNameRef[(ExprNameRef{.id=(expr: bit<32>), .value=(name: internment::Intern<string>)}: ExprNameRef)], Expression[(Expression{.id=(expr: bit<32>), .kind=(NameRef{}: ExprKind), .scope=(scope: bit<32>), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: bit<32>), .declared_in=(_: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)]. */
                                 Rule::ArrangementRule {
                                     description: "InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- ExprNameRef[(ExprNameRef{.id=(expr: bit<32>), .value=(name: internment::Intern<string>)}: ExprNameRef)], Expression[(Expression{.id=(expr: bit<32>), .kind=(NameRef{}: ExprKind), .scope=(scope: bit<32>), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: bit<32>), .declared_in=(_: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)].".to_string(),
                                     arr: ( Relations::ExprNameRef as RelId, 0),
                                     xform: XFormArrangement::Join{
                                                description: "ExprNameRef[(ExprNameRef{.id=(expr: bit<32>), .value=(name: internment::Intern<string>)}: ExprNameRef)], Expression[(Expression{.id=(expr: bit<32>), .kind=(NameRef{}: ExprKind), .scope=(scope: bit<32>), .span=(span: Span)}: Expression)]".to_string(),
                                                ffun: None,
                                                arrangement: (Relations::Expression as RelId,0),
                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                {
                                                    let (ref expr, ref name) = match *unsafe {<::types::ExprNameRef>::from_ddvalue_ref(__v1) } {
                                                        ::types::ExprNameRef{id: ref expr, value: ref name} => ((*expr).clone(), (*name).clone()),
                                                        _ => return None
                                                    };
                                                    let (ref scope, ref span) = match *unsafe {<::types::Expression>::from_ddvalue_ref(__v2) } {
                                                        ::types::Expression{id: _, kind: ::types::ExprKind::NameRef{}, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                        _ => return None
                                                    };
                                                    Some((::types::ddlog_std::tuple3((*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(Some(XFormCollection::Arrange {
                                                                        description: "arrange ExprNameRef[(ExprNameRef{.id=(expr: bit<32>), .value=(name: internment::Intern<string>)}: ExprNameRef)], Expression[(Expression{.id=(expr: bit<32>), .kind=(NameRef{}: ExprKind), .scope=(scope: bit<32>), .span=(span: Span)}: Expression)] by (name, scope)" .to_string(),
                                                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                        {
                                                                            let ::types::ddlog_std::tuple3(ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, u32, ::types::Span>>::from_ddvalue_ref( &__v ) };
                                                                            Some(((::types::ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                        }
                                                                        __f},
                                                                        next: Box::new(XFormArrangement::Antijoin {
                                                                                           description: "ExprNameRef[(ExprNameRef{.id=(expr: bit<32>), .value=(name: internment::Intern<string>)}: ExprNameRef)], Expression[(Expression{.id=(expr: bit<32>), .kind=(NameRef{}: ExprKind), .scope=(scope: bit<32>), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: bit<32>), .declared_in=(_: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)]".to_string(),
                                                                                           ffun: None,
                                                                                           arrangement: (Relations::NameInScope as RelId,0),
                                                                                           next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                   description: "head of InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- ExprNameRef[(ExprNameRef{.id=(expr: bit<32>), .value=(name: internment::Intern<string>)}: ExprNameRef)], Expression[(Expression{.id=(expr: bit<32>), .kind=(NameRef{}: ExprKind), .scope=(scope: bit<32>), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: bit<32>), .declared_in=(_: ddlog_std::Either<StmtId,FuncId>)}: NameInScope)]." .to_string(),
                                                                                                                   fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                   {
                                                                                                                       let ::types::ddlog_std::tuple3(ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, u32, ::types::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                       Some(((::types::InvalidNameUse{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone()})).into_ddvalue())
                                                                                                                   }
                                                                                                                   __f},
                                                                                                                   next: Box::new(None)
                                                                                                               }))
                                                                                       })
                                                                    }))
                                            }
                                 }],
                             arrangements: vec![
                                 ],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let INPUT_VarDecl = Relation {
        name: "INPUT_VarDecl".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::INPUT_VarDecl as RelId,
        rules: vec![
            /* INPUT_VarDecl[x] :- VarDecl[(x: VarDecl)]. */
            Rule::CollectionRule {
                description: "INPUT_VarDecl[x] :- VarDecl[(x: VarDecl)].".to_string(),
                rel: Relations::VarDecl as RelId,
                xform: Some(XFormCollection::FilterMap {
                    description: "head of INPUT_VarDecl[x] :- VarDecl[(x: VarDecl)].".to_string(),
                    fmfun: &{
                        fn __f(__v: DDValue) -> Option<DDValue> {
                            let ref x = match *unsafe { <::types::VarDecl>::from_ddvalue_ref(&__v) }
                            {
                                ref x => (*x).clone(),
                                _ => return None,
                            };
                            Some(((*x).clone()).into_ddvalue())
                        }
                        __f
                    },
                    next: Box::new(None),
                }),
            },
        ],
        arrangements: vec![],
        change_cb: Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone()))),
    };
    let __Null = Relation {
        name: "__Null".to_string(),
        input: false,
        distinct: false,
        caching_mode: CachingMode::Set,
        key_func: None,
        id: Relations::__Null as RelId,
        rules: vec![],
        arrangements: vec![Arrangement::Map {
            name: r###"_ /*join*/"###.to_string(),
            afun: &{
                fn __f(__v: DDValue) -> Option<(DDValue, DDValue)> {
                    let __cloned = __v.clone();
                    match unsafe { <()>::from_ddvalue(__v) } {
                        _ => Some((()).into_ddvalue()),
                        _ => None,
                    }
                    .map(|x| (x, __cloned))
                }
                __f
            },
            queryable: true,
        }],
        change_cb: None,
    };
    Program {
        nodes: vec![
            ProgNode::Rel { rel: ConstDecl },
            ProgNode::Rel {
                rel: INPUT_ConstDecl,
            },
            ProgNode::Rel { rel: ExprBigInt },
            ProgNode::Rel {
                rel: INPUT_ExprBigInt,
            },
            ProgNode::Rel { rel: ExprBool },
            ProgNode::Rel {
                rel: INPUT_ExprBool,
            },
            ProgNode::Rel { rel: ExprNameRef },
            ProgNode::Rel {
                rel: INPUT_ExprNameRef,
            },
            ProgNode::Rel { rel: ExprNumber },
            ProgNode::Rel {
                rel: INPUT_ExprNumber,
            },
            ProgNode::Rel { rel: ExprString },
            ProgNode::Rel {
                rel: INPUT_ExprString,
            },
            ProgNode::Rel { rel: Expression },
            ProgNode::Rel {
                rel: INPUT_Expression,
            },
            ProgNode::Rel { rel: Function },
            ProgNode::Rel {
                rel: INPUT_Function,
            },
            ProgNode::Rel { rel: FunctionArg },
            ProgNode::Rel {
                rel: INPUT_FunctionArg,
            },
            ProgNode::Rel { rel: InputScope },
            ProgNode::Rel { rel: ChildScope },
            ProgNode::Rel {
                rel: INPUT_InputScope,
            },
            ProgNode::Rel { rel: LetDecl },
            ProgNode::Rel { rel: INPUT_LetDecl },
            ProgNode::Rel { rel: Return },
            ProgNode::Rel { rel: INPUT_Return },
            ProgNode::Rel { rel: Statement },
            ProgNode::Rel {
                rel: INPUT_Statement,
            },
            ProgNode::Rel { rel: VarDecl },
            ProgNode::SCC {
                rels: vec![RecursiveRelation {
                    rel: NameInScope,
                    distinct: true,
                }],
            },
            ProgNode::Rel {
                rel: InvalidNameUse,
            },
            ProgNode::Rel { rel: INPUT_VarDecl },
            ProgNode::Rel { rel: __Null },
        ],
        init_data: vec![],
    }
}
