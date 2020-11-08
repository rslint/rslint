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


decl_update_deserializer!(UpdateSerializer,(0, ::types::ChainedWith), (1, ::types::ChildScope), (2, ::types::ClosestFunction), (3, ::types::inputs::Array), (4, ::types::inputs::Arrow), (5, ::types::inputs::ArrowParam), (6, ::types::inputs::Assign), (7, ::types::inputs::Await), (8, ::types::inputs::BinOp), (9, ::types::inputs::BracketAccess), (10, ::types::inputs::Break), (11, ::types::inputs::Call), (12, ::types::inputs::Class), (13, ::types::inputs::ClassExpr), (14, ::types::inputs::ConstDecl), (15, ::types::inputs::Continue), (16, ::types::inputs::DoWhile), (17, ::types::inputs::DotAccess), (18, ::types::inputs::EveryScope), (19, ::types::inputs::ExprBigInt), (20, ::types::inputs::ExprBool), (21, ::types::inputs::ExprNumber), (22, ::types::inputs::ExprString), (23, ::types::inputs::Expression), (24, ::types::inputs::For), (25, ::types::inputs::ForIn), (26, ::types::inputs::Function), (27, ::types::inputs::FunctionArg), (28, ::types::inputs::If), (29, ::types::inputs::ImplicitGlobal), (30, ::types::inputs::ImportDecl), (31, ::types::inputs::InlineFunc), (32, ::types::inputs::InlineFuncParam), (33, ::types::inputs::InputScope), (34, ::types::inputs::Label), (35, ::types::inputs::LetDecl), (36, ::types::inputs::NameRef), (37, ::types::inputs::New), (38, ::types::inputs::Property), (39, ::types::inputs::Return), (40, ::types::inputs::Statement), (41, ::types::inputs::Switch), (42, ::types::inputs::SwitchCase), (43, ::types::inputs::Template), (44, ::types::inputs::Ternary), (45, ::types::inputs::Throw), (46, ::types::inputs::Try), (47, ::types::inputs::UnaryOp), (48, ::types::inputs::VarDecl), (49, ::types::inputs::While), (50, ::types::inputs::With), (51, ::types::inputs::Yield), (52, ::types::InvalidNameUse), (53, ::types::NameInScope), (54, ::types::TypeofUndefinedAlwaysUndefined), (55, ::types::VarUseBeforeDeclaration), (56, ::types::WithinTypeofExpr), (60, ::types::inputs::Array), (61, ::types::inputs::Arrow), (62, ::types::inputs::ArrowParam), (63, ::types::inputs::Assign), (64, ::types::inputs::Await), (65, ::types::inputs::BinOp), (66, ::types::inputs::BracketAccess), (67, ::types::inputs::Break), (68, ::types::inputs::Call), (69, ::types::inputs::Class), (70, ::types::inputs::ClassExpr), (71, ::types::inputs::ConstDecl), (72, ::types::inputs::Continue), (73, ::types::inputs::DoWhile), (74, ::types::inputs::DotAccess), (75, ::types::inputs::EveryScope), (76, ::types::inputs::ExprBigInt), (77, ::types::inputs::ExprBool), (78, ::types::inputs::ExprNumber), (79, ::types::inputs::ExprString), (80, ::types::inputs::Expression), (81, ::types::inputs::For), (82, ::types::inputs::ForIn), (83, ::types::inputs::Function), (84, ::types::inputs::FunctionArg), (85, ::types::inputs::If), (86, ::types::inputs::ImplicitGlobal), (87, ::types::inputs::ImportDecl), (88, ::types::inputs::InlineFunc), (89, ::types::inputs::InlineFuncParam), (90, ::types::inputs::InputScope), (91, ::types::inputs::Label), (92, ::types::inputs::LetDecl), (93, ::types::inputs::NameRef), (94, ::types::inputs::New), (95, ::types::inputs::Property), (96, ::types::inputs::Return), (97, ::types::inputs::Statement), (98, ::types::inputs::Switch), (99, ::types::inputs::SwitchCase), (100, ::types::inputs::Template), (101, ::types::inputs::Ternary), (102, ::types::inputs::Throw), (103, ::types::inputs::Try), (104, ::types::inputs::UnaryOp), (105, ::types::inputs::VarDecl), (106, ::types::inputs::While), (107, ::types::inputs::With), (108, ::types::inputs::Yield));
impl TryFrom<&str> for Relations {
    type Error = ();
    fn try_from(rname: &str) -> ::std::result::Result<Self, ()> {
         match rname {
        "ChainedWith" => Ok(Relations::ChainedWith),
        "ChildScope" => Ok(Relations::ChildScope),
        "ClosestFunction" => Ok(Relations::ClosestFunction),
        "INPUT_inputs::Array" => Ok(Relations::INPUT_inputs_Array),
        "INPUT_inputs::Arrow" => Ok(Relations::INPUT_inputs_Arrow),
        "INPUT_inputs::ArrowParam" => Ok(Relations::INPUT_inputs_ArrowParam),
        "INPUT_inputs::Assign" => Ok(Relations::INPUT_inputs_Assign),
        "INPUT_inputs::Await" => Ok(Relations::INPUT_inputs_Await),
        "INPUT_inputs::BinOp" => Ok(Relations::INPUT_inputs_BinOp),
        "INPUT_inputs::BracketAccess" => Ok(Relations::INPUT_inputs_BracketAccess),
        "INPUT_inputs::Break" => Ok(Relations::INPUT_inputs_Break),
        "INPUT_inputs::Call" => Ok(Relations::INPUT_inputs_Call),
        "INPUT_inputs::Class" => Ok(Relations::INPUT_inputs_Class),
        "INPUT_inputs::ClassExpr" => Ok(Relations::INPUT_inputs_ClassExpr),
        "INPUT_inputs::ConstDecl" => Ok(Relations::INPUT_inputs_ConstDecl),
        "INPUT_inputs::Continue" => Ok(Relations::INPUT_inputs_Continue),
        "INPUT_inputs::DoWhile" => Ok(Relations::INPUT_inputs_DoWhile),
        "INPUT_inputs::DotAccess" => Ok(Relations::INPUT_inputs_DotAccess),
        "INPUT_inputs::EveryScope" => Ok(Relations::INPUT_inputs_EveryScope),
        "INPUT_inputs::ExprBigInt" => Ok(Relations::INPUT_inputs_ExprBigInt),
        "INPUT_inputs::ExprBool" => Ok(Relations::INPUT_inputs_ExprBool),
        "INPUT_inputs::ExprNumber" => Ok(Relations::INPUT_inputs_ExprNumber),
        "INPUT_inputs::ExprString" => Ok(Relations::INPUT_inputs_ExprString),
        "INPUT_inputs::Expression" => Ok(Relations::INPUT_inputs_Expression),
        "INPUT_inputs::For" => Ok(Relations::INPUT_inputs_For),
        "INPUT_inputs::ForIn" => Ok(Relations::INPUT_inputs_ForIn),
        "INPUT_inputs::Function" => Ok(Relations::INPUT_inputs_Function),
        "INPUT_inputs::FunctionArg" => Ok(Relations::INPUT_inputs_FunctionArg),
        "INPUT_inputs::If" => Ok(Relations::INPUT_inputs_If),
        "INPUT_inputs::ImplicitGlobal" => Ok(Relations::INPUT_inputs_ImplicitGlobal),
        "INPUT_inputs::ImportDecl" => Ok(Relations::INPUT_inputs_ImportDecl),
        "INPUT_inputs::InlineFunc" => Ok(Relations::INPUT_inputs_InlineFunc),
        "INPUT_inputs::InlineFuncParam" => Ok(Relations::INPUT_inputs_InlineFuncParam),
        "INPUT_inputs::InputScope" => Ok(Relations::INPUT_inputs_InputScope),
        "INPUT_inputs::Label" => Ok(Relations::INPUT_inputs_Label),
        "INPUT_inputs::LetDecl" => Ok(Relations::INPUT_inputs_LetDecl),
        "INPUT_inputs::NameRef" => Ok(Relations::INPUT_inputs_NameRef),
        "INPUT_inputs::New" => Ok(Relations::INPUT_inputs_New),
        "INPUT_inputs::Property" => Ok(Relations::INPUT_inputs_Property),
        "INPUT_inputs::Return" => Ok(Relations::INPUT_inputs_Return),
        "INPUT_inputs::Statement" => Ok(Relations::INPUT_inputs_Statement),
        "INPUT_inputs::Switch" => Ok(Relations::INPUT_inputs_Switch),
        "INPUT_inputs::SwitchCase" => Ok(Relations::INPUT_inputs_SwitchCase),
        "INPUT_inputs::Template" => Ok(Relations::INPUT_inputs_Template),
        "INPUT_inputs::Ternary" => Ok(Relations::INPUT_inputs_Ternary),
        "INPUT_inputs::Throw" => Ok(Relations::INPUT_inputs_Throw),
        "INPUT_inputs::Try" => Ok(Relations::INPUT_inputs_Try),
        "INPUT_inputs::UnaryOp" => Ok(Relations::INPUT_inputs_UnaryOp),
        "INPUT_inputs::VarDecl" => Ok(Relations::INPUT_inputs_VarDecl),
        "INPUT_inputs::While" => Ok(Relations::INPUT_inputs_While),
        "INPUT_inputs::With" => Ok(Relations::INPUT_inputs_With),
        "INPUT_inputs::Yield" => Ok(Relations::INPUT_inputs_Yield),
        "InvalidNameUse" => Ok(Relations::InvalidNameUse),
        "NameInScope" => Ok(Relations::NameInScope),
        "TypeofUndefinedAlwaysUndefined" => Ok(Relations::TypeofUndefinedAlwaysUndefined),
        "VarUseBeforeDeclaration" => Ok(Relations::VarUseBeforeDeclaration),
        "WithinTypeofExpr" => Ok(Relations::WithinTypeofExpr),
        "__Prefix_0" => Ok(Relations::__Prefix_0),
        "__Prefix_1" => Ok(Relations::__Prefix_1),
        "__Prefix_2" => Ok(Relations::__Prefix_2),
        "inputs::Array" => Ok(Relations::inputs_Array),
        "inputs::Arrow" => Ok(Relations::inputs_Arrow),
        "inputs::ArrowParam" => Ok(Relations::inputs_ArrowParam),
        "inputs::Assign" => Ok(Relations::inputs_Assign),
        "inputs::Await" => Ok(Relations::inputs_Await),
        "inputs::BinOp" => Ok(Relations::inputs_BinOp),
        "inputs::BracketAccess" => Ok(Relations::inputs_BracketAccess),
        "inputs::Break" => Ok(Relations::inputs_Break),
        "inputs::Call" => Ok(Relations::inputs_Call),
        "inputs::Class" => Ok(Relations::inputs_Class),
        "inputs::ClassExpr" => Ok(Relations::inputs_ClassExpr),
        "inputs::ConstDecl" => Ok(Relations::inputs_ConstDecl),
        "inputs::Continue" => Ok(Relations::inputs_Continue),
        "inputs::DoWhile" => Ok(Relations::inputs_DoWhile),
        "inputs::DotAccess" => Ok(Relations::inputs_DotAccess),
        "inputs::EveryScope" => Ok(Relations::inputs_EveryScope),
        "inputs::ExprBigInt" => Ok(Relations::inputs_ExprBigInt),
        "inputs::ExprBool" => Ok(Relations::inputs_ExprBool),
        "inputs::ExprNumber" => Ok(Relations::inputs_ExprNumber),
        "inputs::ExprString" => Ok(Relations::inputs_ExprString),
        "inputs::Expression" => Ok(Relations::inputs_Expression),
        "inputs::For" => Ok(Relations::inputs_For),
        "inputs::ForIn" => Ok(Relations::inputs_ForIn),
        "inputs::Function" => Ok(Relations::inputs_Function),
        "inputs::FunctionArg" => Ok(Relations::inputs_FunctionArg),
        "inputs::If" => Ok(Relations::inputs_If),
        "inputs::ImplicitGlobal" => Ok(Relations::inputs_ImplicitGlobal),
        "inputs::ImportDecl" => Ok(Relations::inputs_ImportDecl),
        "inputs::InlineFunc" => Ok(Relations::inputs_InlineFunc),
        "inputs::InlineFuncParam" => Ok(Relations::inputs_InlineFuncParam),
        "inputs::InputScope" => Ok(Relations::inputs_InputScope),
        "inputs::Label" => Ok(Relations::inputs_Label),
        "inputs::LetDecl" => Ok(Relations::inputs_LetDecl),
        "inputs::NameRef" => Ok(Relations::inputs_NameRef),
        "inputs::New" => Ok(Relations::inputs_New),
        "inputs::Property" => Ok(Relations::inputs_Property),
        "inputs::Return" => Ok(Relations::inputs_Return),
        "inputs::Statement" => Ok(Relations::inputs_Statement),
        "inputs::Switch" => Ok(Relations::inputs_Switch),
        "inputs::SwitchCase" => Ok(Relations::inputs_SwitchCase),
        "inputs::Template" => Ok(Relations::inputs_Template),
        "inputs::Ternary" => Ok(Relations::inputs_Ternary),
        "inputs::Throw" => Ok(Relations::inputs_Throw),
        "inputs::Try" => Ok(Relations::inputs_Try),
        "inputs::UnaryOp" => Ok(Relations::inputs_UnaryOp),
        "inputs::VarDecl" => Ok(Relations::inputs_VarDecl),
        "inputs::While" => Ok(Relations::inputs_While),
        "inputs::With" => Ok(Relations::inputs_With),
        "inputs::Yield" => Ok(Relations::inputs_Yield),
             _  => Err(())
         }
    }
}
impl Relations {
    pub fn is_output(&self) -> bool {
        match self {
        Relations::ChainedWith => true,
        Relations::ChildScope => true,
        Relations::ClosestFunction => true,
        Relations::INPUT_inputs_Array => true,
        Relations::INPUT_inputs_Arrow => true,
        Relations::INPUT_inputs_ArrowParam => true,
        Relations::INPUT_inputs_Assign => true,
        Relations::INPUT_inputs_Await => true,
        Relations::INPUT_inputs_BinOp => true,
        Relations::INPUT_inputs_BracketAccess => true,
        Relations::INPUT_inputs_Break => true,
        Relations::INPUT_inputs_Call => true,
        Relations::INPUT_inputs_Class => true,
        Relations::INPUT_inputs_ClassExpr => true,
        Relations::INPUT_inputs_ConstDecl => true,
        Relations::INPUT_inputs_Continue => true,
        Relations::INPUT_inputs_DoWhile => true,
        Relations::INPUT_inputs_DotAccess => true,
        Relations::INPUT_inputs_EveryScope => true,
        Relations::INPUT_inputs_ExprBigInt => true,
        Relations::INPUT_inputs_ExprBool => true,
        Relations::INPUT_inputs_ExprNumber => true,
        Relations::INPUT_inputs_ExprString => true,
        Relations::INPUT_inputs_Expression => true,
        Relations::INPUT_inputs_For => true,
        Relations::INPUT_inputs_ForIn => true,
        Relations::INPUT_inputs_Function => true,
        Relations::INPUT_inputs_FunctionArg => true,
        Relations::INPUT_inputs_If => true,
        Relations::INPUT_inputs_ImplicitGlobal => true,
        Relations::INPUT_inputs_ImportDecl => true,
        Relations::INPUT_inputs_InlineFunc => true,
        Relations::INPUT_inputs_InlineFuncParam => true,
        Relations::INPUT_inputs_InputScope => true,
        Relations::INPUT_inputs_Label => true,
        Relations::INPUT_inputs_LetDecl => true,
        Relations::INPUT_inputs_NameRef => true,
        Relations::INPUT_inputs_New => true,
        Relations::INPUT_inputs_Property => true,
        Relations::INPUT_inputs_Return => true,
        Relations::INPUT_inputs_Statement => true,
        Relations::INPUT_inputs_Switch => true,
        Relations::INPUT_inputs_SwitchCase => true,
        Relations::INPUT_inputs_Template => true,
        Relations::INPUT_inputs_Ternary => true,
        Relations::INPUT_inputs_Throw => true,
        Relations::INPUT_inputs_Try => true,
        Relations::INPUT_inputs_UnaryOp => true,
        Relations::INPUT_inputs_VarDecl => true,
        Relations::INPUT_inputs_While => true,
        Relations::INPUT_inputs_With => true,
        Relations::INPUT_inputs_Yield => true,
        Relations::InvalidNameUse => true,
        Relations::NameInScope => true,
        Relations::TypeofUndefinedAlwaysUndefined => true,
        Relations::VarUseBeforeDeclaration => true,
        Relations::WithinTypeofExpr => true,
            _  => false
        }
    }
}
impl Relations {
    pub fn is_input(&self) -> bool {
        match self {
        Relations::inputs_Array => true,
        Relations::inputs_Arrow => true,
        Relations::inputs_ArrowParam => true,
        Relations::inputs_Assign => true,
        Relations::inputs_Await => true,
        Relations::inputs_BinOp => true,
        Relations::inputs_BracketAccess => true,
        Relations::inputs_Break => true,
        Relations::inputs_Call => true,
        Relations::inputs_Class => true,
        Relations::inputs_ClassExpr => true,
        Relations::inputs_ConstDecl => true,
        Relations::inputs_Continue => true,
        Relations::inputs_DoWhile => true,
        Relations::inputs_DotAccess => true,
        Relations::inputs_EveryScope => true,
        Relations::inputs_ExprBigInt => true,
        Relations::inputs_ExprBool => true,
        Relations::inputs_ExprNumber => true,
        Relations::inputs_ExprString => true,
        Relations::inputs_Expression => true,
        Relations::inputs_For => true,
        Relations::inputs_ForIn => true,
        Relations::inputs_Function => true,
        Relations::inputs_FunctionArg => true,
        Relations::inputs_If => true,
        Relations::inputs_ImplicitGlobal => true,
        Relations::inputs_ImportDecl => true,
        Relations::inputs_InlineFunc => true,
        Relations::inputs_InlineFuncParam => true,
        Relations::inputs_InputScope => true,
        Relations::inputs_Label => true,
        Relations::inputs_LetDecl => true,
        Relations::inputs_NameRef => true,
        Relations::inputs_New => true,
        Relations::inputs_Property => true,
        Relations::inputs_Return => true,
        Relations::inputs_Statement => true,
        Relations::inputs_Switch => true,
        Relations::inputs_SwitchCase => true,
        Relations::inputs_Template => true,
        Relations::inputs_Ternary => true,
        Relations::inputs_Throw => true,
        Relations::inputs_Try => true,
        Relations::inputs_UnaryOp => true,
        Relations::inputs_VarDecl => true,
        Relations::inputs_While => true,
        Relations::inputs_With => true,
        Relations::inputs_Yield => true,
            _  => false
        }
    }
}
impl TryFrom<RelId> for Relations {
    type Error = ();
    fn try_from(rid: RelId) -> ::std::result::Result<Self, ()> {
         match rid {
        0 => Ok(Relations::ChainedWith),
        1 => Ok(Relations::ChildScope),
        2 => Ok(Relations::ClosestFunction),
        3 => Ok(Relations::INPUT_inputs_Array),
        4 => Ok(Relations::INPUT_inputs_Arrow),
        5 => Ok(Relations::INPUT_inputs_ArrowParam),
        6 => Ok(Relations::INPUT_inputs_Assign),
        7 => Ok(Relations::INPUT_inputs_Await),
        8 => Ok(Relations::INPUT_inputs_BinOp),
        9 => Ok(Relations::INPUT_inputs_BracketAccess),
        10 => Ok(Relations::INPUT_inputs_Break),
        11 => Ok(Relations::INPUT_inputs_Call),
        12 => Ok(Relations::INPUT_inputs_Class),
        13 => Ok(Relations::INPUT_inputs_ClassExpr),
        14 => Ok(Relations::INPUT_inputs_ConstDecl),
        15 => Ok(Relations::INPUT_inputs_Continue),
        16 => Ok(Relations::INPUT_inputs_DoWhile),
        17 => Ok(Relations::INPUT_inputs_DotAccess),
        18 => Ok(Relations::INPUT_inputs_EveryScope),
        19 => Ok(Relations::INPUT_inputs_ExprBigInt),
        20 => Ok(Relations::INPUT_inputs_ExprBool),
        21 => Ok(Relations::INPUT_inputs_ExprNumber),
        22 => Ok(Relations::INPUT_inputs_ExprString),
        23 => Ok(Relations::INPUT_inputs_Expression),
        24 => Ok(Relations::INPUT_inputs_For),
        25 => Ok(Relations::INPUT_inputs_ForIn),
        26 => Ok(Relations::INPUT_inputs_Function),
        27 => Ok(Relations::INPUT_inputs_FunctionArg),
        28 => Ok(Relations::INPUT_inputs_If),
        29 => Ok(Relations::INPUT_inputs_ImplicitGlobal),
        30 => Ok(Relations::INPUT_inputs_ImportDecl),
        31 => Ok(Relations::INPUT_inputs_InlineFunc),
        32 => Ok(Relations::INPUT_inputs_InlineFuncParam),
        33 => Ok(Relations::INPUT_inputs_InputScope),
        34 => Ok(Relations::INPUT_inputs_Label),
        35 => Ok(Relations::INPUT_inputs_LetDecl),
        36 => Ok(Relations::INPUT_inputs_NameRef),
        37 => Ok(Relations::INPUT_inputs_New),
        38 => Ok(Relations::INPUT_inputs_Property),
        39 => Ok(Relations::INPUT_inputs_Return),
        40 => Ok(Relations::INPUT_inputs_Statement),
        41 => Ok(Relations::INPUT_inputs_Switch),
        42 => Ok(Relations::INPUT_inputs_SwitchCase),
        43 => Ok(Relations::INPUT_inputs_Template),
        44 => Ok(Relations::INPUT_inputs_Ternary),
        45 => Ok(Relations::INPUT_inputs_Throw),
        46 => Ok(Relations::INPUT_inputs_Try),
        47 => Ok(Relations::INPUT_inputs_UnaryOp),
        48 => Ok(Relations::INPUT_inputs_VarDecl),
        49 => Ok(Relations::INPUT_inputs_While),
        50 => Ok(Relations::INPUT_inputs_With),
        51 => Ok(Relations::INPUT_inputs_Yield),
        52 => Ok(Relations::InvalidNameUse),
        53 => Ok(Relations::NameInScope),
        54 => Ok(Relations::TypeofUndefinedAlwaysUndefined),
        55 => Ok(Relations::VarUseBeforeDeclaration),
        56 => Ok(Relations::WithinTypeofExpr),
        57 => Ok(Relations::__Prefix_0),
        58 => Ok(Relations::__Prefix_1),
        59 => Ok(Relations::__Prefix_2),
        60 => Ok(Relations::inputs_Array),
        61 => Ok(Relations::inputs_Arrow),
        62 => Ok(Relations::inputs_ArrowParam),
        63 => Ok(Relations::inputs_Assign),
        64 => Ok(Relations::inputs_Await),
        65 => Ok(Relations::inputs_BinOp),
        66 => Ok(Relations::inputs_BracketAccess),
        67 => Ok(Relations::inputs_Break),
        68 => Ok(Relations::inputs_Call),
        69 => Ok(Relations::inputs_Class),
        70 => Ok(Relations::inputs_ClassExpr),
        71 => Ok(Relations::inputs_ConstDecl),
        72 => Ok(Relations::inputs_Continue),
        73 => Ok(Relations::inputs_DoWhile),
        74 => Ok(Relations::inputs_DotAccess),
        75 => Ok(Relations::inputs_EveryScope),
        76 => Ok(Relations::inputs_ExprBigInt),
        77 => Ok(Relations::inputs_ExprBool),
        78 => Ok(Relations::inputs_ExprNumber),
        79 => Ok(Relations::inputs_ExprString),
        80 => Ok(Relations::inputs_Expression),
        81 => Ok(Relations::inputs_For),
        82 => Ok(Relations::inputs_ForIn),
        83 => Ok(Relations::inputs_Function),
        84 => Ok(Relations::inputs_FunctionArg),
        85 => Ok(Relations::inputs_If),
        86 => Ok(Relations::inputs_ImplicitGlobal),
        87 => Ok(Relations::inputs_ImportDecl),
        88 => Ok(Relations::inputs_InlineFunc),
        89 => Ok(Relations::inputs_InlineFuncParam),
        90 => Ok(Relations::inputs_InputScope),
        91 => Ok(Relations::inputs_Label),
        92 => Ok(Relations::inputs_LetDecl),
        93 => Ok(Relations::inputs_NameRef),
        94 => Ok(Relations::inputs_New),
        95 => Ok(Relations::inputs_Property),
        96 => Ok(Relations::inputs_Return),
        97 => Ok(Relations::inputs_Statement),
        98 => Ok(Relations::inputs_Switch),
        99 => Ok(Relations::inputs_SwitchCase),
        100 => Ok(Relations::inputs_Template),
        101 => Ok(Relations::inputs_Ternary),
        102 => Ok(Relations::inputs_Throw),
        103 => Ok(Relations::inputs_Try),
        104 => Ok(Relations::inputs_UnaryOp),
        105 => Ok(Relations::inputs_VarDecl),
        106 => Ok(Relations::inputs_While),
        107 => Ok(Relations::inputs_With),
        108 => Ok(Relations::inputs_Yield),
             _  => Err(())
         }
    }
}
pub fn relid2name(rid: RelId) -> Option<&'static str> {
   match rid {
        0 => Some(&"ChainedWith"),
        1 => Some(&"ChildScope"),
        2 => Some(&"ClosestFunction"),
        3 => Some(&"INPUT_inputs::Array"),
        4 => Some(&"INPUT_inputs::Arrow"),
        5 => Some(&"INPUT_inputs::ArrowParam"),
        6 => Some(&"INPUT_inputs::Assign"),
        7 => Some(&"INPUT_inputs::Await"),
        8 => Some(&"INPUT_inputs::BinOp"),
        9 => Some(&"INPUT_inputs::BracketAccess"),
        10 => Some(&"INPUT_inputs::Break"),
        11 => Some(&"INPUT_inputs::Call"),
        12 => Some(&"INPUT_inputs::Class"),
        13 => Some(&"INPUT_inputs::ClassExpr"),
        14 => Some(&"INPUT_inputs::ConstDecl"),
        15 => Some(&"INPUT_inputs::Continue"),
        16 => Some(&"INPUT_inputs::DoWhile"),
        17 => Some(&"INPUT_inputs::DotAccess"),
        18 => Some(&"INPUT_inputs::EveryScope"),
        19 => Some(&"INPUT_inputs::ExprBigInt"),
        20 => Some(&"INPUT_inputs::ExprBool"),
        21 => Some(&"INPUT_inputs::ExprNumber"),
        22 => Some(&"INPUT_inputs::ExprString"),
        23 => Some(&"INPUT_inputs::Expression"),
        24 => Some(&"INPUT_inputs::For"),
        25 => Some(&"INPUT_inputs::ForIn"),
        26 => Some(&"INPUT_inputs::Function"),
        27 => Some(&"INPUT_inputs::FunctionArg"),
        28 => Some(&"INPUT_inputs::If"),
        29 => Some(&"INPUT_inputs::ImplicitGlobal"),
        30 => Some(&"INPUT_inputs::ImportDecl"),
        31 => Some(&"INPUT_inputs::InlineFunc"),
        32 => Some(&"INPUT_inputs::InlineFuncParam"),
        33 => Some(&"INPUT_inputs::InputScope"),
        34 => Some(&"INPUT_inputs::Label"),
        35 => Some(&"INPUT_inputs::LetDecl"),
        36 => Some(&"INPUT_inputs::NameRef"),
        37 => Some(&"INPUT_inputs::New"),
        38 => Some(&"INPUT_inputs::Property"),
        39 => Some(&"INPUT_inputs::Return"),
        40 => Some(&"INPUT_inputs::Statement"),
        41 => Some(&"INPUT_inputs::Switch"),
        42 => Some(&"INPUT_inputs::SwitchCase"),
        43 => Some(&"INPUT_inputs::Template"),
        44 => Some(&"INPUT_inputs::Ternary"),
        45 => Some(&"INPUT_inputs::Throw"),
        46 => Some(&"INPUT_inputs::Try"),
        47 => Some(&"INPUT_inputs::UnaryOp"),
        48 => Some(&"INPUT_inputs::VarDecl"),
        49 => Some(&"INPUT_inputs::While"),
        50 => Some(&"INPUT_inputs::With"),
        51 => Some(&"INPUT_inputs::Yield"),
        52 => Some(&"InvalidNameUse"),
        53 => Some(&"NameInScope"),
        54 => Some(&"TypeofUndefinedAlwaysUndefined"),
        55 => Some(&"VarUseBeforeDeclaration"),
        56 => Some(&"WithinTypeofExpr"),
        57 => Some(&"__Prefix_0"),
        58 => Some(&"__Prefix_1"),
        59 => Some(&"__Prefix_2"),
        60 => Some(&"inputs::Array"),
        61 => Some(&"inputs::Arrow"),
        62 => Some(&"inputs::ArrowParam"),
        63 => Some(&"inputs::Assign"),
        64 => Some(&"inputs::Await"),
        65 => Some(&"inputs::BinOp"),
        66 => Some(&"inputs::BracketAccess"),
        67 => Some(&"inputs::Break"),
        68 => Some(&"inputs::Call"),
        69 => Some(&"inputs::Class"),
        70 => Some(&"inputs::ClassExpr"),
        71 => Some(&"inputs::ConstDecl"),
        72 => Some(&"inputs::Continue"),
        73 => Some(&"inputs::DoWhile"),
        74 => Some(&"inputs::DotAccess"),
        75 => Some(&"inputs::EveryScope"),
        76 => Some(&"inputs::ExprBigInt"),
        77 => Some(&"inputs::ExprBool"),
        78 => Some(&"inputs::ExprNumber"),
        79 => Some(&"inputs::ExprString"),
        80 => Some(&"inputs::Expression"),
        81 => Some(&"inputs::For"),
        82 => Some(&"inputs::ForIn"),
        83 => Some(&"inputs::Function"),
        84 => Some(&"inputs::FunctionArg"),
        85 => Some(&"inputs::If"),
        86 => Some(&"inputs::ImplicitGlobal"),
        87 => Some(&"inputs::ImportDecl"),
        88 => Some(&"inputs::InlineFunc"),
        89 => Some(&"inputs::InlineFuncParam"),
        90 => Some(&"inputs::InputScope"),
        91 => Some(&"inputs::Label"),
        92 => Some(&"inputs::LetDecl"),
        93 => Some(&"inputs::NameRef"),
        94 => Some(&"inputs::New"),
        95 => Some(&"inputs::Property"),
        96 => Some(&"inputs::Return"),
        97 => Some(&"inputs::Statement"),
        98 => Some(&"inputs::Switch"),
        99 => Some(&"inputs::SwitchCase"),
        100 => Some(&"inputs::Template"),
        101 => Some(&"inputs::Ternary"),
        102 => Some(&"inputs::Throw"),
        103 => Some(&"inputs::Try"),
        104 => Some(&"inputs::UnaryOp"),
        105 => Some(&"inputs::VarDecl"),
        106 => Some(&"inputs::While"),
        107 => Some(&"inputs::With"),
        108 => Some(&"inputs::Yield"),
       _  => None
   }
}
pub fn relid2cname(rid: RelId) -> Option<&'static ::std::ffi::CStr> {
    RELIDMAPC.get(&rid).copied()
}   /// A map of `RelId`s to their name as an `&'static str`
pub static RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(109, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ChainedWith, "ChainedWith");
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::ClosestFunction, "ClosestFunction");
        map.insert(Relations::INPUT_inputs_Array, "INPUT_inputs::Array");
        map.insert(Relations::INPUT_inputs_Arrow, "INPUT_inputs::Arrow");
        map.insert(Relations::INPUT_inputs_ArrowParam, "INPUT_inputs::ArrowParam");
        map.insert(Relations::INPUT_inputs_Assign, "INPUT_inputs::Assign");
        map.insert(Relations::INPUT_inputs_Await, "INPUT_inputs::Await");
        map.insert(Relations::INPUT_inputs_BinOp, "INPUT_inputs::BinOp");
        map.insert(Relations::INPUT_inputs_BracketAccess, "INPUT_inputs::BracketAccess");
        map.insert(Relations::INPUT_inputs_Break, "INPUT_inputs::Break");
        map.insert(Relations::INPUT_inputs_Call, "INPUT_inputs::Call");
        map.insert(Relations::INPUT_inputs_Class, "INPUT_inputs::Class");
        map.insert(Relations::INPUT_inputs_ClassExpr, "INPUT_inputs::ClassExpr");
        map.insert(Relations::INPUT_inputs_ConstDecl, "INPUT_inputs::ConstDecl");
        map.insert(Relations::INPUT_inputs_Continue, "INPUT_inputs::Continue");
        map.insert(Relations::INPUT_inputs_DoWhile, "INPUT_inputs::DoWhile");
        map.insert(Relations::INPUT_inputs_DotAccess, "INPUT_inputs::DotAccess");
        map.insert(Relations::INPUT_inputs_EveryScope, "INPUT_inputs::EveryScope");
        map.insert(Relations::INPUT_inputs_ExprBigInt, "INPUT_inputs::ExprBigInt");
        map.insert(Relations::INPUT_inputs_ExprBool, "INPUT_inputs::ExprBool");
        map.insert(Relations::INPUT_inputs_ExprNumber, "INPUT_inputs::ExprNumber");
        map.insert(Relations::INPUT_inputs_ExprString, "INPUT_inputs::ExprString");
        map.insert(Relations::INPUT_inputs_Expression, "INPUT_inputs::Expression");
        map.insert(Relations::INPUT_inputs_For, "INPUT_inputs::For");
        map.insert(Relations::INPUT_inputs_ForIn, "INPUT_inputs::ForIn");
        map.insert(Relations::INPUT_inputs_Function, "INPUT_inputs::Function");
        map.insert(Relations::INPUT_inputs_FunctionArg, "INPUT_inputs::FunctionArg");
        map.insert(Relations::INPUT_inputs_If, "INPUT_inputs::If");
        map.insert(Relations::INPUT_inputs_ImplicitGlobal, "INPUT_inputs::ImplicitGlobal");
        map.insert(Relations::INPUT_inputs_ImportDecl, "INPUT_inputs::ImportDecl");
        map.insert(Relations::INPUT_inputs_InlineFunc, "INPUT_inputs::InlineFunc");
        map.insert(Relations::INPUT_inputs_InlineFuncParam, "INPUT_inputs::InlineFuncParam");
        map.insert(Relations::INPUT_inputs_InputScope, "INPUT_inputs::InputScope");
        map.insert(Relations::INPUT_inputs_Label, "INPUT_inputs::Label");
        map.insert(Relations::INPUT_inputs_LetDecl, "INPUT_inputs::LetDecl");
        map.insert(Relations::INPUT_inputs_NameRef, "INPUT_inputs::NameRef");
        map.insert(Relations::INPUT_inputs_New, "INPUT_inputs::New");
        map.insert(Relations::INPUT_inputs_Property, "INPUT_inputs::Property");
        map.insert(Relations::INPUT_inputs_Return, "INPUT_inputs::Return");
        map.insert(Relations::INPUT_inputs_Statement, "INPUT_inputs::Statement");
        map.insert(Relations::INPUT_inputs_Switch, "INPUT_inputs::Switch");
        map.insert(Relations::INPUT_inputs_SwitchCase, "INPUT_inputs::SwitchCase");
        map.insert(Relations::INPUT_inputs_Template, "INPUT_inputs::Template");
        map.insert(Relations::INPUT_inputs_Ternary, "INPUT_inputs::Ternary");
        map.insert(Relations::INPUT_inputs_Throw, "INPUT_inputs::Throw");
        map.insert(Relations::INPUT_inputs_Try, "INPUT_inputs::Try");
        map.insert(Relations::INPUT_inputs_UnaryOp, "INPUT_inputs::UnaryOp");
        map.insert(Relations::INPUT_inputs_VarDecl, "INPUT_inputs::VarDecl");
        map.insert(Relations::INPUT_inputs_While, "INPUT_inputs::While");
        map.insert(Relations::INPUT_inputs_With, "INPUT_inputs::With");
        map.insert(Relations::INPUT_inputs_Yield, "INPUT_inputs::Yield");
        map.insert(Relations::InvalidNameUse, "InvalidNameUse");
        map.insert(Relations::NameInScope, "NameInScope");
        map.insert(Relations::TypeofUndefinedAlwaysUndefined, "TypeofUndefinedAlwaysUndefined");
        map.insert(Relations::VarUseBeforeDeclaration, "VarUseBeforeDeclaration");
        map.insert(Relations::WithinTypeofExpr, "WithinTypeofExpr");
        map.insert(Relations::__Prefix_0, "__Prefix_0");
        map.insert(Relations::__Prefix_1, "__Prefix_1");
        map.insert(Relations::__Prefix_2, "__Prefix_2");
        map.insert(Relations::inputs_Array, "inputs::Array");
        map.insert(Relations::inputs_Arrow, "inputs::Arrow");
        map.insert(Relations::inputs_ArrowParam, "inputs::ArrowParam");
        map.insert(Relations::inputs_Assign, "inputs::Assign");
        map.insert(Relations::inputs_Await, "inputs::Await");
        map.insert(Relations::inputs_BinOp, "inputs::BinOp");
        map.insert(Relations::inputs_BracketAccess, "inputs::BracketAccess");
        map.insert(Relations::inputs_Break, "inputs::Break");
        map.insert(Relations::inputs_Call, "inputs::Call");
        map.insert(Relations::inputs_Class, "inputs::Class");
        map.insert(Relations::inputs_ClassExpr, "inputs::ClassExpr");
        map.insert(Relations::inputs_ConstDecl, "inputs::ConstDecl");
        map.insert(Relations::inputs_Continue, "inputs::Continue");
        map.insert(Relations::inputs_DoWhile, "inputs::DoWhile");
        map.insert(Relations::inputs_DotAccess, "inputs::DotAccess");
        map.insert(Relations::inputs_EveryScope, "inputs::EveryScope");
        map.insert(Relations::inputs_ExprBigInt, "inputs::ExprBigInt");
        map.insert(Relations::inputs_ExprBool, "inputs::ExprBool");
        map.insert(Relations::inputs_ExprNumber, "inputs::ExprNumber");
        map.insert(Relations::inputs_ExprString, "inputs::ExprString");
        map.insert(Relations::inputs_Expression, "inputs::Expression");
        map.insert(Relations::inputs_For, "inputs::For");
        map.insert(Relations::inputs_ForIn, "inputs::ForIn");
        map.insert(Relations::inputs_Function, "inputs::Function");
        map.insert(Relations::inputs_FunctionArg, "inputs::FunctionArg");
        map.insert(Relations::inputs_If, "inputs::If");
        map.insert(Relations::inputs_ImplicitGlobal, "inputs::ImplicitGlobal");
        map.insert(Relations::inputs_ImportDecl, "inputs::ImportDecl");
        map.insert(Relations::inputs_InlineFunc, "inputs::InlineFunc");
        map.insert(Relations::inputs_InlineFuncParam, "inputs::InlineFuncParam");
        map.insert(Relations::inputs_InputScope, "inputs::InputScope");
        map.insert(Relations::inputs_Label, "inputs::Label");
        map.insert(Relations::inputs_LetDecl, "inputs::LetDecl");
        map.insert(Relations::inputs_NameRef, "inputs::NameRef");
        map.insert(Relations::inputs_New, "inputs::New");
        map.insert(Relations::inputs_Property, "inputs::Property");
        map.insert(Relations::inputs_Return, "inputs::Return");
        map.insert(Relations::inputs_Statement, "inputs::Statement");
        map.insert(Relations::inputs_Switch, "inputs::Switch");
        map.insert(Relations::inputs_SwitchCase, "inputs::SwitchCase");
        map.insert(Relations::inputs_Template, "inputs::Template");
        map.insert(Relations::inputs_Ternary, "inputs::Ternary");
        map.insert(Relations::inputs_Throw, "inputs::Throw");
        map.insert(Relations::inputs_Try, "inputs::Try");
        map.insert(Relations::inputs_UnaryOp, "inputs::UnaryOp");
        map.insert(Relations::inputs_VarDecl, "inputs::VarDecl");
        map.insert(Relations::inputs_While, "inputs::While");
        map.insert(Relations::inputs_With, "inputs::With");
        map.insert(Relations::inputs_Yield, "inputs::Yield");
        map
    });
    /// A map of `RelId`s to their name as an `&'static CStr`
pub static RELIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<RelId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(109, ::fnv::FnvBuildHasher::default());
        map.insert(0, ::std::ffi::CStr::from_bytes_with_nul(b"ChainedWith\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(1, ::std::ffi::CStr::from_bytes_with_nul(b"ChildScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(2, ::std::ffi::CStr::from_bytes_with_nul(b"ClosestFunction\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(3, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Array\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(4, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Arrow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(5, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ArrowParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(6, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Assign\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(7, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Await\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(8, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::BinOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(9, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::BracketAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(10, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Break\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(11, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Call\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(12, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Class\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(13, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ClassExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(14, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ConstDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(15, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Continue\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(16, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::DoWhile\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(17, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::DotAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(18, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::EveryScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(19, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ExprBigInt\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(20, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ExprBool\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(21, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ExprNumber\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(22, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ExprString\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(23, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Expression\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(24, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::For\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(25, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ForIn\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(26, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Function\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(27, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::FunctionArg\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(28, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::If\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(29, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ImplicitGlobal\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(30, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::ImportDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(31, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::InlineFunc\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(32, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::InlineFuncParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(33, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::InputScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(34, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Label\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(35, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::LetDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(36, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::NameRef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(37, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::New\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(38, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Property\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(39, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Return\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(40, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Statement\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(41, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Switch\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(42, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::SwitchCase\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(43, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Template\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(44, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Ternary\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(45, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Throw\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(46, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Try\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(47, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::UnaryOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(48, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::VarDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(49, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::While\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(50, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::With\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(51, ::std::ffi::CStr::from_bytes_with_nul(b"INPUT_inputs::Yield\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(52, ::std::ffi::CStr::from_bytes_with_nul(b"InvalidNameUse\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(53, ::std::ffi::CStr::from_bytes_with_nul(b"NameInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(54, ::std::ffi::CStr::from_bytes_with_nul(b"TypeofUndefinedAlwaysUndefined\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(55, ::std::ffi::CStr::from_bytes_with_nul(b"VarUseBeforeDeclaration\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(56, ::std::ffi::CStr::from_bytes_with_nul(b"WithinTypeofExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(57, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_0\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(58, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_1\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(59, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_2\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(60, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Array\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(61, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Arrow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(62, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ArrowParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(63, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Assign\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(64, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Await\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(65, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::BinOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(66, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::BracketAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(67, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Break\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(68, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Call\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(69, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Class\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(70, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ClassExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(71, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ConstDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(72, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Continue\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(73, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::DoWhile\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(74, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::DotAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(75, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::EveryScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(76, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprBigInt\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(77, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprBool\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(78, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprNumber\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(79, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprString\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(80, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Expression\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(81, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::For\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(82, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ForIn\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(83, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Function\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(84, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::FunctionArg\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(85, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::If\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(86, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ImplicitGlobal\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(87, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ImportDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(88, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InlineFunc\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(89, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InlineFuncParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(90, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InputScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(91, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Label\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(92, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::LetDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(93, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::NameRef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(94, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::New\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(95, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Property\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(96, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Return\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(97, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Statement\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(98, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Switch\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(99, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::SwitchCase\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(100, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Template\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(101, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Ternary\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(102, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Throw\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(103, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Try\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(104, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::UnaryOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(105, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::VarDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(106, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::While\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(107, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::With\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(108, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Yield\0").expect("Unreachable: A null byte was specifically inserted"));
        map
    });
    /// A map of input `Relations`s to their name as an `&'static str`
pub static INPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(49, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::inputs_Array, "inputs::Array");
        map.insert(Relations::inputs_Arrow, "inputs::Arrow");
        map.insert(Relations::inputs_ArrowParam, "inputs::ArrowParam");
        map.insert(Relations::inputs_Assign, "inputs::Assign");
        map.insert(Relations::inputs_Await, "inputs::Await");
        map.insert(Relations::inputs_BinOp, "inputs::BinOp");
        map.insert(Relations::inputs_BracketAccess, "inputs::BracketAccess");
        map.insert(Relations::inputs_Break, "inputs::Break");
        map.insert(Relations::inputs_Call, "inputs::Call");
        map.insert(Relations::inputs_Class, "inputs::Class");
        map.insert(Relations::inputs_ClassExpr, "inputs::ClassExpr");
        map.insert(Relations::inputs_ConstDecl, "inputs::ConstDecl");
        map.insert(Relations::inputs_Continue, "inputs::Continue");
        map.insert(Relations::inputs_DoWhile, "inputs::DoWhile");
        map.insert(Relations::inputs_DotAccess, "inputs::DotAccess");
        map.insert(Relations::inputs_EveryScope, "inputs::EveryScope");
        map.insert(Relations::inputs_ExprBigInt, "inputs::ExprBigInt");
        map.insert(Relations::inputs_ExprBool, "inputs::ExprBool");
        map.insert(Relations::inputs_ExprNumber, "inputs::ExprNumber");
        map.insert(Relations::inputs_ExprString, "inputs::ExprString");
        map.insert(Relations::inputs_Expression, "inputs::Expression");
        map.insert(Relations::inputs_For, "inputs::For");
        map.insert(Relations::inputs_ForIn, "inputs::ForIn");
        map.insert(Relations::inputs_Function, "inputs::Function");
        map.insert(Relations::inputs_FunctionArg, "inputs::FunctionArg");
        map.insert(Relations::inputs_If, "inputs::If");
        map.insert(Relations::inputs_ImplicitGlobal, "inputs::ImplicitGlobal");
        map.insert(Relations::inputs_ImportDecl, "inputs::ImportDecl");
        map.insert(Relations::inputs_InlineFunc, "inputs::InlineFunc");
        map.insert(Relations::inputs_InlineFuncParam, "inputs::InlineFuncParam");
        map.insert(Relations::inputs_InputScope, "inputs::InputScope");
        map.insert(Relations::inputs_Label, "inputs::Label");
        map.insert(Relations::inputs_LetDecl, "inputs::LetDecl");
        map.insert(Relations::inputs_NameRef, "inputs::NameRef");
        map.insert(Relations::inputs_New, "inputs::New");
        map.insert(Relations::inputs_Property, "inputs::Property");
        map.insert(Relations::inputs_Return, "inputs::Return");
        map.insert(Relations::inputs_Statement, "inputs::Statement");
        map.insert(Relations::inputs_Switch, "inputs::Switch");
        map.insert(Relations::inputs_SwitchCase, "inputs::SwitchCase");
        map.insert(Relations::inputs_Template, "inputs::Template");
        map.insert(Relations::inputs_Ternary, "inputs::Ternary");
        map.insert(Relations::inputs_Throw, "inputs::Throw");
        map.insert(Relations::inputs_Try, "inputs::Try");
        map.insert(Relations::inputs_UnaryOp, "inputs::UnaryOp");
        map.insert(Relations::inputs_VarDecl, "inputs::VarDecl");
        map.insert(Relations::inputs_While, "inputs::While");
        map.insert(Relations::inputs_With, "inputs::With");
        map.insert(Relations::inputs_Yield, "inputs::Yield");
        map
    });
    /// A map of output `Relations`s to their name as an `&'static str`
pub static OUTPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(57, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ChainedWith, "ChainedWith");
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::ClosestFunction, "ClosestFunction");
        map.insert(Relations::INPUT_inputs_Array, "INPUT_inputs::Array");
        map.insert(Relations::INPUT_inputs_Arrow, "INPUT_inputs::Arrow");
        map.insert(Relations::INPUT_inputs_ArrowParam, "INPUT_inputs::ArrowParam");
        map.insert(Relations::INPUT_inputs_Assign, "INPUT_inputs::Assign");
        map.insert(Relations::INPUT_inputs_Await, "INPUT_inputs::Await");
        map.insert(Relations::INPUT_inputs_BinOp, "INPUT_inputs::BinOp");
        map.insert(Relations::INPUT_inputs_BracketAccess, "INPUT_inputs::BracketAccess");
        map.insert(Relations::INPUT_inputs_Break, "INPUT_inputs::Break");
        map.insert(Relations::INPUT_inputs_Call, "INPUT_inputs::Call");
        map.insert(Relations::INPUT_inputs_Class, "INPUT_inputs::Class");
        map.insert(Relations::INPUT_inputs_ClassExpr, "INPUT_inputs::ClassExpr");
        map.insert(Relations::INPUT_inputs_ConstDecl, "INPUT_inputs::ConstDecl");
        map.insert(Relations::INPUT_inputs_Continue, "INPUT_inputs::Continue");
        map.insert(Relations::INPUT_inputs_DoWhile, "INPUT_inputs::DoWhile");
        map.insert(Relations::INPUT_inputs_DotAccess, "INPUT_inputs::DotAccess");
        map.insert(Relations::INPUT_inputs_EveryScope, "INPUT_inputs::EveryScope");
        map.insert(Relations::INPUT_inputs_ExprBigInt, "INPUT_inputs::ExprBigInt");
        map.insert(Relations::INPUT_inputs_ExprBool, "INPUT_inputs::ExprBool");
        map.insert(Relations::INPUT_inputs_ExprNumber, "INPUT_inputs::ExprNumber");
        map.insert(Relations::INPUT_inputs_ExprString, "INPUT_inputs::ExprString");
        map.insert(Relations::INPUT_inputs_Expression, "INPUT_inputs::Expression");
        map.insert(Relations::INPUT_inputs_For, "INPUT_inputs::For");
        map.insert(Relations::INPUT_inputs_ForIn, "INPUT_inputs::ForIn");
        map.insert(Relations::INPUT_inputs_Function, "INPUT_inputs::Function");
        map.insert(Relations::INPUT_inputs_FunctionArg, "INPUT_inputs::FunctionArg");
        map.insert(Relations::INPUT_inputs_If, "INPUT_inputs::If");
        map.insert(Relations::INPUT_inputs_ImplicitGlobal, "INPUT_inputs::ImplicitGlobal");
        map.insert(Relations::INPUT_inputs_ImportDecl, "INPUT_inputs::ImportDecl");
        map.insert(Relations::INPUT_inputs_InlineFunc, "INPUT_inputs::InlineFunc");
        map.insert(Relations::INPUT_inputs_InlineFuncParam, "INPUT_inputs::InlineFuncParam");
        map.insert(Relations::INPUT_inputs_InputScope, "INPUT_inputs::InputScope");
        map.insert(Relations::INPUT_inputs_Label, "INPUT_inputs::Label");
        map.insert(Relations::INPUT_inputs_LetDecl, "INPUT_inputs::LetDecl");
        map.insert(Relations::INPUT_inputs_NameRef, "INPUT_inputs::NameRef");
        map.insert(Relations::INPUT_inputs_New, "INPUT_inputs::New");
        map.insert(Relations::INPUT_inputs_Property, "INPUT_inputs::Property");
        map.insert(Relations::INPUT_inputs_Return, "INPUT_inputs::Return");
        map.insert(Relations::INPUT_inputs_Statement, "INPUT_inputs::Statement");
        map.insert(Relations::INPUT_inputs_Switch, "INPUT_inputs::Switch");
        map.insert(Relations::INPUT_inputs_SwitchCase, "INPUT_inputs::SwitchCase");
        map.insert(Relations::INPUT_inputs_Template, "INPUT_inputs::Template");
        map.insert(Relations::INPUT_inputs_Ternary, "INPUT_inputs::Ternary");
        map.insert(Relations::INPUT_inputs_Throw, "INPUT_inputs::Throw");
        map.insert(Relations::INPUT_inputs_Try, "INPUT_inputs::Try");
        map.insert(Relations::INPUT_inputs_UnaryOp, "INPUT_inputs::UnaryOp");
        map.insert(Relations::INPUT_inputs_VarDecl, "INPUT_inputs::VarDecl");
        map.insert(Relations::INPUT_inputs_While, "INPUT_inputs::While");
        map.insert(Relations::INPUT_inputs_With, "INPUT_inputs::With");
        map.insert(Relations::INPUT_inputs_Yield, "INPUT_inputs::Yield");
        map.insert(Relations::InvalidNameUse, "InvalidNameUse");
        map.insert(Relations::NameInScope, "NameInScope");
        map.insert(Relations::TypeofUndefinedAlwaysUndefined, "TypeofUndefinedAlwaysUndefined");
        map.insert(Relations::VarUseBeforeDeclaration, "VarUseBeforeDeclaration");
        map.insert(Relations::WithinTypeofExpr, "WithinTypeofExpr");
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
        Relations::ChainedWith => {
            Ok(<::types::ChainedWith>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ChildScope => {
            Ok(<::types::ChildScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::ClosestFunction => {
            Ok(<::types::ClosestFunction>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Array => {
            Ok(<::types::inputs::Array>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Arrow => {
            Ok(<::types::inputs::Arrow>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ArrowParam => {
            Ok(<::types::inputs::ArrowParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Assign => {
            Ok(<::types::inputs::Assign>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Await => {
            Ok(<::types::inputs::Await>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_BinOp => {
            Ok(<::types::inputs::BinOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_BracketAccess => {
            Ok(<::types::inputs::BracketAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Break => {
            Ok(<::types::inputs::Break>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Call => {
            Ok(<::types::inputs::Call>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Class => {
            Ok(<::types::inputs::Class>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ClassExpr => {
            Ok(<::types::inputs::ClassExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ConstDecl => {
            Ok(<::types::inputs::ConstDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Continue => {
            Ok(<::types::inputs::Continue>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_DoWhile => {
            Ok(<::types::inputs::DoWhile>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_DotAccess => {
            Ok(<::types::inputs::DotAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_EveryScope => {
            Ok(<::types::inputs::EveryScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ExprBigInt => {
            Ok(<::types::inputs::ExprBigInt>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ExprBool => {
            Ok(<::types::inputs::ExprBool>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ExprNumber => {
            Ok(<::types::inputs::ExprNumber>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ExprString => {
            Ok(<::types::inputs::ExprString>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Expression => {
            Ok(<::types::inputs::Expression>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_For => {
            Ok(<::types::inputs::For>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ForIn => {
            Ok(<::types::inputs::ForIn>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Function => {
            Ok(<::types::inputs::Function>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_FunctionArg => {
            Ok(<::types::inputs::FunctionArg>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_If => {
            Ok(<::types::inputs::If>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ImplicitGlobal => {
            Ok(<::types::inputs::ImplicitGlobal>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_ImportDecl => {
            Ok(<::types::inputs::ImportDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_InlineFunc => {
            Ok(<::types::inputs::InlineFunc>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_InlineFuncParam => {
            Ok(<::types::inputs::InlineFuncParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_InputScope => {
            Ok(<::types::inputs::InputScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Label => {
            Ok(<::types::inputs::Label>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_LetDecl => {
            Ok(<::types::inputs::LetDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_NameRef => {
            Ok(<::types::inputs::NameRef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_New => {
            Ok(<::types::inputs::New>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Property => {
            Ok(<::types::inputs::Property>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Return => {
            Ok(<::types::inputs::Return>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Statement => {
            Ok(<::types::inputs::Statement>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Switch => {
            Ok(<::types::inputs::Switch>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_SwitchCase => {
            Ok(<::types::inputs::SwitchCase>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Template => {
            Ok(<::types::inputs::Template>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Ternary => {
            Ok(<::types::inputs::Ternary>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Throw => {
            Ok(<::types::inputs::Throw>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Try => {
            Ok(<::types::inputs::Try>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_UnaryOp => {
            Ok(<::types::inputs::UnaryOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_VarDecl => {
            Ok(<::types::inputs::VarDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_While => {
            Ok(<::types::inputs::While>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_With => {
            Ok(<::types::inputs::With>::from_record(_rec)?.into_ddvalue())
        },
        Relations::INPUT_inputs_Yield => {
            Ok(<::types::inputs::Yield>::from_record(_rec)?.into_ddvalue())
        },
        Relations::InvalidNameUse => {
            Ok(<::types::InvalidNameUse>::from_record(_rec)?.into_ddvalue())
        },
        Relations::NameInScope => {
            Ok(<::types::NameInScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::TypeofUndefinedAlwaysUndefined => {
            Ok(<::types::TypeofUndefinedAlwaysUndefined>::from_record(_rec)?.into_ddvalue())
        },
        Relations::VarUseBeforeDeclaration => {
            Ok(<::types::VarUseBeforeDeclaration>::from_record(_rec)?.into_ddvalue())
        },
        Relations::WithinTypeofExpr => {
            Ok(<::types::WithinTypeofExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::__Prefix_0 => {
            Ok(<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_record(_rec)?.into_ddvalue())
        },
        Relations::__Prefix_1 => {
            Ok(<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::internment::Intern<::types::ast::Pattern>>>::from_record(_rec)?.into_ddvalue())
        },
        Relations::__Prefix_2 => {
            Ok(<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::internment::Intern<::types::ast::Pattern>>>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Array => {
            Ok(<::types::inputs::Array>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Arrow => {
            Ok(<::types::inputs::Arrow>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ArrowParam => {
            Ok(<::types::inputs::ArrowParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Assign => {
            Ok(<::types::inputs::Assign>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Await => {
            Ok(<::types::inputs::Await>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_BinOp => {
            Ok(<::types::inputs::BinOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_BracketAccess => {
            Ok(<::types::inputs::BracketAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Break => {
            Ok(<::types::inputs::Break>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Call => {
            Ok(<::types::inputs::Call>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Class => {
            Ok(<::types::inputs::Class>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ClassExpr => {
            Ok(<::types::inputs::ClassExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ConstDecl => {
            Ok(<::types::inputs::ConstDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Continue => {
            Ok(<::types::inputs::Continue>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_DoWhile => {
            Ok(<::types::inputs::DoWhile>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_DotAccess => {
            Ok(<::types::inputs::DotAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_EveryScope => {
            Ok(<::types::inputs::EveryScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprBigInt => {
            Ok(<::types::inputs::ExprBigInt>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprBool => {
            Ok(<::types::inputs::ExprBool>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprNumber => {
            Ok(<::types::inputs::ExprNumber>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprString => {
            Ok(<::types::inputs::ExprString>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Expression => {
            Ok(<::types::inputs::Expression>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_For => {
            Ok(<::types::inputs::For>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ForIn => {
            Ok(<::types::inputs::ForIn>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Function => {
            Ok(<::types::inputs::Function>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_FunctionArg => {
            Ok(<::types::inputs::FunctionArg>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_If => {
            Ok(<::types::inputs::If>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ImplicitGlobal => {
            Ok(<::types::inputs::ImplicitGlobal>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ImportDecl => {
            Ok(<::types::inputs::ImportDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_InlineFunc => {
            Ok(<::types::inputs::InlineFunc>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_InlineFuncParam => {
            Ok(<::types::inputs::InlineFuncParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_InputScope => {
            Ok(<::types::inputs::InputScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Label => {
            Ok(<::types::inputs::Label>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_LetDecl => {
            Ok(<::types::inputs::LetDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_NameRef => {
            Ok(<::types::inputs::NameRef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_New => {
            Ok(<::types::inputs::New>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Property => {
            Ok(<::types::inputs::Property>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Return => {
            Ok(<::types::inputs::Return>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Statement => {
            Ok(<::types::inputs::Statement>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Switch => {
            Ok(<::types::inputs::Switch>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_SwitchCase => {
            Ok(<::types::inputs::SwitchCase>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Template => {
            Ok(<::types::inputs::Template>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Ternary => {
            Ok(<::types::inputs::Ternary>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Throw => {
            Ok(<::types::inputs::Throw>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Try => {
            Ok(<::types::inputs::Try>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_UnaryOp => {
            Ok(<::types::inputs::UnaryOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_VarDecl => {
            Ok(<::types::inputs::VarDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_While => {
            Ok(<::types::inputs::While>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_With => {
            Ok(<::types::inputs::With>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Yield => {
            Ok(<::types::inputs::Yield>::from_record(_rec)?.into_ddvalue())
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
            Ok(<::types::ddlog_std::tuple2<::types::ast::Scope, ::types::internment::Intern<String>>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::Index_VariablesForScope => {
            Ok(<::types::ast::Scope>::from_record(_rec)?.into_ddvalue())
        }
    }
}
pub fn indexes2arrid(idx: Indexes) -> ArrId {
    match idx {
        Indexes::Index_InvalidNameUse => ( 52, 0),
        Indexes::Index_VarUseBeforeDeclaration => ( 55, 0),
        Indexes::Index_VariableInScope => ( 53, 5),
        Indexes::Index_VariablesForScope => ( 53, 6),
    }
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Relations {
    ChainedWith = 0,
    ChildScope = 1,
    ClosestFunction = 2,
    INPUT_inputs_Array = 3,
    INPUT_inputs_Arrow = 4,
    INPUT_inputs_ArrowParam = 5,
    INPUT_inputs_Assign = 6,
    INPUT_inputs_Await = 7,
    INPUT_inputs_BinOp = 8,
    INPUT_inputs_BracketAccess = 9,
    INPUT_inputs_Break = 10,
    INPUT_inputs_Call = 11,
    INPUT_inputs_Class = 12,
    INPUT_inputs_ClassExpr = 13,
    INPUT_inputs_ConstDecl = 14,
    INPUT_inputs_Continue = 15,
    INPUT_inputs_DoWhile = 16,
    INPUT_inputs_DotAccess = 17,
    INPUT_inputs_EveryScope = 18,
    INPUT_inputs_ExprBigInt = 19,
    INPUT_inputs_ExprBool = 20,
    INPUT_inputs_ExprNumber = 21,
    INPUT_inputs_ExprString = 22,
    INPUT_inputs_Expression = 23,
    INPUT_inputs_For = 24,
    INPUT_inputs_ForIn = 25,
    INPUT_inputs_Function = 26,
    INPUT_inputs_FunctionArg = 27,
    INPUT_inputs_If = 28,
    INPUT_inputs_ImplicitGlobal = 29,
    INPUT_inputs_ImportDecl = 30,
    INPUT_inputs_InlineFunc = 31,
    INPUT_inputs_InlineFuncParam = 32,
    INPUT_inputs_InputScope = 33,
    INPUT_inputs_Label = 34,
    INPUT_inputs_LetDecl = 35,
    INPUT_inputs_NameRef = 36,
    INPUT_inputs_New = 37,
    INPUT_inputs_Property = 38,
    INPUT_inputs_Return = 39,
    INPUT_inputs_Statement = 40,
    INPUT_inputs_Switch = 41,
    INPUT_inputs_SwitchCase = 42,
    INPUT_inputs_Template = 43,
    INPUT_inputs_Ternary = 44,
    INPUT_inputs_Throw = 45,
    INPUT_inputs_Try = 46,
    INPUT_inputs_UnaryOp = 47,
    INPUT_inputs_VarDecl = 48,
    INPUT_inputs_While = 49,
    INPUT_inputs_With = 50,
    INPUT_inputs_Yield = 51,
    InvalidNameUse = 52,
    NameInScope = 53,
    TypeofUndefinedAlwaysUndefined = 54,
    VarUseBeforeDeclaration = 55,
    WithinTypeofExpr = 56,
    __Prefix_0 = 57,
    __Prefix_1 = 58,
    __Prefix_2 = 59,
    inputs_Array = 60,
    inputs_Arrow = 61,
    inputs_ArrowParam = 62,
    inputs_Assign = 63,
    inputs_Await = 64,
    inputs_BinOp = 65,
    inputs_BracketAccess = 66,
    inputs_Break = 67,
    inputs_Call = 68,
    inputs_Class = 69,
    inputs_ClassExpr = 70,
    inputs_ConstDecl = 71,
    inputs_Continue = 72,
    inputs_DoWhile = 73,
    inputs_DotAccess = 74,
    inputs_EveryScope = 75,
    inputs_ExprBigInt = 76,
    inputs_ExprBool = 77,
    inputs_ExprNumber = 78,
    inputs_ExprString = 79,
    inputs_Expression = 80,
    inputs_For = 81,
    inputs_ForIn = 82,
    inputs_Function = 83,
    inputs_FunctionArg = 84,
    inputs_If = 85,
    inputs_ImplicitGlobal = 86,
    inputs_ImportDecl = 87,
    inputs_InlineFunc = 88,
    inputs_InlineFuncParam = 89,
    inputs_InputScope = 90,
    inputs_Label = 91,
    inputs_LetDecl = 92,
    inputs_NameRef = 93,
    inputs_New = 94,
    inputs_Property = 95,
    inputs_Return = 96,
    inputs_Statement = 97,
    inputs_Switch = 98,
    inputs_SwitchCase = 99,
    inputs_Template = 100,
    inputs_Ternary = 101,
    inputs_Throw = 102,
    inputs_Try = 103,
    inputs_UnaryOp = 104,
    inputs_VarDecl = 105,
    inputs_While = 106,
    inputs_With = 107,
    inputs_Yield = 108
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Indexes {
    Index_InvalidNameUse = 0,
    Index_VarUseBeforeDeclaration = 1,
    Index_VariableInScope = 2,
    Index_VariablesForScope = 3
}
pub fn prog(__update_cb: Box<dyn CBFn>) -> Program {
    let inputs_Array = Relation {
                           name:         "inputs::Array".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Array as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_Array = Relation {
                                 name:         "INPUT_inputs::Array".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Array as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Array[x] :- inputs::Array[(x: inputs::Array)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Array[x] :- inputs::Array[(x: inputs::Array)].".to_string(),
                                         rel: Relations::inputs_Array as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Array[x] :- inputs::Array[(x: inputs::Array)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Array>::from_ddvalue_ref(&__v) } {
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
    let inputs_Arrow = Relation {
                           name:         "inputs::Arrow".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Arrow as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               Arrangement::Map{
                                  name: r###"(inputs::Arrow{.expr_id=(_0: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow) /*join*/"###.to_string(),
                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                   {
                                       let __cloned = __v.clone();
                                       match unsafe {< ::types::inputs::Arrow>::from_ddvalue(__v) } {
                                           ::types::inputs::Arrow{expr_id: ref _0, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                           _ => None
                                       }.map(|x|(x,__cloned))
                                   }
                                   __f},
                                   queryable: false
                               },
                               Arrangement::Map{
                                  name: r###"(inputs::Arrow{.expr_id=(_0: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(_: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow) /*join*/"###.to_string(),
                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                   {
                                       let __cloned = __v.clone();
                                       match unsafe {< ::types::inputs::Arrow>::from_ddvalue(__v) } {
                                           ::types::inputs::Arrow{expr_id: ref _0, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Right{r: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                           _ => None
                                       }.map(|x|(x,__cloned))
                                   }
                                   __f},
                                   queryable: false
                               }],
                           change_cb:    None
                       };
    let INPUT_inputs_Arrow = Relation {
                                 name:         "INPUT_inputs::Arrow".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Arrow as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Arrow[x] :- inputs::Arrow[(x: inputs::Arrow)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Arrow[x] :- inputs::Arrow[(x: inputs::Arrow)].".to_string(),
                                         rel: Relations::inputs_Arrow as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Arrow[x] :- inputs::Arrow[(x: inputs::Arrow)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Arrow>::from_ddvalue_ref(&__v) } {
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
    let inputs_ArrowParam = Relation {
                                name:         "inputs::ArrowParam".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_ArrowParam as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    ],
                                change_cb:    None
                            };
    let INPUT_inputs_ArrowParam = Relation {
                                      name:         "INPUT_inputs::ArrowParam".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_ArrowParam as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::ArrowParam[x] :- inputs::ArrowParam[(x: inputs::ArrowParam)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::ArrowParam[x] :- inputs::ArrowParam[(x: inputs::ArrowParam)].".to_string(),
                                              rel: Relations::inputs_ArrowParam as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::ArrowParam[x] :- inputs::ArrowParam[(x: inputs::ArrowParam)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::ArrowParam>::from_ddvalue_ref(&__v) } {
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
                             /* __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))). */
                             Rule::CollectionRule {
                                 description: "__Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))).".to_string(),
                                 rel: Relations::inputs_ArrowParam as RelId,
                                 xform: Some(XFormCollection::FlatMap{
                                                 description: "inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                 {
                                                     let (ref expr, ref pat) = match *unsafe {<::types::inputs::ArrowParam>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::ArrowParam{expr_id: ref expr, param: ref pat} => ((*expr).clone(), (*pat).clone()),
                                                         _ => return None
                                                     };
                                                     let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                     let expr = (*expr).clone();
                                                     let pat = (*pat).clone();
                                                     Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple3(name.clone(), expr.clone(), pat.clone())).into_ddvalue())))
                                                 }
                                                 __f},
                                                 next: Box::new(Some(XFormCollection::FilterMap{
                                                                         description: "head of __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))." .to_string(),
                                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                         {
                                                                             let ::types::ddlog_std::tuple3(ref name, ref expr, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref( &__v ) };
                                                                             Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*pat).clone())).into_ddvalue())
                                                                         }
                                                                         __f},
                                                                         next: Box::new(None)
                                                                     }))
                                             })
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"((_: ast::Spanned<ast::Name>), (_0: ast::ExprId), (_: internment::Intern<ast::Pattern>)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue(__v) } {
                                         ::types::ddlog_std::tuple3(_, ref _0, _) => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let inputs_Assign = Relation {
                            name:         "inputs::Assign".to_string(),
                            input:        true,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::inputs_Assign as RelId,
                            rules:        vec![
                                ],
                            arrangements: vec![
                                Arrangement::Map{
                                   name: r###"(inputs::Assign{.expr_id=(_0: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign) /*join*/"###.to_string(),
                                    afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                    {
                                        let __cloned = __v.clone();
                                        match unsafe {< ::types::inputs::Assign>::from_ddvalue(__v) } {
                                            ::types::inputs::Assign{expr_id: ref _0, lhs: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: _}}, rhs: _, op: _} => Some(((*_0).clone()).into_ddvalue()),
                                            _ => None
                                        }.map(|x|(x,__cloned))
                                    }
                                    __f},
                                    queryable: false
                                }],
                            change_cb:    None
                        };
    let INPUT_inputs_Assign = Relation {
                                  name:         "INPUT_inputs::Assign".to_string(),
                                  input:        false,
                                  distinct:     false,
                                  caching_mode: CachingMode::Set,
                                  key_func:     None,
                                  id:           Relations::INPUT_inputs_Assign as RelId,
                                  rules:        vec![
                                      /* INPUT_inputs::Assign[x] :- inputs::Assign[(x: inputs::Assign)]. */
                                      Rule::CollectionRule {
                                          description: "INPUT_inputs::Assign[x] :- inputs::Assign[(x: inputs::Assign)].".to_string(),
                                          rel: Relations::inputs_Assign as RelId,
                                          xform: Some(XFormCollection::FilterMap{
                                                          description: "head of INPUT_inputs::Assign[x] :- inputs::Assign[(x: inputs::Assign)]." .to_string(),
                                                          fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                          {
                                                              let ref x = match *unsafe {<::types::inputs::Assign>::from_ddvalue_ref(&__v) } {
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
    let inputs_Await = Relation {
                           name:         "inputs::Await".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Await as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_Await = Relation {
                                 name:         "INPUT_inputs::Await".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Await as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Await[x] :- inputs::Await[(x: inputs::Await)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Await[x] :- inputs::Await[(x: inputs::Await)].".to_string(),
                                         rel: Relations::inputs_Await as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Await[x] :- inputs::Await[(x: inputs::Await)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Await>::from_ddvalue_ref(&__v) } {
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
    let inputs_BinOp = Relation {
                           name:         "inputs::BinOp".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_BinOp as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_BinOp = Relation {
                                 name:         "INPUT_inputs::BinOp".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_BinOp as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::BinOp[x] :- inputs::BinOp[(x: inputs::BinOp)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::BinOp[x] :- inputs::BinOp[(x: inputs::BinOp)].".to_string(),
                                         rel: Relations::inputs_BinOp as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::BinOp[x] :- inputs::BinOp[(x: inputs::BinOp)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::BinOp>::from_ddvalue_ref(&__v) } {
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
    let inputs_BracketAccess = Relation {
                                   name:         "inputs::BracketAccess".to_string(),
                                   input:        true,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::inputs_BracketAccess as RelId,
                                   rules:        vec![
                                       ],
                                   arrangements: vec![
                                       ],
                                   change_cb:    None
                               };
    let INPUT_inputs_BracketAccess = Relation {
                                         name:         "INPUT_inputs::BracketAccess".to_string(),
                                         input:        false,
                                         distinct:     false,
                                         caching_mode: CachingMode::Set,
                                         key_func:     None,
                                         id:           Relations::INPUT_inputs_BracketAccess as RelId,
                                         rules:        vec![
                                             /* INPUT_inputs::BracketAccess[x] :- inputs::BracketAccess[(x: inputs::BracketAccess)]. */
                                             Rule::CollectionRule {
                                                 description: "INPUT_inputs::BracketAccess[x] :- inputs::BracketAccess[(x: inputs::BracketAccess)].".to_string(),
                                                 rel: Relations::inputs_BracketAccess as RelId,
                                                 xform: Some(XFormCollection::FilterMap{
                                                                 description: "head of INPUT_inputs::BracketAccess[x] :- inputs::BracketAccess[(x: inputs::BracketAccess)]." .to_string(),
                                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                 {
                                                                     let ref x = match *unsafe {<::types::inputs::BracketAccess>::from_ddvalue_ref(&__v) } {
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
    let inputs_Break = Relation {
                           name:         "inputs::Break".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Break as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_Break = Relation {
                                 name:         "INPUT_inputs::Break".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Break as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Break[x] :- inputs::Break[(x: inputs::Break)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Break[x] :- inputs::Break[(x: inputs::Break)].".to_string(),
                                         rel: Relations::inputs_Break as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Break[x] :- inputs::Break[(x: inputs::Break)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Break>::from_ddvalue_ref(&__v) } {
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
    let inputs_Call = Relation {
                          name:         "inputs::Call".to_string(),
                          input:        true,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::inputs_Call as RelId,
                          rules:        vec![
                              ],
                          arrangements: vec![
                              ],
                          change_cb:    None
                      };
    let INPUT_inputs_Call = Relation {
                                name:         "INPUT_inputs::Call".to_string(),
                                input:        false,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::INPUT_inputs_Call as RelId,
                                rules:        vec![
                                    /* INPUT_inputs::Call[x] :- inputs::Call[(x: inputs::Call)]. */
                                    Rule::CollectionRule {
                                        description: "INPUT_inputs::Call[x] :- inputs::Call[(x: inputs::Call)].".to_string(),
                                        rel: Relations::inputs_Call as RelId,
                                        xform: Some(XFormCollection::FilterMap{
                                                        description: "head of INPUT_inputs::Call[x] :- inputs::Call[(x: inputs::Call)]." .to_string(),
                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                        {
                                                            let ref x = match *unsafe {<::types::inputs::Call>::from_ddvalue_ref(&__v) } {
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
    let inputs_Class = Relation {
                           name:         "inputs::Class".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Class as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_Class = Relation {
                                 name:         "INPUT_inputs::Class".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Class as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Class[x] :- inputs::Class[(x: inputs::Class)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Class[x] :- inputs::Class[(x: inputs::Class)].".to_string(),
                                         rel: Relations::inputs_Class as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Class[x] :- inputs::Class[(x: inputs::Class)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Class>::from_ddvalue_ref(&__v) } {
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
    let inputs_ClassExpr = Relation {
                               name:         "inputs::ClassExpr".to_string(),
                               input:        true,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::inputs_ClassExpr as RelId,
                               rules:        vec![
                                   ],
                               arrangements: vec![
                                   ],
                               change_cb:    None
                           };
    let INPUT_inputs_ClassExpr = Relation {
                                     name:         "INPUT_inputs::ClassExpr".to_string(),
                                     input:        false,
                                     distinct:     false,
                                     caching_mode: CachingMode::Set,
                                     key_func:     None,
                                     id:           Relations::INPUT_inputs_ClassExpr as RelId,
                                     rules:        vec![
                                         /* INPUT_inputs::ClassExpr[x] :- inputs::ClassExpr[(x: inputs::ClassExpr)]. */
                                         Rule::CollectionRule {
                                             description: "INPUT_inputs::ClassExpr[x] :- inputs::ClassExpr[(x: inputs::ClassExpr)].".to_string(),
                                             rel: Relations::inputs_ClassExpr as RelId,
                                             xform: Some(XFormCollection::FilterMap{
                                                             description: "head of INPUT_inputs::ClassExpr[x] :- inputs::ClassExpr[(x: inputs::ClassExpr)]." .to_string(),
                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                             {
                                                                 let ref x = match *unsafe {<::types::inputs::ClassExpr>::from_ddvalue_ref(&__v) } {
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
    let inputs_ConstDecl = Relation {
                               name:         "inputs::ConstDecl".to_string(),
                               input:        true,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::inputs_ConstDecl as RelId,
                               rules:        vec![
                                   ],
                               arrangements: vec![
                                   ],
                               change_cb:    None
                           };
    let INPUT_inputs_ConstDecl = Relation {
                                     name:         "INPUT_inputs::ConstDecl".to_string(),
                                     input:        false,
                                     distinct:     false,
                                     caching_mode: CachingMode::Set,
                                     key_func:     None,
                                     id:           Relations::INPUT_inputs_ConstDecl as RelId,
                                     rules:        vec![
                                         /* INPUT_inputs::ConstDecl[x] :- inputs::ConstDecl[(x: inputs::ConstDecl)]. */
                                         Rule::CollectionRule {
                                             description: "INPUT_inputs::ConstDecl[x] :- inputs::ConstDecl[(x: inputs::ConstDecl)].".to_string(),
                                             rel: Relations::inputs_ConstDecl as RelId,
                                             xform: Some(XFormCollection::FilterMap{
                                                             description: "head of INPUT_inputs::ConstDecl[x] :- inputs::ConstDecl[(x: inputs::ConstDecl)]." .to_string(),
                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                             {
                                                                 let ref x = match *unsafe {<::types::inputs::ConstDecl>::from_ddvalue_ref(&__v) } {
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
    let inputs_Continue = Relation {
                              name:         "inputs::Continue".to_string(),
                              input:        true,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::inputs_Continue as RelId,
                              rules:        vec![
                                  ],
                              arrangements: vec![
                                  ],
                              change_cb:    None
                          };
    let INPUT_inputs_Continue = Relation {
                                    name:         "INPUT_inputs::Continue".to_string(),
                                    input:        false,
                                    distinct:     false,
                                    caching_mode: CachingMode::Set,
                                    key_func:     None,
                                    id:           Relations::INPUT_inputs_Continue as RelId,
                                    rules:        vec![
                                        /* INPUT_inputs::Continue[x] :- inputs::Continue[(x: inputs::Continue)]. */
                                        Rule::CollectionRule {
                                            description: "INPUT_inputs::Continue[x] :- inputs::Continue[(x: inputs::Continue)].".to_string(),
                                            rel: Relations::inputs_Continue as RelId,
                                            xform: Some(XFormCollection::FilterMap{
                                                            description: "head of INPUT_inputs::Continue[x] :- inputs::Continue[(x: inputs::Continue)]." .to_string(),
                                                            fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                            {
                                                                let ref x = match *unsafe {<::types::inputs::Continue>::from_ddvalue_ref(&__v) } {
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
    let inputs_DoWhile = Relation {
                             name:         "inputs::DoWhile".to_string(),
                             input:        true,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::inputs_DoWhile as RelId,
                             rules:        vec![
                                 ],
                             arrangements: vec![
                                 ],
                             change_cb:    None
                         };
    let INPUT_inputs_DoWhile = Relation {
                                   name:         "INPUT_inputs::DoWhile".to_string(),
                                   input:        false,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::INPUT_inputs_DoWhile as RelId,
                                   rules:        vec![
                                       /* INPUT_inputs::DoWhile[x] :- inputs::DoWhile[(x: inputs::DoWhile)]. */
                                       Rule::CollectionRule {
                                           description: "INPUT_inputs::DoWhile[x] :- inputs::DoWhile[(x: inputs::DoWhile)].".to_string(),
                                           rel: Relations::inputs_DoWhile as RelId,
                                           xform: Some(XFormCollection::FilterMap{
                                                           description: "head of INPUT_inputs::DoWhile[x] :- inputs::DoWhile[(x: inputs::DoWhile)]." .to_string(),
                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                           {
                                                               let ref x = match *unsafe {<::types::inputs::DoWhile>::from_ddvalue_ref(&__v) } {
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
    let inputs_DotAccess = Relation {
                               name:         "inputs::DotAccess".to_string(),
                               input:        true,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::inputs_DotAccess as RelId,
                               rules:        vec![
                                   ],
                               arrangements: vec![
                                   ],
                               change_cb:    None
                           };
    let ChainedWith = Relation {
                          name:         "ChainedWith".to_string(),
                          input:        false,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::ChainedWith as RelId,
                          rules:        vec![
                              /* ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- inputs::BracketAccess[(inputs::BracketAccess{.expr_id=(_: ast::ExprId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(property: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess)]. */
                              Rule::CollectionRule {
                                  description: "ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- inputs::BracketAccess[(inputs::BracketAccess{.expr_id=(_: ast::ExprId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(property: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess)].".to_string(),
                                  rel: Relations::inputs_BracketAccess as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- inputs::BracketAccess[(inputs::BracketAccess{.expr_id=(_: ast::ExprId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(property: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref object, ref property) = match *unsafe {<::types::inputs::BracketAccess>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::BracketAccess{expr_id: _, object: ::types::ddlog_std::Option::Some{x: ref object}, prop: ::types::ddlog_std::Option::Some{x: ref property}} => ((*object).clone(), (*property).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::ChainedWith{object: (*object).clone(), property: (*property).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- inputs::DotAccess[(inputs::DotAccess{.expr_id=(property: ast::ExprId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess)]. */
                              Rule::CollectionRule {
                                  description: "ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- inputs::DotAccess[(inputs::DotAccess{.expr_id=(property: ast::ExprId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess)].".to_string(),
                                  rel: Relations::inputs_DotAccess as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- inputs::DotAccess[(inputs::DotAccess{.expr_id=(property: ast::ExprId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref property, ref object) = match *unsafe {<::types::inputs::DotAccess>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::DotAccess{expr_id: ref property, object: ::types::ddlog_std::Option::Some{x: ref object}, prop: _} => ((*property).clone(), (*object).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::ChainedWith{object: (*object).clone(), property: (*property).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(interum: ast::ExprId)}: ChainedWith)], ChainedWith[(ChainedWith{.object=(interum: ast::ExprId), .property=(property: ast::ExprId)}: ChainedWith)]. */
                              Rule::ArrangementRule {
                                  description: "ChainedWith[(ChainedWith{.object=object, .property=property}: ChainedWith)] :- ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(interum: ast::ExprId)}: ChainedWith)], ChainedWith[(ChainedWith{.object=(interum: ast::ExprId), .property=(property: ast::ExprId)}: ChainedWith)].".to_string(),
                                  arr: ( Relations::ChainedWith as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(interum: ast::ExprId)}: ChainedWith)], ChainedWith[(ChainedWith{.object=(interum: ast::ExprId), .property=(property: ast::ExprId)}: ChainedWith)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::ChainedWith as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref object, ref interum) = match *unsafe {<::types::ChainedWith>::from_ddvalue_ref(__v1) } {
                                                     ::types::ChainedWith{object: ref object, property: ref interum} => ((*object).clone(), (*interum).clone()),
                                                     _ => return None
                                                 };
                                                 let ref property = match *unsafe {<::types::ChainedWith>::from_ddvalue_ref(__v2) } {
                                                     ::types::ChainedWith{object: _, property: ref property} => (*property).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::ChainedWith{object: (*object).clone(), property: (*property).clone()})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              }],
                          arrangements: vec![
                              Arrangement::Map{
                                 name: r###"(ChainedWith{.object=(_: ast::ExprId), .property=(_0: ast::ExprId)}: ChainedWith) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::ChainedWith>::from_ddvalue(__v) } {
                                          ::types::ChainedWith{object: _, property: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(ChainedWith{.object=(_0: ast::ExprId), .property=(_: ast::ExprId)}: ChainedWith) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::ChainedWith>::from_ddvalue(__v) } {
                                          ::types::ChainedWith{object: ref _0, property: _} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Set{
                                  name: r###"(ChainedWith{.object=(_: ast::ExprId), .property=(_0: ast::ExprId)}: ChainedWith) /*antijoin*/"###.to_string(),
                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                  {
                                      match unsafe {< ::types::ChainedWith>::from_ddvalue(__v) } {
                                          ::types::ChainedWith{object: _, property: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }
                                  }
                                  __f},
                                  distinct: true
                              }],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let INPUT_inputs_DotAccess = Relation {
                                     name:         "INPUT_inputs::DotAccess".to_string(),
                                     input:        false,
                                     distinct:     false,
                                     caching_mode: CachingMode::Set,
                                     key_func:     None,
                                     id:           Relations::INPUT_inputs_DotAccess as RelId,
                                     rules:        vec![
                                         /* INPUT_inputs::DotAccess[x] :- inputs::DotAccess[(x: inputs::DotAccess)]. */
                                         Rule::CollectionRule {
                                             description: "INPUT_inputs::DotAccess[x] :- inputs::DotAccess[(x: inputs::DotAccess)].".to_string(),
                                             rel: Relations::inputs_DotAccess as RelId,
                                             xform: Some(XFormCollection::FilterMap{
                                                             description: "head of INPUT_inputs::DotAccess[x] :- inputs::DotAccess[(x: inputs::DotAccess)]." .to_string(),
                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                             {
                                                                 let ref x = match *unsafe {<::types::inputs::DotAccess>::from_ddvalue_ref(&__v) } {
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
    let inputs_EveryScope = Relation {
                                name:         "inputs::EveryScope".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_EveryScope as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    Arrangement::Map{
                                       name: r###"(inputs::EveryScope{.scope=(_: ast::Scope)}: inputs::EveryScope) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::EveryScope>::from_ddvalue(__v) } {
                                                ::types::inputs::EveryScope{scope: _} => Some((()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    }],
                                change_cb:    None
                            };
    let INPUT_inputs_EveryScope = Relation {
                                      name:         "INPUT_inputs::EveryScope".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_EveryScope as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::EveryScope[x] :- inputs::EveryScope[(x: inputs::EveryScope)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::EveryScope[x] :- inputs::EveryScope[(x: inputs::EveryScope)].".to_string(),
                                              rel: Relations::inputs_EveryScope as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::EveryScope[x] :- inputs::EveryScope[(x: inputs::EveryScope)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::EveryScope>::from_ddvalue_ref(&__v) } {
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
    let inputs_ExprBigInt = Relation {
                                name:         "inputs::ExprBigInt".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_ExprBigInt as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    ],
                                change_cb:    None
                            };
    let INPUT_inputs_ExprBigInt = Relation {
                                      name:         "INPUT_inputs::ExprBigInt".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_ExprBigInt as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::ExprBigInt[x] :- inputs::ExprBigInt[(x: inputs::ExprBigInt)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::ExprBigInt[x] :- inputs::ExprBigInt[(x: inputs::ExprBigInt)].".to_string(),
                                              rel: Relations::inputs_ExprBigInt as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::ExprBigInt[x] :- inputs::ExprBigInt[(x: inputs::ExprBigInt)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::ExprBigInt>::from_ddvalue_ref(&__v) } {
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
    let inputs_ExprBool = Relation {
                              name:         "inputs::ExprBool".to_string(),
                              input:        true,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::inputs_ExprBool as RelId,
                              rules:        vec![
                                  ],
                              arrangements: vec![
                                  ],
                              change_cb:    None
                          };
    let INPUT_inputs_ExprBool = Relation {
                                    name:         "INPUT_inputs::ExprBool".to_string(),
                                    input:        false,
                                    distinct:     false,
                                    caching_mode: CachingMode::Set,
                                    key_func:     None,
                                    id:           Relations::INPUT_inputs_ExprBool as RelId,
                                    rules:        vec![
                                        /* INPUT_inputs::ExprBool[x] :- inputs::ExprBool[(x: inputs::ExprBool)]. */
                                        Rule::CollectionRule {
                                            description: "INPUT_inputs::ExprBool[x] :- inputs::ExprBool[(x: inputs::ExprBool)].".to_string(),
                                            rel: Relations::inputs_ExprBool as RelId,
                                            xform: Some(XFormCollection::FilterMap{
                                                            description: "head of INPUT_inputs::ExprBool[x] :- inputs::ExprBool[(x: inputs::ExprBool)]." .to_string(),
                                                            fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                            {
                                                                let ref x = match *unsafe {<::types::inputs::ExprBool>::from_ddvalue_ref(&__v) } {
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
    let inputs_ExprNumber = Relation {
                                name:         "inputs::ExprNumber".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_ExprNumber as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    ],
                                change_cb:    None
                            };
    let INPUT_inputs_ExprNumber = Relation {
                                      name:         "INPUT_inputs::ExprNumber".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_ExprNumber as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::ExprNumber[x] :- inputs::ExprNumber[(x: inputs::ExprNumber)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::ExprNumber[x] :- inputs::ExprNumber[(x: inputs::ExprNumber)].".to_string(),
                                              rel: Relations::inputs_ExprNumber as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::ExprNumber[x] :- inputs::ExprNumber[(x: inputs::ExprNumber)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::ExprNumber>::from_ddvalue_ref(&__v) } {
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
    let inputs_ExprString = Relation {
                                name:         "inputs::ExprString".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_ExprString as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    ],
                                change_cb:    None
                            };
    let INPUT_inputs_ExprString = Relation {
                                      name:         "INPUT_inputs::ExprString".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_ExprString as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::ExprString[x] :- inputs::ExprString[(x: inputs::ExprString)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::ExprString[x] :- inputs::ExprString[(x: inputs::ExprString)].".to_string(),
                                              rel: Relations::inputs_ExprString as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::ExprString[x] :- inputs::ExprString[(x: inputs::ExprString)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::ExprString>::from_ddvalue_ref(&__v) } {
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
    let inputs_Expression = Relation {
                                name:         "inputs::Expression".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_Expression as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, kind: ::types::ast::ExprKind::ExprGrouping{inner: ::types::ddlog_std::Option::Some{x: _}}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(ast::ExprSequence{.exprs=(_: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, kind: ::types::ast::ExprKind::ExprSequence{exprs: _}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    }],
                                change_cb:    None
                            };
    let INPUT_inputs_Expression = Relation {
                                      name:         "INPUT_inputs::Expression".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_Expression as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::Expression[x] :- inputs::Expression[(x: inputs::Expression)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::Expression[x] :- inputs::Expression[(x: inputs::Expression)].".to_string(),
                                              rel: Relations::inputs_Expression as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::Expression[x] :- inputs::Expression[(x: inputs::Expression)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(&__v) } {
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
    let inputs_For = Relation {
                         name:         "inputs::For".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::inputs_For as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_inputs_For = Relation {
                               name:         "INPUT_inputs::For".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_inputs_For as RelId,
                               rules:        vec![
                                   /* INPUT_inputs::For[x] :- inputs::For[(x: inputs::For)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_inputs::For[x] :- inputs::For[(x: inputs::For)].".to_string(),
                                       rel: Relations::inputs_For as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_inputs::For[x] :- inputs::For[(x: inputs::For)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::inputs::For>::from_ddvalue_ref(&__v) } {
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
    let inputs_ForIn = Relation {
                           name:         "inputs::ForIn".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_ForIn as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_ForIn = Relation {
                                 name:         "INPUT_inputs::ForIn".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_ForIn as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::ForIn[x] :- inputs::ForIn[(x: inputs::ForIn)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::ForIn[x] :- inputs::ForIn[(x: inputs::ForIn)].".to_string(),
                                         rel: Relations::inputs_ForIn as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::ForIn[x] :- inputs::ForIn[(x: inputs::ForIn)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::ForIn>::from_ddvalue_ref(&__v) } {
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
    let inputs_Function = Relation {
                              name:         "inputs::Function".to_string(),
                              input:        true,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::inputs_Function as RelId,
                              rules:        vec![
                                  ],
                              arrangements: vec![
                                  Arrangement::Map{
                                     name: r###"(inputs::Function{.id=(_: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(_0: ast::Scope)}: inputs::Function) /*join*/"###.to_string(),
                                      afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                      {
                                          let __cloned = __v.clone();
                                          match unsafe {< ::types::inputs::Function>::from_ddvalue(__v) } {
                                              ::types::inputs::Function{id: _, name: _, scope: _, body: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                              _ => None
                                          }.map(|x|(x,__cloned))
                                      }
                                      __f},
                                      queryable: false
                                  },
                                  Arrangement::Map{
                                     name: r###"(inputs::Function{.id=(_0: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(_: ast::Scope)}: inputs::Function) /*join*/"###.to_string(),
                                      afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                      {
                                          let __cloned = __v.clone();
                                          match unsafe {< ::types::inputs::Function>::from_ddvalue(__v) } {
                                              ::types::inputs::Function{id: ref _0, name: _, scope: _, body: _} => Some(((*_0).clone()).into_ddvalue()),
                                              _ => None
                                          }.map(|x|(x,__cloned))
                                      }
                                      __f},
                                      queryable: false
                                  }],
                              change_cb:    None
                          };
    let INPUT_inputs_Function = Relation {
                                    name:         "INPUT_inputs::Function".to_string(),
                                    input:        false,
                                    distinct:     false,
                                    caching_mode: CachingMode::Set,
                                    key_func:     None,
                                    id:           Relations::INPUT_inputs_Function as RelId,
                                    rules:        vec![
                                        /* INPUT_inputs::Function[x] :- inputs::Function[(x: inputs::Function)]. */
                                        Rule::CollectionRule {
                                            description: "INPUT_inputs::Function[x] :- inputs::Function[(x: inputs::Function)].".to_string(),
                                            rel: Relations::inputs_Function as RelId,
                                            xform: Some(XFormCollection::FilterMap{
                                                            description: "head of INPUT_inputs::Function[x] :- inputs::Function[(x: inputs::Function)]." .to_string(),
                                                            fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                            {
                                                                let ref x = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(&__v) } {
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
    let inputs_FunctionArg = Relation {
                                 name:         "inputs::FunctionArg".to_string(),
                                 input:        true,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::inputs_FunctionArg as RelId,
                                 rules:        vec![
                                     ],
                                 arrangements: vec![
                                     ],
                                 change_cb:    None
                             };
    let INPUT_inputs_FunctionArg = Relation {
                                       name:         "INPUT_inputs::FunctionArg".to_string(),
                                       input:        false,
                                       distinct:     false,
                                       caching_mode: CachingMode::Set,
                                       key_func:     None,
                                       id:           Relations::INPUT_inputs_FunctionArg as RelId,
                                       rules:        vec![
                                           /* INPUT_inputs::FunctionArg[x] :- inputs::FunctionArg[(x: inputs::FunctionArg)]. */
                                           Rule::CollectionRule {
                                               description: "INPUT_inputs::FunctionArg[x] :- inputs::FunctionArg[(x: inputs::FunctionArg)].".to_string(),
                                               rel: Relations::inputs_FunctionArg as RelId,
                                               xform: Some(XFormCollection::FilterMap{
                                                               description: "head of INPUT_inputs::FunctionArg[x] :- inputs::FunctionArg[(x: inputs::FunctionArg)]." .to_string(),
                                                               fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                               {
                                                                   let ref x = match *unsafe {<::types::inputs::FunctionArg>::from_ddvalue_ref(&__v) } {
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
    let inputs_If = Relation {
                        name:         "inputs::If".to_string(),
                        input:        true,
                        distinct:     false,
                        caching_mode: CachingMode::Set,
                        key_func:     None,
                        id:           Relations::inputs_If as RelId,
                        rules:        vec![
                            ],
                        arrangements: vec![
                            ],
                        change_cb:    None
                    };
    let INPUT_inputs_If = Relation {
                              name:         "INPUT_inputs::If".to_string(),
                              input:        false,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::INPUT_inputs_If as RelId,
                              rules:        vec![
                                  /* INPUT_inputs::If[x] :- inputs::If[(x: inputs::If)]. */
                                  Rule::CollectionRule {
                                      description: "INPUT_inputs::If[x] :- inputs::If[(x: inputs::If)].".to_string(),
                                      rel: Relations::inputs_If as RelId,
                                      xform: Some(XFormCollection::FilterMap{
                                                      description: "head of INPUT_inputs::If[x] :- inputs::If[(x: inputs::If)]." .to_string(),
                                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                      {
                                                          let ref x = match *unsafe {<::types::inputs::If>::from_ddvalue_ref(&__v) } {
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
    let inputs_ImplicitGlobal = Relation {
                                    name:         "inputs::ImplicitGlobal".to_string(),
                                    input:        true,
                                    distinct:     false,
                                    caching_mode: CachingMode::Set,
                                    key_func:     None,
                                    id:           Relations::inputs_ImplicitGlobal as RelId,
                                    rules:        vec![
                                        ],
                                    arrangements: vec![
                                        Arrangement::Map{
                                           name: r###"(inputs::ImplicitGlobal{.id=(_: ast::GlobalId), .name=(_: internment::Intern<string>)}: inputs::ImplicitGlobal) /*join*/"###.to_string(),
                                            afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                            {
                                                let __cloned = __v.clone();
                                                match unsafe {< ::types::inputs::ImplicitGlobal>::from_ddvalue(__v) } {
                                                    ::types::inputs::ImplicitGlobal{id: _, name: _} => Some((()).into_ddvalue()),
                                                    _ => None
                                                }.map(|x|(x,__cloned))
                                            }
                                            __f},
                                            queryable: false
                                        }],
                                    change_cb:    None
                                };
    let INPUT_inputs_ImplicitGlobal = Relation {
                                          name:         "INPUT_inputs::ImplicitGlobal".to_string(),
                                          input:        false,
                                          distinct:     false,
                                          caching_mode: CachingMode::Set,
                                          key_func:     None,
                                          id:           Relations::INPUT_inputs_ImplicitGlobal as RelId,
                                          rules:        vec![
                                              /* INPUT_inputs::ImplicitGlobal[x] :- inputs::ImplicitGlobal[(x: inputs::ImplicitGlobal)]. */
                                              Rule::CollectionRule {
                                                  description: "INPUT_inputs::ImplicitGlobal[x] :- inputs::ImplicitGlobal[(x: inputs::ImplicitGlobal)].".to_string(),
                                                  rel: Relations::inputs_ImplicitGlobal as RelId,
                                                  xform: Some(XFormCollection::FilterMap{
                                                                  description: "head of INPUT_inputs::ImplicitGlobal[x] :- inputs::ImplicitGlobal[(x: inputs::ImplicitGlobal)]." .to_string(),
                                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                  {
                                                                      let ref x = match *unsafe {<::types::inputs::ImplicitGlobal>::from_ddvalue_ref(&__v) } {
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
    let inputs_ImportDecl = Relation {
                                name:         "inputs::ImportDecl".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_ImportDecl as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    ],
                                change_cb:    None
                            };
    let INPUT_inputs_ImportDecl = Relation {
                                      name:         "INPUT_inputs::ImportDecl".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_ImportDecl as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::ImportDecl[x] :- inputs::ImportDecl[(x: inputs::ImportDecl)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::ImportDecl[x] :- inputs::ImportDecl[(x: inputs::ImportDecl)].".to_string(),
                                              rel: Relations::inputs_ImportDecl as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::ImportDecl[x] :- inputs::ImportDecl[(x: inputs::ImportDecl)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::ImportDecl>::from_ddvalue_ref(&__v) } {
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
    let inputs_InlineFunc = Relation {
                                name:         "inputs::InlineFunc".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_InlineFunc as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    Arrangement::Map{
                                       name: r###"(inputs::InlineFunc{.expr_id=(_0: ast::ExprId), .name=(ddlog_std::Some{.x=(_: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InlineFunc>::from_ddvalue(__v) } {
                                                ::types::inputs::InlineFunc{expr_id: ref _0, name: ::types::ddlog_std::Option::Some{x: _}, body: _} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::InlineFunc{.expr_id=(_0: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InlineFunc>::from_ddvalue(__v) } {
                                                ::types::inputs::InlineFunc{expr_id: ref _0, name: _, body: ::types::ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    }],
                                change_cb:    None
                            };
    let INPUT_inputs_InlineFunc = Relation {
                                      name:         "INPUT_inputs::InlineFunc".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_InlineFunc as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::InlineFunc[x] :- inputs::InlineFunc[(x: inputs::InlineFunc)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::InlineFunc[x] :- inputs::InlineFunc[(x: inputs::InlineFunc)].".to_string(),
                                              rel: Relations::inputs_InlineFunc as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::InlineFunc[x] :- inputs::InlineFunc[(x: inputs::InlineFunc)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::InlineFunc>::from_ddvalue_ref(&__v) } {
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
    let inputs_InlineFuncParam = Relation {
                                     name:         "inputs::InlineFuncParam".to_string(),
                                     input:        true,
                                     distinct:     false,
                                     caching_mode: CachingMode::Set,
                                     key_func:     None,
                                     id:           Relations::inputs_InlineFuncParam as RelId,
                                     rules:        vec![
                                         ],
                                     arrangements: vec![
                                         ],
                                     change_cb:    None
                                 };
    let INPUT_inputs_InlineFuncParam = Relation {
                                           name:         "INPUT_inputs::InlineFuncParam".to_string(),
                                           input:        false,
                                           distinct:     false,
                                           caching_mode: CachingMode::Set,
                                           key_func:     None,
                                           id:           Relations::INPUT_inputs_InlineFuncParam as RelId,
                                           rules:        vec![
                                               /* INPUT_inputs::InlineFuncParam[x] :- inputs::InlineFuncParam[(x: inputs::InlineFuncParam)]. */
                                               Rule::CollectionRule {
                                                   description: "INPUT_inputs::InlineFuncParam[x] :- inputs::InlineFuncParam[(x: inputs::InlineFuncParam)].".to_string(),
                                                   rel: Relations::inputs_InlineFuncParam as RelId,
                                                   xform: Some(XFormCollection::FilterMap{
                                                                   description: "head of INPUT_inputs::InlineFuncParam[x] :- inputs::InlineFuncParam[(x: inputs::InlineFuncParam)]." .to_string(),
                                                                   fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                   {
                                                                       let ref x = match *unsafe {<::types::inputs::InlineFuncParam>::from_ddvalue_ref(&__v) } {
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
    let inputs_InputScope = Relation {
                                name:         "inputs::InputScope".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_InputScope as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    Arrangement::Map{
                                       name: r###"(inputs::InputScope{.parent=(_: ast::Scope), .child=(_0: ast::Scope)}: inputs::InputScope) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InputScope>::from_ddvalue(__v) } {
                                                ::types::inputs::InputScope{parent: _, child: ref _0} => Some(((*_0).clone()).into_ddvalue()),
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
                             /* ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::Scope), .child=(child: ast::Scope)}: inputs::InputScope)], (parent != child). */
                             Rule::CollectionRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::Scope), .child=(child: ast::Scope)}: inputs::InputScope)], (parent != child).".to_string(),
                                 rel: Relations::inputs_InputScope as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::Scope), .child=(child: ast::Scope)}: inputs::InputScope)], (parent != child)." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref parent, ref child) = match *unsafe {<::types::inputs::InputScope>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::InputScope{parent: ref parent, child: ref child} => ((*parent).clone(), (*child).clone()),
                                                         _ => return None
                                                     };
                                                     if !((&*parent) != (&*child)) {return None;};
                                                     Some(((::types::ChildScope{parent: (*parent).clone(), child: (*child).clone()})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::Scope), .child=(interum: ast::Scope)}: inputs::InputScope)], ChildScope[(ChildScope{.parent=(interum: ast::Scope), .child=(child: ast::Scope)}: ChildScope)], (parent != child). */
                             Rule::ArrangementRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::Scope), .child=(interum: ast::Scope)}: inputs::InputScope)], ChildScope[(ChildScope{.parent=(interum: ast::Scope), .child=(child: ast::Scope)}: ChildScope)], (parent != child).".to_string(),
                                 arr: ( Relations::inputs_InputScope as RelId, 0),
                                 xform: XFormArrangement::Join{
                                            description: "inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::Scope), .child=(interum: ast::Scope)}: inputs::InputScope)], ChildScope[(ChildScope{.parent=(interum: ast::Scope), .child=(child: ast::Scope)}: ChildScope)]".to_string(),
                                            ffun: None,
                                            arrangement: (Relations::ChildScope as RelId,0),
                                            jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                            {
                                                let (ref parent, ref interum) = match *unsafe {<::types::inputs::InputScope>::from_ddvalue_ref(__v1) } {
                                                    ::types::inputs::InputScope{parent: ref parent, child: ref interum} => ((*parent).clone(), (*interum).clone()),
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
                                name: r###"(ChildScope{.parent=(_0: ast::Scope), .child=(_: ast::Scope)}: ChildScope) /*join*/"###.to_string(),
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
                                 name: r###"(ChildScope{.parent=(_0: ast::Scope), .child=(_1: ast::Scope)}: ChildScope) /*semijoin*/"###.to_string(),
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
                                  /* ClosestFunction[(ClosestFunction{.scope=body, .func=func}: ClosestFunction)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)]. */
                                  Rule::CollectionRule {
                                      description: "ClosestFunction[(ClosestFunction{.scope=body, .func=func}: ClosestFunction)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)].".to_string(),
                                      rel: Relations::inputs_Function as RelId,
                                      xform: Some(XFormCollection::FilterMap{
                                                      description: "head of ClosestFunction[(ClosestFunction{.scope=body, .func=func}: ClosestFunction)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)]." .to_string(),
                                                      fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                      {
                                                          let (ref func, ref body) = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(&__v) } {
                                                              ::types::inputs::Function{id: ref func, name: _, scope: _, body: ref body} => ((*func).clone(), (*body).clone()),
                                                              _ => return None
                                                          };
                                                          Some(((::types::ClosestFunction{scope: (*body).clone(), func: (*func).clone()})).into_ddvalue())
                                                      }
                                                      __f},
                                                      next: Box::new(None)
                                                  })
                                  },
                                  /* ClosestFunction[(ClosestFunction{.scope=scope, .func=func}: ClosestFunction)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)], ChildScope[(ChildScope{.parent=(body: ast::Scope), .child=(scope: ast::Scope)}: ChildScope)]. */
                                  Rule::ArrangementRule {
                                      description: "ClosestFunction[(ClosestFunction{.scope=scope, .func=func}: ClosestFunction)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)], ChildScope[(ChildScope{.parent=(body: ast::Scope), .child=(scope: ast::Scope)}: ChildScope)].".to_string(),
                                      arr: ( Relations::inputs_Function as RelId, 0),
                                      xform: XFormArrangement::Join{
                                                 description: "inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)], ChildScope[(ChildScope{.parent=(body: ast::Scope), .child=(scope: ast::Scope)}: ChildScope)]".to_string(),
                                                 ffun: None,
                                                 arrangement: (Relations::ChildScope as RelId,0),
                                                 jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref func, ref body) = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(__v1) } {
                                                         ::types::inputs::Function{id: ref func, name: _, scope: _, body: ref body} => ((*func).clone(), (*body).clone()),
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
                                     name: r###"(ClosestFunction{.scope=(_0: ast::Scope), .func=(_: ast::FuncId)}: ClosestFunction) /*join*/"###.to_string(),
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
                                      name: r###"(ClosestFunction{.scope=(_0: ast::Scope), .func=(_: ast::FuncId)}: ClosestFunction) /*antijoin*/"###.to_string(),
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
    let INPUT_inputs_InputScope = Relation {
                                      name:         "INPUT_inputs::InputScope".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_InputScope as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::InputScope[x] :- inputs::InputScope[(x: inputs::InputScope)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::InputScope[x] :- inputs::InputScope[(x: inputs::InputScope)].".to_string(),
                                              rel: Relations::inputs_InputScope as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::InputScope[x] :- inputs::InputScope[(x: inputs::InputScope)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::InputScope>::from_ddvalue_ref(&__v) } {
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
    let inputs_Label = Relation {
                           name:         "inputs::Label".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Label as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_Label = Relation {
                                 name:         "INPUT_inputs::Label".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Label as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Label[x] :- inputs::Label[(x: inputs::Label)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Label[x] :- inputs::Label[(x: inputs::Label)].".to_string(),
                                         rel: Relations::inputs_Label as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Label[x] :- inputs::Label[(x: inputs::Label)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Label>::from_ddvalue_ref(&__v) } {
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
    let inputs_LetDecl = Relation {
                             name:         "inputs::LetDecl".to_string(),
                             input:        true,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::inputs_LetDecl as RelId,
                             rules:        vec![
                                 ],
                             arrangements: vec![
                                 ],
                             change_cb:    None
                         };
    let INPUT_inputs_LetDecl = Relation {
                                   name:         "INPUT_inputs::LetDecl".to_string(),
                                   input:        false,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::INPUT_inputs_LetDecl as RelId,
                                   rules:        vec![
                                       /* INPUT_inputs::LetDecl[x] :- inputs::LetDecl[(x: inputs::LetDecl)]. */
                                       Rule::CollectionRule {
                                           description: "INPUT_inputs::LetDecl[x] :- inputs::LetDecl[(x: inputs::LetDecl)].".to_string(),
                                           rel: Relations::inputs_LetDecl as RelId,
                                           xform: Some(XFormCollection::FilterMap{
                                                           description: "head of INPUT_inputs::LetDecl[x] :- inputs::LetDecl[(x: inputs::LetDecl)]." .to_string(),
                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                           {
                                                               let ref x = match *unsafe {<::types::inputs::LetDecl>::from_ddvalue_ref(&__v) } {
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
    let inputs_NameRef = Relation {
                             name:         "inputs::NameRef".to_string(),
                             input:        true,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::inputs_NameRef as RelId,
                             rules:        vec![
                                 ],
                             arrangements: vec![
                                 Arrangement::Map{
                                    name: r###"(inputs::NameRef{.expr_id=(_0: ast::ExprId), .value=(_: internment::Intern<string>)}: inputs::NameRef) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::NameRef>::from_ddvalue(__v) } {
                                             ::types::inputs::NameRef{expr_id: ref _0, value: _} => Some(((*_0).clone()).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 }],
                             change_cb:    None
                         };
    let INPUT_inputs_NameRef = Relation {
                                   name:         "INPUT_inputs::NameRef".to_string(),
                                   input:        false,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::INPUT_inputs_NameRef as RelId,
                                   rules:        vec![
                                       /* INPUT_inputs::NameRef[x] :- inputs::NameRef[(x: inputs::NameRef)]. */
                                       Rule::CollectionRule {
                                           description: "INPUT_inputs::NameRef[x] :- inputs::NameRef[(x: inputs::NameRef)].".to_string(),
                                           rel: Relations::inputs_NameRef as RelId,
                                           xform: Some(XFormCollection::FilterMap{
                                                           description: "head of INPUT_inputs::NameRef[x] :- inputs::NameRef[(x: inputs::NameRef)]." .to_string(),
                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                           {
                                                               let ref x = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(&__v) } {
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
    let inputs_New = Relation {
                         name:         "inputs::New".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::inputs_New as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_inputs_New = Relation {
                               name:         "INPUT_inputs::New".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_inputs_New as RelId,
                               rules:        vec![
                                   /* INPUT_inputs::New[x] :- inputs::New[(x: inputs::New)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_inputs::New[x] :- inputs::New[(x: inputs::New)].".to_string(),
                                       rel: Relations::inputs_New as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_inputs::New[x] :- inputs::New[(x: inputs::New)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::inputs::New>::from_ddvalue_ref(&__v) } {
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
    let inputs_Property = Relation {
                              name:         "inputs::Property".to_string(),
                              input:        true,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::inputs_Property as RelId,
                              rules:        vec![
                                  ],
                              arrangements: vec![
                                  ],
                              change_cb:    None
                          };
    let INPUT_inputs_Property = Relation {
                                    name:         "INPUT_inputs::Property".to_string(),
                                    input:        false,
                                    distinct:     false,
                                    caching_mode: CachingMode::Set,
                                    key_func:     None,
                                    id:           Relations::INPUT_inputs_Property as RelId,
                                    rules:        vec![
                                        /* INPUT_inputs::Property[x] :- inputs::Property[(x: inputs::Property)]. */
                                        Rule::CollectionRule {
                                            description: "INPUT_inputs::Property[x] :- inputs::Property[(x: inputs::Property)].".to_string(),
                                            rel: Relations::inputs_Property as RelId,
                                            xform: Some(XFormCollection::FilterMap{
                                                            description: "head of INPUT_inputs::Property[x] :- inputs::Property[(x: inputs::Property)]." .to_string(),
                                                            fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                            {
                                                                let ref x = match *unsafe {<::types::inputs::Property>::from_ddvalue_ref(&__v) } {
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
    let inputs_Return = Relation {
                            name:         "inputs::Return".to_string(),
                            input:        true,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::inputs_Return as RelId,
                            rules:        vec![
                                ],
                            arrangements: vec![
                                ],
                            change_cb:    None
                        };
    let INPUT_inputs_Return = Relation {
                                  name:         "INPUT_inputs::Return".to_string(),
                                  input:        false,
                                  distinct:     false,
                                  caching_mode: CachingMode::Set,
                                  key_func:     None,
                                  id:           Relations::INPUT_inputs_Return as RelId,
                                  rules:        vec![
                                      /* INPUT_inputs::Return[x] :- inputs::Return[(x: inputs::Return)]. */
                                      Rule::CollectionRule {
                                          description: "INPUT_inputs::Return[x] :- inputs::Return[(x: inputs::Return)].".to_string(),
                                          rel: Relations::inputs_Return as RelId,
                                          xform: Some(XFormCollection::FilterMap{
                                                          description: "head of INPUT_inputs::Return[x] :- inputs::Return[(x: inputs::Return)]." .to_string(),
                                                          fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                          {
                                                              let ref x = match *unsafe {<::types::inputs::Return>::from_ddvalue_ref(&__v) } {
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
    let inputs_Statement = Relation {
                               name:         "inputs::Statement".to_string(),
                               input:        true,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::inputs_Statement as RelId,
                               rules:        vec![
                                   ],
                               arrangements: vec![
                                   Arrangement::Map{
                                      name: r###"(inputs::Statement{.id=(_0: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::inputs::Statement>::from_ddvalue(__v) } {
                                               ::types::inputs::Statement{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   },
                                   Arrangement::Map{
                                      name: r###"(inputs::Statement{.id=(_0: ast::StmtId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::inputs::Statement>::from_ddvalue(__v) } {
                                               ::types::inputs::Statement{id: ref _0, kind: ::types::ast::StmtKind::StmtVarDecl{}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   }],
                               change_cb:    None
                           };
    let INPUT_inputs_Statement = Relation {
                                     name:         "INPUT_inputs::Statement".to_string(),
                                     input:        false,
                                     distinct:     false,
                                     caching_mode: CachingMode::Set,
                                     key_func:     None,
                                     id:           Relations::INPUT_inputs_Statement as RelId,
                                     rules:        vec![
                                         /* INPUT_inputs::Statement[x] :- inputs::Statement[(x: inputs::Statement)]. */
                                         Rule::CollectionRule {
                                             description: "INPUT_inputs::Statement[x] :- inputs::Statement[(x: inputs::Statement)].".to_string(),
                                             rel: Relations::inputs_Statement as RelId,
                                             xform: Some(XFormCollection::FilterMap{
                                                             description: "head of INPUT_inputs::Statement[x] :- inputs::Statement[(x: inputs::Statement)]." .to_string(),
                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                             {
                                                                 let ref x = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(&__v) } {
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
    let inputs_Switch = Relation {
                            name:         "inputs::Switch".to_string(),
                            input:        true,
                            distinct:     false,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::inputs_Switch as RelId,
                            rules:        vec![
                                ],
                            arrangements: vec![
                                ],
                            change_cb:    None
                        };
    let INPUT_inputs_Switch = Relation {
                                  name:         "INPUT_inputs::Switch".to_string(),
                                  input:        false,
                                  distinct:     false,
                                  caching_mode: CachingMode::Set,
                                  key_func:     None,
                                  id:           Relations::INPUT_inputs_Switch as RelId,
                                  rules:        vec![
                                      /* INPUT_inputs::Switch[x] :- inputs::Switch[(x: inputs::Switch)]. */
                                      Rule::CollectionRule {
                                          description: "INPUT_inputs::Switch[x] :- inputs::Switch[(x: inputs::Switch)].".to_string(),
                                          rel: Relations::inputs_Switch as RelId,
                                          xform: Some(XFormCollection::FilterMap{
                                                          description: "head of INPUT_inputs::Switch[x] :- inputs::Switch[(x: inputs::Switch)]." .to_string(),
                                                          fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                          {
                                                              let ref x = match *unsafe {<::types::inputs::Switch>::from_ddvalue_ref(&__v) } {
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
    let inputs_SwitchCase = Relation {
                                name:         "inputs::SwitchCase".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_SwitchCase as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    ],
                                change_cb:    None
                            };
    let INPUT_inputs_SwitchCase = Relation {
                                      name:         "INPUT_inputs::SwitchCase".to_string(),
                                      input:        false,
                                      distinct:     false,
                                      caching_mode: CachingMode::Set,
                                      key_func:     None,
                                      id:           Relations::INPUT_inputs_SwitchCase as RelId,
                                      rules:        vec![
                                          /* INPUT_inputs::SwitchCase[x] :- inputs::SwitchCase[(x: inputs::SwitchCase)]. */
                                          Rule::CollectionRule {
                                              description: "INPUT_inputs::SwitchCase[x] :- inputs::SwitchCase[(x: inputs::SwitchCase)].".to_string(),
                                              rel: Relations::inputs_SwitchCase as RelId,
                                              xform: Some(XFormCollection::FilterMap{
                                                              description: "head of INPUT_inputs::SwitchCase[x] :- inputs::SwitchCase[(x: inputs::SwitchCase)]." .to_string(),
                                                              fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                              {
                                                                  let ref x = match *unsafe {<::types::inputs::SwitchCase>::from_ddvalue_ref(&__v) } {
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
    let inputs_Template = Relation {
                              name:         "inputs::Template".to_string(),
                              input:        true,
                              distinct:     false,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::inputs_Template as RelId,
                              rules:        vec![
                                  ],
                              arrangements: vec![
                                  ],
                              change_cb:    None
                          };
    let INPUT_inputs_Template = Relation {
                                    name:         "INPUT_inputs::Template".to_string(),
                                    input:        false,
                                    distinct:     false,
                                    caching_mode: CachingMode::Set,
                                    key_func:     None,
                                    id:           Relations::INPUT_inputs_Template as RelId,
                                    rules:        vec![
                                        /* INPUT_inputs::Template[x] :- inputs::Template[(x: inputs::Template)]. */
                                        Rule::CollectionRule {
                                            description: "INPUT_inputs::Template[x] :- inputs::Template[(x: inputs::Template)].".to_string(),
                                            rel: Relations::inputs_Template as RelId,
                                            xform: Some(XFormCollection::FilterMap{
                                                            description: "head of INPUT_inputs::Template[x] :- inputs::Template[(x: inputs::Template)]." .to_string(),
                                                            fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                            {
                                                                let ref x = match *unsafe {<::types::inputs::Template>::from_ddvalue_ref(&__v) } {
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
    let inputs_Ternary = Relation {
                             name:         "inputs::Ternary".to_string(),
                             input:        true,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::inputs_Ternary as RelId,
                             rules:        vec![
                                 ],
                             arrangements: vec![
                                 ],
                             change_cb:    None
                         };
    let INPUT_inputs_Ternary = Relation {
                                   name:         "INPUT_inputs::Ternary".to_string(),
                                   input:        false,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::INPUT_inputs_Ternary as RelId,
                                   rules:        vec![
                                       /* INPUT_inputs::Ternary[x] :- inputs::Ternary[(x: inputs::Ternary)]. */
                                       Rule::CollectionRule {
                                           description: "INPUT_inputs::Ternary[x] :- inputs::Ternary[(x: inputs::Ternary)].".to_string(),
                                           rel: Relations::inputs_Ternary as RelId,
                                           xform: Some(XFormCollection::FilterMap{
                                                           description: "head of INPUT_inputs::Ternary[x] :- inputs::Ternary[(x: inputs::Ternary)]." .to_string(),
                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                           {
                                                               let ref x = match *unsafe {<::types::inputs::Ternary>::from_ddvalue_ref(&__v) } {
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
    let inputs_Throw = Relation {
                           name:         "inputs::Throw".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Throw as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_Throw = Relation {
                                 name:         "INPUT_inputs::Throw".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Throw as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Throw[x] :- inputs::Throw[(x: inputs::Throw)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Throw[x] :- inputs::Throw[(x: inputs::Throw)].".to_string(),
                                         rel: Relations::inputs_Throw as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Throw[x] :- inputs::Throw[(x: inputs::Throw)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Throw>::from_ddvalue_ref(&__v) } {
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
    let inputs_Try = Relation {
                         name:         "inputs::Try".to_string(),
                         input:        true,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::inputs_Try as RelId,
                         rules:        vec![
                             ],
                         arrangements: vec![
                             ],
                         change_cb:    None
                     };
    let INPUT_inputs_Try = Relation {
                               name:         "INPUT_inputs::Try".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::INPUT_inputs_Try as RelId,
                               rules:        vec![
                                   /* INPUT_inputs::Try[x] :- inputs::Try[(x: inputs::Try)]. */
                                   Rule::CollectionRule {
                                       description: "INPUT_inputs::Try[x] :- inputs::Try[(x: inputs::Try)].".to_string(),
                                       rel: Relations::inputs_Try as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of INPUT_inputs::Try[x] :- inputs::Try[(x: inputs::Try)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let ref x = match *unsafe {<::types::inputs::Try>::from_ddvalue_ref(&__v) } {
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
    let inputs_UnaryOp = Relation {
                             name:         "inputs::UnaryOp".to_string(),
                             input:        true,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::inputs_UnaryOp as RelId,
                             rules:        vec![
                                 ],
                             arrangements: vec![
                                 ],
                             change_cb:    None
                         };
    let INPUT_inputs_UnaryOp = Relation {
                                   name:         "INPUT_inputs::UnaryOp".to_string(),
                                   input:        false,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::INPUT_inputs_UnaryOp as RelId,
                                   rules:        vec![
                                       /* INPUT_inputs::UnaryOp[x] :- inputs::UnaryOp[(x: inputs::UnaryOp)]. */
                                       Rule::CollectionRule {
                                           description: "INPUT_inputs::UnaryOp[x] :- inputs::UnaryOp[(x: inputs::UnaryOp)].".to_string(),
                                           rel: Relations::inputs_UnaryOp as RelId,
                                           xform: Some(XFormCollection::FilterMap{
                                                           description: "head of INPUT_inputs::UnaryOp[x] :- inputs::UnaryOp[(x: inputs::UnaryOp)]." .to_string(),
                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                           {
                                                               let ref x = match *unsafe {<::types::inputs::UnaryOp>::from_ddvalue_ref(&__v) } {
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
    let WithinTypeofExpr = Relation {
                               name:         "WithinTypeofExpr".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::WithinTypeofExpr as RelId,
                               rules:        vec![
                                   /* WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=expr}: WithinTypeofExpr)] :- inputs::UnaryOp[(inputs::UnaryOp{.expr_id=(type_of: ast::ExprId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp)]. */
                                   Rule::CollectionRule {
                                       description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=expr}: WithinTypeofExpr)] :- inputs::UnaryOp[(inputs::UnaryOp{.expr_id=(type_of: ast::ExprId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp)].".to_string(),
                                       rel: Relations::inputs_UnaryOp as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=expr}: WithinTypeofExpr)] :- inputs::UnaryOp[(inputs::UnaryOp{.expr_id=(type_of: ast::ExprId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let (ref type_of, ref expr) = match *unsafe {<::types::inputs::UnaryOp>::from_ddvalue_ref(&__v) } {
                                                               ::types::inputs::UnaryOp{expr_id: ref type_of, op: ::types::ddlog_std::Option::Some{x: ::types::ast::UnaryOperand::UnaryTypeof{}}, expr: ::types::ddlog_std::Option::Some{x: ref expr}} => ((*type_of).clone(), (*expr).clone()),
                                                               _ => return None
                                                           };
                                                           Some(((::types::WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*expr).clone()})).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   },
                                   /* WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=grouped}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]. */
                                   Rule::ArrangementRule {
                                       description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=grouped}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)].".to_string(),
                                       arr: ( Relations::WithinTypeofExpr as RelId, 0),
                                       xform: XFormArrangement::Join{
                                                  description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                  ffun: None,
                                                  arrangement: (Relations::inputs_Expression as RelId,2),
                                                  jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref type_of, ref expr) = match *unsafe {<::types::WithinTypeofExpr>::from_ddvalue_ref(__v1) } {
                                                          ::types::WithinTypeofExpr{type_of: ref type_of, expr: ref expr} => ((*type_of).clone(), (*expr).clone()),
                                                          _ => return None
                                                      };
                                                      let ref grouped = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                          ::types::inputs::Expression{id: _, kind: ::types::ast::ExprKind::ExprGrouping{inner: ::types::ddlog_std::Option::Some{x: ref grouped}}, scope: _, span: _} => (*grouped).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((::types::WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*grouped).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              }
                                   },
                                   /* WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=last}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprSequence{.exprs=(sequence: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], ((ddlog_std::Some{.x=(var last: ast::ExprId)}: ddlog_std::Option<ast::ExprId>) = ((vec::last: function(ddlog_std::Vec<ast::ExprId>):ddlog_std::Option<ast::ExprId>)(sequence))). */
                                   Rule::ArrangementRule {
                                       description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=last}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprSequence{.exprs=(sequence: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], ((ddlog_std::Some{.x=(var last: ast::ExprId)}: ddlog_std::Option<ast::ExprId>) = ((vec::last: function(ddlog_std::Vec<ast::ExprId>):ddlog_std::Option<ast::ExprId>)(sequence))).".to_string(),
                                       arr: ( Relations::WithinTypeofExpr as RelId, 0),
                                       xform: XFormArrangement::Join{
                                                  description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprSequence{.exprs=(sequence: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                  ffun: None,
                                                  arrangement: (Relations::inputs_Expression as RelId,3),
                                                  jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref type_of, ref expr) = match *unsafe {<::types::WithinTypeofExpr>::from_ddvalue_ref(__v1) } {
                                                          ::types::WithinTypeofExpr{type_of: ref type_of, expr: ref expr} => ((*type_of).clone(), (*expr).clone()),
                                                          _ => return None
                                                      };
                                                      let ref sequence = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                          ::types::inputs::Expression{id: _, kind: ::types::ast::ExprKind::ExprSequence{exprs: ref sequence}, scope: _, span: _} => (*sequence).clone(),
                                                          _ => return None
                                                      };
                                                      let ref last: ::types::ast::ExprId = match ::types::vec::last::<::types::ast::ExprId>(sequence) {
                                                          ::types::ddlog_std::Option::Some{x: last} => last,
                                                          _ => return None
                                                      };
                                                      Some(((::types::WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*last).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              }
                                   }],
                               arrangements: vec![
                                   Arrangement::Map{
                                      name: r###"(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(_0: ast::ExprId)}: WithinTypeofExpr) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::WithinTypeofExpr>::from_ddvalue(__v) } {
                                               ::types::WithinTypeofExpr{type_of: _, expr: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   },
                                   Arrangement::Set{
                                       name: r###"(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(_0: ast::ExprId)}: WithinTypeofExpr) /*antijoin*/"###.to_string(),
                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                       {
                                           match unsafe {< ::types::WithinTypeofExpr>::from_ddvalue(__v) } {
                                               ::types::WithinTypeofExpr{type_of: _, expr: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                               _ => None
                                           }
                                       }
                                       __f},
                                       distinct: true
                                   }],
                               change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                           };
    let inputs_VarDecl = Relation {
                             name:         "inputs::VarDecl".to_string(),
                             input:        true,
                             distinct:     false,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::inputs_VarDecl as RelId,
                             rules:        vec![
                                 ],
                             arrangements: vec![
                                 ],
                             change_cb:    None
                         };
    let INPUT_inputs_VarDecl = Relation {
                                   name:         "INPUT_inputs::VarDecl".to_string(),
                                   input:        false,
                                   distinct:     false,
                                   caching_mode: CachingMode::Set,
                                   key_func:     None,
                                   id:           Relations::INPUT_inputs_VarDecl as RelId,
                                   rules:        vec![
                                       /* INPUT_inputs::VarDecl[x] :- inputs::VarDecl[(x: inputs::VarDecl)]. */
                                       Rule::CollectionRule {
                                           description: "INPUT_inputs::VarDecl[x] :- inputs::VarDecl[(x: inputs::VarDecl)].".to_string(),
                                           rel: Relations::inputs_VarDecl as RelId,
                                           xform: Some(XFormCollection::FilterMap{
                                                           description: "head of INPUT_inputs::VarDecl[x] :- inputs::VarDecl[(x: inputs::VarDecl)]." .to_string(),
                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                           {
                                                               let ref x = match *unsafe {<::types::inputs::VarDecl>::from_ddvalue_ref(&__v) } {
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
    let __Prefix_2 = Relation {
                         name:         "__Prefix_2".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::__Prefix_2 as RelId,
                         rules:        vec![
                             /* __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::VarDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))). */
                             Rule::CollectionRule {
                                 description: "__Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::VarDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))).".to_string(),
                                 rel: Relations::inputs_VarDecl as RelId,
                                 xform: Some(XFormCollection::FlatMap{
                                                 description: "inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::VarDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                 {
                                                     let (ref stmt, ref pat) = match *unsafe {<::types::inputs::VarDecl>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::VarDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                         _ => return None
                                                     };
                                                     let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                     let stmt = (*stmt).clone();
                                                     let pat = (*pat).clone();
                                                     Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple3(name.clone(), stmt.clone(), pat.clone())).into_ddvalue())))
                                                 }
                                                 __f},
                                                 next: Box::new(Some(XFormCollection::FilterMap{
                                                                         description: "head of __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::VarDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))." .to_string(),
                                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                         {
                                                                             let ::types::ddlog_std::tuple3(ref name, ref stmt, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref( &__v ) };
                                                                             Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*pat).clone())).into_ddvalue())
                                                                         }
                                                                         __f},
                                                                         next: Box::new(None)
                                                                     }))
                                             })
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"((_: ast::Spanned<ast::Name>), (_0: ast::StmtId), (_: internment::Intern<ast::Pattern>)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue(__v) } {
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
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .span=(ddlog_std::None{}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdGlobal{.global=global}: ast::AnyId)}: NameInScope)] :- inputs::ImplicitGlobal[(inputs::ImplicitGlobal{.id=(global: ast::GlobalId), .name=(name: internment::Intern<string>)}: inputs::ImplicitGlobal)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::Scope)}: inputs::EveryScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .span=(ddlog_std::None{}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdGlobal{.global=global}: ast::AnyId)}: NameInScope)] :- inputs::ImplicitGlobal[(inputs::ImplicitGlobal{.id=(global: ast::GlobalId), .name=(name: internment::Intern<string>)}: inputs::ImplicitGlobal)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::Scope)}: inputs::EveryScope)].".to_string(),
                                  arr: ( Relations::inputs_ImplicitGlobal as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::ImplicitGlobal[(inputs::ImplicitGlobal{.id=(global: ast::GlobalId), .name=(name: internment::Intern<string>)}: inputs::ImplicitGlobal)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::Scope)}: inputs::EveryScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_EveryScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref global, ref name) = match *unsafe {<::types::inputs::ImplicitGlobal>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::ImplicitGlobal{id: ref global, name: ref name} => ((*global).clone(), (*name).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::EveryScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::EveryScope{scope: ref scope} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::None{}), declared_in: (::types::ast::AnyId::AnyIdGlobal{global: (*global).clone()})})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdImport{.import_=id}: ast::AnyId)}: NameInScope)] :- inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], var name = FlatMap((ast::free_variables(clause))), inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::Scope)}: inputs::EveryScope)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdImport{.import_=id}: ast::AnyId)}: NameInScope)] :- inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], var name = FlatMap((ast::free_variables(clause))), inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::Scope)}: inputs::EveryScope)].".to_string(),
                                  rel: Relations::inputs_ImportDecl as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], var name = FlatMap((ast::free_variables(clause)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref id, ref clause) = match *unsafe {<::types::inputs::ImportDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::ImportDecl{id: ref id, clause: ref clause} => ((*id).clone(), (*clause).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::ast::free_variables(clause);
                                                      let id = (*id).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), id.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], var name = FlatMap((ast::free_variables(clause))) by ()" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref id) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ImportId>>::from_ddvalue_ref( &__v ) };
                                                                              Some(((()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*id).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], var name = FlatMap((ast::free_variables(clause))), inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::Scope)}: inputs::EveryScope)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::inputs_EveryScope as RelId,0),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref id) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ImportId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref scope = match *unsafe {<::types::inputs::EveryScope>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::inputs::EveryScope{scope: ref scope} => (*scope).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdImport{import_: (*id).clone()})})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdClass{.class=class}: ast::AnyId)}: NameInScope)] :- inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(scope: ast::Scope)}: inputs::Class)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdClass{.class=class}: ast::AnyId)}: NameInScope)] :- inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(scope: ast::Scope)}: inputs::Class)].".to_string(),
                                  rel: Relations::inputs_Class as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdClass{.class=class}: ast::AnyId)}: NameInScope)] :- inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(scope: ast::Scope)}: inputs::Class)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref class, ref name, ref scope) = match *unsafe {<::types::inputs::Class>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::Class{id: ref class, name: ::types::ddlog_std::Option::Some{x: ref name}, parent: _, elements: _, scope: ref scope} => ((*class).clone(), (*name).clone(), (*scope).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdClass{class: (*class).clone()})})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::LetDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::LetDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)].".to_string(),
                                  rel: Relations::inputs_LetDecl as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::LetDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref stmt, ref pat) = match *unsafe {<::types::inputs::LetDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::LetDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                      let stmt = (*stmt).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), stmt.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::LetDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))) by (stmt)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::LetDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::inputs::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::ConstDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::ConstDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)].".to_string(),
                                  rel: Relations::inputs_ConstDecl as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::ConstDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref stmt, ref pat) = match *unsafe {<::types::inputs::ConstDecl>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::ConstDecl{stmt_id: ref stmt, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _} => ((*stmt).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                      let stmt = (*stmt).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), stmt.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::ConstDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))) by (stmt)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>)}: inputs::ConstDecl)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::inputs::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(decl_scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: ast::Scope), .func=(func: ast::FuncId)}: ClosestFunction)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(scope: ast::Scope)}: inputs::Function)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(decl_scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: ast::Scope), .func=(func: ast::FuncId)}: ClosestFunction)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(scope: ast::Scope)}: inputs::Function)].".to_string(),
                                  arr: ( Relations::__Prefix_2 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(decl_scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref stmt, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref stmt, ref pat) => ((*name).clone(), (*stmt).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref decl_scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Statement{id: _, kind: _, scope: ref decl_scope, span: _} => (*decl_scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*decl_scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(decl_scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)] by (decl_scope)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref stmt, ref decl_scope) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::ast::Scope>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*decl_scope).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "__Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(decl_scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: ast::Scope), .func=(func: ast::FuncId)}: ClosestFunction)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::ClosestFunction as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref func = match *unsafe {<::types::ClosestFunction>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::ClosestFunction{scope: _, func: ref func} => (*func).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*func).clone())).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                description: "arrange __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(decl_scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: ast::Scope), .func=(func: ast::FuncId)}: ClosestFunction)] by (func)" .to_string(),
                                                                                                                afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple3(ref name, ref stmt, ref func) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::ast::FuncId>>::from_ddvalue_ref( &__v ) };
                                                                                                                    Some((((*func).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*stmt).clone())).into_ddvalue()))
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(XFormArrangement::Join{
                                                                                                                                   description: "__Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(decl_scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], ClosestFunction[(ClosestFunction{.scope=(decl_scope: ast::Scope), .func=(func: ast::FuncId)}: ClosestFunction)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(scope: ast::Scope)}: inputs::Function)]".to_string(),
                                                                                                                                   ffun: None,
                                                                                                                                   arrangement: (Relations::inputs_Function as RelId,1),
                                                                                                                                   jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                   {
                                                                                                                                       let ::types::ddlog_std::tuple2(ref name, ref stmt) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                       let ref scope = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(__v2) } {
                                                                                                                                           ::types::inputs::Function{id: _, name: _, scope: _, body: ref scope} => (*scope).clone(),
                                                                                                                                           _ => return None
                                                                                                                                       };
                                                                                                                                       Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                                                                   }
                                                                                                                                   __f},
                                                                                                                                   next: Box::new(None)
                                                                                                                               })
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: ast::Scope), .func=(_: ast::FuncId)}: ClosestFunction)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: ast::Scope), .func=(_: ast::FuncId)}: ClosestFunction)].".to_string(),
                                  arr: ( Relations::__Prefix_2 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref stmt, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref stmt, ref pat) => ((*name).clone(), (*stmt).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)] by (scope)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref stmt, ref scope) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::ast::Scope>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*scope).clone()).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*stmt).clone(), (*scope).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Antijoin {
                                                                                        description: "__Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: ast::Scope), .func=(_: ast::FuncId)}: ClosestFunction)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::ClosestFunction as RelId,1),
                                                                                        next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                description: "head of NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId)}: NameInScope)] :- __Prefix_2[((name: ast::Spanned<ast::Name>), (stmt: ast::StmtId), (pat: internment::Intern<ast::Pattern>))], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)], not ClosestFunction[(ClosestFunction{.scope=(scope: ast::Scope), .func=(_: ast::FuncId)}: ClosestFunction)]." .to_string(),
                                                                                                                fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple3(ref name, ref stmt, ref scope) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::ast::Scope>>::from_ddvalue_ref( &__v ) };
                                                                                                                    Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*stmt).clone()})})).into_ddvalue())
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(None)
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: NameInScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(scope: ast::Scope), .body=(_: ast::Scope)}: inputs::Function)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: NameInScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(scope: ast::Scope), .body=(_: ast::Scope)}: inputs::Function)].".to_string(),
                                  rel: Relations::inputs_Function as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: NameInScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(scope: ast::Scope), .body=(_: ast::Scope)}: inputs::Function)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref func, ref name, ref scope) = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::Function{id: ref func, name: ::types::ddlog_std::Option::Some{x: ref name}, scope: ref scope, body: _} => ((*func).clone(), (*name).clone(), (*scope).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdFunc{func: (*func).clone()})})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=body, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: NameInScope)] :- inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .pattern=(pat: internment::Intern<ast::Pattern>)}: inputs::FunctionArg)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=body, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: NameInScope)] :- inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .pattern=(pat: internment::Intern<ast::Pattern>)}: inputs::FunctionArg)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)].".to_string(),
                                  rel: Relations::inputs_FunctionArg as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .pattern=(pat: internment::Intern<ast::Pattern>)}: inputs::FunctionArg)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref func, ref pat) = match *unsafe {<::types::inputs::FunctionArg>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::FunctionArg{parent_func: ref func, pattern: ref pat} => ((*func).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                      let func = (*func).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), func.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .pattern=(pat: internment::Intern<ast::Pattern>)}: inputs::FunctionArg)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))) by (func)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref func) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::FuncId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*func).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*func).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .pattern=(pat: internment::Intern<ast::Pattern>)}: inputs::FunctionArg)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::Scope), .body=(body: ast::Scope)}: inputs::Function)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::inputs_Function as RelId,1),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref func) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::FuncId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref body = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::inputs::Function{id: _, name: _, scope: _, body: ref body} => (*body).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{name: name.data.clone(), scope: (*body).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdFunc{func: (*func).clone()})})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(expr_body: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(expr_body: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)].".to_string(),
                                  arr: ( Relations::__Prefix_1 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Arrow as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref expr, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref expr, ref pat) => ((*name).clone(), (*expr).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref expr_body = match *unsafe {<::types::inputs::Arrow>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Arrow{expr_id: _, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: ref expr_body}}} => (*expr_body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*expr_body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)] by (expr_body)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref expr, ref expr_body) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*expr_body).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "__Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(expr_body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(expr_body: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::inputs_Expression as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref scope = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::inputs::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(None)
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(stmt_body: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(stmt_body: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)].".to_string(),
                                  arr: ( Relations::__Prefix_1 as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "__Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Arrow as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref expr, ref pat) = match *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref(__v1) } {
                                                     ::types::ddlog_std::tuple3(ref name, ref expr, ref pat) => ((*name).clone(), (*expr).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref stmt_body = match *unsafe {<::types::inputs::Arrow>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Arrow{expr_id: _, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Right{r: ref stmt_body}}} => (*stmt_body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*stmt_body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange __Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)] by (stmt_body)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple3(ref name, ref expr, ref stmt_body) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                         Some((((*stmt_body).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "__Prefix_1[((name: ast::Spanned<ast::Name>), (expr: ast::ExprId), (pat: internment::Intern<ast::Pattern>))], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(stmt_body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(stmt_body: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::inputs_Statement as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::inputs::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(None)
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)].".to_string(),
                                  arr: ( Relations::inputs_InlineFunc as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(ddlog_std::Some{.x=(name: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Expression as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref expr, ref name) = match *unsafe {<::types::inputs::InlineFunc>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::InlineFunc{expr_id: ref expr, name: ::types::ddlog_std::Option::Some{x: ref name}, body: _} => ((*expr).clone(), (*name).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              },
                              /* NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.name=(name.data), .scope=scope, .span=(ddlog_std::Some{.x=(name.span)}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId)}: NameInScope)] :- inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)].".to_string(),
                                  rel: Relations::inputs_InlineFuncParam as RelId,
                                  xform: Some(XFormCollection::FlatMap{
                                                  description: "inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                  {
                                                      let (ref expr, ref pat) = match *unsafe {<::types::inputs::InlineFuncParam>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::InlineFuncParam{expr_id: ref expr, param: ref pat} => ((*expr).clone(), (*pat).clone()),
                                                          _ => return None
                                                      };
                                                      let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                      let expr = (*expr).clone();
                                                      Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), expr.clone())).into_ddvalue())))
                                                  }
                                                  __f},
                                                  next: Box::new(Some(XFormCollection::Arrange {
                                                                          description: "arrange inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))) by (expr)" .to_string(),
                                                                          afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                          {
                                                                              let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                              Some((((*expr).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                          }
                                                                          __f},
                                                                          next: Box::new(XFormArrangement::Join{
                                                                                             description: "inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)]".to_string(),
                                                                                             ffun: None,
                                                                                             arrangement: (Relations::inputs_InlineFunc as RelId,1),
                                                                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                                 let ref body = match *unsafe {<::types::inputs::InlineFunc>::from_ddvalue_ref(__v2) } {
                                                                                                     ::types::inputs::InlineFunc{expr_id: _, name: _, body: ::types::ddlog_std::Option::Some{x: ref body}} => (*body).clone(),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some((::types::ddlog_std::tuple3((*name).clone(), (*expr).clone(), (*body).clone())).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                     description: "arrange inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)] by (body)" .to_string(),
                                                                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let ::types::ddlog_std::tuple3(ref name, ref expr, ref body) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                                                                         Some((((*body).clone()).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*expr).clone())).into_ddvalue()))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     next: Box::new(XFormArrangement::Join{
                                                                                                                                        description: "inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                                                                        ffun: None,
                                                                                                                                        arrangement: (Relations::inputs_Statement as RelId,0),
                                                                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let ::types::ddlog_std::tuple2(ref name, ref expr) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                            let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                                                                ::types::inputs::Statement{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some(((::types::NameInScope{name: name.data.clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: name.span.clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()})})).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(None)
                                                                                                                                    })
                                                                                                                 }))
                                                                                         })
                                                                      }))
                                              })
                              },
                              /* NameInScope[(NameInScope{.name=name, .scope=scope, .span=span, .declared_in=declared_in}: NameInScope)] :- NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: ast::Scope), .span=(span: ddlog_std::Option<ast::Span>), .declared_in=(declared_in: ast::AnyId)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: ast::Scope), .child=(scope: ast::Scope)}: ChildScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.name=name, .scope=scope, .span=span, .declared_in=declared_in}: NameInScope)] :- NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: ast::Scope), .span=(span: ddlog_std::Option<ast::Span>), .declared_in=(declared_in: ast::AnyId)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: ast::Scope), .child=(scope: ast::Scope)}: ChildScope)].".to_string(),
                                  arr: ( Relations::NameInScope as RelId, 2),
                                  xform: XFormArrangement::Join{
                                             description: "NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(interum: ast::Scope), .span=(span: ddlog_std::Option<ast::Span>), .declared_in=(declared_in: ast::AnyId)}: NameInScope)], ChildScope[(ChildScope{.parent=(interum: ast::Scope), .child=(scope: ast::Scope)}: ChildScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::ChildScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref name, ref interum, ref span, ref declared_in) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v1) } {
                                                     ::types::NameInScope{name: ref name, scope: ref interum, span: ref span, declared_in: ref declared_in} => ((*name).clone(), (*interum).clone(), (*span).clone(), (*declared_in).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::ChildScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::ChildScope{parent: _, child: ref scope} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone(), declared_in: (*declared_in).clone()})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              }],
                          arrangements: vec![
                              Arrangement::Set{
                                  name: r###"(NameInScope{.name=(_: internment::Intern<string>), .scope=(_: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=(_0: ast::ExprId)}: ast::AnyId)}: NameInScope) /*semijoin*/"###.to_string(),
                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                  {
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: _, scope: _, span: _, declared_in: ::types::ast::AnyId::AnyIdExpr{expr: ref _0}} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }
                                  }
                                  __f},
                                  distinct: false
                              },
                              Arrangement::Set{
                                  name: r###"(NameInScope{.name=_0, .scope=(_1: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope) /*antijoin*/"###.to_string(),
                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                  {
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: ref _0, scope: ref _1, span: _, declared_in: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }
                                  }
                                  __f},
                                  distinct: true
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=(_: internment::Intern<string>), .scope=(_0: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: _, scope: ref _0, span: _, declared_in: _} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Set{
                                  name: r###"(NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope) /*antijoin*/"###.to_string(),
                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                  {
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: ref _0, scope: ref _1, span: _, declared_in: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }
                                  }
                                  __f},
                                  distinct: true
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=(_0: internment::Intern<string>), .scope=(_1: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(_: ast::StmtId)}: ast::AnyId)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: ref _0, scope: ref _1, span: _, declared_in: ::types::ast::AnyId::AnyIdStmt{stmt: _}} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=_1, .scope=_0, .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: ref _1, scope: ref _0, span: _, declared_in: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: true
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.name=(_: internment::Intern<string>), .scope=_0, .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{name: _, scope: ref _0, span: _, declared_in: _} => Some(((*_0).clone()).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: true
                              }],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let TypeofUndefinedAlwaysUndefined = Relation {
                                             name:         "TypeofUndefinedAlwaysUndefined".to_string(),
                                             input:        false,
                                             distinct:     true,
                                             caching_mode: CachingMode::Set,
                                             key_func:     None,
                                             id:           Relations::TypeofUndefinedAlwaysUndefined as RelId,
                                             rules:        vec![
                                                 /* TypeofUndefinedAlwaysUndefined[(TypeofUndefinedAlwaysUndefined{.whole_expr=whole_expr, .undefined_expr=undefined_expr}: TypeofUndefinedAlwaysUndefined)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)], WithinTypeofExpr[(WithinTypeofExpr{.type_of=(whole_expr: ast::ExprId), .expr=(undefined_expr: ast::ExprId)}: WithinTypeofExpr)]. */
                                                 Rule::ArrangementRule {
                                                     description: "TypeofUndefinedAlwaysUndefined[(TypeofUndefinedAlwaysUndefined{.whole_expr=whole_expr, .undefined_expr=undefined_expr}: TypeofUndefinedAlwaysUndefined)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)], WithinTypeofExpr[(WithinTypeofExpr{.type_of=(whole_expr: ast::ExprId), .expr=(undefined_expr: ast::ExprId)}: WithinTypeofExpr)].".to_string(),
                                                     arr: ( Relations::inputs_NameRef as RelId, 0),
                                                     xform: XFormArrangement::Join{
                                                                description: "inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)]".to_string(),
                                                                ffun: None,
                                                                arrangement: (Relations::inputs_Expression as RelId,1),
                                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                {
                                                                    let (ref undefined_expr, ref name) = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v1) } {
                                                                        ::types::inputs::NameRef{expr_id: ref undefined_expr, value: ref name} => ((*undefined_expr).clone(), (*name).clone()),
                                                                        _ => return None
                                                                    };
                                                                    let (ref scope, ref span) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                                        ::types::inputs::Expression{id: _, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                                        _ => return None
                                                                    };
                                                                    Some((::types::ddlog_std::tuple3((*undefined_expr).clone(), (*name).clone(), (*scope).clone())).into_ddvalue())
                                                                }
                                                                __f},
                                                                next: Box::new(Some(XFormCollection::Arrange {
                                                                                        description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)] by (name, scope)" .to_string(),
                                                                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple3(ref undefined_expr, ref name, ref scope) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope>>::from_ddvalue_ref( &__v ) };
                                                                                            Some(((::types::ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), ((*undefined_expr).clone()).into_ddvalue()))
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(XFormArrangement::Antijoin {
                                                                                                           description: "inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)]".to_string(),
                                                                                                           ffun: None,
                                                                                                           arrangement: (Relations::NameInScope as RelId,3),
                                                                                                           next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                   description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)] by (undefined_expr)" .to_string(),
                                                                                                                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                   {
                                                                                                                                       let ref undefined_expr = *unsafe {<::types::ast::ExprId>::from_ddvalue_ref( &__v ) };
                                                                                                                                       Some((((*undefined_expr).clone()).into_ddvalue(), ((*undefined_expr).clone()).into_ddvalue()))
                                                                                                                                   }
                                                                                                                                   __f},
                                                                                                                                   next: Box::new(XFormArrangement::Join{
                                                                                                                                                      description: "inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)], WithinTypeofExpr[(WithinTypeofExpr{.type_of=(whole_expr: ast::ExprId), .expr=(undefined_expr: ast::ExprId)}: WithinTypeofExpr)]".to_string(),
                                                                                                                                                      ffun: None,
                                                                                                                                                      arrangement: (Relations::WithinTypeofExpr as RelId,0),
                                                                                                                                                      jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                      {
                                                                                                                                                          let ref undefined_expr = *unsafe {<::types::ast::ExprId>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                          let ref whole_expr = match *unsafe {<::types::WithinTypeofExpr>::from_ddvalue_ref(__v2) } {
                                                                                                                                                              ::types::WithinTypeofExpr{type_of: ref whole_expr, expr: _} => (*whole_expr).clone(),
                                                                                                                                                              _ => return None
                                                                                                                                                          };
                                                                                                                                                          Some(((::types::TypeofUndefinedAlwaysUndefined{whole_expr: (*whole_expr).clone(), undefined_expr: (*undefined_expr).clone()})).into_ddvalue())
                                                                                                                                                      }
                                                                                                                                                      __f},
                                                                                                                                                      next: Box::new(None)
                                                                                                                                                  })
                                                                                                                               }))
                                                                                                       })
                                                                                    }))
                                                            }
                                                 }],
                                             arrangements: vec![
                                                 ],
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
                                          /* VarUseBeforeDeclaration[(VarUseBeforeDeclaration{.name=name, .used_in=used_in, .declared_in=declared_in}: VarUseBeforeDeclaration)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::Scope), .span=(declared_in: ast::Span)}: inputs::Statement)], ChildScope[(ChildScope{.parent=(used_scope: ast::Scope), .child=(declared_scope: ast::Scope)}: ChildScope)]. */
                                          Rule::ArrangementRule {
                                              description: "VarUseBeforeDeclaration[(VarUseBeforeDeclaration{.name=name, .used_in=used_in, .declared_in=declared_in}: VarUseBeforeDeclaration)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::Scope), .span=(declared_in: ast::Span)}: inputs::Statement)], ChildScope[(ChildScope{.parent=(used_scope: ast::Scope), .child=(declared_scope: ast::Scope)}: ChildScope)].".to_string(),
                                              arr: ( Relations::inputs_NameRef as RelId, 0),
                                              xform: XFormArrangement::Join{
                                                         description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)]".to_string(),
                                                         ffun: None,
                                                         arrangement: (Relations::inputs_Expression as RelId,1),
                                                         jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                         {
                                                             let (ref expr, ref name) = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v1) } {
                                                                 ::types::inputs::NameRef{expr_id: ref expr, value: ref name} => ((*expr).clone(), (*name).clone()),
                                                                 _ => return None
                                                             };
                                                             let (ref used_scope, ref used_in) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                                 ::types::inputs::Expression{id: _, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: ref used_scope, span: ref used_in} => ((*used_scope).clone(), (*used_in).clone()),
                                                                 _ => return None
                                                             };
                                                             Some((::types::ddlog_std::tuple3((*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue())
                                                         }
                                                         __f},
                                                         next: Box::new(Some(XFormCollection::Arrange {
                                                                                 description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)] by (name, used_scope)" .to_string(),
                                                                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                 {
                                                                                     let ::types::ddlog_std::tuple3(ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                     Some(((::types::ddlog_std::tuple2((*name).clone(), (*used_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue()))
                                                                                 }
                                                                                 __f},
                                                                                 next: Box::new(XFormArrangement::Join{
                                                                                                    description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)}: NameInScope)]".to_string(),
                                                                                                    ffun: None,
                                                                                                    arrangement: (Relations::NameInScope as RelId,4),
                                                                                                    jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                    {
                                                                                                        let ::types::ddlog_std::tuple3(ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                        let ref stmt = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v2) } {
                                                                                                            ::types::NameInScope{name: _, scope: _, span: _, declared_in: ::types::ast::AnyId::AnyIdStmt{stmt: ref stmt}} => (*stmt).clone(),
                                                                                                            _ => return None
                                                                                                        };
                                                                                                        Some((::types::ddlog_std::tuple4((*name).clone(), (*used_scope).clone(), (*used_in).clone(), (*stmt).clone())).into_ddvalue())
                                                                                                    }
                                                                                                    __f},
                                                                                                    next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                            description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)}: NameInScope)] by (stmt)" .to_string(),
                                                                                                                            afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let ::types::ddlog_std::tuple4(ref name, ref used_scope, ref used_in, ref stmt) = *unsafe {<::types::ddlog_std::tuple4<::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                Some((((*stmt).clone()).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue()))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            next: Box::new(XFormArrangement::Join{
                                                                                                                                               description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::Scope), .span=(declared_in: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                                                                               ffun: None,
                                                                                                                                               arrangement: (Relations::inputs_Statement as RelId,1),
                                                                                                                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                               {
                                                                                                                                                   let ::types::ddlog_std::tuple3(ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                   let (ref declared_scope, ref declared_in) = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                                                                       ::types::inputs::Statement{id: _, kind: ::types::ast::StmtKind::StmtVarDecl{}, scope: ref declared_scope, span: ref declared_in} => ((*declared_scope).clone(), (*declared_in).clone()),
                                                                                                                                                       _ => return None
                                                                                                                                                   };
                                                                                                                                                   Some((::types::ddlog_std::tuple5((*name).clone(), (*used_scope).clone(), (*used_in).clone(), (*declared_scope).clone(), (*declared_in).clone())).into_ddvalue())
                                                                                                                                               }
                                                                                                                                               __f},
                                                                                                                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                       description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::Scope), .span=(declared_in: ast::Span)}: inputs::Statement)] by (used_scope, declared_scope)" .to_string(),
                                                                                                                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                       {
                                                                                                                                                                           let ::types::ddlog_std::tuple5(ref name, ref used_scope, ref used_in, ref declared_scope, ref declared_in) = *unsafe {<::types::ddlog_std::tuple5<::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                           Some(((::types::ddlog_std::tuple2((*used_scope).clone(), (*declared_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*used_in).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                                       }
                                                                                                                                                                       __f},
                                                                                                                                                                       next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                          description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::Scope), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(used_scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::Scope), .span=(declared_in: ast::Span)}: inputs::Statement)], ChildScope[(ChildScope{.parent=(used_scope: ast::Scope), .child=(declared_scope: ast::Scope)}: ChildScope)]".to_string(),
                                                                                                                                                                                          ffun: None,
                                                                                                                                                                                          arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                          {
                                                                                                                                                                                              let ::types::ddlog_std::tuple3(ref name, ref used_in, ref declared_in) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ast::Span, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
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
                                             name: r###"(VarUseBeforeDeclaration{.name=_0, .used_in=(_: ast::Span), .declared_in=(_: ast::Span)}: VarUseBeforeDeclaration) /*join*/"###.to_string(),
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
    let __Prefix_0 = Relation {
                         name:         "__Prefix_0".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::__Prefix_0 as RelId,
                         rules:        vec![
                             /* __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)]. */
                             Rule::ArrangementRule {
                                 description: "__Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)].".to_string(),
                                 arr: ( Relations::inputs_NameRef as RelId, 0),
                                 xform: XFormArrangement::Join{
                                            description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)]".to_string(),
                                            ffun: None,
                                            arrangement: (Relations::inputs_Expression as RelId,1),
                                            jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                            {
                                                let (ref expr, ref name) = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v1) } {
                                                    ::types::inputs::NameRef{expr_id: ref expr, value: ref name} => ((*expr).clone(), (*name).clone()),
                                                    _ => return None
                                                };
                                                let (ref scope, ref span) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                    ::types::inputs::Expression{id: _, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                    _ => return None
                                                };
                                                Some((::types::ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue())
                                            }
                                            __f},
                                            next: Box::new(Some(XFormCollection::Arrange {
                                                                    description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)] by (name, scope)" .to_string(),
                                                                    afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                    {
                                                                        let ::types::ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                        Some(((::types::ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                    }
                                                                    __f},
                                                                    next: Box::new(XFormArrangement::Antijoin {
                                                                                       description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)]".to_string(),
                                                                                       ffun: None,
                                                                                       arrangement: (Relations::NameInScope as RelId,3),
                                                                                       next: Box::new(Some(XFormCollection::Arrange {
                                                                                                               description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)] by (expr)" .to_string(),
                                                                                                               afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                               {
                                                                                                                   let ::types::ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                   Some((((*expr).clone()).into_ddvalue(), (::types::ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                                                               }
                                                                                                               __f},
                                                                                                               next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                  description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)]".to_string(),
                                                                                                                                  ffun: None,
                                                                                                                                  arrangement: (Relations::WithinTypeofExpr as RelId,1),
                                                                                                                                  next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                          description: "head of __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::Scope), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.name=(name: internment::Intern<string>), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId)}: WithinTypeofExpr)]." .to_string(),
                                                                                                                                                          fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                          {
                                                                                                                                                              let ::types::ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                              Some((::types::ddlog_std::tuple4((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue())
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
                                name: r###"((_0: ast::ExprId), (_: internment::Intern<string>), (_: ast::Scope), (_: ast::Span)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue(__v) } {
                                         ::types::ddlog_std::tuple4(ref _0, _, _, _) => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let InvalidNameUse = Relation {
                             name:         "InvalidNameUse".to_string(),
                             input:        false,
                             distinct:     true,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::InvalidNameUse as RelId,
                             rules:        vec![
                                 /* InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)]. */
                                 Rule::ArrangementRule {
                                     description: "InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)].".to_string(),
                                     arr: ( Relations::__Prefix_0 as RelId, 0),
                                     xform: XFormArrangement::Antijoin {
                                                description: "__Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)]".to_string(),
                                                ffun: None,
                                                arrangement: (Relations::ChainedWith as RelId,2),
                                                next: Box::new(Some(XFormCollection::FilterMap{
                                                                        description: "head of InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)]." .to_string(),
                                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                        {
                                                                            let (ref expr, ref name, ref scope, ref span) = match *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref(&__v) } {
                                                                                ::types::ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) => ((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone()),
                                                                                _ => return None
                                                                            };
                                                                            Some(((::types::InvalidNameUse{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone()})).into_ddvalue())
                                                                        }
                                                                        __f},
                                                                        next: Box::new(None)
                                                                    }))
                                            }
                                 },
                                 /* InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)], NameInScope[(NameInScope{.name=(_: internment::Intern<string>), .scope=(_: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=(object: ast::ExprId)}: ast::AnyId)}: NameInScope)]. */
                                 Rule::ArrangementRule {
                                     description: "InvalidNameUse[(InvalidNameUse{.name=name, .scope=scope, .span=span}: InvalidNameUse)] :- __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)], NameInScope[(NameInScope{.name=(_: internment::Intern<string>), .scope=(_: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=(object: ast::ExprId)}: ast::AnyId)}: NameInScope)].".to_string(),
                                     arr: ( Relations::__Prefix_0 as RelId, 0),
                                     xform: XFormArrangement::Join{
                                                description: "__Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)]".to_string(),
                                                ffun: None,
                                                arrangement: (Relations::ChainedWith as RelId,0),
                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                {
                                                    let (ref expr, ref name, ref scope, ref span) = match *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref(__v1) } {
                                                        ::types::ddlog_std::tuple4(ref expr, ref name, ref scope, ref span) => ((*expr).clone(), (*name).clone(), (*scope).clone(), (*span).clone()),
                                                        _ => return None
                                                    };
                                                    let ref object = match *unsafe {<::types::ChainedWith>::from_ddvalue_ref(__v2) } {
                                                        ::types::ChainedWith{object: ref object, property: _} => (*object).clone(),
                                                        _ => return None
                                                    };
                                                    Some((::types::ddlog_std::tuple4((*name).clone(), (*scope).clone(), (*span).clone(), (*object).clone())).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(Some(XFormCollection::Arrange {
                                                                        description: "arrange __Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)] by (object)" .to_string(),
                                                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                        {
                                                                            let ::types::ddlog_std::tuple4(ref name, ref scope, ref span, ref object) = *unsafe {<::types::ddlog_std::tuple4<::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                            Some((((*object).clone()).into_ddvalue(), (::types::ddlog_std::tuple3((*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                        }
                                                                        __f},
                                                                        next: Box::new(XFormArrangement::Semijoin{
                                                                                           description: "__Prefix_0[((expr: ast::ExprId), (name: internment::Intern<string>), (scope: ast::Scope), (span: ast::Span))], ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(expr: ast::ExprId)}: ChainedWith)], NameInScope[(NameInScope{.name=(_: internment::Intern<string>), .scope=(_: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=(object: ast::ExprId)}: ast::AnyId)}: NameInScope)]".to_string(),
                                                                                           ffun: None,
                                                                                           arrangement: (Relations::NameInScope as RelId,0),
                                                                                           jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                           {
                                                                                               let ::types::ddlog_std::tuple3(ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple3<::types::internment::Intern<String>, ::types::ast::Scope, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                               Some(((::types::InvalidNameUse{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone()})).into_ddvalue())
                                                                                           }
                                                                                           __f},
                                                                                           next: Box::new(None)
                                                                                       })
                                                                    }))
                                            }
                                 },
                                 /* InvalidNameUse[(InvalidNameUse{.name=(name.data), .scope=scope, .span=(name.span)}: InvalidNameUse)] :- inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), not NameInScope[(NameInScope{.name=(name.data), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)]. */
                                 Rule::ArrangementRule {
                                     description: "InvalidNameUse[(InvalidNameUse{.name=(name.data), .scope=scope, .span=(name.span)}: InvalidNameUse)] :- inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), not NameInScope[(NameInScope{.name=(name.data), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)].".to_string(),
                                     arr: ( Relations::inputs_Assign as RelId, 0),
                                     xform: XFormArrangement::Join{
                                                description: "inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                ffun: None,
                                                arrangement: (Relations::inputs_Expression as RelId,0),
                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                {
                                                    let (ref expr, ref pat) = match *unsafe {<::types::inputs::Assign>::from_ddvalue_ref(__v1) } {
                                                        ::types::inputs::Assign{expr_id: ref expr, lhs: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: ref pat}}, rhs: _, op: _} => ((*expr).clone(), (*pat).clone()),
                                                        _ => return None
                                                    };
                                                    let ref scope = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                        ::types::inputs::Expression{id: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                        _ => return None
                                                    };
                                                    Some((::types::ddlog_std::tuple2((*pat).clone(), (*scope).clone())).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(Some(XFormCollection::FlatMap{
                                                                        description: "inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                        fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                        {
                                                                            let ::types::ddlog_std::tuple2(ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple2<::types::internment::Intern<::types::ast::Pattern>, ::types::ast::Scope>>::from_ddvalue_ref( &__v ) };
                                                                            let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                            let scope = (*scope).clone();
                                                                            Some(Box::new(__flattened.into_iter().map(move |name|(::types::ddlog_std::tuple2(name.clone(), scope.clone())).into_ddvalue())))
                                                                        }
                                                                        __f},
                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                description: "arrange inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))) by ((name.data), scope)" .to_string(),
                                                                                                afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                {
                                                                                                    let ::types::ddlog_std::tuple2(ref name, ref scope) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::Scope>>::from_ddvalue_ref( &__v ) };
                                                                                                    Some(((::types::ddlog_std::tuple2(name.data.clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple2((*name).clone(), (*scope).clone())).into_ddvalue()))
                                                                                                }
                                                                                                __f},
                                                                                                next: Box::new(XFormArrangement::Antijoin {
                                                                                                                   description: "inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), not NameInScope[(NameInScope{.name=(name.data), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)]".to_string(),
                                                                                                                   ffun: None,
                                                                                                                   arrangement: (Relations::NameInScope as RelId,1),
                                                                                                                   next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                           description: "head of InvalidNameUse[(InvalidNameUse{.name=(name.data), .scope=scope, .span=(name.span)}: InvalidNameUse)] :- inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(scope: ast::Scope), .span=(_: ast::Span)}: inputs::Expression)], var name = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), not NameInScope[(NameInScope{.name=(name.data), .scope=(scope: ast::Scope), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId)}: NameInScope)]." .to_string(),
                                                                                                                                           fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                           {
                                                                                                                                               let ::types::ddlog_std::tuple2(ref name, ref scope) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::Scope>>::from_ddvalue_ref( &__v ) };
                                                                                                                                               Some(((::types::InvalidNameUse{name: name.data.clone(), scope: (*scope).clone(), span: name.span.clone()})).into_ddvalue())
                                                                                                                                           }
                                                                                                                                           __f},
                                                                                                                                           next: Box::new(None)
                                                                                                                                       }))
                                                                                                               })
                                                                                            }))
                                                                    }))
                                            }
                                 }],
                             arrangements: vec![
                                 Arrangement::Map{
                                    name: r###"(InvalidNameUse{.name=_0, .scope=(_: ast::Scope), .span=(_: ast::Span)}: InvalidNameUse) /*join*/"###.to_string(),
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
    let inputs_While = Relation {
                           name:         "inputs::While".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_While as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_While = Relation {
                                 name:         "INPUT_inputs::While".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_While as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::While[x] :- inputs::While[(x: inputs::While)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::While[x] :- inputs::While[(x: inputs::While)].".to_string(),
                                         rel: Relations::inputs_While as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::While[x] :- inputs::While[(x: inputs::While)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::While>::from_ddvalue_ref(&__v) } {
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
    let inputs_With = Relation {
                          name:         "inputs::With".to_string(),
                          input:        true,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::inputs_With as RelId,
                          rules:        vec![
                              ],
                          arrangements: vec![
                              ],
                          change_cb:    None
                      };
    let INPUT_inputs_With = Relation {
                                name:         "INPUT_inputs::With".to_string(),
                                input:        false,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::INPUT_inputs_With as RelId,
                                rules:        vec![
                                    /* INPUT_inputs::With[x] :- inputs::With[(x: inputs::With)]. */
                                    Rule::CollectionRule {
                                        description: "INPUT_inputs::With[x] :- inputs::With[(x: inputs::With)].".to_string(),
                                        rel: Relations::inputs_With as RelId,
                                        xform: Some(XFormCollection::FilterMap{
                                                        description: "head of INPUT_inputs::With[x] :- inputs::With[(x: inputs::With)]." .to_string(),
                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                        {
                                                            let ref x = match *unsafe {<::types::inputs::With>::from_ddvalue_ref(&__v) } {
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
    let inputs_Yield = Relation {
                           name:         "inputs::Yield".to_string(),
                           input:        true,
                           distinct:     false,
                           caching_mode: CachingMode::Set,
                           key_func:     None,
                           id:           Relations::inputs_Yield as RelId,
                           rules:        vec![
                               ],
                           arrangements: vec![
                               ],
                           change_cb:    None
                       };
    let INPUT_inputs_Yield = Relation {
                                 name:         "INPUT_inputs::Yield".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::INPUT_inputs_Yield as RelId,
                                 rules:        vec![
                                     /* INPUT_inputs::Yield[x] :- inputs::Yield[(x: inputs::Yield)]. */
                                     Rule::CollectionRule {
                                         description: "INPUT_inputs::Yield[x] :- inputs::Yield[(x: inputs::Yield)].".to_string(),
                                         rel: Relations::inputs_Yield as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of INPUT_inputs::Yield[x] :- inputs::Yield[(x: inputs::Yield)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let ref x = match *unsafe {<::types::inputs::Yield>::from_ddvalue_ref(&__v) } {
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
            ProgNode::Rel{rel: inputs_Array},
            ProgNode::Rel{rel: INPUT_inputs_Array},
            ProgNode::Rel{rel: inputs_Arrow},
            ProgNode::Rel{rel: INPUT_inputs_Arrow},
            ProgNode::Rel{rel: inputs_ArrowParam},
            ProgNode::Rel{rel: INPUT_inputs_ArrowParam},
            ProgNode::Rel{rel: __Prefix_1},
            ProgNode::Rel{rel: inputs_Assign},
            ProgNode::Rel{rel: INPUT_inputs_Assign},
            ProgNode::Rel{rel: inputs_Await},
            ProgNode::Rel{rel: INPUT_inputs_Await},
            ProgNode::Rel{rel: inputs_BinOp},
            ProgNode::Rel{rel: INPUT_inputs_BinOp},
            ProgNode::Rel{rel: inputs_BracketAccess},
            ProgNode::Rel{rel: INPUT_inputs_BracketAccess},
            ProgNode::Rel{rel: inputs_Break},
            ProgNode::Rel{rel: INPUT_inputs_Break},
            ProgNode::Rel{rel: inputs_Call},
            ProgNode::Rel{rel: INPUT_inputs_Call},
            ProgNode::Rel{rel: inputs_Class},
            ProgNode::Rel{rel: INPUT_inputs_Class},
            ProgNode::Rel{rel: inputs_ClassExpr},
            ProgNode::Rel{rel: INPUT_inputs_ClassExpr},
            ProgNode::Rel{rel: inputs_ConstDecl},
            ProgNode::Rel{rel: INPUT_inputs_ConstDecl},
            ProgNode::Rel{rel: inputs_Continue},
            ProgNode::Rel{rel: INPUT_inputs_Continue},
            ProgNode::Rel{rel: inputs_DoWhile},
            ProgNode::Rel{rel: INPUT_inputs_DoWhile},
            ProgNode::Rel{rel: inputs_DotAccess},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: ChainedWith, distinct: true}]},
            ProgNode::Rel{rel: INPUT_inputs_DotAccess},
            ProgNode::Rel{rel: inputs_EveryScope},
            ProgNode::Rel{rel: INPUT_inputs_EveryScope},
            ProgNode::Rel{rel: inputs_ExprBigInt},
            ProgNode::Rel{rel: INPUT_inputs_ExprBigInt},
            ProgNode::Rel{rel: inputs_ExprBool},
            ProgNode::Rel{rel: INPUT_inputs_ExprBool},
            ProgNode::Rel{rel: inputs_ExprNumber},
            ProgNode::Rel{rel: INPUT_inputs_ExprNumber},
            ProgNode::Rel{rel: inputs_ExprString},
            ProgNode::Rel{rel: INPUT_inputs_ExprString},
            ProgNode::Rel{rel: inputs_Expression},
            ProgNode::Rel{rel: INPUT_inputs_Expression},
            ProgNode::Rel{rel: inputs_For},
            ProgNode::Rel{rel: INPUT_inputs_For},
            ProgNode::Rel{rel: inputs_ForIn},
            ProgNode::Rel{rel: INPUT_inputs_ForIn},
            ProgNode::Rel{rel: inputs_Function},
            ProgNode::Rel{rel: INPUT_inputs_Function},
            ProgNode::Rel{rel: inputs_FunctionArg},
            ProgNode::Rel{rel: INPUT_inputs_FunctionArg},
            ProgNode::Rel{rel: inputs_If},
            ProgNode::Rel{rel: INPUT_inputs_If},
            ProgNode::Rel{rel: inputs_ImplicitGlobal},
            ProgNode::Rel{rel: INPUT_inputs_ImplicitGlobal},
            ProgNode::Rel{rel: inputs_ImportDecl},
            ProgNode::Rel{rel: INPUT_inputs_ImportDecl},
            ProgNode::Rel{rel: inputs_InlineFunc},
            ProgNode::Rel{rel: INPUT_inputs_InlineFunc},
            ProgNode::Rel{rel: inputs_InlineFuncParam},
            ProgNode::Rel{rel: INPUT_inputs_InlineFuncParam},
            ProgNode::Rel{rel: inputs_InputScope},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: ChildScope, distinct: true}]},
            ProgNode::Rel{rel: ClosestFunction},
            ProgNode::Rel{rel: INPUT_inputs_InputScope},
            ProgNode::Rel{rel: inputs_Label},
            ProgNode::Rel{rel: INPUT_inputs_Label},
            ProgNode::Rel{rel: inputs_LetDecl},
            ProgNode::Rel{rel: INPUT_inputs_LetDecl},
            ProgNode::Rel{rel: inputs_NameRef},
            ProgNode::Rel{rel: INPUT_inputs_NameRef},
            ProgNode::Rel{rel: inputs_New},
            ProgNode::Rel{rel: INPUT_inputs_New},
            ProgNode::Rel{rel: inputs_Property},
            ProgNode::Rel{rel: INPUT_inputs_Property},
            ProgNode::Rel{rel: inputs_Return},
            ProgNode::Rel{rel: INPUT_inputs_Return},
            ProgNode::Rel{rel: inputs_Statement},
            ProgNode::Rel{rel: INPUT_inputs_Statement},
            ProgNode::Rel{rel: inputs_Switch},
            ProgNode::Rel{rel: INPUT_inputs_Switch},
            ProgNode::Rel{rel: inputs_SwitchCase},
            ProgNode::Rel{rel: INPUT_inputs_SwitchCase},
            ProgNode::Rel{rel: inputs_Template},
            ProgNode::Rel{rel: INPUT_inputs_Template},
            ProgNode::Rel{rel: inputs_Ternary},
            ProgNode::Rel{rel: INPUT_inputs_Ternary},
            ProgNode::Rel{rel: inputs_Throw},
            ProgNode::Rel{rel: INPUT_inputs_Throw},
            ProgNode::Rel{rel: inputs_Try},
            ProgNode::Rel{rel: INPUT_inputs_Try},
            ProgNode::Rel{rel: inputs_UnaryOp},
            ProgNode::Rel{rel: INPUT_inputs_UnaryOp},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: WithinTypeofExpr, distinct: true}]},
            ProgNode::Rel{rel: inputs_VarDecl},
            ProgNode::Rel{rel: INPUT_inputs_VarDecl},
            ProgNode::Rel{rel: __Prefix_2},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: NameInScope, distinct: true}]},
            ProgNode::Rel{rel: TypeofUndefinedAlwaysUndefined},
            ProgNode::Rel{rel: VarUseBeforeDeclaration},
            ProgNode::Rel{rel: __Prefix_0},
            ProgNode::Rel{rel: InvalidNameUse},
            ProgNode::Rel{rel: inputs_While},
            ProgNode::Rel{rel: INPUT_inputs_While},
            ProgNode::Rel{rel: inputs_With},
            ProgNode::Rel{rel: INPUT_inputs_With},
            ProgNode::Rel{rel: inputs_Yield},
            ProgNode::Rel{rel: INPUT_inputs_Yield}
        ],
        init_data: vec![
        ]
    }
}