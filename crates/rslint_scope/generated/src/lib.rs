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


decl_update_deserializer!(UpdateSerializer,(0, ::types::Array), (1, ::types::Arrow), (2, ::types::ArrowParam), (3, ::types::Assign), (4, ::types::Await), (5, ::types::BinOp), (6, ::types::BracketAccess), (7, ::types::Break), (8, ::types::Call), (9, ::types::ChildScope), (10, ::types::Class), (11, ::types::ClassExpr), (12, ::types::ClosestFunction), (13, ::types::ConstDecl), (14, ::types::Continue), (15, ::types::DoWhile), (16, ::types::DotAccess), (17, ::types::EveryScope), (18, ::types::ExprBigInt), (19, ::types::ExprBool), (20, ::types::ExprNumber), (21, ::types::ExprString), (22, ::types::Expression), (23, ::types::For), (24, ::types::ForIn), (25, ::types::Function), (26, ::types::FunctionArg), (27, ::types::Array), (28, ::types::Arrow), (29, ::types::ArrowParam), (30, ::types::Assign), (31, ::types::Await), (32, ::types::BinOp), (33, ::types::BracketAccess), (34, ::types::Break), (35, ::types::Call), (36, ::types::Class), (37, ::types::ClassExpr), (38, ::types::ConstDecl), (39, ::types::Continue), (40, ::types::DoWhile), (41, ::types::DotAccess), (42, ::types::EveryScope), (43, ::types::ExprBigInt), (44, ::types::ExprBool), (45, ::types::ExprNumber), (46, ::types::ExprString), (47, ::types::Expression), (48, ::types::For), (49, ::types::ForIn), (50, ::types::Function), (51, ::types::FunctionArg), (52, ::types::If), (53, ::types::ImplicitGlobal), (54, ::types::InlineFunc), (55, ::types::InlineFuncParam), (56, ::types::InputScope), (57, ::types::Label), (58, ::types::LetDecl), (59, ::types::NameRef), (60, ::types::New), (61, ::types::Property), (62, ::types::Return), (63, ::types::Statement), (64, ::types::Switch), (65, ::types::SwitchCase), (66, ::types::Template), (67, ::types::Ternary), (68, ::types::Throw), (69, ::types::Try), (70, ::types::UnaryOp), (71, ::types::VarDecl), (72, ::types::While), (73, ::types::With), (74, ::types::Yield), (75, ::types::If), (76, ::types::ImplicitGlobal), (77, ::types::InlineFunc), (78, ::types::InlineFuncParam), (79, ::types::InputScope), (80, ::types::InvalidNameUse), (81, ::types::Label), (82, ::types::LetDecl), (83, ::types::NameInScope), (84, ::types::NameRef), (85, ::types::New), (86, ::types::Property), (87, ::types::Return), (88, ::types::Statement), (89, ::types::Switch), (90, ::types::SwitchCase), (91, ::types::Template), (92, ::types::Ternary), (93, ::types::Throw), (94, ::types::Try), (95, ::types::UnaryOp), (96, ::types::VarDecl), (97, ::types::VarUseBeforeDeclaration), (98, ::types::While), (99, ::types::With), (100, ::types::WithinTypeOf), (101, ::types::Yield));
impl TryFrom<&str> for Relations {
    type Error = ();
    fn try_from(rname: &str) -> ::std::result::Result<Self, ()> {
         match rname {
        "Array" => Ok(Relations::Array),
        "Arrow" => Ok(Relations::Arrow),
        "ArrowParam" => Ok(Relations::ArrowParam),
        "Assign" => Ok(Relations::Assign),
        "Await" => Ok(Relations::Await),
        "BinOp" => Ok(Relations::BinOp),
        "BracketAccess" => Ok(Relations::BracketAccess),
        "Break" => Ok(Relations::Break),
        "Call" => Ok(Relations::Call),
        "ChildScope" => Ok(Relations::ChildScope),
        "Class" => Ok(Relations::Class),
        "ClassExpr" => Ok(Relations::ClassExpr),
        "ClosestFunction" => Ok(Relations::ClosestFunction),
        "ConstDecl" => Ok(Relations::ConstDecl),
        "Continue" => Ok(Relations::Continue),
        "DoWhile" => Ok(Relations::DoWhile),
        "DotAccess" => Ok(Relations::DotAccess),
        "EveryScope" => Ok(Relations::EveryScope),
        "ExprBigInt" => Ok(Relations::ExprBigInt),
        "ExprBool" => Ok(Relations::ExprBool),
        "ExprNumber" => Ok(Relations::ExprNumber),
        "ExprString" => Ok(Relations::ExprString),
        "Expression" => Ok(Relations::Expression),
        "For" => Ok(Relations::For),
        "ForIn" => Ok(Relations::ForIn),
        "Function" => Ok(Relations::Function),
        "FunctionArg" => Ok(Relations::FunctionArg),
        "INPUT_Array" => Ok(Relations::INPUT_Array),
        "INPUT_Arrow" => Ok(Relations::INPUT_Arrow),
        "INPUT_ArrowParam" => Ok(Relations::INPUT_ArrowParam),
        "INPUT_Assign" => Ok(Relations::INPUT_Assign),
        "INPUT_Await" => Ok(Relations::INPUT_Await),
        "INPUT_BinOp" => Ok(Relations::INPUT_BinOp),
        "INPUT_BracketAccess" => Ok(Relations::INPUT_BracketAccess),
        "INPUT_Break" => Ok(Relations::INPUT_Break),
        "INPUT_Call" => Ok(Relations::INPUT_Call),
        "INPUT_Class" => Ok(Relations::INPUT_Class),
        "INPUT_ClassExpr" => Ok(Relations::INPUT_ClassExpr),
        "INPUT_ConstDecl" => Ok(Relations::INPUT_ConstDecl),
        "INPUT_Continue" => Ok(Relations::INPUT_Continue),
        "INPUT_DoWhile" => Ok(Relations::INPUT_DoWhile),
        "INPUT_DotAccess" => Ok(Relations::INPUT_DotAccess),
        "INPUT_EveryScope" => Ok(Relations::INPUT_EveryScope),
        "INPUT_ExprBigInt" => Ok(Relations::INPUT_ExprBigInt),
        "INPUT_ExprBool" => Ok(Relations::INPUT_ExprBool),
        "INPUT_ExprNumber" => Ok(Relations::INPUT_ExprNumber),
        "INPUT_ExprString" => Ok(Relations::INPUT_ExprString),
        "INPUT_Expression" => Ok(Relations::INPUT_Expression),
        "INPUT_For" => Ok(Relations::INPUT_For),
        "INPUT_ForIn" => Ok(Relations::INPUT_ForIn),
        "INPUT_Function" => Ok(Relations::INPUT_Function),
        "INPUT_FunctionArg" => Ok(Relations::INPUT_FunctionArg),
        "INPUT_If" => Ok(Relations::INPUT_If),
        "INPUT_ImplicitGlobal" => Ok(Relations::INPUT_ImplicitGlobal),
        "INPUT_InlineFunc" => Ok(Relations::INPUT_InlineFunc),
        "INPUT_InlineFuncParam" => Ok(Relations::INPUT_InlineFuncParam),
        "INPUT_InputScope" => Ok(Relations::INPUT_InputScope),
        "INPUT_Label" => Ok(Relations::INPUT_Label),
        "INPUT_LetDecl" => Ok(Relations::INPUT_LetDecl),
        "INPUT_NameRef" => Ok(Relations::INPUT_NameRef),
        "INPUT_New" => Ok(Relations::INPUT_New),
        "INPUT_Property" => Ok(Relations::INPUT_Property),
        "INPUT_Return" => Ok(Relations::INPUT_Return),
        "INPUT_Statement" => Ok(Relations::INPUT_Statement),
        "INPUT_Switch" => Ok(Relations::INPUT_Switch),
        "INPUT_SwitchCase" => Ok(Relations::INPUT_SwitchCase),
        "INPUT_Template" => Ok(Relations::INPUT_Template),
        "INPUT_Ternary" => Ok(Relations::INPUT_Ternary),
        "INPUT_Throw" => Ok(Relations::INPUT_Throw),
        "INPUT_Try" => Ok(Relations::INPUT_Try),
        "INPUT_UnaryOp" => Ok(Relations::INPUT_UnaryOp),
        "INPUT_VarDecl" => Ok(Relations::INPUT_VarDecl),
        "INPUT_While" => Ok(Relations::INPUT_While),
        "INPUT_With" => Ok(Relations::INPUT_With),
        "INPUT_Yield" => Ok(Relations::INPUT_Yield),
        "If" => Ok(Relations::If),
        "ImplicitGlobal" => Ok(Relations::ImplicitGlobal),
        "InlineFunc" => Ok(Relations::InlineFunc),
        "InlineFuncParam" => Ok(Relations::InlineFuncParam),
        "InputScope" => Ok(Relations::InputScope),
        "InvalidNameUse" => Ok(Relations::InvalidNameUse),
        "Label" => Ok(Relations::Label),
        "LetDecl" => Ok(Relations::LetDecl),
        "NameInScope" => Ok(Relations::NameInScope),
        "NameRef" => Ok(Relations::NameRef),
        "New" => Ok(Relations::New),
        "Property" => Ok(Relations::Property),
        "Return" => Ok(Relations::Return),
        "Statement" => Ok(Relations::Statement),
        "Switch" => Ok(Relations::Switch),
        "SwitchCase" => Ok(Relations::SwitchCase),
        "Template" => Ok(Relations::Template),
        "Ternary" => Ok(Relations::Ternary),
        "Throw" => Ok(Relations::Throw),
        "Try" => Ok(Relations::Try),
        "UnaryOp" => Ok(Relations::UnaryOp),
        "VarDecl" => Ok(Relations::VarDecl),
        "VarUseBeforeDeclaration" => Ok(Relations::VarUseBeforeDeclaration),
        "While" => Ok(Relations::While),
        "With" => Ok(Relations::With),
        "WithinTypeOf" => Ok(Relations::WithinTypeOf),
        "Yield" => Ok(Relations::Yield),
        "__Prefix_0" => Ok(Relations::__Prefix_0),
        "__Prefix_1" => Ok(Relations::__Prefix_1),
             _  => Err(())
         }
    }
}
impl Relations {
    pub fn is_output(&self) -> bool {
        match self {
        Relations::ChildScope => true,
        Relations::ClosestFunction => true,
        Relations::INPUT_Array => true,
        Relations::INPUT_Arrow => true,
        Relations::INPUT_ArrowParam => true,
        Relations::INPUT_Assign => true,
        Relations::INPUT_Await => true,
        Relations::INPUT_BinOp => true,
        Relations::INPUT_BracketAccess => true,
        Relations::INPUT_Break => true,
        Relations::INPUT_Call => true,
        Relations::INPUT_Class => true,
        Relations::INPUT_ClassExpr => true,
        Relations::INPUT_ConstDecl => true,
        Relations::INPUT_Continue => true,
        Relations::INPUT_DoWhile => true,
        Relations::INPUT_DotAccess => true,
        Relations::INPUT_EveryScope => true,
        Relations::INPUT_ExprBigInt => true,
        Relations::INPUT_ExprBool => true,
        Relations::INPUT_ExprNumber => true,
        Relations::INPUT_ExprString => true,
        Relations::INPUT_Expression => true,
        Relations::INPUT_For => true,
        Relations::INPUT_ForIn => true,
        Relations::INPUT_Function => true,
        Relations::INPUT_FunctionArg => true,
        Relations::INPUT_If => true,
        Relations::INPUT_ImplicitGlobal => true,
        Relations::INPUT_InlineFunc => true,
        Relations::INPUT_InlineFuncParam => true,
        Relations::INPUT_InputScope => true,
        Relations::INPUT_Label => true,
        Relations::INPUT_LetDecl => true,
        Relations::INPUT_NameRef => true,
        Relations::INPUT_New => true,
        Relations::INPUT_Property => true,
        Relations::INPUT_Return => true,
        Relations::INPUT_Statement => true,
        Relations::INPUT_Switch => true,
        Relations::INPUT_SwitchCase => true,
        Relations::INPUT_Template => true,
        Relations::INPUT_Ternary => true,
        Relations::INPUT_Throw => true,
        Relations::INPUT_Try => true,
        Relations::INPUT_UnaryOp => true,
        Relations::INPUT_VarDecl => true,
        Relations::INPUT_While => true,
        Relations::INPUT_With => true,
        Relations::INPUT_Yield => true,
        Relations::InvalidNameUse => true,
        Relations::NameInScope => true,
        Relations::VarUseBeforeDeclaration => true,
        Relations::WithinTypeOf => true,
            _  => false
        }
    }
}
impl Relations {
    pub fn is_input(&self) -> bool {
        match self {
        Relations::Array => true,
        Relations::Arrow => true,
        Relations::ArrowParam => true,
        Relations::Assign => true,
        Relations::Await => true,
        Relations::BinOp => true,
        Relations::BracketAccess => true,
        Relations::Break => true,
        Relations::Call => true,
        Relations::Class => true,
        Relations::ClassExpr => true,
        Relations::ConstDecl => true,
        Relations::Continue => true,
        Relations::DoWhile => true,
        Relations::DotAccess => true,
        Relations::EveryScope => true,
        Relations::ExprBigInt => true,
        Relations::ExprBool => true,
        Relations::ExprNumber => true,
        Relations::ExprString => true,
        Relations::Expression => true,
        Relations::For => true,
        Relations::ForIn => true,
        Relations::Function => true,
        Relations::FunctionArg => true,
        Relations::If => true,
        Relations::ImplicitGlobal => true,
        Relations::InlineFunc => true,
        Relations::InlineFuncParam => true,
        Relations::InputScope => true,
        Relations::Label => true,
        Relations::LetDecl => true,
        Relations::NameRef => true,
        Relations::New => true,
        Relations::Property => true,
        Relations::Return => true,
        Relations::Statement => true,
        Relations::Switch => true,
        Relations::SwitchCase => true,
        Relations::Template => true,
        Relations::Ternary => true,
        Relations::Throw => true,
        Relations::Try => true,
        Relations::UnaryOp => true,
        Relations::VarDecl => true,
        Relations::While => true,
        Relations::With => true,
        Relations::Yield => true,
            _  => false
        }
    }
}
impl TryFrom<RelId> for Relations {
    type Error = ();
    fn try_from(rid: RelId) -> ::std::result::Result<Self, ()> {
         match rid {
        0 => Ok(Relations::Array),
        1 => Ok(Relations::Arrow),
        2 => Ok(Relations::ArrowParam),
        3 => Ok(Relations::Assign),
        4 => Ok(Relations::Await),
        5 => Ok(Relations::BinOp),
        6 => Ok(Relations::BracketAccess),
        7 => Ok(Relations::Break),
        8 => Ok(Relations::Call),
        9 => Ok(Relations::ChildScope),
        10 => Ok(Relations::Class),
        11 => Ok(Relations::ClassExpr),
        12 => Ok(Relations::ClosestFunction),
        13 => Ok(Relations::ConstDecl),
        14 => Ok(Relations::Continue),
        15 => Ok(Relations::DoWhile),
        16 => Ok(Relations::DotAccess),
        17 => Ok(Relations::EveryScope),
        18 => Ok(Relations::ExprBigInt),
        19 => Ok(Relations::ExprBool),
        20 => Ok(Relations::ExprNumber),
        21 => Ok(Relations::ExprString),
        22 => Ok(Relations::Expression),
        23 => Ok(Relations::For),
        24 => Ok(Relations::ForIn),
        25 => Ok(Relations::Function),
        26 => Ok(Relations::FunctionArg),
        27 => Ok(Relations::INPUT_Array),
        28 => Ok(Relations::INPUT_Arrow),
        29 => Ok(Relations::INPUT_ArrowParam),
        30 => Ok(Relations::INPUT_Assign),
        31 => Ok(Relations::INPUT_Await),
        32 => Ok(Relations::INPUT_BinOp),
        33 => Ok(Relations::INPUT_BracketAccess),
        34 => Ok(Relations::INPUT_Break),
        35 => Ok(Relations::INPUT_Call),
        36 => Ok(Relations::INPUT_Class),
        37 => Ok(Relations::INPUT_ClassExpr),
        38 => Ok(Relations::INPUT_ConstDecl),
        39 => Ok(Relations::INPUT_Continue),
        40 => Ok(Relations::INPUT_DoWhile),
        41 => Ok(Relations::INPUT_DotAccess),
        42 => Ok(Relations::INPUT_EveryScope),
        43 => Ok(Relations::INPUT_ExprBigInt),
        44 => Ok(Relations::INPUT_ExprBool),
        45 => Ok(Relations::INPUT_ExprNumber),
        46 => Ok(Relations::INPUT_ExprString),
        47 => Ok(Relations::INPUT_Expression),
        48 => Ok(Relations::INPUT_For),
        49 => Ok(Relations::INPUT_ForIn),
        50 => Ok(Relations::INPUT_Function),
        51 => Ok(Relations::INPUT_FunctionArg),
        52 => Ok(Relations::INPUT_If),
        53 => Ok(Relations::INPUT_ImplicitGlobal),
        54 => Ok(Relations::INPUT_InlineFunc),
        55 => Ok(Relations::INPUT_InlineFuncParam),
        56 => Ok(Relations::INPUT_InputScope),
        57 => Ok(Relations::INPUT_Label),
        58 => Ok(Relations::INPUT_LetDecl),
        59 => Ok(Relations::INPUT_NameRef),
        60 => Ok(Relations::INPUT_New),
        61 => Ok(Relations::INPUT_Property),
        62 => Ok(Relations::INPUT_Return),
        63 => Ok(Relations::INPUT_Statement),
        64 => Ok(Relations::INPUT_Switch),
        65 => Ok(Relations::INPUT_SwitchCase),
        66 => Ok(Relations::INPUT_Template),
        67 => Ok(Relations::INPUT_Ternary),
        68 => Ok(Relations::INPUT_Throw),
        69 => Ok(Relations::INPUT_Try),
        70 => Ok(Relations::INPUT_UnaryOp),
        71 => Ok(Relations::INPUT_VarDecl),
        72 => Ok(Relations::INPUT_While),
        73 => Ok(Relations::INPUT_With),
        74 => Ok(Relations::INPUT_Yield),
        75 => Ok(Relations::If),
        76 => Ok(Relations::ImplicitGlobal),
        77 => Ok(Relations::InlineFunc),
        78 => Ok(Relations::InlineFuncParam),
        79 => Ok(Relations::InputScope),
        80 => Ok(Relations::InvalidNameUse),
        81 => Ok(Relations::Label),
        82 => Ok(Relations::LetDecl),
        83 => Ok(Relations::NameInScope),
        84 => Ok(Relations::NameRef),
        85 => Ok(Relations::New),
        86 => Ok(Relations::Property),
        87 => Ok(Relations::Return),
        88 => Ok(Relations::Statement),
        89 => Ok(Relations::Switch),
        90 => Ok(Relations::SwitchCase),
        91 => Ok(Relations::Template),
        92 => Ok(Relations::Ternary),
        93 => Ok(Relations::Throw),
        94 => Ok(Relations::Try),
        95 => Ok(Relations::UnaryOp),
        96 => Ok(Relations::VarDecl),
        97 => Ok(Relations::VarUseBeforeDeclaration),
        98 => Ok(Relations::While),
        99 => Ok(Relations::With),
        100 => Ok(Relations::WithinTypeOf),
        101 => Ok(Relations::Yield),
        102 => Ok(Relations::__Prefix_0),
        103 => Ok(Relations::__Prefix_1),
             _  => Err(())
         }
    }
}
pub fn relid2name(rid: RelId) -> Option<&'static str> {
   match rid {
        0 => Some(&"Array"),
        1 => Some(&"Arrow"),
        2 => Some(&"ArrowParam"),
        3 => Some(&"Assign"),
        4 => Some(&"Await"),
        5 => Some(&"BinOp"),
        6 => Some(&"BracketAccess"),
        7 => Some(&"Break"),
        8 => Some(&"Call"),
        9 => Some(&"ChildScope"),
        10 => Some(&"Class"),
        11 => Some(&"ClassExpr"),
        12 => Some(&"ClosestFunction"),
        13 => Some(&"ConstDecl"),
        14 => Some(&"Continue"),
        15 => Some(&"DoWhile"),
        16 => Some(&"DotAccess"),
        17 => Some(&"EveryScope"),
        18 => Some(&"ExprBigInt"),
        19 => Some(&"ExprBool"),
        20 => Some(&"ExprNumber"),
        21 => Some(&"ExprString"),
        22 => Some(&"Expression"),
        23 => Some(&"For"),
        24 => Some(&"ForIn"),
        25 => Some(&"Function"),
        26 => Some(&"FunctionArg"),
        27 => Some(&"INPUT_Array"),
        28 => Some(&"INPUT_Arrow"),
        29 => Some(&"INPUT_ArrowParam"),
        30 => Some(&"INPUT_Assign"),
        31 => Some(&"INPUT_Await"),
        32 => Some(&"INPUT_BinOp"),
        33 => Some(&"INPUT_BracketAccess"),
        34 => Some(&"INPUT_Break"),
        35 => Some(&"INPUT_Call"),
        36 => Some(&"INPUT_Class"),
        37 => Some(&"INPUT_ClassExpr"),
        38 => Some(&"INPUT_ConstDecl"),
        39 => Some(&"INPUT_Continue"),
        40 => Some(&"INPUT_DoWhile"),
        41 => Some(&"INPUT_DotAccess"),
        42 => Some(&"INPUT_EveryScope"),
        43 => Some(&"INPUT_ExprBigInt"),
        44 => Some(&"INPUT_ExprBool"),
        45 => Some(&"INPUT_ExprNumber"),
        46 => Some(&"INPUT_ExprString"),
        47 => Some(&"INPUT_Expression"),
        48 => Some(&"INPUT_For"),
        49 => Some(&"INPUT_ForIn"),
        50 => Some(&"INPUT_Function"),
        51 => Some(&"INPUT_FunctionArg"),
        52 => Some(&"INPUT_If"),
        53 => Some(&"INPUT_ImplicitGlobal"),
        54 => Some(&"INPUT_InlineFunc"),
        55 => Some(&"INPUT_InlineFuncParam"),
        56 => Some(&"INPUT_InputScope"),
        57 => Some(&"INPUT_Label"),
        58 => Some(&"INPUT_LetDecl"),
        59 => Some(&"INPUT_NameRef"),
        60 => Some(&"INPUT_New"),
        61 => Some(&"INPUT_Property"),
        62 => Some(&"INPUT_Return"),
        63 => Some(&"INPUT_Statement"),
        64 => Some(&"INPUT_Switch"),
        65 => Some(&"INPUT_SwitchCase"),
        66 => Some(&"INPUT_Template"),
        67 => Some(&"INPUT_Ternary"),
        68 => Some(&"INPUT_Throw"),
        69 => Some(&"INPUT_Try"),
        70 => Some(&"INPUT_UnaryOp"),
        71 => Some(&"INPUT_VarDecl"),
        72 => Some(&"INPUT_While"),
        73 => Some(&"INPUT_With"),
        74 => Some(&"INPUT_Yield"),
        75 => Some(&"If"),
        76 => Some(&"ImplicitGlobal"),
        77 => Some(&"InlineFunc"),
        78 => Some(&"InlineFuncParam"),
        79 => Some(&"InputScope"),
        80 => Some(&"InvalidNameUse"),
        81 => Some(&"Label"),
        82 => Some(&"LetDecl"),
        83 => Some(&"NameInScope"),
        84 => Some(&"NameRef"),
        85 => Some(&"New"),
        86 => Some(&"Property"),
        87 => Some(&"Return"),
        88 => Some(&"Statement"),
        89 => Some(&"Switch"),
        90 => Some(&"SwitchCase"),
        91 => Some(&"Template"),
        92 => Some(&"Ternary"),
        93 => Some(&"Throw"),
        94 => Some(&"Try"),
        95 => Some(&"UnaryOp"),
        96 => Some(&"VarDecl"),
        97 => Some(&"VarUseBeforeDeclaration"),
        98 => Some(&"While"),
        99 => Some(&"With"),
        100 => Some(&"WithinTypeOf"),
        101 => Some(&"Yield"),
        102 => Some(&"__Prefix_0"),
        103 => Some(&"__Prefix_1"),
       _  => None
   }
}
pub fn relid2cname(rid: RelId) -> Option<&'static ::std::ffi::CStr> {
    RELIDMAPC.get(&rid).copied()
}   /// A map of `RelId`s to their name as an `&'static str`
pub static RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(104, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::Array, "Array");
        map.insert(Relations::Arrow, "Arrow");
        map.insert(Relations::ArrowParam, "ArrowParam");
        map.insert(Relations::Assign, "Assign");
        map.insert(Relations::Await, "Await");
        map.insert(Relations::BinOp, "BinOp");
        map.insert(Relations::BracketAccess, "BracketAccess");
        map.insert(Relations::Break, "Break");
        map.insert(Relations::Call, "Call");
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::Class, "Class");
        map.insert(Relations::ClassExpr, "ClassExpr");
        map.insert(Relations::ClosestFunction, "ClosestFunction");
        map.insert(Relations::ConstDecl, "ConstDecl");
        map.insert(Relations::Continue, "Continue");
        map.insert(Relations::DoWhile, "DoWhile");
        map.insert(Relations::DotAccess, "DotAccess");
        map.insert(Relations::EveryScope, "EveryScope");
        map.insert(Relations::ExprBigInt, "ExprBigInt");
        map.insert(Relations::ExprBool, "ExprBool");
        map.insert(Relations::ExprNumber, "ExprNumber");
        map.insert(Relations::ExprString, "ExprString");
        map.insert(Relations::Expression, "Expression");
        map.insert(Relations::For, "For");
        map.insert(Relations::ForIn, "ForIn");
        map.insert(Relations::Function, "Function");
        map.insert(Relations::FunctionArg, "FunctionArg");
        map.insert(Relations::INPUT_Array, "INPUT_Array");
        map.insert(Relations::INPUT_Arrow, "INPUT_Arrow");
        map.insert(Relations::INPUT_ArrowParam, "INPUT_ArrowParam");
        map.insert(Relations::INPUT_Assign, "INPUT_Assign");
        map.insert(Relations::INPUT_Await, "INPUT_Await");
        map.insert(Relations::INPUT_BinOp, "INPUT_BinOp");
        map.insert(Relations::INPUT_BracketAccess, "INPUT_BracketAccess");
        map.insert(Relations::INPUT_Break, "INPUT_Break");
        map.insert(Relations::INPUT_Call, "INPUT_Call");
        map.insert(Relations::INPUT_Class, "INPUT_Class");
        map.insert(Relations::INPUT_ClassExpr, "INPUT_ClassExpr");
        map.insert(Relations::INPUT_ConstDecl, "INPUT_ConstDecl");
        map.insert(Relations::INPUT_Continue, "INPUT_Continue");
        map.insert(Relations::INPUT_DoWhile, "INPUT_DoWhile");
        map.insert(Relations::INPUT_DotAccess, "INPUT_DotAccess");
        map.insert(Relations::INPUT_EveryScope, "INPUT_EveryScope");
        map.insert(Relations::INPUT_ExprBigInt, "INPUT_ExprBigInt");
        map.insert(Relations::INPUT_ExprBool, "INPUT_ExprBool");
        map.insert(Relations::INPUT_ExprNumber, "INPUT_ExprNumber");
        map.insert(Relations::INPUT_ExprString, "INPUT_ExprString");
        map.insert(Relations::INPUT_Expression, "INPUT_Expression");
        map.insert(Relations::INPUT_For, "INPUT_For");
        map.insert(Relations::INPUT_ForIn, "INPUT_ForIn");
        map.insert(Relations::INPUT_Function, "INPUT_Function");
        map.insert(Relations::INPUT_FunctionArg, "INPUT_FunctionArg");
        map.insert(Relations::INPUT_If, "INPUT_If");
        map.insert(Relations::INPUT_ImplicitGlobal, "INPUT_ImplicitGlobal");
        map.insert(Relations::INPUT_InlineFunc, "INPUT_InlineFunc");
        map.insert(Relations::INPUT_InlineFuncParam, "INPUT_InlineFuncParam");
        map.insert(Relations::INPUT_InputScope, "INPUT_InputScope");
        map.insert(Relations::INPUT_Label, "INPUT_Label");
        map.insert(Relations::INPUT_LetDecl, "INPUT_LetDecl");
        map.insert(Relations::INPUT_NameRef, "INPUT_NameRef");
        map.insert(Relations::INPUT_New, "INPUT_New");
        map.insert(Relations::INPUT_Property, "INPUT_Property");
        map.insert(Relations::INPUT_Return, "INPUT_Return");
        map.insert(Relations::INPUT_Statement, "INPUT_Statement");
        map.insert(Relations::INPUT_Switch, "INPUT_Switch");
        map.insert(Relations::INPUT_SwitchCase, "INPUT_SwitchCase");
        map.insert(Relations::INPUT_Template, "INPUT_Template");
        map.insert(Relations::INPUT_Ternary, "INPUT_Ternary");
        map.insert(Relations::INPUT_Throw, "INPUT_Throw");
        map.insert(Relations::INPUT_Try, "INPUT_Try");
        map.insert(Relations::INPUT_UnaryOp, "INPUT_UnaryOp");
        map.insert(Relations::INPUT_VarDecl, "INPUT_VarDecl");
        map.insert(Relations::INPUT_While, "INPUT_While");
        map.insert(Relations::INPUT_With, "INPUT_With");
        map.insert(Relations::INPUT_Yield, "INPUT_Yield");
        map.insert(Relations::If, "If");
        map.insert(Relations::ImplicitGlobal, "ImplicitGlobal");
        map.insert(Relations::InlineFunc, "InlineFunc");
        map.insert(Relations::InlineFuncParam, "InlineFuncParam");
        map.insert(Relations::InputScope, "InputScope");
        map.insert(Relations::InvalidNameUse, "InvalidNameUse");
        map.insert(Relations::Label, "Label");
        map.insert(Relations::LetDecl, "LetDecl");
        map.insert(Relations::NameInScope, "NameInScope");
        map.insert(Relations::NameRef, "NameRef");
        map.insert(Relations::New, "New");
        map.insert(Relations::Property, "Property");
        map.insert(Relations::Return, "Return");
        map.insert(Relations::Statement, "Statement");
        map.insert(Relations::Switch, "Switch");
        map.insert(Relations::SwitchCase, "SwitchCase");
        map.insert(Relations::Template, "Template");
        map.insert(Relations::Ternary, "Ternary");
        map.insert(Relations::Throw, "Throw");
        map.insert(Relations::Try, "Try");
        map.insert(Relations::UnaryOp, "UnaryOp");
        map.insert(Relations::VarDecl, "VarDecl");
        map.insert(Relations::VarUseBeforeDeclaration, "VarUseBeforeDeclaration");
        map.insert(Relations::While, "While");
        map.insert(Relations::With, "With");
        map.insert(Relations::WithinTypeOf, "WithinTypeOf");
        map.insert(Relations::Yield, "Yield");
        map.insert(Relations::__Prefix_0, "__Prefix_0");
        map.insert(Relations::__Prefix_1, "__Prefix_1");
        map
    });
    /// A map of `RelId`s to their name as an `&'static CStr`
pub static RELIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<RelId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(104, ::fnv::FnvBuildHasher::default());
        map.insert(0, ::std::ffi::CStr::from_bytes_with_nul(b"Array\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(1, ::std::ffi::CStr::from_bytes_with_nul(b"Arrow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(2, ::std::ffi::CStr::from_bytes_with_nul(b"ArrowParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(3, ::std::ffi::CStr::from_bytes_with_nul(b"Assign\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(4, ::std::ffi::CStr::from_bytes_with_nul(b"Await\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(5, ::std::ffi::CStr::from_bytes_with_nul(b"BinOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(6, ::std::ffi::CStr::from_bytes_with_nul(b"BracketAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(7, ::std::ffi::CStr::from_bytes_with_nul(b"Break\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(8, ::std::ffi::CStr::from_bytes_with_nul(b"Call\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(9, ::std::ffi::CStr::from_bytes_with_nul(b"ChildScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(10, ::std::ffi::CStr::from_bytes_with_nul(b"Class\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(11, ::std::ffi::CStr::from_bytes_with_nul(b"ClassExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(12, ::std::ffi::CStr::from_bytes_with_nul(b"ClosestFunction\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(13, ::std::ffi::CStr::from_bytes_with_nul(b"ConstDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(14, ::std::ffi::CStr::from_bytes_with_nul(b"Continue\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(15, ::std::ffi::CStr::from_bytes_with_nul(b"DoWhile\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(16, ::std::ffi::CStr::from_bytes_with_nul(b"DotAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(17, ::std::ffi::CStr::from_bytes_with_nul(b"EveryScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(18, ::std::ffi::CStr::from_bytes_with_nul(b"ExprBigInt\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(19, ::std::ffi::CStr::from_bytes_with_nul(b"ExprBool\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(20, ::std::ffi::CStr::from_bytes_with_nul(b"ExprNumber\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(21, ::std::ffi::CStr::from_bytes_with_nul(b"ExprString\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(22, ::std::ffi::CStr::from_bytes_with_nul(b"Expression\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(23, ::std::ffi::CStr::from_bytes_with_nul(b"For\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(24, ::std::ffi::CStr::from_bytes_with_nul(b"ForIn\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(25, ::std::ffi::CStr::from_bytes_with_nul(b"Function\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(26, ::std::ffi::CStr::from_bytes_with_nul(b"FunctionArg\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(27, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Array\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(28, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Arrow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(29, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ArrowParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(30, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Assign\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(31, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Await\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(32, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_BinOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(33, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_BracketAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(34, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Break\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(35, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Call\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(36, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Class\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(37, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ClassExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(38, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ConstDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(39, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Continue\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(40, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_DoWhile\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(41, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_DotAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(42, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_EveryScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(43, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprBigInt\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(44, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprBool\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(45, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprNumber\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(46, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ExprString\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(47, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Expression\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(48, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_For\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(49, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ForIn\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(50, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Function\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(51, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_FunctionArg\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(52, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_If\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(53, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_ImplicitGlobal\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(54, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_InlineFunc\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(55, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_InlineFuncParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(56, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_InputScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(57, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Label\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(58, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_LetDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(59, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_NameRef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(60, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_New\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(61, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Property\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(62, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Return\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(63, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Statement\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(64, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Switch\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(65, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_SwitchCase\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(66, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Template\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(67, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Ternary\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(68, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Throw\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(69, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Try\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(70, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_UnaryOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(71, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_VarDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(72, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_While\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(73, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_With\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(74, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_Yield\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(75, ::std::ffi::CStr::from_bytes_with_nul(b"If\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(76, ::std::ffi::CStr::from_bytes_with_nul(b"ImplicitGlobal\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(77, ::std::ffi::CStr::from_bytes_with_nul(b"InlineFunc\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(78, ::std::ffi::CStr::from_bytes_with_nul(b"InlineFuncParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(79, ::std::ffi::CStr::from_bytes_with_nul(b"InputScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(80, ::std::ffi::CStr::from_bytes_with_nul(b"InvalidNameUse\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(81, ::std::ffi::CStr::from_bytes_with_nul(b"Label\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(82, ::std::ffi::CStr::from_bytes_with_nul(b"LetDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(83, ::std::ffi::CStr::from_bytes_with_nul(b"NameInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(84, ::std::ffi::CStr::from_bytes_with_nul(b"NameRef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(85, ::std::ffi::CStr::from_bytes_with_nul(b"New\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(86, ::std::ffi::CStr::from_bytes_with_nul(b"Property\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(87, ::std::ffi::CStr::from_bytes_with_nul(b"Return\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(88, ::std::ffi::CStr::from_bytes_with_nul(b"Statement\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(89, ::std::ffi::CStr::from_bytes_with_nul(b"Switch\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(90, ::std::ffi::CStr::from_bytes_with_nul(b"SwitchCase\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(91, ::std::ffi::CStr::from_bytes_with_nul(b"Template\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(92, ::std::ffi::CStr::from_bytes_with_nul(b"Ternary\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(93, ::std::ffi::CStr::from_bytes_with_nul(b"Throw\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(94, ::std::ffi::CStr::from_bytes_with_nul(b"Try\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(95, ::std::ffi::CStr::from_bytes_with_nul(b"UnaryOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(96, ::std::ffi::CStr::from_bytes_with_nul(b"VarDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(97, ::std::ffi::CStr::from_bytes_with_nul(b"VarUseBeforeDeclaration\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(98, ::std::ffi::CStr::from_bytes_with_nul(b"While\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(99, ::std::ffi::CStr::from_bytes_with_nul(b"With\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(100, ::std::ffi::CStr::from_bytes_with_nul(b"WithinTypeOf\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(101, ::std::ffi::CStr::from_bytes_with_nul(b"Yield\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(102, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_0\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(103, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_1\0").expect("Unreachable: A null byte was specifically inserted"));
        map
    });
    /// A map of input `Relations`s to their name as an `&'static str`
pub static INPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(48, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::Array, "Array");
        map.insert(Relations::Arrow, "Arrow");
        map.insert(Relations::ArrowParam, "ArrowParam");
        map.insert(Relations::Assign, "Assign");
        map.insert(Relations::Await, "Await");
        map.insert(Relations::BinOp, "BinOp");
        map.insert(Relations::BracketAccess, "BracketAccess");
        map.insert(Relations::Break, "Break");
        map.insert(Relations::Call, "Call");
        map.insert(Relations::Class, "Class");
        map.insert(Relations::ClassExpr, "ClassExpr");
        map.insert(Relations::ConstDecl, "ConstDecl");
        map.insert(Relations::Continue, "Continue");
        map.insert(Relations::DoWhile, "DoWhile");
        map.insert(Relations::DotAccess, "DotAccess");
        map.insert(Relations::EveryScope, "EveryScope");
        map.insert(Relations::ExprBigInt, "ExprBigInt");
        map.insert(Relations::ExprBool, "ExprBool");
        map.insert(Relations::ExprNumber, "ExprNumber");
        map.insert(Relations::ExprString, "ExprString");
        map.insert(Relations::Expression, "Expression");
        map.insert(Relations::For, "For");
        map.insert(Relations::ForIn, "ForIn");
        map.insert(Relations::Function, "Function");
        map.insert(Relations::FunctionArg, "FunctionArg");
        map.insert(Relations::If, "If");
        map.insert(Relations::ImplicitGlobal, "ImplicitGlobal");
        map.insert(Relations::InlineFunc, "InlineFunc");
        map.insert(Relations::InlineFuncParam, "InlineFuncParam");
        map.insert(Relations::InputScope, "InputScope");
        map.insert(Relations::Label, "Label");
        map.insert(Relations::LetDecl, "LetDecl");
        map.insert(Relations::NameRef, "NameRef");
        map.insert(Relations::New, "New");
        map.insert(Relations::Property, "Property");
        map.insert(Relations::Return, "Return");
        map.insert(Relations::Statement, "Statement");
        map.insert(Relations::Switch, "Switch");
        map.insert(Relations::SwitchCase, "SwitchCase");
        map.insert(Relations::Template, "Template");
        map.insert(Relations::Ternary, "Ternary");
        map.insert(Relations::Throw, "Throw");
        map.insert(Relations::Try, "Try");
        map.insert(Relations::UnaryOp, "UnaryOp");
        map.insert(Relations::VarDecl, "VarDecl");
        map.insert(Relations::While, "While");
        map.insert(Relations::With, "With");
        map.insert(Relations::Yield, "Yield");
        map
    });
    /// A map of output `Relations`s to their name as an `&'static str`
pub static OUTPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(54, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::ClosestFunction, "ClosestFunction");
        map.insert(Relations::INPUT_Array, "INPUT_Array");
        map.insert(Relations::INPUT_Arrow, "INPUT_Arrow");
        map.insert(Relations::INPUT_ArrowParam, "INPUT_ArrowParam");
        map.insert(Relations::INPUT_Assign, "INPUT_Assign");
        map.insert(Relations::INPUT_Await, "INPUT_Await");
        map.insert(Relations::INPUT_BinOp, "INPUT_BinOp");
        map.insert(Relations::INPUT_BracketAccess, "INPUT_BracketAccess");
        map.insert(Relations::INPUT_Break, "INPUT_Break");
        map.insert(Relations::INPUT_Call, "INPUT_Call");
        map.insert(Relations::INPUT_Class, "INPUT_Class");
        map.insert(Relations::INPUT_ClassExpr, "INPUT_ClassExpr");
        map.insert(Relations::INPUT_ConstDecl, "INPUT_ConstDecl");
        map.insert(Relations::INPUT_Continue, "INPUT_Continue");
        map.insert(Relations::INPUT_DoWhile, "INPUT_DoWhile");
        map.insert(Relations::INPUT_DotAccess, "INPUT_DotAccess");
        map.insert(Relations::INPUT_EveryScope, "INPUT_EveryScope");
        map.insert(Relations::INPUT_ExprBigInt, "INPUT_ExprBigInt");
        map.insert(Relations::INPUT_ExprBool, "INPUT_ExprBool");
        map.insert(Relations::INPUT_ExprNumber, "INPUT_ExprNumber");
        map.insert(Relations::INPUT_ExprString, "INPUT_ExprString");
        map.insert(Relations::INPUT_Expression, "INPUT_Expression");
        map.insert(Relations::INPUT_For, "INPUT_For");
        map.insert(Relations::INPUT_ForIn, "INPUT_ForIn");
        map.insert(Relations::INPUT_Function, "INPUT_Function");
        map.insert(Relations::INPUT_FunctionArg, "INPUT_FunctionArg");
        map.insert(Relations::INPUT_If, "INPUT_If");
        map.insert(Relations::INPUT_ImplicitGlobal, "INPUT_ImplicitGlobal");
        map.insert(Relations::INPUT_InlineFunc, "INPUT_InlineFunc");
        map.insert(Relations::INPUT_InlineFuncParam, "INPUT_InlineFuncParam");
        map.insert(Relations::INPUT_InputScope, "INPUT_InputScope");
        map.insert(Relations::INPUT_Label, "INPUT_Label");
        map.insert(Relations::INPUT_LetDecl, "INPUT_LetDecl");
        map.insert(Relations::INPUT_NameRef, "INPUT_NameRef");
        map.insert(Relations::INPUT_New, "INPUT_New");
        map.insert(Relations::INPUT_Property, "INPUT_Property");
        map.insert(Relations::INPUT_Return, "INPUT_Return");
        map.insert(Relations::INPUT_Statement, "INPUT_Statement");
        map.insert(Relations::INPUT_Switch, "INPUT_Switch");
        map.insert(Relations::INPUT_SwitchCase, "INPUT_SwitchCase");
        map.insert(Relations::INPUT_Template, "INPUT_Template");
        map.insert(Relations::INPUT_Ternary, "INPUT_Ternary");
        map.insert(Relations::INPUT_Throw, "INPUT_Throw");
        map.insert(Relations::INPUT_Try, "INPUT_Try");
        map.insert(Relations::INPUT_UnaryOp, "INPUT_UnaryOp");
        map.insert(Relations::INPUT_VarDecl, "INPUT_VarDecl");
        map.insert(Relations::INPUT_While, "INPUT_While");
        map.insert(Relations::INPUT_With, "INPUT_With");
        map.insert(Relations::INPUT_Yield, "INPUT_Yield");
        map.insert(Relations::InvalidNameUse, "InvalidNameUse");
        map.insert(Relations::NameInScope, "NameInScope");
        map.insert(Relations::VarUseBeforeDeclaration, "VarUseBeforeDeclaration");
        map.insert(Relations::WithinTypeOf, "WithinTypeOf");
        map
    });
impl TryFrom<&str> for Indexes {
    type Error = ();
    fn try_from(iname: &str) -> ::std::result::Result<Self, ()> {
         match iname {
        "Index_InvalidNameUse" => Ok(Indexes::Index_InvalidNameUse),
        "Index_VarUseBeforeDeclaration" => Ok(Indexes::Index_VarUseBeforeDeclaration),
        "Index_VariableInScope" => Ok(Indexes::Index_VariableInScope),
        "Index_VariablesForScope" => Ok(Indexes::Index_VariablesForScope),
             _  => Err(())
         }
    }
}
impl TryFrom<IdxId> for Indexes {
    type Error = ();
    fn try_from(iid: IdxId) -> ::core::result::Result<Self, ()> {
         match iid {
        0 => Ok(Indexes::Index_InvalidNameUse),
        1 => Ok(Indexes::Index_VarUseBeforeDeclaration),
        2 => Ok(Indexes::Index_VariableInScope),
        3 => Ok(Indexes::Index_VariablesForScope),
             _  => Err(())
         }
    }
}
pub fn indexid2name(iid: IdxId) -> Option<&'static str> {
   match iid {
        0 => Some(&"Index_InvalidNameUse"),
        1 => Some(&"Index_VarUseBeforeDeclaration"),
        2 => Some(&"Index_VariableInScope"),
        3 => Some(&"Index_VariablesForScope"),
       _  => None
   }
}
pub fn indexid2cname(iid: IdxId) -> Option<&'static ::std::ffi::CStr> {
    IDXIDMAPC.get(&iid).copied()
}   /// A map of `Indexes` to their name as an `&'static str`
pub static IDXIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Indexes, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(4, ::fnv::FnvBuildHasher::default());
        map.insert(Indexes::Index_InvalidNameUse, "Index_InvalidNameUse");
        map.insert(Indexes::Index_VarUseBeforeDeclaration, "Index_VarUseBeforeDeclaration");
        map.insert(Indexes::Index_VariableInScope, "Index_VariableInScope");
        map.insert(Indexes::Index_VariablesForScope, "Index_VariablesForScope");
        map
    });
    /// A map of `IdxId`s to their name as an `&'static CStr`
pub static IDXIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<IdxId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(4, ::fnv::FnvBuildHasher::default());
        map.insert(0, ::std::ffi::CStr::from_bytes_with_nul(b"Index_InvalidNameUse\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(1, ::std::ffi::CStr::from_bytes_with_nul(b"Index_VarUseBeforeDeclaration\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(2, ::std::ffi::CStr::from_bytes_with_nul(b"Index_VariableInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(3, ::std::ffi::CStr::from_bytes_with_nul(b"Index_VariablesForScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map
    });
pub fn relval_from_record(rel: Relations, _rec: &differential_datalog::record::Record) -> ::std::result::Result<DDValue, String> {
    match rel {
        Relations::Array => {
            Ok(<::types::Array>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Arrow => {
            Ok(<::types::Arrow>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ArrowParam => {
            Ok(<::types::ArrowParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Assign => {
            Ok(<::types::Assign>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Await => {
            Ok(<::types::Await>::from_record(_rec)?.into_ddvalue())
        },
        Relations::BinOp => {
            Ok(<::types::BinOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::BracketAccess => {
            Ok(<::types::BracketAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Break => {
            Ok(<::types::Break>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Call => {
            Ok(<::types::Call>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ChildScope => {
            Ok(<::types::ChildScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Class => {
            Ok(<::types::Class>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ClassExpr => {
            Ok(<::types::ClassExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ClosestFunction => {
            Ok(<::types::ClosestFunction>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ConstDecl => {
            Ok(<::types::ConstDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Continue => {
            Ok(<::types::Continue>::from_record(_rec)?.into_ddvalue())
        },
        Relations::DoWhile => {
            Ok(<::types::DoWhile>::from_record(_rec)?.into_ddvalue())
        },
        Relations::DotAccess => {
            Ok(<::types::DotAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::EveryScope => {
            Ok(<::types::EveryScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ExprBigInt => {
            Ok(<::types::ExprBigInt>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ExprBool => {
            Ok(<::types::ExprBool>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ExprNumber => {
            Ok(<::types::ExprNumber>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ExprString => {
            Ok(<::types::ExprString>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Expression => {
            Ok(<::types::Expression>::from_record(_rec)?.into_ddvalue())
        },
        Relations::For => {
            Ok(<::types::For>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ForIn => {
            Ok(<::types::ForIn>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Function => {
            Ok(<::types::Function>::from_record(_rec)?.into_ddvalue())
        },
        Relations::FunctionArg => {
            Ok(<::types::FunctionArg>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Array => {
            Ok(<::types::Array>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Arrow => {
            Ok(<::types::Arrow>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ArrowParam => {
            Ok(<::types::ArrowParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Assign => {
            Ok(<::types::Assign>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Await => {
            Ok(<::types::Await>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_BinOp => {
            Ok(<::types::BinOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_BracketAccess => {
            Ok(<::types::BracketAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Break => {
            Ok(<::types::Break>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Call => {
            Ok(<::types::Call>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Class => {
            Ok(<::types::Class>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ClassExpr => {
            Ok(<::types::ClassExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ConstDecl => {
            Ok(<::types::ConstDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Continue => {
            Ok(<::types::Continue>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_DoWhile => {
            Ok(<::types::DoWhile>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_DotAccess => {
            Ok(<::types::DotAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_EveryScope => {
            Ok(<::types::EveryScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ExprBigInt => {
            Ok(<::types::ExprBigInt>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ExprBool => {
            Ok(<::types::ExprBool>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ExprNumber => {
            Ok(<::types::ExprNumber>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ExprString => {
            Ok(<::types::ExprString>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Expression => {
            Ok(<::types::Expression>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_For => {
            Ok(<::types::For>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ForIn => {
            Ok(<::types::ForIn>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Function => {
            Ok(<::types::Function>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_FunctionArg => {
            Ok(<::types::FunctionArg>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_If => {
            Ok(<::types::If>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_ImplicitGlobal => {
            Ok(<::types::ImplicitGlobal>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_InlineFunc => {
            Ok(<::types::InlineFunc>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_InlineFuncParam => {
            Ok(<::types::InlineFuncParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_InputScope => {
            Ok(<::types::InputScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Label => {
            Ok(<::types::Label>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_LetDecl => {
            Ok(<::types::LetDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_NameRef => {
            Ok(<::types::NameRef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_New => {
            Ok(<::types::New>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Property => {
            Ok(<::types::Property>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Return => {
            Ok(<::types::Return>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Statement => {
            Ok(<::types::Statement>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Switch => {
            Ok(<::types::Switch>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_SwitchCase => {
            Ok(<::types::SwitchCase>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Template => {
            Ok(<::types::Template>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Ternary => {
            Ok(<::types::Ternary>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Throw => {
            Ok(<::types::Throw>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Try => {
            Ok(<::types::Try>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_UnaryOp => {
            Ok(<::types::UnaryOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_VarDecl => {
            Ok(<::types::VarDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_While => {
            Ok(<::types::While>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_With => {
            Ok(<::types::With>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_Yield => {
            Ok(<::types::Yield>::from_record(_rec)?.into_ddvalue())
        },
        Relations::If => {
            Ok(<::types::If>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ImplicitGlobal => {
            Ok(<::types::ImplicitGlobal>::from_record(_rec)?.into_ddvalue())
        },
        Relations::InlineFunc => {
            Ok(<::types::InlineFunc>::from_record(_rec)?.into_ddvalue())
        },
        Relations::InlineFuncParam => {
            Ok(<::types::InlineFuncParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::InputScope => {
            Ok(<::types::InputScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::InvalidNameUse => {
            Ok(<::types::InvalidNameUse>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Label => {
            Ok(<::types::Label>::from_record(_rec)?.into_ddvalue())
        },
        Relations::LetDecl => {
            Ok(<::types::LetDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::NameInScope => {
            Ok(<::types::NameInScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::NameRef => {
            Ok(<::types::NameRef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::New => {
            Ok(<::types::New>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Property => {
            Ok(<::types::Property>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Return => {
            Ok(<::types::Return>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Statement => {
            Ok(<::types::Statement>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Switch => {
            Ok(<::types::Switch>::from_record(_rec)?.into_ddvalue())
        },
        Relations::SwitchCase => {
            Ok(<::types::SwitchCase>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Template => {
            Ok(<::types::Template>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Ternary => {
            Ok(<::types::Ternary>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Throw => {
            Ok(<::types::Throw>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Try => {
            Ok(<::types::Try>::from_record(_rec)?.into_ddvalue())
        },
        Relations::UnaryOp => {
            Ok(<::types::UnaryOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::VarDecl => {
            Ok(<::types::VarDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::VarUseBeforeDeclaration => {
            Ok(<::types::VarUseBeforeDeclaration>::from_record(_rec)?.into_ddvalue())
        },
        Relations::While => {
            Ok(<::types::While>::from_record(_rec)?.into_ddvalue())
        },
        Relations::With => {
            Ok(<::types::With>::from_record(_rec)?.into_ddvalue())
        },
        Relations::WithinTypeOf => {
            Ok(<::types::WithinTypeOf>::from_record(_rec)?.into_ddvalue())
        },
        Relations::Yield => {
            Ok(<::types::Yield>::from_record(_rec)?.into_ddvalue())
        },
        Relations::__Prefix_0 => {
            Ok(<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::internment::Intern<::types::Pattern>>>::from_record(_rec)?.into_ddvalue())
        },
        Relations::__Prefix_1 => {
            Ok(<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::internment::Intern<::types::Pattern>>>::from_record(_rec)?.into_ddvalue())
        }
    }
}
pub fn relkey_from_record(rel: Relations, _rec: &differential_datalog::record::Record) -> ::std::result::Result<DDValue, String> {
    match rel {
        _ => Err(format!("relation {:?} does not have a primary key", rel))
    }
}
pub fn idxkey_from_record(idx: Indexes, _rec: &differential_datalog::record::Record) -> ::std::result::Result<DDValue, String> {
    match idx {
        Indexes::Index_InvalidNameUse => {
            Ok(<::types::internment::Intern<String>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::Index_VarUseBeforeDeclaration => {
            Ok(<::types::internment::Intern<String>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::Index_VariableInScope => {
            Ok(<::types::ddlog_std::tuple2<::types::Scope, ::types::internment::Intern<String>>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::Index_VariablesForScope => {
            Ok(<::types::Scope>::from_record(_rec)?.into_ddvalue())
        }
    }
}
pub fn indexes2arrid(idx: Indexes) -> ArrId {
    match idx {
        Indexes::Index_InvalidNameUse => ( 80, 0),
        Indexes::Index_VarUseBeforeDeclaration => ( 97, 0),
        Indexes::Index_VariableInScope => ( 83, 3),
        Indexes::Index_VariablesForScope => ( 83, 4),
    }
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Relations {
    Array = 0,
    Arrow = 1,
    ArrowParam = 2,
    Assign = 3,
    Await = 4,
    BinOp = 5,
    BracketAccess = 6,
    Break = 7,
    Call = 8,
    ChildScope = 9,
    Class = 10,
    ClassExpr = 11,
    ClosestFunction = 12,
    ConstDecl = 13,
    Continue = 14,
    DoWhile = 15,
    DotAccess = 16,
    EveryScope = 17,
    ExprBigInt = 18,
    ExprBool = 19,
    ExprNumber = 20,
    ExprString = 21,
    Expression = 22,
    For = 23,
    ForIn = 24,
    Function = 25,
    FunctionArg = 26,
    INPUT_Array = 27,
    INPUT_Arrow = 28,
    INPUT_ArrowParam = 29,
    INPUT_Assign = 30,
    INPUT_Await = 31,
    INPUT_BinOp = 32,
    INPUT_BracketAccess = 33,
    INPUT_Break = 34,
    INPUT_Call = 35,
    INPUT_Class = 36,
    INPUT_ClassExpr = 37,
    INPUT_ConstDecl = 38,
    INPUT_Continue = 39,
    INPUT_DoWhile = 40,
    INPUT_DotAccess = 41,
    INPUT_EveryScope = 42,
    INPUT_ExprBigInt = 43,
    INPUT_ExprBool = 44,
    INPUT_ExprNumber = 45,
    INPUT_ExprString = 46,
    INPUT_Expression = 47,
    INPUT_For = 48,
    INPUT_ForIn = 49,
    INPUT_Function = 50,
    INPUT_FunctionArg = 51,
    INPUT_If = 52,
    INPUT_ImplicitGlobal = 53,
    INPUT_InlineFunc = 54,
    INPUT_InlineFuncParam = 55,
    INPUT_InputScope = 56,
    INPUT_Label = 57,
    INPUT_LetDecl = 58,
    INPUT_NameRef = 59,
    INPUT_New = 60,
    INPUT_Property = 61,
    INPUT_Return = 62,
    INPUT_Statement = 63,
    INPUT_Switch = 64,
    INPUT_SwitchCase = 65,
    INPUT_Template = 66,
    INPUT_Ternary = 67,
    INPUT_Throw = 68,
    INPUT_Try = 69,
    INPUT_UnaryOp = 70,
    INPUT_VarDecl = 71,
    INPUT_While = 72,
    INPUT_With = 73,
    INPUT_Yield = 74,
    If = 75,
    ImplicitGlobal = 76,
    InlineFunc = 77,
    InlineFuncParam = 78,
    InputScope = 79,
    InvalidNameUse = 80,
    Label = 81,
    LetDecl = 82,
    NameInScope = 83,
    NameRef = 84,
    New = 85,
    Property = 86,
    Return = 87,
    Statement = 88,
    Switch = 89,
    SwitchCase = 90,
    Template = 91,
    Ternary = 92,
    Throw = 93,
    Try = 94,
    UnaryOp = 95,
    VarDecl = 96,
    VarUseBeforeDeclaration = 97,
    While = 98,
    With = 99,
    WithinTypeOf = 100,
    Yield = 101,
    __Prefix_0 = 102,
    __Prefix_1 = 103
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Indexes {
    Index_InvalidNameUse = 0,
    Index_VarUseBeforeDeclaration = 1,
    Index_VariableInScope = 2,
    Index_VariablesForScope = 3
}
pub fn prog(__update_cb: Box<dyn CBFn>) -> Program {
    let Array = Relation {
                    name:         "Array".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Array as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_Array = Relation {
                          name:         "INPUT_Array".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Array as RelId,
                          rules:        vec![
                              /* INPUT_Array[x] :- Array[(x: Array)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Array[x] :- Array[(x: Array)].".to_string(),
                                  rel: Relations::Array as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Array[x] :- Array[(x: Array)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Array>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let Arrow = Relation {
                    name:         "Arrow".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Arrow as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        Arrangement::Map{
                           name: r###"(Arrow{.expr_id=(_0: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: ExprId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow) /*join*/"###.to_string(),
                            afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                            {
                                let __cloned = __v.clone();
                                match unsafe {< ::types::Arrow>::from_ddvalue(__v) } {
                                    ::types::Arrow{expr_id: ref _0, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                    _ => None
                                }.map(|x|(x,__cloned))
                            }
                            __f},
                            queryable: false
                        },
                        Arrangement::Map{
                           name: r###"(Arrow{.expr_id=(_0: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(_: StmtId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow) /*join*/"###.to_string(),
                            afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                            {
                                let __cloned = __v.clone();
                                match unsafe {< ::types::Arrow>::from_ddvalue(__v) } {
                                    ::types::Arrow{expr_id: ref _0, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Right{r: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                    _ => None
                                }.map(|x|(x,__cloned))
                            }
                            __f},
                            queryable: false
                        }],
                    change_cb:    None
                };
    let INPUT_Arrow = Relation {
                          name:         "INPUT_Arrow".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Arrow as RelId,
                          rules:        vec![
                              /* INPUT_Arrow[x] :- Arrow[(x: Arrow)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Arrow[x] :- Arrow[(x: Arrow)].".to_string(),
                                  rel: Relations::Arrow as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Arrow[x] :- Arrow[(x: Arrow)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Arrow>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let ArrowParam = Relation {
                         name:         "ArrowParam".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::ArrowParam as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_ArrowParam = Relation {
                               name:         "INPUT_ArrowParam".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_ArrowParam as RelId,
                               rules:        vec![
                                   /* INPUT_ArrowParam[x] :- ArrowParam[(x: ArrowParam)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_ArrowParam[x] :- ArrowParam[(x: ArrowParam)].".to_string(),
                                       rel: Relations::ArrowParam as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_ArrowParam[x] :- ArrowParam[(x: ArrowParam)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::ArrowParam>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let __Prefix_0 = Relation {
                         name:         "__Prefix_0".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::__Prefix_0 as RelId,
                         rules:        vec![
                             /* __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))] :- ArrowParam[(ArrowParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: ArrowParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))). */
                             Rule::CollectionRule {
                                 description: "__Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))] :- ArrowParam[(ArrowParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: ArrowParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))).".to_string(),
                                 rel: Relations::ArrowParam as RelId,
                                 xform: Some(XFormCollection::FlatMap{
                                                 description: "ArrowParam[(ArrowParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: ArrowParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))" .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                 {
                                                     let (ref expr, ref pat) = match *unsafe {<::types::ArrowParam>::from_ddvalue_ref(&__v) } {
                                                         ::types::ArrowParam{expr_id: ref expr, param: ref pat} => ((*expr).clone(), (*pat).clone()),
                                                         _ => return None
                                                     };
                                                     let __flattened = ::types::bound_vars_internment_Intern__Pattern_ddlog_std_Vec__internment_Intern____Stringval(pat);
                                                     let expr = (*expr).clone();
                                                     let pat = (*pat).clone();
                                                     Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple3(name.clone(), expr.clone(), pat.clone())).into_ddvalue())))
                                                 }
                                                 __f},
                                                 next: Box::new(Some(XFormCollection::FilterMap{
                                                                         description: "head of __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))] :- ArrowParam[(ArrowParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: ArrowParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))." .to_string(),
                                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                         {
                                                                             let ::types::ddlog_std::tuple3(ref name, ref expr, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue_ref( &__v ) };
                                                                             Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*pat).clone())).into_ddvalue())
                                                                         }
                                                                         __f},
                                                                         next: Box::new(None)
                                                                     }))
                                             })
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"((_: Name), (_0: ExprId), (_: internment::Intern<Pattern>)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue(__v) } {
                                         ::types::ddlog_std::tuple3(_, ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let Assign = Relation {
                     name:         "Assign".to_string(),
                     input:        true,
                     distinct:     false,
                     caching_mode: CachingMode::Set,
                     key_func:     None,
                     id:           Relations::Assign as RelId,
                     rules:        vec![
                         ],
                     arrangements: vec![
                         ],
                     change_cb:    None
                 };
    let INPUT_Assign = Relation {
                           name:         "INPUT_Assign".to_string(),
                           input:        false,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::INPUT_Assign as RelId,
                           rules:        vec![
                               /* INPUT_Assign[x] :- Assign[(x: Assign)]. */
                               Rule::CollectionRule {
                                   description: "INPUT_Assign[x] :- Assign[(x: Assign)].".to_string(),
                                   rel: Relations::Assign as RelId,
                                   xform: Some(XFormCollection::FilterMap{
                                                   description: "head of INPUT_Assign[x] :- Assign[(x: Assign)]." .to_string(),
                                                   fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                   {
                                                       let ref x = match *unsafe {<::types::Assign>::from_ddvalue_ref(&__v) } {
                                                           ref x => (*x).clone(),
                                                           _ => return None
                                                       };
                                                       Some(((*x).clone()).into_ddvalue())
                                                   }
                                                   __f},
                                                   next: Box::new(None)
                                               })
                               }],
                           arrangements: vec![
                               ],
                           change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                       };
    let Await = Relation {
                    name:         "Await".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Await as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_Await = Relation {
                          name:         "INPUT_Await".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Await as RelId,
                          rules:        vec![
                              /* INPUT_Await[x] :- Await[(x: Await)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Await[x] :- Await[(x: Await)].".to_string(),
                                  rel: Relations::Await as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Await[x] :- Await[(x: Await)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Await>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let BinOp = Relation {
                    name:         "BinOp".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::BinOp as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_BinOp = Relation {
                          name:         "INPUT_BinOp".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_BinOp as RelId,
                          rules:        vec![
                              /* INPUT_BinOp[x] :- BinOp[(x: BinOp)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_BinOp[x] :- BinOp[(x: BinOp)].".to_string(),
                                  rel: Relations::BinOp as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_BinOp[x] :- BinOp[(x: BinOp)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::BinOp>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let BracketAccess = Relation {
                            name:         "BracketAccess".to_string(),
                            input:        true,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::BracketAccess as RelId,
                            rules:        vec![
                                ],
                            arrangements: vec![
                                ],
                            change_cb:    None
                        };
    let INPUT_BracketAccess = Relation {
                                  name:         "INPUT_BracketAccess".to_string(),
                                  input:        false,
                                  distinct:     false,
                                  caching_mode: CachingMode::Set,
                                  key_func:     None,
                                  id:           Relations::INPUT_BracketAccess as RelId,
                                  rules:        vec![
                                      /* INPUT_BracketAccess[x] :- BracketAccess[(x: BracketAccess)]. */
                                      Rule::CollectionRule {
                                          description: "INPUT_BracketAccess[x] :- BracketAccess[(x: BracketAccess)].".to_string(),
                                          rel: Relations::BracketAccess as RelId,
                                          xform: Some(XFormCollection::FilterMap{
                                                          description: "head of INPUT_BracketAccess[x] :- BracketAccess[(x: BracketAccess)]." .to_string(),
                                                          fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                          {
                                                              let ref x = match *unsafe {<::types::BracketAccess>::from_ddvalue_ref(&__v) } {
                                                                  ref x => (*x).clone(),
                                                                  _ => return None
                                                              };
                                                              Some(((*x).clone()).into_ddvalue())
                                                          }
                                                          __f},
                                                          next: Box::new(None)
                                                      })
                                      }],
                                  arrangements: vec![
                                      ],
                                  change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                              };
    let Break = Relation {
                    name:         "Break".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Break as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_Break = Relation {
                          name:         "INPUT_Break".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Break as RelId,
                          rules:        vec![
                              /* INPUT_Break[x] :- Break[(x: Break)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Break[x] :- Break[(x: Break)].".to_string(),
                                  rel: Relations::Break as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Break[x] :- Break[(x: Break)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Break>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let Call = Relation {
                   name:         "Call".to_string(),
                   input:        true,
                   distinct:     false,
                   caching_mode: CachingMode::Set,
                   key_func:     None,
                   id:           Relations::Call as RelId,
                   rules:        vec![
                       ],
                   arrangements: vec![
                       ],
                   change_cb:    None
               };
    let INPUT_Call = Relation {
                         name:         "INPUT_Call".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::INPUT_Call as RelId,
                         rules:        vec![
                             /* INPUT_Call[x] :- Call[(x: Call)]. */
                             Rule::CollectionRule {
                                 description: "INPUT_Call[x] :- Call[(x: Call)].".to_string(),
                                 rel: Relations::Call as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of INPUT_Call[x] :- Call[(x: Call)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let ref x = match *unsafe {<::types::Call>::from_ddvalue_ref(&__v) } {
                                                         ref x => (*x).clone(),
                                                         _ => return None
                                                     };
                                                     Some(((*x).clone()).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             }],
                         arrangements: vec![
                             ],
                         change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                     };
    let Class = Relation {
                    name:         "Class".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Class as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_Class = Relation {
                          name:         "INPUT_Class".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Class as RelId,
                          rules:        vec![
                              /* INPUT_Class[x] :- Class[(x: Class)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Class[x] :- Class[(x: Class)].".to_string(),
                                  rel: Relations::Class as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Class[x] :- Class[(x: Class)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Class>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let ClassExpr = Relation {
                        name:         "ClassExpr".to_string(),
                        input:        true,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::ClassExpr as RelId,
                        rules:        vec![
                            ],
                        arrangements: vec![
                            ],
                        change_cb:    None
                    };
    let INPUT_ClassExpr = Relation {
                              name:         "INPUT_ClassExpr".to_string(),
                              input:        false,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::INPUT_ClassExpr as RelId,
                              rules:        vec![
                                  /* INPUT_ClassExpr[x] :- ClassExpr[(x: ClassExpr)]. */
                                  Rule::CollectionRule {
                                      description: "INPUT_ClassExpr[x] :- ClassExpr[(x: ClassExpr)].".to_string(),
                                      rel: Relations::ClassExpr as RelId,
                                      xform: Some(XFormCollection::FilterMap{
                                                      description: "head of INPUT_ClassExpr[x] :- ClassExpr[(x: ClassExpr)]." .to_string(),
                                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                      {
                                                          let ref x = match *unsafe {<::types::ClassExpr>::from_ddvalue_ref(&__v) } {
                                                              ref x => (*x).clone(),
                                                              _ => return None
                                                          };
                                                          Some(((*x).clone()).into_ddvalue())
                                                      }
                                                      __f},
                                                      next: Box::new(None)
                                                  })
                                  }],
                              arrangements: vec![
                                  ],
                              change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                          };
    let ConstDecl = Relation {
                        name:         "ConstDecl".to_string(),
                        input:        true,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::ConstDecl as RelId,
                        rules:        vec![
                            ],
                        arrangements: vec![
                            ],
                        change_cb:    None
                    };
    let INPUT_ConstDecl = Relation {
                              name:         "INPUT_ConstDecl".to_string(),
                              input:        false,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::INPUT_ConstDecl as RelId,
                              rules:        vec![
                                  /* INPUT_ConstDecl[x] :- ConstDecl[(x: ConstDecl)]. */
                                  Rule::CollectionRule {
                                      description: "INPUT_ConstDecl[x] :- ConstDecl[(x: ConstDecl)].".to_string(),
                                      rel: Relations::ConstDecl as RelId,
                                      xform: Some(XFormCollection::FilterMap{
                                                      description: "head of INPUT_ConstDecl[x] :- ConstDecl[(x: ConstDecl)]." .to_string(),
                                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                      {
                                                          let ref x = match *unsafe {<::types::ConstDecl>::from_ddvalue_ref(&__v) } {
                                                              ref x => (*x).clone(),
                                                              _ => return None
                                                          };
                                                          Some(((*x).clone()).into_ddvalue())
                                                      }
                                                      __f},
                                                      next: Box::new(None)
                                                  })
                                  }],
                              arrangements: vec![
                                  ],
                              change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                          };
    let Continue = Relation {
                       name:         "Continue".to_string(),
                       input:        true,
                       distinct:     false,
                       caching_mode: CachingMode::Set,
                       key_func:     None,
                       id:           Relations::Continue as RelId,
                       rules:        vec![
                           ],
                       arrangements: vec![
                           ],
                       change_cb:    None
                   };
    let INPUT_Continue = Relation {
                             name:         "INPUT_Continue".to_string(),
                             input:        false,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::INPUT_Continue as RelId,
                             rules:        vec![
                                 /* INPUT_Continue[x] :- Continue[(x: Continue)]. */
                                 Rule::CollectionRule {
                                     description: "INPUT_Continue[x] :- Continue[(x: Continue)].".to_string(),
                                     rel: Relations::Continue as RelId,
                                     xform: Some(XFormCollection::FilterMap{
                                                     description: "head of INPUT_Continue[x] :- Continue[(x: Continue)]." .to_string(),
                                                     fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                     {
                                                         let ref x = match *unsafe {<::types::Continue>::from_ddvalue_ref(&__v) } {
                                                             ref x => (*x).clone(),
                                                             _ => return None
                                                         };
                                                         Some(((*x).clone()).into_ddvalue())
                                                     }
                                                     __f},
                                                     next: Box::new(None)
                                                 })
                                 }],
                             arrangements: vec![
                                 ],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let DoWhile = Relation {
                      name:         "DoWhile".to_string(),
                      input:        true,
                      distinct:     false,
                      caching_mode: CachingMode::Set,
                      key_func:     None,
                      id:           Relations::DoWhile as RelId,
                      rules:        vec![
                          ],
                      arrangements: vec![
                          ],
                      change_cb:    None
                  };
    let INPUT_DoWhile = Relation {
                            name:         "INPUT_DoWhile".to_string(),
                            input:        false,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::INPUT_DoWhile as RelId,
                            rules:        vec![
                                /* INPUT_DoWhile[x] :- DoWhile[(x: DoWhile)]. */
                                Rule::CollectionRule {
                                    description: "INPUT_DoWhile[x] :- DoWhile[(x: DoWhile)].".to_string(),
                                    rel: Relations::DoWhile as RelId,
                                    xform: Some(XFormCollection::FilterMap{
                                                    description: "head of INPUT_DoWhile[x] :- DoWhile[(x: DoWhile)]." .to_string(),
                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                    {
                                                        let ref x = match *unsafe {<::types::DoWhile>::from_ddvalue_ref(&__v) } {
                                                            ref x => (*x).clone(),
                                                            _ => return None
                                                        };
                                                        Some(((*x).clone()).into_ddvalue())
                                                    }
                                                    __f},
                                                    next: Box::new(None)
                                                })
                                }],
                            arrangements: vec![
                                ],
                            change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                        };
    let DotAccess = Relation {
                        name:         "DotAccess".to_string(),
                        input:        true,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::DotAccess as RelId,
                        rules:        vec![
                            ],
                        arrangements: vec![
                            ],
                        change_cb:    None
                    };
    let INPUT_DotAccess = Relation {
                              name:         "INPUT_DotAccess".to_string(),
                              input:        false,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::INPUT_DotAccess as RelId,
                              rules:        vec![
                                  /* INPUT_DotAccess[x] :- DotAccess[(x: DotAccess)]. */
                                  Rule::CollectionRule {
                                      description: "INPUT_DotAccess[x] :- DotAccess[(x: DotAccess)].".to_string(),
                                      rel: Relations::DotAccess as RelId,
                                      xform: Some(XFormCollection::FilterMap{
                                                      description: "head of INPUT_DotAccess[x] :- DotAccess[(x: DotAccess)]." .to_string(),
                                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                      {
                                                          let ref x = match *unsafe {<::types::DotAccess>::from_ddvalue_ref(&__v) } {
                                                              ref x => (*x).clone(),
                                                              _ => return None
                                                          };
                                                          Some(((*x).clone()).into_ddvalue())
                                                      }
                                                      __f},
                                                      next: Box::new(None)
                                                  })
                                  }],
                              arrangements: vec![
                                  ],
                              change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                          };
    let EveryScope = Relation {
                         name:         "EveryScope".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::EveryScope as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"(EveryScope{.scope=(_: Scope)}: EveryScope) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::EveryScope>::from_ddvalue(__v) } {
                                         ::types::EveryScope{scope: _} => Some((()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let INPUT_EveryScope = Relation {
                               name:         "INPUT_EveryScope".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_EveryScope as RelId,
                               rules:        vec![
                                   /* INPUT_EveryScope[x] :- EveryScope[(x: EveryScope)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_EveryScope[x] :- EveryScope[(x: EveryScope)].".to_string(),
                                       rel: Relations::EveryScope as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_EveryScope[x] :- EveryScope[(x: EveryScope)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::EveryScope>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let ExprBigInt = Relation {
                         name:         "ExprBigInt".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::ExprBigInt as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_ExprBigInt = Relation {
                               name:         "INPUT_ExprBigInt".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_ExprBigInt as RelId,
                               rules:        vec![
                                   /* INPUT_ExprBigInt[x] :- ExprBigInt[(x: ExprBigInt)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_ExprBigInt[x] :- ExprBigInt[(x: ExprBigInt)].".to_string(),
                                       rel: Relations::ExprBigInt as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_ExprBigInt[x] :- ExprBigInt[(x: ExprBigInt)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::ExprBigInt>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let ExprBool = Relation {
                       name:         "ExprBool".to_string(),
                       input:        true,
                       distinct:     false,
                       caching_mode: CachingMode::Set,
                       key_func:     None,
                       id:           Relations::ExprBool as RelId,
                       rules:        vec![
                           ],
                       arrangements: vec![
                           ],
                       change_cb:    None
                   };
    let INPUT_ExprBool = Relation {
                             name:         "INPUT_ExprBool".to_string(),
                             input:        false,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::INPUT_ExprBool as RelId,
                             rules:        vec![
                                 /* INPUT_ExprBool[x] :- ExprBool[(x: ExprBool)]. */
                                 Rule::CollectionRule {
                                     description: "INPUT_ExprBool[x] :- ExprBool[(x: ExprBool)].".to_string(),
                                     rel: Relations::ExprBool as RelId,
                                     xform: Some(XFormCollection::FilterMap{
                                                     description: "head of INPUT_ExprBool[x] :- ExprBool[(x: ExprBool)]." .to_string(),
                                                     fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                     {
                                                         let ref x = match *unsafe {<::types::ExprBool>::from_ddvalue_ref(&__v) } {
                                                             ref x => (*x).clone(),
                                                             _ => return None
                                                         };
                                                         Some(((*x).clone()).into_ddvalue())
                                                     }
                                                     __f},
                                                     next: Box::new(None)
                                                 })
                                 }],
                             arrangements: vec![
                                 ],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let ExprNumber = Relation {
                         name:         "ExprNumber".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::ExprNumber as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_ExprNumber = Relation {
                               name:         "INPUT_ExprNumber".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_ExprNumber as RelId,
                               rules:        vec![
                                   /* INPUT_ExprNumber[x] :- ExprNumber[(x: ExprNumber)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_ExprNumber[x] :- ExprNumber[(x: ExprNumber)].".to_string(),
                                       rel: Relations::ExprNumber as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_ExprNumber[x] :- ExprNumber[(x: ExprNumber)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::ExprNumber>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let ExprString = Relation {
                         name:         "ExprString".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::ExprString as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_ExprString = Relation {
                               name:         "INPUT_ExprString".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_ExprString as RelId,
                               rules:        vec![
                                   /* INPUT_ExprString[x] :- ExprString[(x: ExprString)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_ExprString[x] :- ExprString[(x: ExprString)].".to_string(),
                                       rel: Relations::ExprString as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_ExprString[x] :- ExprString[(x: ExprString)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::ExprString>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
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
                                name: r###"(Expression{.id=(_0: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::Expression>::from_ddvalue(__v) } {
                                         ::types::Expression{id: ref _0, kind: ::types::ExprKind::ExprNameRef{}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             },
                             Arrangement::Map{
                                name: r###"(Expression{.id=(_0: ExprId), .kind=(_: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::Expression>::from_ddvalue(__v) } {
                                         ::types::Expression{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             },
                             Arrangement::Map{
                                name: r###"(Expression{.id=(_0: ExprId), .kind=(ExprGrouping{.inner=(ddlog_std::Some{.x=(_: ExprId)}: ddlog_std::Option<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::Expression>::from_ddvalue(__v) } {
                                         ::types::Expression{id: ref _0, kind: ::types::ExprKind::ExprGrouping{inner: ::types::ddlog_std::Option::Some{x: _}}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             },
                             Arrangement::Map{
                                name: r###"(Expression{.id=(_0: ExprId), .kind=(ExprSequence{.exprs=(_: ddlog_std::Vec<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::Expression>::from_ddvalue(__v) } {
                                         ::types::Expression{id: ref _0, kind: ::types::ExprKind::ExprSequence{exprs: _}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let INPUT_Expression = Relation {
                               name:         "INPUT_Expression".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_Expression as RelId,
                               rules:        vec![
                                   /* INPUT_Expression[x] :- Expression[(x: Expression)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_Expression[x] :- Expression[(x: Expression)].".to_string(),
                                       rel: Relations::Expression as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_Expression[x] :- Expression[(x: Expression)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::Expression>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let For = Relation {
                  name:         "For".to_string(),
                  input:        true,
                  distinct:     false,
                  caching_mode: CachingMode::Set,
                  key_func:     None,
                  id:           Relations::For as RelId,
                  rules:        vec![
                      ],
                  arrangements: vec![
                      ],
                  change_cb:    None
              };
    let INPUT_For = Relation {
                        name:         "INPUT_For".to_string(),
                        input:        false,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::INPUT_For as RelId,
                        rules:        vec![
                            /* INPUT_For[x] :- For[(x: For)]. */
                            Rule::CollectionRule {
                                description: "INPUT_For[x] :- For[(x: For)].".to_string(),
                                rel: Relations::For as RelId,
                                xform: Some(XFormCollection::FilterMap{
                                                description: "head of INPUT_For[x] :- For[(x: For)]." .to_string(),
                                                fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                {
                                                    let ref x = match *unsafe {<::types::For>::from_ddvalue_ref(&__v) } {
                                                        ref x => (*x).clone(),
                                                        _ => return None
                                                    };
                                                    Some(((*x).clone()).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(None)
                                            })
                            }],
                        arrangements: vec![
                            ],
                        change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                    };
    let ForIn = Relation {
                    name:         "ForIn".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::ForIn as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_ForIn = Relation {
                          name:         "INPUT_ForIn".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_ForIn as RelId,
                          rules:        vec![
                              /* INPUT_ForIn[x] :- ForIn[(x: ForIn)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_ForIn[x] :- ForIn[(x: ForIn)].".to_string(),
                                  rel: Relations::ForIn as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_ForIn[x] :- ForIn[(x: ForIn)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::ForIn>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
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
                              name: r###"(Function{.id=(_: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(_0: Scope)}: Function) /*join*/"###.to_string(),
                               afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                               {
                                   let __cloned = __v.clone();
                                   match unsafe {< ::types::Function>::from_ddvalue(__v) } {
                                       ::types::Function{id: _, name: _, scope: _, body: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                       _ => None
                                   }.map(|x|(x,__cloned))
                               }
                               __f},
                               queryable: false
                           },
                           Arrangement::Map{
                              name: r###"(Function{.id=(_0: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(_: Scope)}: Function) /*join*/"###.to_string(),
                               afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                               {
                                   let __cloned = __v.clone();
                                   match unsafe {< ::types::Function>::from_ddvalue(__v) } {
                                       ::types::Function{id: ref _0, name: _, scope: _, body: _} => Some(((*_0).clone()).into_ddvalue()),
                                       _ => None
                                   }.map(|x|(x,__cloned))
                               }
                               __f},
                               queryable: false
                           }],
                       change_cb:    None
                   };
    let INPUT_Function = Relation {
                             name:         "INPUT_Function".to_string(),
                             input:        false,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::INPUT_Function as RelId,
                             rules:        vec![
                                 /* INPUT_Function[x] :- Function[(x: Function)]. */
                                 Rule::CollectionRule {
                                     description: "INPUT_Function[x] :- Function[(x: Function)].".to_string(),
                                     rel: Relations::Function as RelId,
                                     xform: Some(XFormCollection::FilterMap{
                                                     description: "head of INPUT_Function[x] :- Function[(x: Function)]." .to_string(),
                                                     fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                     {
                                                         let ref x = match *unsafe {<::types::Function>::from_ddvalue_ref(&__v) } {
                                                             ref x => (*x).clone(),
                                                             _ => return None
                                                         };
                                                         Some(((*x).clone()).into_ddvalue())
                                                     }
                                                     __f},
                                                     next: Box::new(None)
                                                 })
                                 }],
                             arrangements: vec![
                                 ],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let FunctionArg = Relation {
                          name:         "FunctionArg".to_string(),
                          input:        true,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::FunctionArg as RelId,
                          rules:        vec![
                              ],
                          arrangements: vec![
                              ],
                          change_cb:    None
                      };
    let INPUT_FunctionArg = Relation {
                                name:         "INPUT_FunctionArg".to_string(),
                                input:        false,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::INPUT_FunctionArg as RelId,
                                rules:        vec![
                                    /* INPUT_FunctionArg[x] :- FunctionArg[(x: FunctionArg)]. */
                                    Rule::CollectionRule {
                                        description: "INPUT_FunctionArg[x] :- FunctionArg[(x: FunctionArg)].".to_string(),
                                        rel: Relations::FunctionArg as RelId,
                                        xform: Some(XFormCollection::FilterMap{
                                                        description: "head of INPUT_FunctionArg[x] :- FunctionArg[(x: FunctionArg)]." .to_string(),
                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                        {
                                                            let ref x = match *unsafe {<::types::FunctionArg>::from_ddvalue_ref(&__v) } {
                                                                ref x => (*x).clone(),
                                                                _ => return None
                                                            };
                                                            Some(((*x).clone()).into_ddvalue())
                                                        }
                                                        __f},
                                                        next: Box::new(None)
                                                    })
                                    }],
                                arrangements: vec![
                                    ],
                                change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                            };
    let If = Relation {
                 name:         "If".to_string(),
                 input:        true,
                 distinct:     false,
                 caching_mode: CachingMode::Set,
                 key_func:     None,
                 id:           Relations::If as RelId,
                 rules:        vec![
                     ],
                 arrangements: vec![
                     ],
                 change_cb:    None
             };
    let INPUT_If = Relation {
                       name:         "INPUT_If".to_string(),
                       input:        false,
                       distinct:     false,
                       caching_mode: CachingMode::Set,
                       key_func:     None,
                       id:           Relations::INPUT_If as RelId,
                       rules:        vec![
                           /* INPUT_If[x] :- If[(x: If)]. */
                           Rule::CollectionRule {
                               description: "INPUT_If[x] :- If[(x: If)].".to_string(),
                               rel: Relations::If as RelId,
                               xform: Some(XFormCollection::FilterMap{
                                               description: "head of INPUT_If[x] :- If[(x: If)]." .to_string(),
                                               fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                               {
                                                   let ref x = match *unsafe {<::types::If>::from_ddvalue_ref(&__v) } {
                                                       ref x => (*x).clone(),
                                                       _ => return None
                                                   };
                                                   Some(((*x).clone()).into_ddvalue())
                                               }
                                               __f},
                                               next: Box::new(None)
                                           })
                           }],
                       arrangements: vec![
                           ],
                       change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                   };
    let ImplicitGlobal = Relation {
                             name:         "ImplicitGlobal".to_string(),
                             input:        true,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::ImplicitGlobal as RelId,
                             rules:        vec![
                                 ],
                             arrangements: vec![
                                 Arrangement::Map{
                                    name: r###"(ImplicitGlobal{.id=(_: GlobalId), .name=(_: internment::Intern<string>)}: ImplicitGlobal) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::ImplicitGlobal>::from_ddvalue(__v) } {
                                             ::types::ImplicitGlobal{id: _, name: _} => Some((()).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 }],
                             change_cb:    None
                         };
    let INPUT_ImplicitGlobal = Relation {
                                   name:         "INPUT_ImplicitGlobal".to_string(),
                                   input:        false,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::INPUT_ImplicitGlobal as RelId,
                                   rules:        vec![
                                       /* INPUT_ImplicitGlobal[x] :- ImplicitGlobal[(x: ImplicitGlobal)]. */
                                       Rule::CollectionRule {
                                           description: "INPUT_ImplicitGlobal[x] :- ImplicitGlobal[(x: ImplicitGlobal)].".to_string(),
                                           rel: Relations::ImplicitGlobal as RelId,
                                           xform: Some(XFormCollection::FilterMap{
                                                           description: "head of INPUT_ImplicitGlobal[x] :- ImplicitGlobal[(x: ImplicitGlobal)]." .to_string(),
                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                           {
                                                               let ref x = match *unsafe {<::types::ImplicitGlobal>::from_ddvalue_ref(&__v) } {
                                                                   ref x => (*x).clone(),
                                                                   _ => return None
                                                               };
                                                               Some(((*x).clone()).into_ddvalue())
                                                           }
                                                           __f},
                                                           next: Box::new(None)
                                                       })
                                       }],
                                   arrangements: vec![
                                       ],
                                   change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                               };
    let InlineFunc = Relation {
                         name:         "InlineFunc".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::InlineFunc as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"(InlineFunc{.expr_id=(_0: ExprId), .name=(ddlog_std::Some{.x=(_: internment::Intern<string>)}: ddlog_std::Option<Name>), .body=(_: ddlog_std::Option<StmtId>)}: InlineFunc) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::InlineFunc>::from_ddvalue(__v) } {
                                         ::types::InlineFunc{expr_id: ref _0, name: ::types::ddlog_std::Option::Some{x: _}, body: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             },
                             Arrangement::Map{
                                name: r###"(InlineFunc{.expr_id=(_0: ExprId), .name=(_: ddlog_std::Option<Name>), .body=(ddlog_std::Some{.x=(_: StmtId)}: ddlog_std::Option<StmtId>)}: InlineFunc) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::InlineFunc>::from_ddvalue(__v) } {
                                         ::types::InlineFunc{expr_id: ref _0, name: _, body: ::types::ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let INPUT_InlineFunc = Relation {
                               name:         "INPUT_InlineFunc".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_InlineFunc as RelId,
                               rules:        vec![
                                   /* INPUT_InlineFunc[x] :- InlineFunc[(x: InlineFunc)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_InlineFunc[x] :- InlineFunc[(x: InlineFunc)].".to_string(),
                                       rel: Relations::InlineFunc as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_InlineFunc[x] :- InlineFunc[(x: InlineFunc)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::InlineFunc>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let InlineFuncParam = Relation {
                              name:         "InlineFuncParam".to_string(),
                              input:        true,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::InlineFuncParam as RelId,
                              rules:        vec![
                                  ],
                              arrangements: vec![
                                  ],
                              change_cb:    None
                          };
    let INPUT_InlineFuncParam = Relation {
                                    name:         "INPUT_InlineFuncParam".to_string(),
                                    input:        false,
                                    distinct:     false,
                                    caching_mode: CachingMode::Set,
                                    key_func:     None,
                                    id:           Relations::INPUT_InlineFuncParam as RelId,
                                    rules:        vec![
                                        /* INPUT_InlineFuncParam[x] :- InlineFuncParam[(x: InlineFuncParam)]. */
                                        Rule::CollectionRule {
                                            description: "INPUT_InlineFuncParam[x] :- InlineFuncParam[(x: InlineFuncParam)].".to_string(),
                                            rel: Relations::InlineFuncParam as RelId,
                                            xform: Some(XFormCollection::FilterMap{
                                                            description: "head of INPUT_InlineFuncParam[x] :- InlineFuncParam[(x: InlineFuncParam)]." .to_string(),
                                                            fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                            {
                                                                let ref x = match *unsafe {<::types::InlineFuncParam>::from_ddvalue_ref(&__v) } {
                                                                    ref x => (*x).clone(),
                                                                    _ => return None
                                                                };
                                                                Some(((*x).clone()).into_ddvalue())
                                                            }
                                                            __f},
                                                            next: Box::new(None)
                                                        })
                                        }],
                                    arrangements: vec![
                                        ],
                                    change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
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
                                name: r###"(InputScope{.parent=(_: Scope), .child=(_0: Scope)}: InputScope) /*join*/"###.to_string(),
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
                             }],
                         change_cb:    None
                     };
    let ChildScope = Relation {
                         name:         "ChildScope".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::ChildScope as RelId,
                         rules:        vec![
                             /* ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: Scope), .child=(child: Scope)}: InputScope)], (parent != child). */
                             Rule::CollectionRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: Scope), .child=(child: Scope)}: InputScope)], (parent != child).".to_string(),
                                 rel: Relations::InputScope as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: Scope), .child=(child: Scope)}: InputScope)], (parent != child)." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref parent, ref child) = match *unsafe {<::types::InputScope>::from_ddvalue_ref(&__v) } {
                                                         ::types::InputScope{parent: ref parent, child: ref child} => ((*parent).clone(), (*child).clone()),
                                                         _ => return None
                                                     };
                                                     if !((&*parent) != (&*child)) {return None;};
                                                     Some(((::types::ChildScope{parent: (*parent).clone(), child: (*child).clone()})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: Scope), .child=(interum: Scope)}: InputScope)], ChildScope[(ChildScope{.parent=(interum: Scope), .child=(child: Scope)}: ChildScope)], (parent != child). */
                             Rule::ArrangementRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- InputScope[(InputScope{.parent=(parent: Scope), .child=(interum: Scope)}: InputScope)], ChildScope[(ChildScope{.parent=(interum: Scope), .child=(child: Scope)}: ChildScope)], (parent != child).".to_string(),
                                 arr: ( Relations::InputScope as RelId, 0),
                                 xform: XFormArrangement::Join{
                                            description: "InputScope[(InputScope{.parent=(parent: Scope), .child=(interum: Scope)}: InputScope)], ChildScope[(ChildScope{.parent=(interum: Scope), .child=(child: Scope)}: ChildScope)]".to_string(),
                                            ffun: None,
                                            arrangement: (Relations::ChildScope as RelId,0),
                                            jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                            {
                                                let (ref parent, ref interum) = match *unsafe {<::types::InputScope>::from_ddvalue_ref(__v1) } {
                                                    ::types::InputScope{parent: ref parent, child: ref interum} => ((*parent).clone(), (*interum).clone()),
                                                    _ => return None
                                                };
                                                let ref child = match *unsafe {<::types::ChildScope>::from_ddvalue_ref(__v2) } {
                                                    ::types::ChildScope{parent: _, child: ref child} => (*child).clone(),
                                                    _ => return None
                                                };
                                                if !((&*parent) != (&*child)) {return None;};
                                                Some(((::types::ChildScope{parent: (*parent).clone(), child: (*child).clone()})).into_ddvalue())
                                            }
                                            __f},
                                            next: Box::new(None)
                                        }
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"(ChildScope{.parent=(_0: Scope), .child=(_: Scope)}: ChildScope) /*join*/"###.to_string(),
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
                             },
                             Arrangement::Set{
                                 name: r###"(ChildScope{.parent=(_0: Scope), .child=(_1: Scope)}: ChildScope) /*semijoin*/"###.to_string(),
                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                 {
                                     match unsafe {< ::types::ChildScope>::from_ddvalue(__v) } {
                                         ::types::ChildScope{parent: ref _0, child: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                         _ => None
                                     }
                                 }
                                 __f},
                                 distinct: false
                             }],
                         change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                     };
    let ClosestFunction = Relation {
                              name:         "ClosestFunction".to_string(),
                              input:        false,
                              distinct:     true,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::ClosestFunction as RelId,
                              rules:        vec![
                                  /* ClosestFunction[(ClosestFunction{.scope=body, .func=func}: ClosestFunction)] :- Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)]. */
                                  Rule::CollectionRule {
                                      description: "ClosestFunction[(ClosestFunction{.scope=body, .func=func}: ClosestFunction)] :- Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)].".to_string(),
                                      rel: Relations::Function as RelId,
                                      xform: Some(XFormCollection::FilterMap{
                                                      description: "head of ClosestFunction[(ClosestFunction{.scope=body, .func=func}: ClosestFunction)] :- Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)]." .to_string(),
                                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                      {
                                                          let (ref func, ref body) = match *unsafe {<::types::Function>::from_ddvalue_ref(&__v) } {
                                                              ::types::Function{id: ref func, name: _, scope: _, body: ref body} => ((*func).clone(), (*body).clone()),
                                                              _ => return None
                                                          };
                                                          Some(((::types::ClosestFunction{scope: (*body).clone(), func: (*func).clone()})).into_ddvalue())
                                                      }
                                                      __f},
                                                      next: Box::new(None)
                                                  })
                                  },
                                  /* ClosestFunction[(ClosestFunction{.scope=scope, .func=func}: ClosestFunction)] :- Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)], ChildScope[(ChildScope{.parent=(body: Scope), .child=(scope: Scope)}: ChildScope)]. */
                                  Rule::ArrangementRule {
                                      description: "ClosestFunction[(ClosestFunction{.scope=scope, .func=func}: ClosestFunction)] :- Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)], ChildScope[(ChildScope{.parent=(body: Scope), .child=(scope: Scope)}: ChildScope)].".to_string(),
                                      arr: ( Relations::Function as RelId, 0),
                                      xform: XFormArrangement::Join{
                                                 description: "Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)], ChildScope[(ChildScope{.parent=(body: Scope), .child=(scope: Scope)}: ChildScope)]".to_string(),
                                                 ffun: None,
                                                 arrangement: (Relations::ChildScope as RelId,0),
                                                 jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref func, ref body) = match *unsafe {<::types::Function>::from_ddvalue_ref(__v1) } {
                                                         ::types::Function{id: ref func, name: _, scope: _, body: ref body} => ((*func).clone(), (*body).clone()),
                                                         _ => return None
                                                     };
                                                     let ref scope = match *unsafe {<::types::ChildScope>::from_ddvalue_ref(__v2) } {
                                                         ::types::ChildScope{parent: _, child: ref scope} => (*scope).clone(),
                                                         _ => return None
                                                     };
                                                     Some(((::types::ClosestFunction{scope: (*scope).clone(), func: (*func).clone()})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             }
                                  }],
                              arrangements: vec![
                                  Arrangement::Map{
                                     name: r###"(ClosestFunction{.scope=(_0: Scope), .func=(_: FuncId)}: ClosestFunction) /*join*/"###.to_string(),
                                      afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                      {
                                          let __cloned = __v.clone();
                                          match unsafe {< ::types::ClosestFunction>::from_ddvalue(__v) } {
                                              ::types::ClosestFunction{scope: ref _0, func: _} => Some(((*_0).clone()).into_ddvalue()),
                                              _ => None
                                          }.map(|x|(x,__cloned))
                                      }
                                      __f},
                                      queryable: false
                                  },
                                  Arrangement::Set{
                                      name: r###"(ClosestFunction{.scope=(_0: Scope), .func=(_: FuncId)}: ClosestFunction) /*antijoin*/"###.to_string(),
                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                      {
                                          match unsafe {< ::types::ClosestFunction>::from_ddvalue(__v) } {
                                              ::types::ClosestFunction{scope: ref _0, func: _} => Some(((*_0).clone()).into_ddvalue()),
                                              _ => None
                                          }
                                      }
                                      __f},
                                      distinct: true
                                  }],
                              change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                          };
    let INPUT_InputScope = Relation {
                               name:         "INPUT_InputScope".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_InputScope as RelId,
                               rules:        vec![
                                   /* INPUT_InputScope[x] :- InputScope[(x: InputScope)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_InputScope[x] :- InputScope[(x: InputScope)].".to_string(),
                                       rel: Relations::InputScope as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_InputScope[x] :- InputScope[(x: InputScope)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::InputScope>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let Label = Relation {
                    name:         "Label".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Label as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_Label = Relation {
                          name:         "INPUT_Label".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Label as RelId,
                          rules:        vec![
                              /* INPUT_Label[x] :- Label[(x: Label)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Label[x] :- Label[(x: Label)].".to_string(),
                                  rel: Relations::Label as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Label[x] :- Label[(x: Label)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Label>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let LetDecl = Relation {
                      name:         "LetDecl".to_string(),
                      input:        true,
                      distinct:     false,
                      caching_mode: CachingMode::Set,
                      key_func:     None,
                      id:           Relations::LetDecl as RelId,
                      rules:        vec![
                          ],
                      arrangements: vec![
                          ],
                      change_cb:    None
                  };
    let INPUT_LetDecl = Relation {
                            name:         "INPUT_LetDecl".to_string(),
                            input:        false,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::INPUT_LetDecl as RelId,
                            rules:        vec![
                                /* INPUT_LetDecl[x] :- LetDecl[(x: LetDecl)]. */
                                Rule::CollectionRule {
                                    description: "INPUT_LetDecl[x] :- LetDecl[(x: LetDecl)].".to_string(),
                                    rel: Relations::LetDecl as RelId,
                                    xform: Some(XFormCollection::FilterMap{
                                                    description: "head of INPUT_LetDecl[x] :- LetDecl[(x: LetDecl)]." .to_string(),
                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                    {
                                                        let ref x = match *unsafe {<::types::LetDecl>::from_ddvalue_ref(&__v) } {
                                                            ref x => (*x).clone(),
                                                            _ => return None
                                                        };
                                                        Some(((*x).clone()).into_ddvalue())
                                                    }
                                                    __f},
                                                    next: Box::new(None)
                                                })
                                }],
                            arrangements: vec![
                                ],
                            change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                        };
    let NameRef = Relation {
                      name:         "NameRef".to_string(),
                      input:        true,
                      distinct:     false,
                      caching_mode: CachingMode::Set,
                      key_func:     None,
                      id:           Relations::NameRef as RelId,
                      rules:        vec![
                          ],
                      arrangements: vec![
                          Arrangement::Map{
                             name: r###"(NameRef{.expr_id=(_0: ExprId), .value=(_: internment::Intern<string>)}: NameRef) /*join*/"###.to_string(),
                              afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                              {
                                  let __cloned = __v.clone();
                                  match unsafe {< ::types::NameRef>::from_ddvalue(__v) } {
                                      ::types::NameRef{expr_id: ref _0, value: _} => Some(((*_0).clone()).into_ddvalue()),
                                      _ => None
                                  }.map(|x|(x,__cloned))
                              }
                              __f},
                              queryable: false
                          }],
                      change_cb:    None
                  };
    let INPUT_NameRef = Relation {
                            name:         "INPUT_NameRef".to_string(),
                            input:        false,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::INPUT_NameRef as RelId,
                            rules:        vec![
                                /* INPUT_NameRef[x] :- NameRef[(x: NameRef)]. */
                                Rule::CollectionRule {
                                    description: "INPUT_NameRef[x] :- NameRef[(x: NameRef)].".to_string(),
                                    rel: Relations::NameRef as RelId,
                                    xform: Some(XFormCollection::FilterMap{
                                                    description: "head of INPUT_NameRef[x] :- NameRef[(x: NameRef)]." .to_string(),
                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                    {
                                                        let ref x = match *unsafe {<::types::NameRef>::from_ddvalue_ref(&__v) } {
                                                            ref x => (*x).clone(),
                                                            _ => return None
                                                        };
                                                        Some(((*x).clone()).into_ddvalue())
                                                    }
                                                    __f},
                                                    next: Box::new(None)
                                                })
                                }],
                            arrangements: vec![
                                ],
                            change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                        };
    let New = Relation {
                  name:         "New".to_string(),
                  input:        true,
                  distinct:     false,
                  caching_mode: CachingMode::Set,
                  key_func:     None,
                  id:           Relations::New as RelId,
                  rules:        vec![
                      ],
                  arrangements: vec![
                      ],
                  change_cb:    None
              };
    let INPUT_New = Relation {
                        name:         "INPUT_New".to_string(),
                        input:        false,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::INPUT_New as RelId,
                        rules:        vec![
                            /* INPUT_New[x] :- New[(x: New)]. */
                            Rule::CollectionRule {
                                description: "INPUT_New[x] :- New[(x: New)].".to_string(),
                                rel: Relations::New as RelId,
                                xform: Some(XFormCollection::FilterMap{
                                                description: "head of INPUT_New[x] :- New[(x: New)]." .to_string(),
                                                fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                {
                                                    let ref x = match *unsafe {<::types::New>::from_ddvalue_ref(&__v) } {
                                                        ref x => (*x).clone(),
                                                        _ => return None
                                                    };
                                                    Some(((*x).clone()).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(None)
                                            })
                            }],
                        arrangements: vec![
                            ],
                        change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                    };
    let Property = Relation {
                       name:         "Property".to_string(),
                       input:        true,
                       distinct:     false,
                       caching_mode: CachingMode::Set,
                       key_func:     None,
                       id:           Relations::Property as RelId,
                       rules:        vec![
                           ],
                       arrangements: vec![
                           ],
                       change_cb:    None
                   };
    let INPUT_Property = Relation {
                             name:         "INPUT_Property".to_string(),
                             input:        false,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::INPUT_Property as RelId,
                             rules:        vec![
                                 /* INPUT_Property[x] :- Property[(x: Property)]. */
                                 Rule::CollectionRule {
                                     description: "INPUT_Property[x] :- Property[(x: Property)].".to_string(),
                                     rel: Relations::Property as RelId,
                                     xform: Some(XFormCollection::FilterMap{
                                                     description: "head of INPUT_Property[x] :- Property[(x: Property)]." .to_string(),
                                                     fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                     {
                                                         let ref x = match *unsafe {<::types::Property>::from_ddvalue_ref(&__v) } {
                                                             ref x => (*x).clone(),
                                                             _ => return None
                                                         };
                                                         Some(((*x).clone()).into_ddvalue())
                                                     }
                                                     __f},
                                                     next: Box::new(None)
                                                 })
                                 }],
                             arrangements: vec![
                                 ],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let Return = Relation {
                     name:         "Return".to_string(),
                     input:        true,
                     distinct:     false,
                     caching_mode: CachingMode::Set,
                     key_func:     None,
                     id:           Relations::Return as RelId,
                     rules:        vec![
                         ],
                     arrangements: vec![
                         ],
                     change_cb:    None
                 };
    let INPUT_Return = Relation {
                           name:         "INPUT_Return".to_string(),
                           input:        false,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::INPUT_Return as RelId,
                           rules:        vec![
                               /* INPUT_Return[x] :- Return[(x: Return)]. */
                               Rule::CollectionRule {
                                   description: "INPUT_Return[x] :- Return[(x: Return)].".to_string(),
                                   rel: Relations::Return as RelId,
                                   xform: Some(XFormCollection::FilterMap{
                                                   description: "head of INPUT_Return[x] :- Return[(x: Return)]." .to_string(),
                                                   fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                   {
                                                       let ref x = match *unsafe {<::types::Return>::from_ddvalue_ref(&__v) } {
                                                           ref x => (*x).clone(),
                                                           _ => return None
                                                       };
                                                       Some(((*x).clone()).into_ddvalue())
                                                   }
                                                   __f},
                                                   next: Box::new(None)
                                               })
                               }],
                           arrangements: vec![
                               ],
                           change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
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
                               name: r###"(Statement{.id=(_0: StmtId), .kind=(_: StmtKind), .scope=(_: Scope), .span=(_: Span)}: Statement) /*join*/"###.to_string(),
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
                            Arrangement::Map{
                               name: r###"(Statement{.id=(_0: StmtId), .kind=(StmtVarDecl{}: StmtKind), .scope=(_: Scope), .span=(_: Span)}: Statement) /*join*/"###.to_string(),
                                afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                {
                                    let __cloned = __v.clone();
                                    match unsafe {< ::types::Statement>::from_ddvalue(__v) } {
                                        ::types::Statement{id: ref _0, kind: ::types::StmtKind::StmtVarDecl{}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                        _ => None
                                    }.map(|x|(x,__cloned))
                                }
                                __f},
                                queryable: false
                            }],
                        change_cb:    None
                    };
    let INPUT_Statement = Relation {
                              name:         "INPUT_Statement".to_string(),
                              input:        false,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::INPUT_Statement as RelId,
                              rules:        vec![
                                  /* INPUT_Statement[x] :- Statement[(x: Statement)]. */
                                  Rule::CollectionRule {
                                      description: "INPUT_Statement[x] :- Statement[(x: Statement)].".to_string(),
                                      rel: Relations::Statement as RelId,
                                      xform: Some(XFormCollection::FilterMap{
                                                      description: "head of INPUT_Statement[x] :- Statement[(x: Statement)]." .to_string(),
                                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                      {
                                                          let ref x = match *unsafe {<::types::Statement>::from_ddvalue_ref(&__v) } {
                                                              ref x => (*x).clone(),
                                                              _ => return None
                                                          };
                                                          Some(((*x).clone()).into_ddvalue())
                                                      }
                                                      __f},
                                                      next: Box::new(None)
                                                  })
                                  }],
                              arrangements: vec![
                                  ],
                              change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                          };
    let Switch = Relation {
                     name:         "Switch".to_string(),
                     input:        true,
                     distinct:     false,
                     caching_mode: CachingMode::Set,
                     key_func:     None,
                     id:           Relations::Switch as RelId,
                     rules:        vec![
                         ],
                     arrangements: vec![
                         ],
                     change_cb:    None
                 };
    let INPUT_Switch = Relation {
                           name:         "INPUT_Switch".to_string(),
                           input:        false,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::INPUT_Switch as RelId,
                           rules:        vec![
                               /* INPUT_Switch[x] :- Switch[(x: Switch)]. */
                               Rule::CollectionRule {
                                   description: "INPUT_Switch[x] :- Switch[(x: Switch)].".to_string(),
                                   rel: Relations::Switch as RelId,
                                   xform: Some(XFormCollection::FilterMap{
                                                   description: "head of INPUT_Switch[x] :- Switch[(x: Switch)]." .to_string(),
                                                   fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                   {
                                                       let ref x = match *unsafe {<::types::Switch>::from_ddvalue_ref(&__v) } {
                                                           ref x => (*x).clone(),
                                                           _ => return None
                                                       };
                                                       Some(((*x).clone()).into_ddvalue())
                                                   }
                                                   __f},
                                                   next: Box::new(None)
                                               })
                               }],
                           arrangements: vec![
                               ],
                           change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                       };
    let SwitchCase = Relation {
                         name:         "SwitchCase".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::SwitchCase as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_SwitchCase = Relation {
                               name:         "INPUT_SwitchCase".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_SwitchCase as RelId,
                               rules:        vec![
                                   /* INPUT_SwitchCase[x] :- SwitchCase[(x: SwitchCase)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_SwitchCase[x] :- SwitchCase[(x: SwitchCase)].".to_string(),
                                       rel: Relations::SwitchCase as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_SwitchCase[x] :- SwitchCase[(x: SwitchCase)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::SwitchCase>::from_ddvalue_ref(&__v) } {
                                                               ref x => (*x).clone(),
                                                               _ => return None
                                                           };
                                                           Some(((*x).clone()).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   }],
                               arrangements: vec![
                                   ],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let Template = Relation {
                       name:         "Template".to_string(),
                       input:        true,
                       distinct:     false,
                       caching_mode: CachingMode::Set,
                       key_func:     None,
                       id:           Relations::Template as RelId,
                       rules:        vec![
                           ],
                       arrangements: vec![
                           ],
                       change_cb:    None
                   };
    let INPUT_Template = Relation {
                             name:         "INPUT_Template".to_string(),
                             input:        false,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::INPUT_Template as RelId,
                             rules:        vec![
                                 /* INPUT_Template[x] :- Template[(x: Template)]. */
                                 Rule::CollectionRule {
                                     description: "INPUT_Template[x] :- Template[(x: Template)].".to_string(),
                                     rel: Relations::Template as RelId,
                                     xform: Some(XFormCollection::FilterMap{
                                                     description: "head of INPUT_Template[x] :- Template[(x: Template)]." .to_string(),
                                                     fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                     {
                                                         let ref x = match *unsafe {<::types::Template>::from_ddvalue_ref(&__v) } {
                                                             ref x => (*x).clone(),
                                                             _ => return None
                                                         };
                                                         Some(((*x).clone()).into_ddvalue())
                                                     }
                                                     __f},
                                                     next: Box::new(None)
                                                 })
                                 }],
                             arrangements: vec![
                                 ],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let Ternary = Relation {
                      name:         "Ternary".to_string(),
                      input:        true,
                      distinct:     false,
                      caching_mode: CachingMode::Set,
                      key_func:     None,
                      id:           Relations::Ternary as RelId,
                      rules:        vec![
                          ],
                      arrangements: vec![
                          ],
                      change_cb:    None
                  };
    let INPUT_Ternary = Relation {
                            name:         "INPUT_Ternary".to_string(),
                            input:        false,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::INPUT_Ternary as RelId,
                            rules:        vec![
                                /* INPUT_Ternary[x] :- Ternary[(x: Ternary)]. */
                                Rule::CollectionRule {
                                    description: "INPUT_Ternary[x] :- Ternary[(x: Ternary)].".to_string(),
                                    rel: Relations::Ternary as RelId,
                                    xform: Some(XFormCollection::FilterMap{
                                                    description: "head of INPUT_Ternary[x] :- Ternary[(x: Ternary)]." .to_string(),
                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                    {
                                                        let ref x = match *unsafe {<::types::Ternary>::from_ddvalue_ref(&__v) } {
                                                            ref x => (*x).clone(),
                                                            _ => return None
                                                        };
                                                        Some(((*x).clone()).into_ddvalue())
                                                    }
                                                    __f},
                                                    next: Box::new(None)
                                                })
                                }],
                            arrangements: vec![
                                ],
                            change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                        };
    let Throw = Relation {
                    name:         "Throw".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Throw as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_Throw = Relation {
                          name:         "INPUT_Throw".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Throw as RelId,
                          rules:        vec![
                              /* INPUT_Throw[x] :- Throw[(x: Throw)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Throw[x] :- Throw[(x: Throw)].".to_string(),
                                  rel: Relations::Throw as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Throw[x] :- Throw[(x: Throw)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Throw>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let Try = Relation {
                  name:         "Try".to_string(),
                  input:        true,
                  distinct:     false,
                  caching_mode: CachingMode::Set,
                  key_func:     None,
                  id:           Relations::Try as RelId,
                  rules:        vec![
                      ],
                  arrangements: vec![
                      ],
                  change_cb:    None
              };
    let INPUT_Try = Relation {
                        name:         "INPUT_Try".to_string(),
                        input:        false,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::INPUT_Try as RelId,
                        rules:        vec![
                            /* INPUT_Try[x] :- Try[(x: Try)]. */
                            Rule::CollectionRule {
                                description: "INPUT_Try[x] :- Try[(x: Try)].".to_string(),
                                rel: Relations::Try as RelId,
                                xform: Some(XFormCollection::FilterMap{
                                                description: "head of INPUT_Try[x] :- Try[(x: Try)]." .to_string(),
                                                fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                {
                                                    let ref x = match *unsafe {<::types::Try>::from_ddvalue_ref(&__v) } {
                                                        ref x => (*x).clone(),
                                                        _ => return None
                                                    };
                                                    Some(((*x).clone()).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(None)
                                            })
                            }],
                        arrangements: vec![
                            ],
                        change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                    };
    let UnaryOp = Relation {
                      name:         "UnaryOp".to_string(),
                      input:        true,
                      distinct:     false,
                      caching_mode: CachingMode::Set,
                      key_func:     None,
                      id:           Relations::UnaryOp as RelId,
                      rules:        vec![
                          ],
                      arrangements: vec![
                          ],
                      change_cb:    None
                  };
    let WithinTypeOf = Relation {
                           name:         "WithinTypeOf".to_string(),
                           input:        false,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::WithinTypeOf as RelId,
                           rules:        vec![
                               /* WithinTypeOf[(WithinTypeOf{.type_of=type_of, .expr=expr}: WithinTypeOf)] :- UnaryOp[(UnaryOp{.expr_id=(type_of: ExprId), .op=(ddlog_std::Some{.x=(UnaryTypeof{}: UnaryOperand)}: ddlog_std::Option<UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ExprId)}: ddlog_std::Option<ExprId>)}: UnaryOp)]. */
                               Rule::CollectionRule {
                                   description: "WithinTypeOf[(WithinTypeOf{.type_of=type_of, .expr=expr}: WithinTypeOf)] :- UnaryOp[(UnaryOp{.expr_id=(type_of: ExprId), .op=(ddlog_std::Some{.x=(UnaryTypeof{}: UnaryOperand)}: ddlog_std::Option<UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ExprId)}: ddlog_std::Option<ExprId>)}: UnaryOp)].".to_string(),
                                   rel: Relations::UnaryOp as RelId,
                                   xform: Some(XFormCollection::FilterMap{
                                                   description: "head of WithinTypeOf[(WithinTypeOf{.type_of=type_of, .expr=expr}: WithinTypeOf)] :- UnaryOp[(UnaryOp{.expr_id=(type_of: ExprId), .op=(ddlog_std::Some{.x=(UnaryTypeof{}: UnaryOperand)}: ddlog_std::Option<UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ExprId)}: ddlog_std::Option<ExprId>)}: UnaryOp)]." .to_string(),
                                                   fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                   {
                                                       let (ref type_of, ref expr) = match *unsafe {<::types::UnaryOp>::from_ddvalue_ref(&__v) } {
                                                           ::types::UnaryOp{expr_id: ref type_of, op: ::types::ddlog_std::Option::Some{x: ::types::UnaryOperand::UnaryTypeof{}}, expr: ::types::ddlog_std::Option::Some{x: ref expr}} => ((*type_of).clone(), (*expr).clone()),
                                                           _ => return None
                                                       };
                                                       Some(((::types::WithinTypeOf{type_of: (*type_of).clone(), expr: (*expr).clone()})).into_ddvalue())
                                                   }
                                                   __f},
                                                   next: Box::new(None)
                                               })
                               },
                               /* WithinTypeOf[(WithinTypeOf{.type_of=type_of, .expr=grouped}: WithinTypeOf)] :- WithinTypeOf[(WithinTypeOf{.type_of=(type_of: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ExprId)}: ddlog_std::Option<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression)]. */
                               Rule::ArrangementRule {
                                   description: "WithinTypeOf[(WithinTypeOf{.type_of=type_of, .expr=grouped}: WithinTypeOf)] :- WithinTypeOf[(WithinTypeOf{.type_of=(type_of: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ExprId)}: ddlog_std::Option<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression)].".to_string(),
                                   arr: ( Relations::WithinTypeOf as RelId, 1),
                                   xform: XFormArrangement::Join{
                                              description: "WithinTypeOf[(WithinTypeOf{.type_of=(type_of: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ExprId)}: ddlog_std::Option<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression)]".to_string(),
                                              ffun: None,
                                              arrangement: (Relations::Expression as RelId,2),
                                              jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                              {
                                                  let (ref type_of, ref expr) = match *unsafe {<::types::WithinTypeOf>::from_ddvalue_ref(__v1) } {
                                                      ::types::WithinTypeOf{type_of: ref type_of, expr: ref expr} => ((*type_of).clone(), (*expr).clone()),
                                                      _ => return None
                                                  };
                                                  let ref grouped = match *unsafe {<::types::Expression>::from_ddvalue_ref(__v2) } {
                                                      ::types::Expression{id: _, kind: ::types::ExprKind::ExprGrouping{inner: ::types::ddlog_std::Option::Some{x: ref grouped}}, scope: _, span: _} => (*grouped).clone(),
                                                      _ => return None
                                                  };
                                                  Some(((::types::WithinTypeOf{type_of: (*type_of).clone(), expr: (*grouped).clone()})).into_ddvalue())
                                              }
                                              __f},
                                              next: Box::new(None)
                                          }
                               },
                               /* WithinTypeOf[(WithinTypeOf{.type_of=type_of, .expr=last}: WithinTypeOf)] :- WithinTypeOf[(WithinTypeOf{.type_of=(type_of: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprSequence{.exprs=(sequence: ddlog_std::Vec<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression)], ((ddlog_std::Some{.x=(var last: ExprId)}: ddlog_std::Option<ExprId>) = ((last: function(ddlog_std::Vec<ExprId>):ddlog_std::Option<ExprId>)(sequence))). */
                               Rule::ArrangementRule {
                                   description: "WithinTypeOf[(WithinTypeOf{.type_of=type_of, .expr=last}: WithinTypeOf)] :- WithinTypeOf[(WithinTypeOf{.type_of=(type_of: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprSequence{.exprs=(sequence: ddlog_std::Vec<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression)], ((ddlog_std::Some{.x=(var last: ExprId)}: ddlog_std::Option<ExprId>) = ((last: function(ddlog_std::Vec<ExprId>):ddlog_std::Option<ExprId>)(sequence))).".to_string(),
                                   arr: ( Relations::WithinTypeOf as RelId, 1),
                                   xform: XFormArrangement::Join{
                                              description: "WithinTypeOf[(WithinTypeOf{.type_of=(type_of: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprSequence{.exprs=(sequence: ddlog_std::Vec<ExprId>)}: ExprKind), .scope=(_: Scope), .span=(_: Span)}: Expression)]".to_string(),
                                              ffun: None,
                                              arrangement: (Relations::Expression as RelId,3),
                                              jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                              {
                                                  let (ref type_of, ref expr) = match *unsafe {<::types::WithinTypeOf>::from_ddvalue_ref(__v1) } {
                                                      ::types::WithinTypeOf{type_of: ref type_of, expr: ref expr} => ((*type_of).clone(), (*expr).clone()),
                                                      _ => return None
                                                  };
                                                  let ref sequence = match *unsafe {<::types::Expression>::from_ddvalue_ref(__v2) } {
                                                      ::types::Expression{id: _, kind: ::types::ExprKind::ExprSequence{exprs: ref sequence}, scope: _, span: _} => (*sequence).clone(),
                                                      _ => return None
                                                  };
                                                  let ref last: ::types::ExprId = match ::types::last::<::types::ExprId>(sequence) {
                                                      ::types::ddlog_std::Option::Some{x: last} => last,
                                                      _ => return None
                                                  };
                                                  Some(((::types::WithinTypeOf{type_of: (*type_of).clone(), expr: (*last).clone()})).into_ddvalue())
                                              }
                                              __f},
                                              next: Box::new(None)
                                          }
                               }],
                           arrangements: vec![
                               Arrangement::Set{
                                   name: r###"(WithinTypeOf{.type_of=(_: ExprId), .expr=(_0: ExprId)}: WithinTypeOf) /*antijoin*/"###.to_string(),
                                   fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                   {
                                       match unsafe {< ::types::WithinTypeOf>::from_ddvalue(__v) } {
                                           ::types::WithinTypeOf{type_of: _, expr: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                           _ => None
                                       }
                                   }
                                   __f},
                                   distinct: true
                               },
                               Arrangement::Map{
                                  name: r###"(WithinTypeOf{.type_of=(_: ExprId), .expr=(_0: ExprId)}: WithinTypeOf) /*join*/"###.to_string(),
                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                   {
                                       let __cloned = __v.clone();
                                       match unsafe {< ::types::WithinTypeOf>::from_ddvalue(__v) } {
                                           ::types::WithinTypeOf{type_of: _, expr: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                           _ => None
                                       }.map(|x|(x,__cloned))
                                   }
                                   __f},
                                   queryable: false
                               }],
                           change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                       };
    let INPUT_UnaryOp = Relation {
                            name:         "INPUT_UnaryOp".to_string(),
                            input:        false,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::INPUT_UnaryOp as RelId,
                            rules:        vec![
                                /* INPUT_UnaryOp[x] :- UnaryOp[(x: UnaryOp)]. */
                                Rule::CollectionRule {
                                    description: "INPUT_UnaryOp[x] :- UnaryOp[(x: UnaryOp)].".to_string(),
                                    rel: Relations::UnaryOp as RelId,
                                    xform: Some(XFormCollection::FilterMap{
                                                    description: "head of INPUT_UnaryOp[x] :- UnaryOp[(x: UnaryOp)]." .to_string(),
                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                    {
                                                        let ref x = match *unsafe {<::types::UnaryOp>::from_ddvalue_ref(&__v) } {
                                                            ref x => (*x).clone(),
                                                            _ => return None
                                                        };
                                                        Some(((*x).clone()).into_ddvalue())
                                                    }
                                                    __f},
                                                    next: Box::new(None)
                                                })
                                }],
                            arrangements: vec![
                                ],
                            change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                        };
    let VarDecl = Relation {
                      name:         "VarDecl".to_string(),
                      input:        true,
                      distinct:     false,
                      caching_mode: CachingMode::Set,
                      key_func:     None,
                      id:           Relations::VarDecl as RelId,
                      rules:        vec![
                          ],
                      arrangements: vec![
                          ],
                      change_cb:    None
                  };
    let INPUT_VarDecl = Relation {
                            name:         "INPUT_VarDecl".to_string(),
                            input:        false,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::INPUT_VarDecl as RelId,
                            rules:        vec![
                                /* INPUT_VarDecl[x] :- VarDecl[(x: VarDecl)]. */
                                Rule::CollectionRule {
                                    description: "INPUT_VarDecl[x] :- VarDecl[(x: VarDecl)].".to_string(),
                                    rel: Relations::VarDecl as RelId,
                                    xform: Some(XFormCollection::FilterMap{
                                                    description: "head of INPUT_VarDecl[x] :- VarDecl[(x: VarDecl)]." .to_string(),
                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                    {
                                                        let ref x = match *unsafe {<::types::VarDecl>::from_ddvalue_ref(&__v) } {
                                                            ref x => (*x).clone(),
                                                            _ => return None
                                                        };
                                                        Some(((*x).clone()).into_ddvalue())
                                                    }
                                                    __f},
                                                    next: Box::new(None)
                                                })
                                }],
                            arrangements: vec![
                                ],
                            change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                        };
    let __Prefix_1 = Relation {
                         name:         "__Prefix_1".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::__Prefix_1 as RelId,
                         rules:        vec![
                             /* __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))] :- VarDecl[(VarDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))). */
                             Rule::CollectionRule {
                                 description: "__Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))] :- VarDecl[(VarDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))).".to_string(),
                                 rel: Relations::VarDecl as RelId,
                                 xform: Some(XFormCollection::FlatMap{
                                                 description: "VarDecl[(VarDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))" .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                 {
                                                     let (ref stmt, ref pat) = match *unsafe {<::types::VarDecl>::from_ddvalue_ref(&__v) } {
                                                         ::types::VarDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                         _ => return None
                                                     };
                                                     let __flattened = ::types::bound_vars_internment_Intern__Pattern_ddlog_std_Vec__internment_Intern____Stringval(pat);
                                                     let stmt = (*stmt).clone();
                                                     let pat = (*pat).clone();
                                                     Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple3(name.clone(), stmt.clone(), pat.clone())).into_ddvalue())))
                                                 }
                                                 __f},
                                                 next: Box::new(Some(XFormCollection::FilterMap{
                                                                         description: "head of __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))] :- VarDecl[(VarDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: VarDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))." .to_string(),
                                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                         {
                                                                             let ::types::ddlog_std::tuple3(ref name, ref stmt, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue_ref( &__v ) };
                                                                             Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*pat).clone())).into_ddvalue())
                                                                         }
                                                                         __f},
                                                                         next: Box::new(None)
                                                                     }))
                                             })
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"((_: Name), (_0: StmtId), (_: internment::Intern<Pattern>)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue(__v) } {
                                         ::types::ddlog_std::tuple3(_, ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let NameInScope = Relation {
                          name:         "NameInScope".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::NameInScope as RelId,
                          rules:        vec![
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdGlobal{.global=global}: AnyId)}: NameInScope)] :- ImplicitGlobal[(ImplicitGlobal{.id=(global: GlobalId), .name=(name: internment::Intern<string>)}: ImplicitGlobal)], EveryScope[(EveryScope{.scope=(scope: Scope)}: EveryScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdGlobal{.global=global}: AnyId)}: NameInScope)] :- ImplicitGlobal[(ImplicitGlobal{.id=(global: GlobalId), .name=(name: internment::Intern<string>)}: ImplicitGlobal)], EveryScope[(EveryScope{.scope=(scope: Scope)}: EveryScope)].".to_string(),
                                  arr: ( Relations::ImplicitGlobal as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "ImplicitGlobal[(ImplicitGlobal{.id=(global: GlobalId), .name=(name: internment::Intern<string>)}: ImplicitGlobal)], EveryScope[(EveryScope{.scope=(scope: Scope)}: EveryScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::EveryScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref global, ref name) = match *unsafe {<::types::ImplicitGlobal>::from_ddvalue_ref(__v1) } {
                                                     ::types::ImplicitGlobal{id: ref global, name: ref name} => ((*global).clone(), (*name).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::EveryScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::EveryScope{scope: ref scope} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdGlobal{global: (*global).clone()})})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdClass{.class=class}: AnyId)}: NameInScope)] :- Class[(Class{.id=(class: ClassId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .parent=(_: ddlog_std::Option<ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<IClassElement>>), .scope=(scope: Scope)}: Class)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdClass{.class=class}: AnyId)}: NameInScope)] :- Class[(Class{.id=(class: ClassId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .parent=(_: ddlog_std::Option<ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<IClassElement>>), .scope=(scope: Scope)}: Class)].".to_string(),
                                  rel: Relations::Class as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdClass{.class=class}: AnyId)}: NameInScope)] :- Class[(Class{.id=(class: ClassId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .parent=(_: ddlog_std::Option<ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<IClassElement>>), .scope=(scope: Scope)}: Class)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref class, ref name, ref scope) = match *unsafe {<::types::Class>::from_ddvalue_ref(&__v) } {
                                                          ::types::Class{id: ref class, name: ::types::ddlog_std::Option::Some{x: ref name}, parent: _, elements: _, scope: ref scope} => ((*class).clone(), (*name).clone(), (*scope).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdClass{class: (*class).clone()})})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- LetDecl[(LetDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- LetDecl[(LetDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)].".to_string(),
                                  rel: Relations::LetDecl as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "LetDecl[(LetDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref stmt, ref pat) = match *unsafe {<::types::LetDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::LetDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::bound_vars_internment_Intern__Pattern_ddlog_std_Vec__internment_Intern____Stringval(pat);
                                                      let stmt = (*stmt).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), stmt.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange LetDecl[(LetDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))) by (stmt)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "LetDecl[(LetDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: LetDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::Statement as RelId,0),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- ConstDecl[(ConstDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- ConstDecl[(ConstDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)].".to_string(),
                                  rel: Relations::ConstDecl as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "ConstDecl[(ConstDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref stmt, ref pat) = match *unsafe {<::types::ConstDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::ConstDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::bound_vars_internment_Intern__Pattern_ddlog_std_Vec__internment_Intern____Stringval(pat);
                                                      let stmt = (*stmt).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), stmt.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange ConstDecl[(ConstDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))) by (stmt)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "ConstDecl[(ConstDecl{.stmt_id=(stmt: StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<Pattern>)}: ddlog_std::Option<IPattern>), .value=(_: ddlog_std::Option<ExprId>)}: ConstDecl)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::Statement as RelId,0),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(decl_scope: Scope), .span=(_: Span)}: Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: Scope), .func=(func: FuncId)}: ClosestFunction)], Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(scope: Scope)}: Function)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(decl_scope: Scope), .span=(_: Span)}: Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: Scope), .func=(func: FuncId)}: ClosestFunction)], Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(scope: Scope)}: Function)].".to_string(),
                                  arr: ( Relations::__Prefix_1 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(decl_scope: Scope), .span=(_: Span)}: Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref stmt, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref stmt, ref pat) => ((*name).clone(), (*stmt).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref decl_scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::Statement{id: _, kind: _, scope: ref decl_scope, span: _} => (*decl_scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*decl_scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(decl_scope: Scope), .span=(_: Span)}: Statement)] by (decl_scope)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref stmt, ref decl_scope) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::Scope>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*decl_scope).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "__Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(decl_scope: Scope), .span=(_: Span)}: Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: Scope), .func=(func: FuncId)}: ClosestFunction)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::ClosestFunction as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref func = match *unsafe {<::types::ClosestFunction>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::ClosestFunction{scope: _, func: ref func} => (*func).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*func).clone())).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                description: "arrange __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(decl_scope: Scope), .span=(_: Span)}: Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: Scope), .func=(func: FuncId)}: ClosestFunction)] by (func)" .to_string(),
                                                                                                                afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple3(ref name, ref stmt, ref func) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::FuncId>>::from_ddvalue_ref( &__v ) };
                                                                                                                    Some((((*func).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(XFormArrangement::Join{
                                                                                                                                   description: "__Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(decl_scope: Scope), .span=(_: Span)}: Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: Scope), .func=(func: FuncId)}: ClosestFunction)], Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(scope: Scope)}: Function)]".to_string(),
                                                                                                                                   ffun: None,
                                                                                                                                   arrangement: (Relations::Function as RelId,1),
                                                                                                                                   jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                   {
                                                                                                                                       let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                       let ref scope = match *unsafe {<::types::Function>::from_ddvalue_ref(__v2) } {
                                                                                                                                           ::types::Function{id: _, name: _, scope: _, body: ref scope} => (*scope).clone(),
                                                                                                                                           _ => return None
                                                                                                                                       };
                                                                                                                                       Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                                                                   }
                                                                                                                                   __f},
                                                                                                                                   next: Box::new(None)
                                                                                                                               })
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: Scope), .func=(_: FuncId)}: ClosestFunction)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: Scope), .func=(_: FuncId)}: ClosestFunction)].".to_string(),
                                  arr: ( Relations::__Prefix_1 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref stmt, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref stmt, ref pat) => ((*name).clone(), (*stmt).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)] by (scope)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref stmt, ref scope) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::Scope>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*scope).clone()).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*scope).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Antijoin {
                                                                                        description: "__Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: Scope), .func=(_: FuncId)}: ClosestFunction)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::ClosestFunction as RelId,1),
                                                                                        next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                description: "head of NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdStmt{.stmt=stmt}: AnyId)}: NameInScope)] :- __Prefix_1[((name: Name), (stmt: StmtId), (pat: internment::Intern<Pattern>))], Statement[(Statement{.id=(stmt: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: Scope), .func=(_: FuncId)}: ClosestFunction)]." .to_string(),
                                                                                                                fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple3(ref name, ref stmt, ref scope) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::StmtId, ::types::Scope>>::from_ddvalue_ref( &__v ) };
                                                                                                                    Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(None)
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdFunc{.func=func}: AnyId)}: NameInScope)] :- Function[(Function{.id=(func: FuncId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .scope=(scope: Scope), .body=(_: Scope)}: Function)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdFunc{.func=func}: AnyId)}: NameInScope)] :- Function[(Function{.id=(func: FuncId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .scope=(scope: Scope), .body=(_: Scope)}: Function)].".to_string(),
                                  rel: Relations::Function as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdFunc{.func=func}: AnyId)}: NameInScope)] :- Function[(Function{.id=(func: FuncId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .scope=(scope: Scope), .body=(_: Scope)}: Function)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref func, ref name, ref scope) = match *unsafe {<::types::Function>::from_ddvalue_ref(&__v) } {
                                                          ::types::Function{id: ref func, name: ::types::ddlog_std::Option::Some{x: ref name}, scope: ref scope, body: _} => ((*func).clone(), (*name).clone(), (*scope).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdFunc{func: (*func).clone()})})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=body, .declared_in=(AnyIdFunc{.func=func}: AnyId)}: NameInScope)] :- FunctionArg[(FunctionArg{.parent_func=(func: FuncId), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=body, .declared_in=(AnyIdFunc{.func=func}: AnyId)}: NameInScope)] :- FunctionArg[(FunctionArg{.parent_func=(func: FuncId), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)].".to_string(),
                                  rel: Relations::FunctionArg as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "FunctionArg[(FunctionArg{.parent_func=(func: FuncId), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref func, ref pat) = match *unsafe {<::types::FunctionArg>::from_ddvalue_ref(&__v) } {
                                                          ::types::FunctionArg{parent_func: ref func, pattern: ref pat} => ((*func).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::bound_vars_internment_Intern__Pattern_ddlog_std_Vec__internment_Intern____Stringval(pat);
                                                      let func = (*func).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), func.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange FunctionArg[(FunctionArg{.parent_func=(func: FuncId), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))) by (func)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref func) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::FuncId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*func).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*func).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "FunctionArg[(FunctionArg{.parent_func=(func: FuncId), .pattern=(pat: internment::Intern<Pattern>)}: FunctionArg)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), Function[(Function{.id=(func: FuncId), .name=(_: ddlog_std::Option<Name>), .scope=(_: Scope), .body=(body: Scope)}: Function)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::Function as RelId,1),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref func) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::FuncId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref body = match *unsafe {<::types::Function>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::Function{id: _, name: _, scope: _, body: ref body} => (*body).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*body).clone(), declared_in: (::types::AnyId::AnyIdFunc{func: (*func).clone()})})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ExprId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)], Expression[(Expression{.id=(expr_body: ExprId), .kind=(_: ExprKind), .scope=(scope: Scope), .span=(_: Span)}: Expression)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ExprId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)], Expression[(Expression{.id=(expr_body: ExprId), .kind=(_: ExprKind), .scope=(scope: Scope), .span=(_: Span)}: Expression)].".to_string(),
                                  arr: ( Relations::__Prefix_0 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ExprId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::Arrow as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref expr, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref expr, ref pat) => ((*name).clone(), (*expr).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref expr_body = match *unsafe {<::types::Arrow>::from_ddvalue_ref(__v2) } {
                                                     ::types::Arrow{expr_id: _, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: ref expr_body}}} => (*expr_body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*expr_body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ExprId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)] by (expr_body)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref expr, ref expr_body) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*expr_body).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "__Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ExprId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)], Expression[(Expression{.id=(expr_body: ExprId), .kind=(_: ExprKind), .scope=(scope: Scope), .span=(_: Span)}: Expression)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::Expression as RelId,1),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref scope = match *unsafe {<::types::Expression>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(None)
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: StmtId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)], Statement[(Statement{.id=(stmt_body: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: StmtId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)], Statement[(Statement{.id=(stmt_body: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)].".to_string(),
                                  arr: ( Relations::__Prefix_0 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: StmtId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::Arrow as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref expr, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::internment::Intern<::types::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref expr, ref pat) => ((*name).clone(), (*expr).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref stmt_body = match *unsafe {<::types::Arrow>::from_ddvalue_ref(__v2) } {
                                                     ::types::Arrow{expr_id: _, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Right{r: ref stmt_body}}} => (*stmt_body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*stmt_body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: StmtId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)] by (stmt_body)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref expr, ref stmt_body) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*stmt_body).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "__Prefix_0[((name: Name), (expr: ExprId), (pat: internment::Intern<Pattern>))], Arrow[(Arrow{.expr_id=(expr: ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: StmtId)}: ddlog_std::Either<ExprId,StmtId>)}: ddlog_std::Option<ddlog_std::Either<ExprId,StmtId>>)}: Arrow)], Statement[(Statement{.id=(stmt_body: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::Statement as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(None)
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .body=(_: ddlog_std::Option<StmtId>)}: InlineFunc)], Expression[(Expression{.id=(expr: ExprId), .kind=(_: ExprKind), .scope=(scope: Scope), .span=(_: Span)}: Expression)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .body=(_: ddlog_std::Option<StmtId>)}: InlineFunc)], Expression[(Expression{.id=(expr: ExprId), .kind=(_: ExprKind), .scope=(scope: Scope), .span=(_: Span)}: Expression)].".to_string(),
                                  arr: ( Relations::InlineFunc as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(ddlog_std::Some{.x=(name: internment::Intern<string>)}: ddlog_std::Option<Name>), .body=(_: ddlog_std::Option<StmtId>)}: InlineFunc)], Expression[(Expression{.id=(expr: ExprId), .kind=(_: ExprKind), .scope=(scope: Scope), .span=(_: Span)}: Expression)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::Expression as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref expr, ref name) = match *unsafe {<::types::InlineFunc>::from_ddvalue_ref(__v1) } {
                                                     ::types::InlineFunc{expr_id: ref expr, name: ::types::ddlog_std::Option::Some{x: ref name}, body: _} => ((*expr).clone(), (*name).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::Expression>::from_ddvalue_ref(__v2) } {
                                                     ::types::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- InlineFuncParam[(InlineFuncParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: InlineFuncParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(_: ddlog_std::Option<Name>), .body=(ddlog_std::Some{.x=(body: StmtId)}: ddlog_std::Option<StmtId>)}: InlineFunc)], Statement[(Statement{.id=(body: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=(AnyIdExpr{.expr=expr}: AnyId)}: NameInScope)] :- InlineFuncParam[(InlineFuncParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: InlineFuncParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(_: ddlog_std::Option<Name>), .body=(ddlog_std::Some{.x=(body: StmtId)}: ddlog_std::Option<StmtId>)}: InlineFunc)], Statement[(Statement{.id=(body: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)].".to_string(),
                                  rel: Relations::InlineFuncParam as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "InlineFuncParam[(InlineFuncParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: InlineFuncParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref expr, ref pat) = match *unsafe {<::types::InlineFuncParam>::from_ddvalue_ref(&__v) } {
                                                          ::types::InlineFuncParam{expr_id: ref expr, param: ref pat} => ((*expr).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::bound_vars_internment_Intern__Pattern_ddlog_std_Vec__internment_Intern____Stringval(pat);
                                                      let expr = (*expr).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), expr.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange InlineFuncParam[(InlineFuncParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: InlineFuncParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))) by (expr)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*expr).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "InlineFuncParam[(InlineFuncParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: InlineFuncParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(_: ddlog_std::Option<Name>), .body=(ddlog_std::Some{.x=(body: StmtId)}: ddlog_std::Option<StmtId>)}: InlineFunc)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::InlineFunc as RelId,1),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref body = match *unsafe {<::types::InlineFunc>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::InlineFunc{expr_id: _, name: _, body: ::types::ddlog_std::Option::Some{x: ref body}} => (*body).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*body).clone())).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                     description: "arrange InlineFuncParam[(InlineFuncParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: InlineFuncParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(_: ddlog_std::Option<Name>), .body=(ddlog_std::Some{.x=(body: StmtId)}: ddlog_std::Option<StmtId>)}: InlineFunc)] by (body)" .to_string(),
                                                                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let ::types::ddlog_std::tuple3(ref name, ref expr, ref body) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ExprId, ::types::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                                                                         Some((((*body).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     next: Box::new(XFormArrangement::Join{
                                                                                                                                        description: "InlineFuncParam[(InlineFuncParam{.expr_id=(expr: ExprId), .param=(pat: internment::Intern<Pattern>)}: InlineFuncParam)], var name = FlatMap(((bound_vars: function(internment::Intern<Pattern>):ddlog_std::Vec<Name>)(pat))), InlineFunc[(InlineFunc{.expr_id=(expr: ExprId), .name=(_: ddlog_std::Option<Name>), .body=(ddlog_std::Some{.x=(body: StmtId)}: ddlog_std::Option<StmtId>)}: InlineFunc)], Statement[(Statement{.id=(body: StmtId), .kind=(_: StmtKind), .scope=(scope: Scope), .span=(_: Span)}: Statement)]".to_string(),
                                                                                                                                        ffun: None,
                                                                                                                                        arrangement: (Relations::Statement as RelId,0),
                                                                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<String>, ::types::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                            let ref scope = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                                                                ::types::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), declared_in: (::types::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(None)
                                                                                                                                    })
                                                                                                                 }))
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=declared_in}: NameInScope)] :- NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: Scope), .declared_in=(declared_in: AnyId)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: Scope), .child=(scope: Scope)}: ChildScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .declared_in=declared_in}: NameInScope)] :- NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: Scope), .declared_in=(declared_in: AnyId)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: Scope), .child=(scope: Scope)}: ChildScope)].".to_string(),
                                  arr: ( Relations::NameInScope as RelId, 1),
                                  xform: XFormArrangement::Join{
                                             description: "NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: Scope), .declared_in=(declared_in: AnyId)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: Scope), .child=(scope: Scope)}: ChildScope)]".to_string(),
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
                                  name: r###"(NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: Scope), .declared_in=(_: AnyId)}: NameInScope) /*antijoin*/"###.to_string(),
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
                                 name: r###"(NameInScope{.name=(_: internment::Intern<string>), .scope=(_0: Scope), .declared_in=(_: AnyId)}: NameInScope) /*join*/"###.to_string(),
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
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: Scope), .declared_in=(AnyIdStmt{.stmt=(_: StmtId)}: AnyId)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: ref _0, scope: ref _1, declared_in: ::types::AnyId::AnyIdStmt{stmt: _}} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=_1, .scope=_0, .declared_in=(_: AnyId)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: ref _1, scope: ref _0, declared_in: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: true
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=(_: internment::Intern<string>), .scope=_0, .declared_in=(_: AnyId)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: _, scope: ref _0, declared_in: _} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: true
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
                                 /* InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: Scope), .declared_in=(_: AnyId)}: NameInScope)], not WithinTypeOf[(WithinTypeOf{.type_of=(_: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)]. */
                                 Rule::ArrangementRule {
                                     description: "InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: Scope), .declared_in=(_: AnyId)}: NameInScope)], not WithinTypeOf[(WithinTypeOf{.type_of=(_: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)].".to_string(),
                                     arr: ( Relations::NameRef as RelId, 0),
                                     xform: XFormArrangement::Join{
                                                description: "NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)]".to_string(),
                                                ffun: None,
                                                arrangement: (Relations::Expression as RelId,0),
                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                {
                                                    let (ref expr, ref name) = match *unsafe {<::types::NameRef>::from_ddvalue_ref(__v1) } {
                                                        ::types::NameRef{expr_id: ref expr, value: ref name} => ((*expr).clone(), (*name).clone()),
                                                        _ => return None
                                                    };
                                                    let (ref scope, ref span) = match *unsafe {<::types::Expression>::from_ddvalue_ref(__v2) } {
                                                        ::types::Expression{id: _, kind: ::types::ExprKind::ExprNameRef{}, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                        _ => return None
                                                    };
                                                    Some((::types::ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(Some(XFormCollection::Arrange {
                                                                        description: "arrange NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)] by (name, scope)" .to_string(),
                                                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                        {
                                                                            let ::types::ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple4<::types::ExprId, ::types::internment::Intern<String>, ::types::Scope, ::types::Span>>::from_ddvalue_ref( &__v ) };
                                                                            Some(((::types::ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                        }
                                                                        __f},
                                                                        next: Box::new(XFormArrangement::Antijoin {
                                                                                           description: "NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: Scope), .declared_in=(_: AnyId)}: NameInScope)]".to_string(),
                                                                                           ffun: None,
                                                                                           arrangement: (Relations::NameInScope as RelId,0),
                                                                                           next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                   description: "arrange NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: Scope), .declared_in=(_: AnyId)}: NameInScope)] by (expr)" .to_string(),
                                                                                                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                   {
                                                                                                                       let ::types::ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple4<::types::ExprId, ::types::internment::Intern<String>, ::types::Scope, ::types::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                       Some((((*expr).clone()).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                                                                   }
                                                                                                                   __f},
                                                                                                                   next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                      description: "NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: Scope), .declared_in=(_: AnyId)}: NameInScope)], not WithinTypeOf[(WithinTypeOf{.type_of=(_: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)]".to_string(),
                                                                                                                                      ffun: None,
                                                                                                                                      arrangement: (Relations::WithinTypeOf as RelId,0),
                                                                                                                                      next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                              description: "head of InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(scope: Scope), .span=(span: Span)}: Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: Scope), .declared_in=(_: AnyId)}: NameInScope)], not WithinTypeOf[(WithinTypeOf{.type_of=(_: ExprId), .expr=(expr: ExprId)}: WithinTypeOf)]." .to_string(),
                                                                                                                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                              {
                                                                                                                                                                  let ::types::ddlog_std::tuple3(ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::Scope, ::types::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                  Some(((::types::InvalidNameUse{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone()})).into_ddvalue())
                                                                                                                                                              }
                                                                                                                                                              __f},
                                                                                                                                                              next: Box::new(None)
                                                                                                                                                          }))
                                                                                                                                  })
                                                                                                               }))
                                                                                       })
                                                                    }))
                                            }
                                 }],
                             arrangements: vec![
                                 Arrangement::Map{
                                    name: r###"(InvalidNameUse{.name=_0, .scope=(_: Scope), .span=(_: Span)}: InvalidNameUse) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::InvalidNameUse>::from_ddvalue(__v) } {
                                             ::types::InvalidNameUse{name: ref _0, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: true
                                 }],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let VarUseBeforeDeclaration = Relation {
                                      name:         "VarUseBeforeDeclaration".to_string(),
                                      input:        false,
                                      distinct:     true,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::VarUseBeforeDeclaration as RelId,
                                      rules:        vec![
                                          /* VarUseBeforeDeclaration[(VarUseBeforeDeclaration{.name=name, .used_in=used_in, .declared_in=declared_in}: VarUseBeforeDeclaration)] :- NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: Scope), .declared_in=(AnyIdStmt{.stmt=(stmt: StmtId)}: AnyId)}: NameInScope)], Statement[(Statement{.id=(stmt: StmtId), .kind=(StmtVarDecl{}: StmtKind), .scope=(declared_scope: Scope), .span=(declared_in: Span)}: Statement)], ChildScope[(ChildScope{.parent=(used_scope: Scope), .child=(declared_scope: Scope)}: ChildScope)]. */
                                          Rule::ArrangementRule {
                                              description: "VarUseBeforeDeclaration[(VarUseBeforeDeclaration{.name=name, .used_in=used_in, .declared_in=declared_in}: VarUseBeforeDeclaration)] :- NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: Scope), .declared_in=(AnyIdStmt{.stmt=(stmt: StmtId)}: AnyId)}: NameInScope)], Statement[(Statement{.id=(stmt: StmtId), .kind=(StmtVarDecl{}: StmtKind), .scope=(declared_scope: Scope), .span=(declared_in: Span)}: Statement)], ChildScope[(ChildScope{.parent=(used_scope: Scope), .child=(declared_scope: Scope)}: ChildScope)].".to_string(),
                                              arr: ( Relations::NameRef as RelId, 0),
                                              xform: XFormArrangement::Join{
                                                         description: "NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)]".to_string(),
                                                         ffun: None,
                                                         arrangement: (Relations::Expression as RelId,0),
                                                         jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                         {
                                                             let (ref expr, ref name) = match *unsafe {<::types::NameRef>::from_ddvalue_ref(__v1) } {
                                                                 ::types::NameRef{expr_id: ref expr, value: ref name} => ((*expr).clone(), (*name).clone()),
                                                                 _ => return None
                                                             };
                                                             let (ref used_scope, ref used_in) = match *unsafe {<::types::Expression>::from_ddvalue_ref(__v2) } {
                                                                 ::types::Expression{id: _, kind: ::types::ExprKind::ExprNameRef{}, scope: ref used_scope, span: ref used_in} => ((*used_scope).clone(), (*used_in).clone()),
                                                                 _ => return None
                                                             };
                                                             Some((::types::ddlog_std::tuple3((*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue())
                                                         }
                                                         __f},
                                                         next: Box::new(Some(XFormCollection::Arrange {
                                                                                 description: "arrange NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)] by (name, used_scope)" .to_string(),
                                                                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                 {
                                                                                     let ::types::ddlog_std::tuple3(ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::Scope, ::types::Span>>::from_ddvalue_ref( &__v ) };
                                                                                     Some(((::types::ddlog_std::tuple2((*name).clone(), (*used_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue()))
                                                                                 }
                                                                                 __f},
                                                                                 next: Box::new(XFormArrangement::Join{
                                                                                                    description: "NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: Scope), .declared_in=(AnyIdStmt{.stmt=(stmt: StmtId)}: AnyId)}: NameInScope)]".to_string(),
                                                                                                    ffun: None,
                                                                                                    arrangement: (Relations::NameInScope as RelId,2),
                                                                                                    jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                    {
                                                                                                        let ::types::ddlog_std::tuple3(ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::Scope, ::types::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                        let ref stmt = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v2) } {
                                                                                                            ::types::NameInScope{name: _, scope: _, declared_in: ::types::AnyId::AnyIdStmt{stmt: ref stmt}} => (*stmt).clone(),
                                                                                                            _ => return None
                                                                                                        };
                                                                                                        Some((::types::ddlog_std::tuple4((*name).clone(), (*used_scope).clone(), (*used_in).clone(), (*stmt).clone())).into_ddvalue())
                                                                                                    }
                                                                                                    __f},
                                                                                                    next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                            description: "arrange NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: Scope), .declared_in=(AnyIdStmt{.stmt=(stmt: StmtId)}: AnyId)}: NameInScope)] by (stmt)" .to_string(),
                                                                                                                            afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let ::types::ddlog_std::tuple4(ref name, ref used_scope, ref used_in, ref stmt) = *unsafe {<::types::ddlog_std::tuple4<::types::internment::Intern<String>, ::types::Scope, ::types::Span, ::types::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue()))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            next: Box::new(XFormArrangement::Join{
                                                                                                                                               description: "NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: Scope), .declared_in=(AnyIdStmt{.stmt=(stmt: StmtId)}: AnyId)}: NameInScope)], Statement[(Statement{.id=(stmt: StmtId), .kind=(StmtVarDecl{}: StmtKind), .scope=(declared_scope: Scope), .span=(declared_in: Span)}: Statement)]".to_string(),
                                                                                                                                               ffun: None,
                                                                                                                                               arrangement: (Relations::Statement as RelId,1),
                                                                                                                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                               {
                                                                                                                                                   let ::types::ddlog_std::tuple3(ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::Scope, ::types::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                   let (ref declared_scope, ref declared_in) = match *unsafe {<::types::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                                                                       ::types::Statement{id: _, kind: ::types::StmtKind::StmtVarDecl{}, scope: ref declared_scope, span: ref declared_in} => ((*declared_scope).clone(), (*declared_in).clone()),
                                                                                                                                                       _ => return None
                                                                                                                                                   };
                                                                                                                                                   Some((::types::ddlog_std::tuple5((*name).clone(), (*used_scope).clone(), (*used_in).clone(), (*declared_scope).clone(), (*declared_in).clone())).into_ddvalue())
                                                                                                                                               }
                                                                                                                                               __f},
                                                                                                                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                       description: "arrange NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: Scope), .declared_in=(AnyIdStmt{.stmt=(stmt: StmtId)}: AnyId)}: NameInScope)], Statement[(Statement{.id=(stmt: StmtId), .kind=(StmtVarDecl{}: StmtKind), .scope=(declared_scope: Scope), .span=(declared_in: Span)}: Statement)] by (used_scope, declared_scope)" .to_string(),
                                                                                                                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                       {
                                                                                                                                                                           let ::types::ddlog_std::tuple5(ref name, ref used_scope, ref used_in, ref declared_scope, ref declared_in) = *unsafe {<::types::ddlog_std::tuple5<::types::internment::Intern<String>, ::types::Scope, ::types::Span, ::types::Scope, ::types::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                           Some(((::types::ddlog_std::tuple2((*used_scope).clone(), (*declared_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*used_in).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                                       }
                                                                                                                                                                       __f},
                                                                                                                                                                       next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                          description: "NameRef[(NameRef{.expr_id=(expr: ExprId), .value=(name: internment::Intern<string>)}: NameRef)], Expression[(Expression{.id=(expr: ExprId), .kind=(ExprNameRef{}: ExprKind), .scope=(used_scope: Scope), .span=(used_in: Span)}: Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: Scope), .declared_in=(AnyIdStmt{.stmt=(stmt: StmtId)}: AnyId)}: NameInScope)], Statement[(Statement{.id=(stmt: StmtId), .kind=(StmtVarDecl{}: StmtKind), .scope=(declared_scope: Scope), .span=(declared_in: Span)}: Statement)], ChildScope[(ChildScope{.parent=(used_scope: Scope), .child=(declared_scope: Scope)}: ChildScope)]".to_string(),
                                                                                                                                                                                          ffun: None,
                                                                                                                                                                                          arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                          {
                                                                                                                                                                                              let ::types::ddlog_std::tuple3(ref name, ref used_in, ref declared_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::Span, ::types::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                              Some(((::types::VarUseBeforeDeclaration{name: (*name).clone(), used_in: (*used_in).clone(), declared_in: (*declared_in).clone()})).into_ddvalue())
                                                                                                                                                                                          }
                                                                                                                                                                                          __f},
                                                                                                                                                                                          next: Box::new(None)
                                                                                                                                                                                      })
                                                                                                                                                                   }))
                                                                                                                                           })
                                                                                                                        }))
                                                                                                })
                                                                             }))
                                                     }
                                          }],
                                      arrangements: vec![
                                          Arrangement::Map{
                                             name: r###"(VarUseBeforeDeclaration{.name=_0, .used_in=(_: Span), .declared_in=(_: Span)}: VarUseBeforeDeclaration) /*join*/"###.to_string(),
                                              afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                              {
                                                  let __cloned = __v.clone();
                                                  match unsafe {< ::types::VarUseBeforeDeclaration>::from_ddvalue(__v) } {
                                                      ::types::VarUseBeforeDeclaration{name: ref _0, used_in: _, declared_in: _} => Some(((*_0).clone()).into_ddvalue()),
                                                      _ => None
                                                  }.map(|x|(x,__cloned))
                                              }
                                              __f},
                                              queryable: true
                                          }],
                                      change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                                  };
    let While = Relation {
                    name:         "While".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::While as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_While = Relation {
                          name:         "INPUT_While".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_While as RelId,
                          rules:        vec![
                              /* INPUT_While[x] :- While[(x: While)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_While[x] :- While[(x: While)].".to_string(),
                                  rel: Relations::While as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_While[x] :- While[(x: While)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::While>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let With = Relation {
                   name:         "With".to_string(),
                   input:        true,
                   distinct:     false,
                   caching_mode: CachingMode::Set,
                   key_func:     None,
                   id:           Relations::With as RelId,
                   rules:        vec![
                       ],
                   arrangements: vec![
                       ],
                   change_cb:    None
               };
    let INPUT_With = Relation {
                         name:         "INPUT_With".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::INPUT_With as RelId,
                         rules:        vec![
                             /* INPUT_With[x] :- With[(x: With)]. */
                             Rule::CollectionRule {
                                 description: "INPUT_With[x] :- With[(x: With)].".to_string(),
                                 rel: Relations::With as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of INPUT_With[x] :- With[(x: With)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let ref x = match *unsafe {<::types::With>::from_ddvalue_ref(&__v) } {
                                                         ref x => (*x).clone(),
                                                         _ => return None
                                                     };
                                                     Some(((*x).clone()).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             }],
                         arrangements: vec![
                             ],
                         change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                     };
    let Yield = Relation {
                    name:         "Yield".to_string(),
                    input:        true,
                    distinct:     false,
                    caching_mode: CachingMode::Set,
                    key_func:     None,
                    id:           Relations::Yield as RelId,
                    rules:        vec![
                        ],
                    arrangements: vec![
                        ],
                    change_cb:    None
                };
    let INPUT_Yield = Relation {
                          name:         "INPUT_Yield".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::INPUT_Yield as RelId,
                          rules:        vec![
                              /* INPUT_Yield[x] :- Yield[(x: Yield)]. */
                              Rule::CollectionRule {
                                  description: "INPUT_Yield[x] :- Yield[(x: Yield)].".to_string(),
                                  rel: Relations::Yield as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of INPUT_Yield[x] :- Yield[(x: Yield)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let ref x = match *unsafe {<::types::Yield>::from_ddvalue_ref(&__v) } {
                                                          ref x => (*x).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((*x).clone()).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              }],
                          arrangements: vec![
                              ],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    Program {
        nodes: vec![
            ProgNode::Rel{rel: Array},
            ProgNode::Rel{rel: INPUT_Array},
            ProgNode::Rel{rel: Arrow},
            ProgNode::Rel{rel: INPUT_Arrow},
            ProgNode::Rel{rel: ArrowParam},
            ProgNode::Rel{rel: INPUT_ArrowParam},
            ProgNode::Rel{rel: __Prefix_0},
            ProgNode::Rel{rel: Assign},
            ProgNode::Rel{rel: INPUT_Assign},
            ProgNode::Rel{rel: Await},
            ProgNode::Rel{rel: INPUT_Await},
            ProgNode::Rel{rel: BinOp},
            ProgNode::Rel{rel: INPUT_BinOp},
            ProgNode::Rel{rel: BracketAccess},
            ProgNode::Rel{rel: INPUT_BracketAccess},
            ProgNode::Rel{rel: Break},
            ProgNode::Rel{rel: INPUT_Break},
            ProgNode::Rel{rel: Call},
            ProgNode::Rel{rel: INPUT_Call},
            ProgNode::Rel{rel: Class},
            ProgNode::Rel{rel: INPUT_Class},
            ProgNode::Rel{rel: ClassExpr},
            ProgNode::Rel{rel: INPUT_ClassExpr},
            ProgNode::Rel{rel: ConstDecl},
            ProgNode::Rel{rel: INPUT_ConstDecl},
            ProgNode::Rel{rel: Continue},
            ProgNode::Rel{rel: INPUT_Continue},
            ProgNode::Rel{rel: DoWhile},
            ProgNode::Rel{rel: INPUT_DoWhile},
            ProgNode::Rel{rel: DotAccess},
            ProgNode::Rel{rel: INPUT_DotAccess},
            ProgNode::Rel{rel: EveryScope},
            ProgNode::Rel{rel: INPUT_EveryScope},
            ProgNode::Rel{rel: ExprBigInt},
            ProgNode::Rel{rel: INPUT_ExprBigInt},
            ProgNode::Rel{rel: ExprBool},
            ProgNode::Rel{rel: INPUT_ExprBool},
            ProgNode::Rel{rel: ExprNumber},
            ProgNode::Rel{rel: INPUT_ExprNumber},
            ProgNode::Rel{rel: ExprString},
            ProgNode::Rel{rel: INPUT_ExprString},
            ProgNode::Rel{rel: Expression},
            ProgNode::Rel{rel: INPUT_Expression},
            ProgNode::Rel{rel: For},
            ProgNode::Rel{rel: INPUT_For},
            ProgNode::Rel{rel: ForIn},
            ProgNode::Rel{rel: INPUT_ForIn},
            ProgNode::Rel{rel: Function},
            ProgNode::Rel{rel: INPUT_Function},
            ProgNode::Rel{rel: FunctionArg},
            ProgNode::Rel{rel: INPUT_FunctionArg},
            ProgNode::Rel{rel: If},
            ProgNode::Rel{rel: INPUT_If},
            ProgNode::Rel{rel: ImplicitGlobal},
            ProgNode::Rel{rel: INPUT_ImplicitGlobal},
            ProgNode::Rel{rel: InlineFunc},
            ProgNode::Rel{rel: INPUT_InlineFunc},
            ProgNode::Rel{rel: InlineFuncParam},
            ProgNode::Rel{rel: INPUT_InlineFuncParam},
            ProgNode::Rel{rel: InputScope},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: ChildScope, distinct: true}]},
            ProgNode::Rel{rel: ClosestFunction},
            ProgNode::Rel{rel: INPUT_InputScope},
            ProgNode::Rel{rel: Label},
            ProgNode::Rel{rel: INPUT_Label},
            ProgNode::Rel{rel: LetDecl},
            ProgNode::Rel{rel: INPUT_LetDecl},
            ProgNode::Rel{rel: NameRef},
            ProgNode::Rel{rel: INPUT_NameRef},
            ProgNode::Rel{rel: New},
            ProgNode::Rel{rel: INPUT_New},
            ProgNode::Rel{rel: Property},
            ProgNode::Rel{rel: INPUT_Property},
            ProgNode::Rel{rel: Return},
            ProgNode::Rel{rel: INPUT_Return},
            ProgNode::Rel{rel: Statement},
            ProgNode::Rel{rel: INPUT_Statement},
            ProgNode::Rel{rel: Switch},
            ProgNode::Rel{rel: INPUT_Switch},
            ProgNode::Rel{rel: SwitchCase},
            ProgNode::Rel{rel: INPUT_SwitchCase},
            ProgNode::Rel{rel: Template},
            ProgNode::Rel{rel: INPUT_Template},
            ProgNode::Rel{rel: Ternary},
            ProgNode::Rel{rel: INPUT_Ternary},
            ProgNode::Rel{rel: Throw},
            ProgNode::Rel{rel: INPUT_Throw},
            ProgNode::Rel{rel: Try},
            ProgNode::Rel{rel: INPUT_Try},
            ProgNode::Rel{rel: UnaryOp},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: WithinTypeOf, distinct: true}]},
            ProgNode::Rel{rel: INPUT_UnaryOp},
            ProgNode::Rel{rel: VarDecl},
            ProgNode::Rel{rel: INPUT_VarDecl},
            ProgNode::Rel{rel: __Prefix_1},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: NameInScope, distinct: true}]},
            ProgNode::Rel{rel: InvalidNameUse},
            ProgNode::Rel{rel: VarUseBeforeDeclaration},
            ProgNode::Rel{rel: While},
            ProgNode::Rel{rel: INPUT_While},
            ProgNode::Rel{rel: With},
            ProgNode::Rel{rel: INPUT_With},
            ProgNode::Rel{rel: Yield},
            ProgNode::Rel{rel: INPUT_Yield}
        ],
        init_data: vec![
        ]
    }
}