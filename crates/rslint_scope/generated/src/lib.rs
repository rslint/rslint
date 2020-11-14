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
    clippy::toplevel_ref_arg,
    clippy::double_parens,
    clippy::clone_on_copy,
    clippy::just_underscores_and_digits,
    clippy::match_single_binding,
    clippy::op_ref,
    clippy::nonminimal_bool,
    clippy::redundant_clone
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


decl_update_deserializer!(UpdateSerializer,(0, ::types::ChainedWith), (1, ::types::ChildScope), (2, ::types::FunctionLevelScope), (3, ::types::IsExported), (4, ::types::NameInScope), (5, ::types::NoUndef), (6, ::types::TypeofUndef), (7, ::types::UnusedVariables), (8, ::types::UseBeforeDecl), (9, ::types::VariableUsages), (10, ::types::WithinTypeofExpr), (13, ::types::inputs::Array), (14, ::types::inputs::Arrow), (15, ::types::inputs::ArrowParam), (16, ::types::inputs::Assign), (17, ::types::inputs::Await), (18, ::types::inputs::BinOp), (19, ::types::inputs::BracketAccess), (20, ::types::inputs::Break), (21, ::types::inputs::Call), (22, ::types::inputs::Class), (23, ::types::inputs::ClassExpr), (24, ::types::inputs::ConstDecl), (25, ::types::inputs::Continue), (26, ::types::inputs::DoWhile), (27, ::types::inputs::DotAccess), (28, ::types::inputs::EveryScope), (29, ::types::inputs::ExprBigInt), (30, ::types::inputs::ExprBool), (31, ::types::inputs::ExprNumber), (32, ::types::inputs::ExprString), (33, ::types::inputs::Expression), (34, ::types::inputs::File), (35, ::types::inputs::FileExport), (36, ::types::inputs::For), (37, ::types::inputs::ForIn), (38, ::types::inputs::Function), (39, ::types::inputs::FunctionArg), (40, ::types::inputs::If), (41, ::types::inputs::ImplicitGlobal), (42, ::types::inputs::ImportDecl), (43, ::types::inputs::InlineFunc), (44, ::types::inputs::InlineFuncParam), (45, ::types::inputs::InputScope), (46, ::types::inputs::Label), (47, ::types::inputs::LetDecl), (48, ::types::inputs::NameRef), (49, ::types::inputs::New), (50, ::types::inputs::Property), (51, ::types::inputs::Return), (52, ::types::inputs::Statement), (53, ::types::inputs::Switch), (54, ::types::inputs::SwitchCase), (55, ::types::inputs::Template), (56, ::types::inputs::Ternary), (57, ::types::inputs::Throw), (58, ::types::inputs::Try), (59, ::types::inputs::UnaryOp), (60, ::types::inputs::VarDecl), (61, ::types::inputs::While), (62, ::types::inputs::With), (63, ::types::inputs::Yield));
impl TryFrom<&str> for Relations {
    type Error = ();
    fn try_from(rname: &str) -> ::std::result::Result<Self, ()> {
         match rname {
        "ChainedWith" => Ok(Relations::ChainedWith),
        "ChildScope" => Ok(Relations::ChildScope),
        "FunctionLevelScope" => Ok(Relations::FunctionLevelScope),
        "IsExported" => Ok(Relations::IsExported),
        "NameInScope" => Ok(Relations::NameInScope),
        "NoUndef" => Ok(Relations::NoUndef),
        "TypeofUndef" => Ok(Relations::TypeofUndef),
        "UnusedVariables" => Ok(Relations::UnusedVariables),
        "UseBeforeDecl" => Ok(Relations::UseBeforeDecl),
        "VariableUsages" => Ok(Relations::VariableUsages),
        "WithinTypeofExpr" => Ok(Relations::WithinTypeofExpr),
        "__Prefix_0" => Ok(Relations::__Prefix_0),
        "__Prefix_1" => Ok(Relations::__Prefix_1),
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
        "inputs::File" => Ok(Relations::inputs_File),
        "inputs::FileExport" => Ok(Relations::inputs_FileExport),
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
        Relations::FunctionLevelScope => true,
        Relations::IsExported => true,
        Relations::NameInScope => true,
        Relations::NoUndef => true,
        Relations::TypeofUndef => true,
        Relations::UnusedVariables => true,
        Relations::UseBeforeDecl => true,
        Relations::VariableUsages => true,
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
        Relations::inputs_File => true,
        Relations::inputs_FileExport => true,
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
        2 => Ok(Relations::FunctionLevelScope),
        3 => Ok(Relations::IsExported),
        4 => Ok(Relations::NameInScope),
        5 => Ok(Relations::NoUndef),
        6 => Ok(Relations::TypeofUndef),
        7 => Ok(Relations::UnusedVariables),
        8 => Ok(Relations::UseBeforeDecl),
        9 => Ok(Relations::VariableUsages),
        10 => Ok(Relations::WithinTypeofExpr),
        11 => Ok(Relations::__Prefix_0),
        12 => Ok(Relations::__Prefix_1),
        13 => Ok(Relations::inputs_Array),
        14 => Ok(Relations::inputs_Arrow),
        15 => Ok(Relations::inputs_ArrowParam),
        16 => Ok(Relations::inputs_Assign),
        17 => Ok(Relations::inputs_Await),
        18 => Ok(Relations::inputs_BinOp),
        19 => Ok(Relations::inputs_BracketAccess),
        20 => Ok(Relations::inputs_Break),
        21 => Ok(Relations::inputs_Call),
        22 => Ok(Relations::inputs_Class),
        23 => Ok(Relations::inputs_ClassExpr),
        24 => Ok(Relations::inputs_ConstDecl),
        25 => Ok(Relations::inputs_Continue),
        26 => Ok(Relations::inputs_DoWhile),
        27 => Ok(Relations::inputs_DotAccess),
        28 => Ok(Relations::inputs_EveryScope),
        29 => Ok(Relations::inputs_ExprBigInt),
        30 => Ok(Relations::inputs_ExprBool),
        31 => Ok(Relations::inputs_ExprNumber),
        32 => Ok(Relations::inputs_ExprString),
        33 => Ok(Relations::inputs_Expression),
        34 => Ok(Relations::inputs_File),
        35 => Ok(Relations::inputs_FileExport),
        36 => Ok(Relations::inputs_For),
        37 => Ok(Relations::inputs_ForIn),
        38 => Ok(Relations::inputs_Function),
        39 => Ok(Relations::inputs_FunctionArg),
        40 => Ok(Relations::inputs_If),
        41 => Ok(Relations::inputs_ImplicitGlobal),
        42 => Ok(Relations::inputs_ImportDecl),
        43 => Ok(Relations::inputs_InlineFunc),
        44 => Ok(Relations::inputs_InlineFuncParam),
        45 => Ok(Relations::inputs_InputScope),
        46 => Ok(Relations::inputs_Label),
        47 => Ok(Relations::inputs_LetDecl),
        48 => Ok(Relations::inputs_NameRef),
        49 => Ok(Relations::inputs_New),
        50 => Ok(Relations::inputs_Property),
        51 => Ok(Relations::inputs_Return),
        52 => Ok(Relations::inputs_Statement),
        53 => Ok(Relations::inputs_Switch),
        54 => Ok(Relations::inputs_SwitchCase),
        55 => Ok(Relations::inputs_Template),
        56 => Ok(Relations::inputs_Ternary),
        57 => Ok(Relations::inputs_Throw),
        58 => Ok(Relations::inputs_Try),
        59 => Ok(Relations::inputs_UnaryOp),
        60 => Ok(Relations::inputs_VarDecl),
        61 => Ok(Relations::inputs_While),
        62 => Ok(Relations::inputs_With),
        63 => Ok(Relations::inputs_Yield),
             _  => Err(())
         }
    }
}
pub fn relid2name(rid: RelId) -> Option<&'static str> {
   match rid {
        0 => Some(&"ChainedWith"),
        1 => Some(&"ChildScope"),
        2 => Some(&"FunctionLevelScope"),
        3 => Some(&"IsExported"),
        4 => Some(&"NameInScope"),
        5 => Some(&"NoUndef"),
        6 => Some(&"TypeofUndef"),
        7 => Some(&"UnusedVariables"),
        8 => Some(&"UseBeforeDecl"),
        9 => Some(&"VariableUsages"),
        10 => Some(&"WithinTypeofExpr"),
        11 => Some(&"__Prefix_0"),
        12 => Some(&"__Prefix_1"),
        13 => Some(&"inputs::Array"),
        14 => Some(&"inputs::Arrow"),
        15 => Some(&"inputs::ArrowParam"),
        16 => Some(&"inputs::Assign"),
        17 => Some(&"inputs::Await"),
        18 => Some(&"inputs::BinOp"),
        19 => Some(&"inputs::BracketAccess"),
        20 => Some(&"inputs::Break"),
        21 => Some(&"inputs::Call"),
        22 => Some(&"inputs::Class"),
        23 => Some(&"inputs::ClassExpr"),
        24 => Some(&"inputs::ConstDecl"),
        25 => Some(&"inputs::Continue"),
        26 => Some(&"inputs::DoWhile"),
        27 => Some(&"inputs::DotAccess"),
        28 => Some(&"inputs::EveryScope"),
        29 => Some(&"inputs::ExprBigInt"),
        30 => Some(&"inputs::ExprBool"),
        31 => Some(&"inputs::ExprNumber"),
        32 => Some(&"inputs::ExprString"),
        33 => Some(&"inputs::Expression"),
        34 => Some(&"inputs::File"),
        35 => Some(&"inputs::FileExport"),
        36 => Some(&"inputs::For"),
        37 => Some(&"inputs::ForIn"),
        38 => Some(&"inputs::Function"),
        39 => Some(&"inputs::FunctionArg"),
        40 => Some(&"inputs::If"),
        41 => Some(&"inputs::ImplicitGlobal"),
        42 => Some(&"inputs::ImportDecl"),
        43 => Some(&"inputs::InlineFunc"),
        44 => Some(&"inputs::InlineFuncParam"),
        45 => Some(&"inputs::InputScope"),
        46 => Some(&"inputs::Label"),
        47 => Some(&"inputs::LetDecl"),
        48 => Some(&"inputs::NameRef"),
        49 => Some(&"inputs::New"),
        50 => Some(&"inputs::Property"),
        51 => Some(&"inputs::Return"),
        52 => Some(&"inputs::Statement"),
        53 => Some(&"inputs::Switch"),
        54 => Some(&"inputs::SwitchCase"),
        55 => Some(&"inputs::Template"),
        56 => Some(&"inputs::Ternary"),
        57 => Some(&"inputs::Throw"),
        58 => Some(&"inputs::Try"),
        59 => Some(&"inputs::UnaryOp"),
        60 => Some(&"inputs::VarDecl"),
        61 => Some(&"inputs::While"),
        62 => Some(&"inputs::With"),
        63 => Some(&"inputs::Yield"),
       _  => None
   }
}
#[cfg(feature = "c_api")]
pub fn relid2cname(rid: RelId) -> Option<&'static ::std::ffi::CStr> {
    RELIDMAPC.get(&rid).copied()
}   /// A map of `RelId`s to their name as an `&'static str`
#[cfg(feature = "globals")]
pub static RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(64, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ChainedWith, "ChainedWith");
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::FunctionLevelScope, "FunctionLevelScope");
        map.insert(Relations::IsExported, "IsExported");
        map.insert(Relations::NameInScope, "NameInScope");
        map.insert(Relations::NoUndef, "NoUndef");
        map.insert(Relations::TypeofUndef, "TypeofUndef");
        map.insert(Relations::UnusedVariables, "UnusedVariables");
        map.insert(Relations::UseBeforeDecl, "UseBeforeDecl");
        map.insert(Relations::VariableUsages, "VariableUsages");
        map.insert(Relations::WithinTypeofExpr, "WithinTypeofExpr");
        map.insert(Relations::__Prefix_0, "__Prefix_0");
        map.insert(Relations::__Prefix_1, "__Prefix_1");
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
        map.insert(Relations::inputs_File, "inputs::File");
        map.insert(Relations::inputs_FileExport, "inputs::FileExport");
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
#[cfg(feature = "c_api")]
pub static RELIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<RelId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(64, ::fnv::FnvBuildHasher::default());
        map.insert(0, ::std::ffi::CStr::from_bytes_with_nul(b"ChainedWith\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(1, ::std::ffi::CStr::from_bytes_with_nul(b"ChildScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(2, ::std::ffi::CStr::from_bytes_with_nul(b"FunctionLevelScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(3, ::std::ffi::CStr::from_bytes_with_nul(b"IsExported\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(4, ::std::ffi::CStr::from_bytes_with_nul(b"NameInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(5, ::std::ffi::CStr::from_bytes_with_nul(b"NoUndef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(6, ::std::ffi::CStr::from_bytes_with_nul(b"TypeofUndef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(7, ::std::ffi::CStr::from_bytes_with_nul(b"UnusedVariables\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(8, ::std::ffi::CStr::from_bytes_with_nul(b"UseBeforeDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(9, ::std::ffi::CStr::from_bytes_with_nul(b"VariableUsages\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(10, ::std::ffi::CStr::from_bytes_with_nul(b"WithinTypeofExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(11, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_0\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(12, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_1\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(13, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Array\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(14, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Arrow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(15, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ArrowParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(16, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Assign\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(17, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Await\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(18, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::BinOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(19, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::BracketAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(20, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Break\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(21, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Call\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(22, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Class\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(23, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ClassExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(24, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ConstDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(25, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Continue\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(26, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::DoWhile\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(27, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::DotAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(28, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::EveryScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(29, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprBigInt\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(30, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprBool\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(31, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprNumber\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(32, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprString\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(33, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Expression\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(34, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::File\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(35, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::FileExport\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(36, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::For\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(37, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ForIn\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(38, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Function\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(39, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::FunctionArg\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(40, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::If\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(41, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ImplicitGlobal\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(42, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ImportDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(43, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InlineFunc\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(44, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InlineFuncParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(45, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InputScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(46, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Label\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(47, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::LetDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(48, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::NameRef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(49, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::New\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(50, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Property\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(51, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Return\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(52, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Statement\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(53, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Switch\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(54, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::SwitchCase\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(55, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Template\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(56, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Ternary\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(57, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Throw\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(58, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Try\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(59, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::UnaryOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(60, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::VarDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(61, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::While\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(62, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::With\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(63, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Yield\0").expect("Unreachable: A null byte was specifically inserted"));
        map
    });
    /// A map of input `Relations`s to their name as an `&'static str`
pub static INPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(51, ::fnv::FnvBuildHasher::default());
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
        map.insert(Relations::inputs_File, "inputs::File");
        map.insert(Relations::inputs_FileExport, "inputs::FileExport");
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
#[cfg(feature = "globals")]
pub static OUTPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(11, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::ChainedWith, "ChainedWith");
        map.insert(Relations::ChildScope, "ChildScope");
        map.insert(Relations::FunctionLevelScope, "FunctionLevelScope");
        map.insert(Relations::IsExported, "IsExported");
        map.insert(Relations::NameInScope, "NameInScope");
        map.insert(Relations::NoUndef, "NoUndef");
        map.insert(Relations::TypeofUndef, "TypeofUndef");
        map.insert(Relations::UnusedVariables, "UnusedVariables");
        map.insert(Relations::UseBeforeDecl, "UseBeforeDecl");
        map.insert(Relations::VariableUsages, "VariableUsages");
        map.insert(Relations::WithinTypeofExpr, "WithinTypeofExpr");
        map
    });
impl TryFrom<&str> for Indexes {
    type Error = ();
    fn try_from(iname: &str) -> ::std::result::Result<Self, ()> {
         match iname {
        "ChildScopeByParent" => Ok(Indexes::ChildScopeByParent),
        "Index_VariableInScope" => Ok(Indexes::Index_VariableInScope),
        "Index_VariablesForScope" => Ok(Indexes::Index_VariablesForScope),
        "inputs::ExpressionById" => Ok(Indexes::inputs_ExpressionById),
        "inputs::ExpressionBySpan" => Ok(Indexes::inputs_ExpressionBySpan),
        "inputs::InputScopeByChild" => Ok(Indexes::inputs_InputScopeByChild),
        "inputs::InputScopeByParent" => Ok(Indexes::inputs_InputScopeByParent),
             _  => Err(())
         }
    }
}
impl TryFrom<IdxId> for Indexes {
    type Error = ();
    fn try_from(iid: IdxId) -> ::core::result::Result<Self, ()> {
         match iid {
        0 => Ok(Indexes::ChildScopeByParent),
        1 => Ok(Indexes::Index_VariableInScope),
        2 => Ok(Indexes::Index_VariablesForScope),
        3 => Ok(Indexes::inputs_ExpressionById),
        4 => Ok(Indexes::inputs_ExpressionBySpan),
        5 => Ok(Indexes::inputs_InputScopeByChild),
        6 => Ok(Indexes::inputs_InputScopeByParent),
             _  => Err(())
         }
    }
}
pub fn indexid2name(iid: IdxId) -> Option<&'static str> {
   match iid {
        0 => Some(&"ChildScopeByParent"),
        1 => Some(&"Index_VariableInScope"),
        2 => Some(&"Index_VariablesForScope"),
        3 => Some(&"inputs::ExpressionById"),
        4 => Some(&"inputs::ExpressionBySpan"),
        5 => Some(&"inputs::InputScopeByChild"),
        6 => Some(&"inputs::InputScopeByParent"),
       _  => None
   }
}
#[cfg(feature = "c_api")]
pub fn indexid2cname(iid: IdxId) -> Option<&'static ::std::ffi::CStr> {
    IDXIDMAPC.get(&iid).copied()
}   /// A map of `Indexes` to their name as an `&'static str`
#[cfg(feature = "globals")]
pub static IDXIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Indexes, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(7, ::fnv::FnvBuildHasher::default());
        map.insert(Indexes::ChildScopeByParent, "ChildScopeByParent");
        map.insert(Indexes::Index_VariableInScope, "Index_VariableInScope");
        map.insert(Indexes::Index_VariablesForScope, "Index_VariablesForScope");
        map.insert(Indexes::inputs_ExpressionById, "inputs::ExpressionById");
        map.insert(Indexes::inputs_ExpressionBySpan, "inputs::ExpressionBySpan");
        map.insert(Indexes::inputs_InputScopeByChild, "inputs::InputScopeByChild");
        map.insert(Indexes::inputs_InputScopeByParent, "inputs::InputScopeByParent");
        map
    });
    /// A map of `IdxId`s to their name as an `&'static CStr`
#[cfg(feature = "c_api")]
pub static IDXIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<IdxId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(7, ::fnv::FnvBuildHasher::default());
        map.insert(0, ::std::ffi::CStr::from_bytes_with_nul(b"ChildScopeByParent\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(1, ::std::ffi::CStr::from_bytes_with_nul(b"Index_VariableInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(2, ::std::ffi::CStr::from_bytes_with_nul(b"Index_VariablesForScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(3, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExpressionById\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(4, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExpressionBySpan\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(5, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InputScopeByChild\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(6, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InputScopeByParent\0").expect("Unreachable: A null byte was specifically inserted"));
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
        Relations::FunctionLevelScope => {
            Ok(<::types::FunctionLevelScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::IsExported => {
            Ok(<::types::IsExported>::from_record(_rec)?.into_ddvalue())
        },
        Relations::NameInScope => {
            Ok(<::types::NameInScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::NoUndef => {
            Ok(<::types::NoUndef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::TypeofUndef => {
            Ok(<::types::TypeofUndef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::UnusedVariables => {
            Ok(<::types::UnusedVariables>::from_record(_rec)?.into_ddvalue())
        },
        Relations::UseBeforeDecl => {
            Ok(<::types::UseBeforeDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::VariableUsages => {
            Ok(<::types::VariableUsages>::from_record(_rec)?.into_ddvalue())
        },
        Relations::WithinTypeofExpr => {
            Ok(<::types::WithinTypeofExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::__Prefix_0 => {
            Ok(<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::StmtId>>::from_record(_rec)?.into_ddvalue())
        },
        Relations::__Prefix_1 => {
            Ok(<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span>>::from_record(_rec)?.into_ddvalue())
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
        Relations::inputs_File => {
            Ok(<::types::inputs::File>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_FileExport => {
            Ok(<::types::inputs::FileExport>::from_record(_rec)?.into_ddvalue())
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
        Indexes::ChildScopeByParent => {
            Ok(<::types::ddlog_std::tuple2<::types::ast::ScopeId, ::types::ast::FileId>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::Index_VariableInScope => {
            Ok(<::types::ddlog_std::tuple3<::types::ast::FileId, ::types::ast::ScopeId, ::types::internment::Intern<String>>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::Index_VariablesForScope => {
            Ok(<::types::ddlog_std::tuple2<::types::ast::FileId, ::types::ast::ScopeId>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::inputs_ExpressionById => {
            Ok(<::types::ddlog_std::tuple2<::types::ast::ExprId, ::types::ast::FileId>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::inputs_ExpressionBySpan => {
            Ok(<::types::ddlog_std::tuple2<::types::ast::Span, ::types::ast::FileId>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::inputs_InputScopeByChild => {
            Ok(<::types::ddlog_std::tuple2<::types::ast::ScopeId, ::types::ast::FileId>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::inputs_InputScopeByParent => {
            Ok(<::types::ddlog_std::tuple2<::types::ast::ScopeId, ::types::ast::FileId>>::from_record(_rec)?.into_ddvalue())
        }
    }
}
pub fn indexes2arrid(idx: Indexes) -> ArrId {
    match idx {
        Indexes::ChildScopeByParent => ( 1, 2),
        Indexes::Index_VariableInScope => ( 4, 9),
        Indexes::Index_VariablesForScope => ( 4, 10),
        Indexes::inputs_ExpressionById => ( 33, 5),
        Indexes::inputs_ExpressionBySpan => ( 33, 6),
        Indexes::inputs_InputScopeByChild => ( 45, 1),
        Indexes::inputs_InputScopeByParent => ( 45, 2),
    }
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Relations {
    ChainedWith = 0,
    ChildScope = 1,
    FunctionLevelScope = 2,
    IsExported = 3,
    NameInScope = 4,
    NoUndef = 5,
    TypeofUndef = 6,
    UnusedVariables = 7,
    UseBeforeDecl = 8,
    VariableUsages = 9,
    WithinTypeofExpr = 10,
    __Prefix_0 = 11,
    __Prefix_1 = 12,
    inputs_Array = 13,
    inputs_Arrow = 14,
    inputs_ArrowParam = 15,
    inputs_Assign = 16,
    inputs_Await = 17,
    inputs_BinOp = 18,
    inputs_BracketAccess = 19,
    inputs_Break = 20,
    inputs_Call = 21,
    inputs_Class = 22,
    inputs_ClassExpr = 23,
    inputs_ConstDecl = 24,
    inputs_Continue = 25,
    inputs_DoWhile = 26,
    inputs_DotAccess = 27,
    inputs_EveryScope = 28,
    inputs_ExprBigInt = 29,
    inputs_ExprBool = 30,
    inputs_ExprNumber = 31,
    inputs_ExprString = 32,
    inputs_Expression = 33,
    inputs_File = 34,
    inputs_FileExport = 35,
    inputs_For = 36,
    inputs_ForIn = 37,
    inputs_Function = 38,
    inputs_FunctionArg = 39,
    inputs_If = 40,
    inputs_ImplicitGlobal = 41,
    inputs_ImportDecl = 42,
    inputs_InlineFunc = 43,
    inputs_InlineFuncParam = 44,
    inputs_InputScope = 45,
    inputs_Label = 46,
    inputs_LetDecl = 47,
    inputs_NameRef = 48,
    inputs_New = 49,
    inputs_Property = 50,
    inputs_Return = 51,
    inputs_Statement = 52,
    inputs_Switch = 53,
    inputs_SwitchCase = 54,
    inputs_Template = 55,
    inputs_Ternary = 56,
    inputs_Throw = 57,
    inputs_Try = 58,
    inputs_UnaryOp = 59,
    inputs_VarDecl = 60,
    inputs_While = 61,
    inputs_With = 62,
    inputs_Yield = 63
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Indexes {
    ChildScopeByParent = 0,
    Index_VariableInScope = 1,
    Index_VariablesForScope = 2,
    inputs_ExpressionById = 3,
    inputs_ExpressionBySpan = 4,
    inputs_InputScopeByChild = 5,
    inputs_InputScopeByParent = 6
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
                                  name: r###"(inputs::Arrow{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow) /*join*/"###.to_string(),
                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                   {
                                       let __cloned = __v.clone();
                                       match unsafe {< ::types::inputs::Arrow>::from_ddvalue(__v) } {
                                           ::types::inputs::Arrow{expr_id: ref _0, file: ref _1, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: _}}} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                           _ => None
                                       }.map(|x|(x,__cloned))
                                   }
                                   __f},
                                   queryable: false
                               },
                               Arrangement::Map{
                                  name: r###"(inputs::Arrow{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(_: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow) /*join*/"###.to_string(),
                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                   {
                                       let __cloned = __v.clone();
                                       match unsafe {< ::types::inputs::Arrow>::from_ddvalue(__v) } {
                                           ::types::inputs::Arrow{expr_id: ref _0, file: ref _1, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Right{r: _}}} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                           _ => None
                                       }.map(|x|(x,__cloned))
                                   }
                                   __f},
                                   queryable: false
                               }],
                           change_cb:    None
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
                                    Arrangement::Map{
                                       name: r###"(inputs::ArrowParam{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .param=(_: internment::Intern<ast::Pattern>)}: inputs::ArrowParam) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::ArrowParam>::from_ddvalue(__v) } {
                                                ::types::inputs::ArrowParam{expr_id: ref _0, file: ref _1, param: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
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
                                   name: r###"(inputs::Assign{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign) /*join*/"###.to_string(),
                                    afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                    {
                                        let __cloned = __v.clone();
                                        match unsafe {< ::types::inputs::Assign>::from_ddvalue(__v) } {
                                            ::types::inputs::Assign{expr_id: ref _0, file: ref _1, lhs: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: _}}, rhs: _, op: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                            _ => None
                                        }.map(|x|(x,__cloned))
                                    }
                                    __f},
                                    queryable: false
                                }],
                            change_cb:    None
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
                              Arrangement::Map{
                                 name: r###"(inputs::Call{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .callee=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::inputs::Call>::from_ddvalue(__v) } {
                                          ::types::inputs::Call{expr_id: ref _0, file: ref _1, callee: ::types::ddlog_std::Option::Some{x: _}, args: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              }],
                          change_cb:    None
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
                               Arrangement::Map{
                                  name: r###"(inputs::Class{.id=(_0: ast::ClassId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Class) /*join*/"###.to_string(),
                                   afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                   {
                                       let __cloned = __v.clone();
                                       match unsafe {< ::types::inputs::Class>::from_ddvalue(__v) } {
                                           ::types::inputs::Class{id: ref _0, file: ref _1, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: _, span: _}}, parent: _, elements: _, scope: _, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                           _ => None
                                       }.map(|x|(x,__cloned))
                                   }
                                   __f},
                                   queryable: false
                               }],
                           change_cb:    None
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
                                   Arrangement::Set{
                                       name: r###"(inputs::ClassExpr{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr) /*semijoin*/"###.to_string(),
                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                       {
                                           match unsafe {< ::types::inputs::ClassExpr>::from_ddvalue(__v) } {
                                               ::types::inputs::ClassExpr{expr_id: ref _0, file: ref _1, elements: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                               _ => None
                                           }
                                       }
                                       __f},
                                       distinct: false
                                   }],
                               change_cb:    None
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
                                   Arrangement::Map{
                                      name: r###"(inputs::ConstDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::inputs::ConstDecl>::from_ddvalue(__v) } {
                                               ::types::inputs::ConstDecl{stmt_id: ref _0, file: ref _1, pattern: ::types::ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   },
                                   Arrangement::Map{
                                      name: r###"(inputs::ConstDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::inputs::ConstDecl>::from_ddvalue(__v) } {
                                               ::types::inputs::ConstDecl{stmt_id: ref _0, file: ref _1, pattern: _, value: ::types::ddlog_std::Option::Some{x: _}, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   }],
                               change_cb:    None
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
                              /* ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- inputs::BracketAccess[(inputs::BracketAccess{.expr_id=(_: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(property: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess)]. */
                              Rule::CollectionRule {
                                  description: "ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- inputs::BracketAccess[(inputs::BracketAccess{.expr_id=(_: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(property: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess)].".to_string(),
                                  rel: Relations::inputs_BracketAccess as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- inputs::BracketAccess[(inputs::BracketAccess{.expr_id=(_: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(property: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref file, ref object, ref property) = match *unsafe {<::types::inputs::BracketAccess>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::BracketAccess{expr_id: _, file: ref file, object: ::types::ddlog_std::Option::Some{x: ref object}, prop: ::types::ddlog_std::Option::Some{x: ref property}} => ((*file).clone(), (*object).clone(), (*property).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::ChainedWith{object: (*object).clone(), property: (*property).clone(), file: (*file).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- inputs::DotAccess[(inputs::DotAccess{.expr_id=(property: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess)]. */
                              Rule::CollectionRule {
                                  description: "ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- inputs::DotAccess[(inputs::DotAccess{.expr_id=(property: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess)].".to_string(),
                                  rel: Relations::inputs_DotAccess as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- inputs::DotAccess[(inputs::DotAccess{.expr_id=(property: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref property, ref file, ref object) = match *unsafe {<::types::inputs::DotAccess>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::DotAccess{expr_id: ref property, file: ref file, object: ::types::ddlog_std::Option::Some{x: ref object}, prop: _} => ((*property).clone(), (*file).clone(), (*object).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::ChainedWith{object: (*object).clone(), property: (*property).clone(), file: (*file).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(interum: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)], ChainedWith[(ChainedWith{.object=(interum: ast::ExprId), .property=(property: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)]. */
                              Rule::ArrangementRule {
                                  description: "ChainedWith[(ChainedWith{.object=object, .property=property, .file=file}: ChainedWith)] :- ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(interum: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)], ChainedWith[(ChainedWith{.object=(interum: ast::ExprId), .property=(property: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)].".to_string(),
                                  arr: ( Relations::ChainedWith as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "ChainedWith[(ChainedWith{.object=(object: ast::ExprId), .property=(interum: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)], ChainedWith[(ChainedWith{.object=(interum: ast::ExprId), .property=(property: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::ChainedWith as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref object, ref interum, ref file) = match *unsafe {<::types::ChainedWith>::from_ddvalue_ref(__v1) } {
                                                     ::types::ChainedWith{object: ref object, property: ref interum, file: ref file} => ((*object).clone(), (*interum).clone(), (*file).clone()),
                                                     _ => return None
                                                 };
                                                 let ref property = match *unsafe {<::types::ChainedWith>::from_ddvalue_ref(__v2) } {
                                                     ::types::ChainedWith{object: _, property: ref property, file: _} => (*property).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::ChainedWith{object: (*object).clone(), property: (*property).clone(), file: (*file).clone()})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              }],
                          arrangements: vec![
                              Arrangement::Map{
                                 name: r###"(ChainedWith{.object=(_: ast::ExprId), .property=(_0: ast::ExprId), .file=(_1: ast::FileId)}: ChainedWith) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::ChainedWith>::from_ddvalue(__v) } {
                                          ::types::ChainedWith{object: _, property: ref _0, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(ChainedWith{.object=(_0: ast::ExprId), .property=(_: ast::ExprId), .file=(_1: ast::FileId)}: ChainedWith) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::ChainedWith>::from_ddvalue(__v) } {
                                          ::types::ChainedWith{object: ref _0, property: _, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Set{
                                  name: r###"(ChainedWith{.object=(_: ast::ExprId), .property=(_0: ast::ExprId), .file=(_1: ast::FileId)}: ChainedWith) /*antijoin*/"###.to_string(),
                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                  {
                                      match unsafe {< ::types::ChainedWith>::from_ddvalue(__v) } {
                                          ::types::ChainedWith{object: _, property: ref _0, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }
                                  }
                                  __f},
                                  distinct: true
                              }],
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
                                       name: r###"(inputs::EveryScope{.scope=(_: ast::ScopeId), .file=(_0: ast::FileId)}: inputs::EveryScope) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::EveryScope>::from_ddvalue(__v) } {
                                                ::types::inputs::EveryScope{scope: _, file: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    }],
                                change_cb:    None
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
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, file: ref _1, kind: _, scope: _, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, file: ref _1, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: _, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Set{
                                        name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(_: ast::ExprKind), .scope=(_2: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*semijoin*/"###.to_string(),
                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                        {
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, file: ref _1, kind: _, scope: ref _2, span: _} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                                _ => None
                                            }
                                        }
                                        __f},
                                        distinct: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, file: ref _1, kind: ::types::ast::ExprKind::ExprGrouping{inner: ::types::ddlog_std::Option::Some{x: _}}, scope: _, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(ast::ExprSequence{.exprs=(_: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, file: ref _1, kind: ::types::ast::ExprKind::ExprSequence{exprs: _}, scope: _, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=_0, .file=_1, .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: ref _0, file: ref _1, kind: _, scope: _, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: true
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::Expression{.id=(_: ast::ExprId), .file=_1, .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=_0}: inputs::Expression) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::Expression>::from_ddvalue(__v) } {
                                                ::types::inputs::Expression{id: _, file: ref _1, kind: _, scope: _, span: ref _0} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: true
                                    }],
                                change_cb:    None
                            };
    let inputs_File = Relation {
                          name:         "inputs::File".to_string(),
                          input:        true,
                          distinct:     false,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::inputs_File as RelId,
                          rules:        vec![
                              ],
                          arrangements: vec![
                              ],
                          change_cb:    None
                      };
    let inputs_FileExport = Relation {
                                name:         "inputs::FileExport".to_string(),
                                input:        true,
                                distinct:     false,
                                caching_mode: CachingMode::Set,
                                key_func:     None,
                                id:           Relations::inputs_FileExport as RelId,
                                rules:        vec![
                                    ],
                                arrangements: vec![
                                    ],
                                change_cb:    None
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
                                     name: r###"(inputs::Function{.id=(_: ast::FuncId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_0: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###.to_string(),
                                      afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                      {
                                          let __cloned = __v.clone();
                                          match unsafe {< ::types::inputs::Function>::from_ddvalue(__v) } {
                                              ::types::inputs::Function{id: _, file: ref _1, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: _, span: _}}, scope: ref _0, body: _, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                              _ => None
                                          }.map(|x|(x,__cloned))
                                      }
                                      __f},
                                      queryable: false
                                  },
                                  Arrangement::Map{
                                     name: r###"(inputs::Function{.id=(_0: ast::FuncId), .file=(_1: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###.to_string(),
                                      afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                      {
                                          let __cloned = __v.clone();
                                          match unsafe {< ::types::inputs::Function>::from_ddvalue(__v) } {
                                              ::types::inputs::Function{id: ref _0, file: ref _1, name: _, scope: _, body: _, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                              _ => None
                                          }.map(|x|(x,__cloned))
                                      }
                                      __f},
                                      queryable: false
                                  },
                                  Arrangement::Map{
                                     name: r###"(inputs::Function{.id=(_0: ast::FuncId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###.to_string(),
                                      afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                      {
                                          let __cloned = __v.clone();
                                          match unsafe {< ::types::inputs::Function>::from_ddvalue(__v) } {
                                              ::types::inputs::Function{id: ref _0, file: ref _1, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: _, span: _}}, scope: _, body: _, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                              _ => None
                                          }.map(|x|(x,__cloned))
                                      }
                                      __f},
                                      queryable: false
                                  }],
                              change_cb:    None
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
                                     Arrangement::Map{
                                        name: r###"(inputs::FunctionArg{.parent_func=(_0: ast::FuncId), .file=(_1: ast::FileId), .pattern=(_: internment::Intern<ast::Pattern>), .implicit=(_: bool)}: inputs::FunctionArg) /*join*/"###.to_string(),
                                         afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                         {
                                             let __cloned = __v.clone();
                                             match unsafe {< ::types::inputs::FunctionArg>::from_ddvalue(__v) } {
                                                 ::types::inputs::FunctionArg{parent_func: ref _0, file: ref _1, pattern: _, implicit: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                 _ => None
                                             }.map(|x|(x,__cloned))
                                         }
                                         __f},
                                         queryable: false
                                     }],
                                 change_cb:    None
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
                                           name: r###"(inputs::ImplicitGlobal{.id=(_: ast::GlobalId), .file=(_0: ast::FileId), .name=(_: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::ImplicitGlobal) /*join*/"###.to_string(),
                                            afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                            {
                                                let __cloned = __v.clone();
                                                match unsafe {< ::types::inputs::ImplicitGlobal>::from_ddvalue(__v) } {
                                                    ::types::inputs::ImplicitGlobal{id: _, file: ref _0, name: _, privileges: _} => Some(((*_0).clone()).into_ddvalue()),
                                                    _ => None
                                                }.map(|x|(x,__cloned))
                                            }
                                            __f},
                                            queryable: false
                                        }],
                                    change_cb:    None
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
                                    Arrangement::Map{
                                       name: r###"(inputs::ImportDecl{.id=(_: ast::ImportId), .file=(_0: ast::FileId), .clause=(_: ast::ImportClause)}: inputs::ImportDecl) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::ImportDecl>::from_ddvalue(__v) } {
                                                ::types::inputs::ImportDecl{id: _, file: ref _0, clause: _} => Some(((*_0).clone()).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    }],
                                change_cb:    None
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
                                       name: r###"(inputs::InlineFunc{.expr_id=(_: ast::ExprId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_0: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InlineFunc>::from_ddvalue(__v) } {
                                                ::types::inputs::InlineFunc{expr_id: _, file: ref _1, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: _, span: _}}, body: ::types::ddlog_std::Option::Some{x: ref _0}} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::InlineFunc{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InlineFunc>::from_ddvalue(__v) } {
                                                ::types::inputs::InlineFunc{expr_id: ref _0, file: ref _1, name: _, body: ::types::ddlog_std::Option::Some{x: _}} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    }],
                                change_cb:    None
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
                                         Arrangement::Map{
                                            name: r###"(inputs::InlineFuncParam{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .param=(_: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam) /*join*/"###.to_string(),
                                             afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                             {
                                                 let __cloned = __v.clone();
                                                 match unsafe {< ::types::inputs::InlineFuncParam>::from_ddvalue(__v) } {
                                                     ::types::inputs::InlineFuncParam{expr_id: ref _0, file: ref _1, param: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                     _ => None
                                                 }.map(|x|(x,__cloned))
                                             }
                                             __f},
                                             queryable: false
                                         }],
                                     change_cb:    None
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
                                       name: r###"(inputs::InputScope{.parent=(_: ast::ScopeId), .child=(_0: ast::ScopeId), .file=(_1: ast::FileId)}: inputs::InputScope) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InputScope>::from_ddvalue(__v) } {
                                                ::types::inputs::InputScope{parent: _, child: ref _0, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: false
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::InputScope{.parent=(_: ast::ScopeId), .child=_0, .file=_1}: inputs::InputScope) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InputScope>::from_ddvalue(__v) } {
                                                ::types::inputs::InputScope{parent: _, child: ref _0, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: true
                                    },
                                    Arrangement::Map{
                                       name: r###"(inputs::InputScope{.parent=_0, .child=(_: ast::ScopeId), .file=_1}: inputs::InputScope) /*join*/"###.to_string(),
                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                        {
                                            let __cloned = __v.clone();
                                            match unsafe {< ::types::inputs::InputScope>::from_ddvalue(__v) } {
                                                ::types::inputs::InputScope{parent: ref _0, child: _, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                _ => None
                                            }.map(|x|(x,__cloned))
                                        }
                                        __f},
                                        queryable: true
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
                             /* ChildScope[(ChildScope{.parent=parent, .child=child, .file=file}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], (parent != child). */
                             Rule::CollectionRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child, .file=file}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], (parent != child).".to_string(),
                                 rel: Relations::inputs_InputScope as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of ChildScope[(ChildScope{.parent=parent, .child=child, .file=file}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], (parent != child)." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref parent, ref child, ref file) = match *unsafe {<::types::inputs::InputScope>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::InputScope{parent: ref parent, child: ref child, file: ref file} => ((*parent).clone(), (*child).clone(), (*file).clone()),
                                                         _ => return None
                                                     };
                                                     if !((&*parent) != (&*child)) {return None;};
                                                     Some(((::types::ChildScope{parent: (*parent).clone(), child: (*child).clone(), file: (*file).clone()})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* ChildScope[(ChildScope{.parent=parent, .child=child, .file=file}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(interum: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], ChildScope[(ChildScope{.parent=(interum: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)], (parent != child). */
                             Rule::ArrangementRule {
                                 description: "ChildScope[(ChildScope{.parent=parent, .child=child, .file=file}: ChildScope)] :- inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(interum: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], ChildScope[(ChildScope{.parent=(interum: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)], (parent != child).".to_string(),
                                 arr: ( Relations::inputs_InputScope as RelId, 0),
                                 xform: XFormArrangement::Join{
                                            description: "inputs::InputScope[(inputs::InputScope{.parent=(parent: ast::ScopeId), .child=(interum: ast::ScopeId), .file=(file: ast::FileId)}: inputs::InputScope)], ChildScope[(ChildScope{.parent=(interum: ast::ScopeId), .child=(child: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                            ffun: None,
                                            arrangement: (Relations::ChildScope as RelId,0),
                                            jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                            {
                                                let (ref parent, ref interum, ref file) = match *unsafe {<::types::inputs::InputScope>::from_ddvalue_ref(__v1) } {
                                                    ::types::inputs::InputScope{parent: ref parent, child: ref interum, file: ref file} => ((*parent).clone(), (*interum).clone(), (*file).clone()),
                                                    _ => return None
                                                };
                                                let ref child = match *unsafe {<::types::ChildScope>::from_ddvalue_ref(__v2) } {
                                                    ::types::ChildScope{parent: _, child: ref child, file: _} => (*child).clone(),
                                                    _ => return None
                                                };
                                                if !((&*parent) != (&*child)) {return None;};
                                                Some(((::types::ChildScope{parent: (*parent).clone(), child: (*child).clone(), file: (*file).clone()})).into_ddvalue())
                                            }
                                            __f},
                                            next: Box::new(None)
                                        }
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"(ChildScope{.parent=(_0: ast::ScopeId), .child=(_: ast::ScopeId), .file=(_1: ast::FileId)}: ChildScope) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ChildScope>::from_ddvalue(__v) } {
                                         ::types::ChildScope{parent: ref _0, child: _, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             },
                             Arrangement::Set{
                                 name: r###"(ChildScope{.parent=(_0: ast::ScopeId), .child=(_1: ast::ScopeId), .file=(_2: ast::FileId)}: ChildScope) /*semijoin*/"###.to_string(),
                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                 {
                                     match unsafe {< ::types::ChildScope>::from_ddvalue(__v) } {
                                         ::types::ChildScope{parent: ref _0, child: ref _1, file: ref _2} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                         _ => None
                                     }
                                 }
                                 __f},
                                 distinct: false
                             },
                             Arrangement::Map{
                                name: r###"(ChildScope{.parent=_0, .child=(_: ast::ScopeId), .file=_1}: ChildScope) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ChildScope>::from_ddvalue(__v) } {
                                         ::types::ChildScope{parent: ref _0, child: _, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: true
                             }],
                         change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                     };
    let FunctionLevelScope = Relation {
                                 name:         "FunctionLevelScope".to_string(),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: CachingMode::Set,
                                 key_func:     None,
                                 id:           Relations::FunctionLevelScope as RelId,
                                 rules:        vec![
                                     /* FunctionLevelScope[(FunctionLevelScope{.scope=body, .nearest=body, .file=file, .id=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: FunctionLevelScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)]. */
                                     Rule::CollectionRule {
                                         description: "FunctionLevelScope[(FunctionLevelScope{.scope=body, .nearest=body, .file=file, .id=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: FunctionLevelScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)].".to_string(),
                                         rel: Relations::inputs_Function as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of FunctionLevelScope[(FunctionLevelScope{.scope=body, .nearest=body, .file=file, .id=(ast::AnyIdFunc{.func=func}: ast::AnyId)}: FunctionLevelScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let (ref func, ref file, ref body) = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(&__v) } {
                                                                 ::types::inputs::Function{id: ref func, file: ref file, name: _, scope: _, body: ref body, exported: _} => ((*func).clone(), (*file).clone(), (*body).clone()),
                                                                 _ => return None
                                                             };
                                                             Some(((::types::FunctionLevelScope{scope: (*body).clone(), nearest: (*body).clone(), file: (*file).clone(), id: (::types::ast::AnyId::AnyIdFunc{func: (*func).clone()})})).into_ddvalue())
                                                         }
                                                         __f},
                                                         next: Box::new(None)
                                                     })
                                     },
                                     /* FunctionLevelScope[(FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdFile{.file=file}: ast::AnyId)}: FunctionLevelScope)] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(scope: ast::ScopeId)}: inputs::File)]. */
                                     Rule::CollectionRule {
                                         description: "FunctionLevelScope[(FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdFile{.file=file}: ast::AnyId)}: FunctionLevelScope)] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(scope: ast::ScopeId)}: inputs::File)].".to_string(),
                                         rel: Relations::inputs_File as RelId,
                                         xform: Some(XFormCollection::FilterMap{
                                                         description: "head of FunctionLevelScope[(FunctionLevelScope{.scope=scope, .nearest=scope, .file=file, .id=(ast::AnyIdFile{.file=file}: ast::AnyId)}: FunctionLevelScope)] :- inputs::File[(inputs::File{.id=(file: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(scope: ast::ScopeId)}: inputs::File)]." .to_string(),
                                                         fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                         {
                                                             let (ref file, ref scope) = match *unsafe {<::types::inputs::File>::from_ddvalue_ref(&__v) } {
                                                                 ::types::inputs::File{id: ref file, kind: _, top_level_scope: ref scope} => ((*file).clone(), (*scope).clone()),
                                                                 _ => return None
                                                             };
                                                             Some(((::types::FunctionLevelScope{scope: (*scope).clone(), nearest: (*scope).clone(), file: (*file).clone(), id: (::types::ast::AnyId::AnyIdFile{file: (*file).clone()})})).into_ddvalue())
                                                         }
                                                         __f},
                                                         next: Box::new(None)
                                                     })
                                     },
                                     /* FunctionLevelScope[(FunctionLevelScope{.scope=from, .nearest=to, .file=file, .id=id}: FunctionLevelScope)] :- FunctionLevelScope[(FunctionLevelScope{.scope=(from: ast::ScopeId), .nearest=(interum: ast::ScopeId), .file=(file: ast::FileId), .id=(id: ast::AnyId)}: FunctionLevelScope)], ChildScope[(ChildScope{.parent=(interum: ast::ScopeId), .child=(to: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                                     Rule::ArrangementRule {
                                         description: "FunctionLevelScope[(FunctionLevelScope{.scope=from, .nearest=to, .file=file, .id=id}: FunctionLevelScope)] :- FunctionLevelScope[(FunctionLevelScope{.scope=(from: ast::ScopeId), .nearest=(interum: ast::ScopeId), .file=(file: ast::FileId), .id=(id: ast::AnyId)}: FunctionLevelScope)], ChildScope[(ChildScope{.parent=(interum: ast::ScopeId), .child=(to: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                         arr: ( Relations::FunctionLevelScope as RelId, 0),
                                         xform: XFormArrangement::Join{
                                                    description: "FunctionLevelScope[(FunctionLevelScope{.scope=(from: ast::ScopeId), .nearest=(interum: ast::ScopeId), .file=(file: ast::FileId), .id=(id: ast::AnyId)}: FunctionLevelScope)], ChildScope[(ChildScope{.parent=(interum: ast::ScopeId), .child=(to: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                                    ffun: None,
                                                    arrangement: (Relations::ChildScope as RelId,0),
                                                    jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                    {
                                                        let (ref from, ref interum, ref file, ref id) = match *unsafe {<::types::FunctionLevelScope>::from_ddvalue_ref(__v1) } {
                                                            ::types::FunctionLevelScope{scope: ref from, nearest: ref interum, file: ref file, id: ref id} => ((*from).clone(), (*interum).clone(), (*file).clone(), (*id).clone()),
                                                            _ => return None
                                                        };
                                                        let ref to = match *unsafe {<::types::ChildScope>::from_ddvalue_ref(__v2) } {
                                                            ::types::ChildScope{parent: _, child: ref to, file: _} => (*to).clone(),
                                                            _ => return None
                                                        };
                                                        Some(((::types::FunctionLevelScope{scope: (*from).clone(), nearest: (*to).clone(), file: (*file).clone(), id: (*id).clone()})).into_ddvalue())
                                                    }
                                                    __f},
                                                    next: Box::new(None)
                                                }
                                     }],
                                 arrangements: vec![
                                     Arrangement::Map{
                                        name: r###"(FunctionLevelScope{.scope=(_: ast::ScopeId), .nearest=(_0: ast::ScopeId), .file=(_1: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope) /*join*/"###.to_string(),
                                         afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                         {
                                             let __cloned = __v.clone();
                                             match unsafe {< ::types::FunctionLevelScope>::from_ddvalue(__v) } {
                                                 ::types::FunctionLevelScope{scope: _, nearest: ref _0, file: ref _1, id: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                 _ => None
                                             }.map(|x|(x,__cloned))
                                         }
                                         __f},
                                         queryable: false
                                     }],
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
                                 Arrangement::Map{
                                    name: r###"(inputs::LetDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::LetDecl>::from_ddvalue(__v) } {
                                             ::types::inputs::LetDecl{stmt_id: ref _0, file: ref _1, pattern: ::types::ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 },
                                 Arrangement::Map{
                                    name: r###"(inputs::LetDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::LetDecl>::from_ddvalue(__v) } {
                                             ::types::inputs::LetDecl{stmt_id: ref _0, file: ref _1, pattern: _, value: ::types::ddlog_std::Option::Some{x: _}, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 }],
                             change_cb:    None
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
                                    name: r###"(inputs::NameRef{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .value=(_: internment::Intern<string>)}: inputs::NameRef) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::NameRef>::from_ddvalue(__v) } {
                                             ::types::inputs::NameRef{expr_id: ref _0, file: ref _1, value: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 },
                                 Arrangement::Map{
                                    name: r###"(inputs::NameRef{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .value=(_: internment::Intern<string>)}: inputs::NameRef) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::NameRef>::from_ddvalue(__v) } {
                                             ::types::inputs::NameRef{expr_id: _, file: ref _0, value: _} => Some(((*_0).clone()).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 },
                                 Arrangement::Map{
                                    name: r###"(inputs::NameRef{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .value=(_1: internment::Intern<string>)}: inputs::NameRef) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::NameRef>::from_ddvalue(__v) } {
                                             ::types::inputs::NameRef{expr_id: _, file: ref _0, value: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 }],
                             change_cb:    None
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
                             Arrangement::Map{
                                name: r###"(inputs::New{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .object=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::inputs::New>::from_ddvalue(__v) } {
                                         ::types::inputs::New{expr_id: ref _0, file: ref _1, object: ::types::ddlog_std::Option::Some{x: _}, args: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let __Prefix_1 = Relation {
                         name:         "__Prefix_1".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::__Prefix_1 as RelId,
                         rules:        vec![
                             /* __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))] :- inputs::New[(inputs::New{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)]. */
                             Rule::ArrangementRule {
                                 description: "__Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))] :- inputs::New[(inputs::New{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)].".to_string(),
                                 arr: ( Relations::inputs_New as RelId, 0),
                                 xform: XFormArrangement::Join{
                                            description: "inputs::New[(inputs::New{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .object=(ddlog_std::Some{.x=(object: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)]".to_string(),
                                            ffun: None,
                                            arrangement: (Relations::inputs_Expression as RelId,0),
                                            jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                            {
                                                let (ref expr, ref file, ref object) = match *unsafe {<::types::inputs::New>::from_ddvalue_ref(__v1) } {
                                                    ::types::inputs::New{expr_id: ref expr, file: ref file, object: ::types::ddlog_std::Option::Some{x: ref object}, args: _} => ((*expr).clone(), (*file).clone(), (*object).clone()),
                                                    _ => return None
                                                };
                                                let (ref used_scope, ref used_in) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                    ::types::inputs::Expression{id: _, file: _, kind: _, scope: ref used_scope, span: ref used_in} => ((*used_scope).clone(), (*used_in).clone()),
                                                    _ => return None
                                                };
                                                Some((::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue())
                                            }
                                            __f},
                                            next: Box::new(None)
                                        }
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"((_: ast::ExprId), (_0: ast::FileId), (_: ast::ExprId), (_: ast::ScopeId), (_: ast::Span)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue(__v) } {
                                         ::types::ddlog_std::tuple5(_, ref _0, _, _, _) => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             },
                             Arrangement::Map{
                                name: r###"((_: ast::ExprId), (_1: ast::FileId), (_0: ast::ExprId), (_: ast::ScopeId), (_: ast::Span)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue(__v) } {
                                         ::types::ddlog_std::tuple5(_, ref _1, ref _0, _, _) => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
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
                                      name: r###"(inputs::Statement{.id=(_0: ast::StmtId), .file=(_1: ast::FileId), .kind=(_: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::inputs::Statement>::from_ddvalue(__v) } {
                                               ::types::inputs::Statement{id: ref _0, file: ref _1, kind: _, scope: _, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   },
                                   Arrangement::Map{
                                      name: r###"(inputs::Statement{.id=(_: ast::StmtId), .file=(_0: ast::FileId), .kind=(_: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::inputs::Statement>::from_ddvalue(__v) } {
                                               ::types::inputs::Statement{id: _, file: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   },
                                   Arrangement::Map{
                                      name: r###"(inputs::Statement{.id=(_0: ast::StmtId), .file=(_1: ast::FileId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::inputs::Statement>::from_ddvalue(__v) } {
                                               ::types::inputs::Statement{id: ref _0, file: ref _1, kind: ::types::ast::StmtKind::StmtVarDecl{}, scope: _, span: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
                                   }],
                               change_cb:    None
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
                             Arrangement::Map{
                                name: r###"(inputs::Try{.stmt_id=(_: ast::StmtId), .file=(_0: ast::FileId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(_: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::inputs::Try>::from_ddvalue(__v) } {
                                         ::types::inputs::Try{stmt_id: _, file: ref _0, body: _, handler: ::types::ast::TryHandler{error: ::types::ddlog_std::Option::Some{x: _}, body: ::types::ddlog_std::Option::Some{x: _}}, finalizer: _} => Some(((*_0).clone()).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
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
    let WithinTypeofExpr = Relation {
                               name:         "WithinTypeofExpr".to_string(),
                               input:        false,
                               distinct:     false,
                               caching_mode: CachingMode::Set,
                               key_func:     None,
                               id:           Relations::WithinTypeofExpr as RelId,
                               rules:        vec![
                                   /* WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=expr, .file=file}: WithinTypeofExpr)] :- inputs::UnaryOp[(inputs::UnaryOp{.expr_id=(type_of: ast::ExprId), .file=(file: ast::FileId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp)]. */
                                   Rule::CollectionRule {
                                       description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=expr, .file=file}: WithinTypeofExpr)] :- inputs::UnaryOp[(inputs::UnaryOp{.expr_id=(type_of: ast::ExprId), .file=(file: ast::FileId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp)].".to_string(),
                                       rel: Relations::inputs_UnaryOp as RelId,
                                       xform: Some(XFormCollection::FilterMap{
                                                       description: "head of WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=expr, .file=file}: WithinTypeofExpr)] :- inputs::UnaryOp[(inputs::UnaryOp{.expr_id=(type_of: ast::ExprId), .file=(file: ast::FileId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(expr: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp)]." .to_string(),
                                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                       {
                                                           let (ref type_of, ref file, ref expr) = match *unsafe {<::types::inputs::UnaryOp>::from_ddvalue_ref(&__v) } {
                                                               ::types::inputs::UnaryOp{expr_id: ref type_of, file: ref file, op: ::types::ddlog_std::Option::Some{x: ::types::ast::UnaryOperand::UnaryTypeof{}}, expr: ::types::ddlog_std::Option::Some{x: ref expr}} => ((*type_of).clone(), (*file).clone(), (*expr).clone()),
                                                               _ => return None
                                                           };
                                                           Some(((::types::WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*expr).clone(), file: (*file).clone()})).into_ddvalue())
                                                       }
                                                       __f},
                                                       next: Box::new(None)
                                                   })
                                   },
                                   /* WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=grouped, .file=file}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]. */
                                   Rule::ArrangementRule {
                                       description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=grouped, .file=file}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)].".to_string(),
                                       arr: ( Relations::WithinTypeofExpr as RelId, 1),
                                       xform: XFormArrangement::Join{
                                                  description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(grouped: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                  ffun: None,
                                                  arrangement: (Relations::inputs_Expression as RelId,3),
                                                  jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref type_of, ref expr, ref file) = match *unsafe {<::types::WithinTypeofExpr>::from_ddvalue_ref(__v1) } {
                                                          ::types::WithinTypeofExpr{type_of: ref type_of, expr: ref expr, file: ref file} => ((*type_of).clone(), (*expr).clone(), (*file).clone()),
                                                          _ => return None
                                                      };
                                                      let ref grouped = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                          ::types::inputs::Expression{id: _, file: _, kind: ::types::ast::ExprKind::ExprGrouping{inner: ::types::ddlog_std::Option::Some{x: ref grouped}}, scope: _, span: _} => (*grouped).clone(),
                                                          _ => return None
                                                      };
                                                      Some(((::types::WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*grouped).clone(), file: (*file).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              }
                                   },
                                   /* WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=last, .file=file}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprSequence{.exprs=(sequence: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], ((ddlog_std::Some{.x=(var last: ast::ExprId)}: ddlog_std::Option<ast::ExprId>) = ((vec::last: function(ddlog_std::Vec<ast::ExprId>):ddlog_std::Option<ast::ExprId>)(sequence))). */
                                   Rule::ArrangementRule {
                                       description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=type_of, .expr=last, .file=file}: WithinTypeofExpr)] :- WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprSequence{.exprs=(sequence: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], ((ddlog_std::Some{.x=(var last: ast::ExprId)}: ddlog_std::Option<ast::ExprId>) = ((vec::last: function(ddlog_std::Vec<ast::ExprId>):ddlog_std::Option<ast::ExprId>)(sequence))).".to_string(),
                                       arr: ( Relations::WithinTypeofExpr as RelId, 1),
                                       xform: XFormArrangement::Join{
                                                  description: "WithinTypeofExpr[(WithinTypeofExpr{.type_of=(type_of: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprSequence{.exprs=(sequence: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                  ffun: None,
                                                  arrangement: (Relations::inputs_Expression as RelId,4),
                                                  jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref type_of, ref expr, ref file) = match *unsafe {<::types::WithinTypeofExpr>::from_ddvalue_ref(__v1) } {
                                                          ::types::WithinTypeofExpr{type_of: ref type_of, expr: ref expr, file: ref file} => ((*type_of).clone(), (*expr).clone(), (*file).clone()),
                                                          _ => return None
                                                      };
                                                      let ref sequence = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                          ::types::inputs::Expression{id: _, file: _, kind: ::types::ast::ExprKind::ExprSequence{exprs: ref sequence}, scope: _, span: _} => (*sequence).clone(),
                                                          _ => return None
                                                      };
                                                      let ref last: ::types::ast::ExprId = match ::types::vec::last::<::types::ast::ExprId>(sequence) {
                                                          ::types::ddlog_std::Option::Some{x: last} => last,
                                                          _ => return None
                                                      };
                                                      Some(((::types::WithinTypeofExpr{type_of: (*type_of).clone(), expr: (*last).clone(), file: (*file).clone()})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              }
                                   }],
                               arrangements: vec![
                                   Arrangement::Set{
                                       name: r###"(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(_0: ast::ExprId), .file=(_1: ast::FileId)}: WithinTypeofExpr) /*antijoin*/"###.to_string(),
                                       fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                       {
                                           match unsafe {< ::types::WithinTypeofExpr>::from_ddvalue(__v) } {
                                               ::types::WithinTypeofExpr{type_of: _, expr: ref _0, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                               _ => None
                                           }
                                       }
                                       __f},
                                       distinct: true
                                   },
                                   Arrangement::Map{
                                      name: r###"(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(_0: ast::ExprId), .file=(_1: ast::FileId)}: WithinTypeofExpr) /*join*/"###.to_string(),
                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                       {
                                           let __cloned = __v.clone();
                                           match unsafe {< ::types::WithinTypeofExpr>::from_ddvalue(__v) } {
                                               ::types::WithinTypeofExpr{type_of: _, expr: ref _0, file: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                               _ => None
                                           }.map(|x|(x,__cloned))
                                       }
                                       __f},
                                       queryable: false
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
                                 Arrangement::Map{
                                    name: r###"(inputs::VarDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::VarDecl>::from_ddvalue(__v) } {
                                             ::types::inputs::VarDecl{stmt_id: ref _0, file: ref _1, pattern: ::types::ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                             _ => None
                                         }.map(|x|(x,__cloned))
                                     }
                                     __f},
                                     queryable: false
                                 },
                                 Arrangement::Map{
                                    name: r###"(inputs::VarDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl) /*join*/"###.to_string(),
                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                     {
                                         let __cloned = __v.clone();
                                         match unsafe {< ::types::inputs::VarDecl>::from_ddvalue(__v) } {
                                             ::types::inputs::VarDecl{stmt_id: ref _0, file: ref _1, pattern: _, value: ::types::ddlog_std::Option::Some{x: _}, exported: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
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
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::None{}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdGlobal{.global=global}: ast::AnyId), .implicit=true}: NameInScope)] :- inputs::ImplicitGlobal[(inputs::ImplicitGlobal{.id=(global: ast::GlobalId), .file=(file: ast::FileId), .name=(name: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::ImplicitGlobal)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::None{}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdGlobal{.global=global}: ast::AnyId), .implicit=true}: NameInScope)] :- inputs::ImplicitGlobal[(inputs::ImplicitGlobal{.id=(global: ast::GlobalId), .file=(file: ast::FileId), .name=(name: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::ImplicitGlobal)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)].".to_string(),
                                  arr: ( Relations::inputs_ImplicitGlobal as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::ImplicitGlobal[(inputs::ImplicitGlobal{.id=(global: ast::GlobalId), .file=(file: ast::FileId), .name=(name: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::ImplicitGlobal)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_EveryScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref global, ref file, ref name) = match *unsafe {<::types::inputs::ImplicitGlobal>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::ImplicitGlobal{id: ref global, file: ref file, name: ref name, privileges: _} => ((*global).clone(), (*file).clone(), (*name).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::EveryScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::EveryScope{scope: ref scope, file: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::None{}), declared_in: (::types::ast::AnyId::AnyIdGlobal{global: (*global).clone()}), implicit: true})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdImport{.import_=id}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .file=(file: ast::FileId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)], var free_var = FlatMap((ast::free_variables(clause))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = free_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdImport{.import_=id}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .file=(file: ast::FileId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)], var free_var = FlatMap((ast::free_variables(clause))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = free_var).".to_string(),
                                  arr: ( Relations::inputs_ImportDecl as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .file=(file: ast::FileId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_EveryScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref id, ref file, ref clause) = match *unsafe {<::types::inputs::ImportDecl>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::ImportDecl{id: ref id, file: ref file, clause: ref clause} => ((*id).clone(), (*file).clone(), (*clause).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::EveryScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::EveryScope{scope: ref scope, file: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*id).clone(), (*file).clone(), (*clause).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::FlatMap{
                                                                     description: "inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .file=(file: ast::FileId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)], var free_var = FlatMap((ast::free_variables(clause)))" .to_string(),
                                                                     fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref id, ref file, ref clause, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ImportId, ::types::ast::FileId, ::types::ast::ImportClause, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                         let __flattened = ::types::ast::free_variables(clause);
                                                                         let id = (*id).clone();
                                                                         let file = (*file).clone();
                                                                         let scope = (*scope).clone();
                                                                         Some(Box::new(__flattened.into_iter().map(move |free_var|(::types::ddlog_std::tuple4(free_var.clone(), id.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(Some(XFormCollection::FilterMap{
                                                                                             description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdImport{.import_=id}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ImportDecl[(inputs::ImportDecl{.id=(id: ast::ImportId), .file=(file: ast::FileId), .clause=(clause: ast::ImportClause)}: inputs::ImportDecl)], inputs::EveryScope[(inputs::EveryScope{.scope=(scope: ast::ScopeId), .file=(file: ast::FileId)}: inputs::EveryScope)], var free_var = FlatMap((ast::free_variables(clause))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = free_var)." .to_string(),
                                                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple4(ref free_var, ref id, ref file, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ImportId, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                 let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*free_var).clone() {
                                                                                                     ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdImport{import_: (*id).clone()}), implicit: false})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         }))
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdClass{.class=class}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)]. */
                              Rule::CollectionRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdClass{.class=class}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)].".to_string(),
                                  rel: Relations::inputs_Class as RelId,
                                  xform: Some(XFormCollection::FilterMap{
                                                  description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdClass{.class=class}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)]." .to_string(),
                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                  {
                                                      let (ref class, ref file, ref name, ref span, ref scope) = match *unsafe {<::types::inputs::Class>::from_ddvalue_ref(&__v) } {
                                                          ::types::inputs::Class{id: ref class, file: ref file, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: ref name, span: ref span}}, parent: _, elements: _, scope: ref scope, exported: _} => ((*class).clone(), (*file).clone(), (*name).clone(), (*span).clone(), (*scope).clone()),
                                                          _ => return None
                                                      };
                                                      Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdClass{class: (*class).clone()}), implicit: false})).into_ddvalue())
                                                  }
                                                  __f},
                                                  next: Box::new(None)
                                              })
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_LetDecl as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref stmt, ref file, ref pat) = match *unsafe {<::types::inputs::LetDecl>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::LetDecl{stmt_id: ref stmt, file: ref file, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _, exported: _} => ((*stmt).clone(), (*file).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*stmt).clone(), (*file).clone(), (*pat).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::FlatMap{
                                                                     description: "inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                     fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref stmt, ref file, ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::StmtId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                         let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                         let stmt = (*stmt).clone();
                                                                         let file = (*file).clone();
                                                                         let scope = (*scope).clone();
                                                                         Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple4(bound_var.clone(), stmt.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(Some(XFormCollection::FilterMap{
                                                                                             description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple4(ref bound_var, ref stmt, ref file, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                 let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                     ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*stmt).clone()}), implicit: false})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         }))
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_ConstDecl as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref stmt, ref file, ref pat) = match *unsafe {<::types::inputs::ConstDecl>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::ConstDecl{stmt_id: ref stmt, file: ref file, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _, exported: _} => ((*stmt).clone(), (*file).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*stmt).clone(), (*file).clone(), (*pat).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::FlatMap{
                                                                     description: "inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                     fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref stmt, ref file, ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::StmtId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                         let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                         let stmt = (*stmt).clone();
                                                                         let file = (*file).clone();
                                                                         let scope = (*scope).clone();
                                                                         Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple4(bound_var.clone(), stmt.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(Some(XFormCollection::FilterMap{
                                                                                             description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple4(ref bound_var, ref stmt, ref file, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                 let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                     ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*stmt).clone()}), implicit: false})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         }))
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=nearest, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=nearest, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_VarDecl as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref stmt, ref file, ref pat) = match *unsafe {<::types::inputs::VarDecl>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::VarDecl{stmt_id: ref stmt, file: ref file, pattern: ::types::ddlog_std::Option::Some{x: ref pat}, value: _, exported: _} => ((*stmt).clone(), (*file).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*stmt).clone(), (*file).clone(), (*pat).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)] by (scope, file)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref stmt, ref file, ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::StmtId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                         Some(((::types::ddlog_std::tuple2((*scope).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*stmt).clone(), (*file).clone(), (*pat).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::FunctionLevelScope as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple3(ref stmt, ref file, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::StmtId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref nearest = match *unsafe {<::types::FunctionLevelScope>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::FunctionLevelScope{scope: ref nearest, nearest: _, file: _, id: _} => (*nearest).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some((::types::ddlog_std::tuple4((*stmt).clone(), (*file).clone(), (*pat).clone(), (*nearest).clone())).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(Some(XFormCollection::FlatMap{
                                                                                                                description: "inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                                                                fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple4(ref stmt, ref file, ref pat, ref nearest) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::StmtId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                    let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                                                                    let stmt = (*stmt).clone();
                                                                                                                    let file = (*file).clone();
                                                                                                                    let nearest = (*nearest).clone();
                                                                                                                    Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple4(bound_var.clone(), stmt.clone(), file.clone(), nearest.clone())).into_ddvalue())))
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                        description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=nearest, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=stmt}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .pattern=(ddlog_std::Some{.x=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let ::types::ddlog_std::tuple4(ref bound_var, ref stmt, ref file, ref nearest) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::StmtId, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                            let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                                                                ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*nearest).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*stmt).clone()}), implicit: false})).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(None)
                                                                                                                                    }))
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)].".to_string(),
                                  arr: ( Relations::inputs_Function as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], FunctionLevelScope[(FunctionLevelScope{.scope=(nearest: ast::ScopeId), .nearest=(scope: ast::ScopeId), .file=(file: ast::FileId), .id=(_: ast::AnyId)}: FunctionLevelScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::FunctionLevelScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref func, ref file, ref name, ref span, ref scope) = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::Function{id: ref func, file: ref file, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: ref name, span: ref span}}, scope: ref scope, body: _, exported: _} => ((*func).clone(), (*file).clone(), (*name).clone(), (*span).clone(), (*scope).clone()),
                                                     _ => return None
                                                 };
                                                 let ref nearest = match *unsafe {<::types::FunctionLevelScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::FunctionLevelScope{scope: ref nearest, nearest: _, file: _, id: _} => (*nearest).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdFunc{func: (*func).clone()}), implicit: false})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=body, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId), .implicit=implicit}: NameInScope)] :- inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .file=(file: ast::FileId), .pattern=(pat: internment::Intern<ast::Pattern>), .implicit=(implicit: bool)}: inputs::FunctionArg)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=body, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId), .implicit=implicit}: NameInScope)] :- inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .file=(file: ast::FileId), .pattern=(pat: internment::Intern<ast::Pattern>), .implicit=(implicit: bool)}: inputs::FunctionArg)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_FunctionArg as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .file=(file: ast::FileId), .pattern=(pat: internment::Intern<ast::Pattern>), .implicit=(implicit: bool)}: inputs::FunctionArg)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Function as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref func, ref file, ref pat, ref implicit) = match *unsafe {<::types::inputs::FunctionArg>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::FunctionArg{parent_func: ref func, file: ref file, pattern: ref pat, implicit: ref implicit} => ((*func).clone(), (*file).clone(), (*pat).clone(), (*implicit).clone()),
                                                     _ => return None
                                                 };
                                                 let ref body = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Function{id: _, file: _, name: _, scope: _, body: ref body, exported: _} => (*body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple5((*func).clone(), (*file).clone(), (*pat).clone(), (*implicit).clone(), (*body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::FlatMap{
                                                                     description: "inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .file=(file: ast::FileId), .pattern=(pat: internment::Intern<ast::Pattern>), .implicit=(implicit: bool)}: inputs::FunctionArg)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                     fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                     {
                                                                         let ::types::ddlog_std::tuple5(ref func, ref file, ref pat, ref implicit, ref body) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::FuncId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, bool, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                         let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                         let func = (*func).clone();
                                                                         let file = (*file).clone();
                                                                         let implicit = (*implicit).clone();
                                                                         let body = (*body).clone();
                                                                         Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple5(bound_var.clone(), func.clone(), file.clone(), implicit.clone(), body.clone())).into_ddvalue())))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(Some(XFormCollection::FilterMap{
                                                                                             description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=body, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=func}: ast::AnyId), .implicit=implicit}: NameInScope)] :- inputs::FunctionArg[(inputs::FunctionArg{.parent_func=(func: ast::FuncId), .file=(file: ast::FileId), .pattern=(pat: internment::Intern<ast::Pattern>), .implicit=(implicit: bool)}: inputs::FunctionArg)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(body: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple5(ref bound_var, ref func, ref file, ref implicit, ref body) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::FuncId, ::types::ast::FileId, bool, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                 let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                     ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*body).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdFunc{func: (*func).clone()}), implicit: (*implicit).clone()})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         }))
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(body: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(body: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_ArrowParam as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Arrow as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref expr, ref file, ref pat) = match *unsafe {<::types::inputs::ArrowParam>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::ArrowParam{expr_id: ref expr, file: ref file, param: ref pat} => ((*expr).clone(), (*file).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref body = match *unsafe {<::types::inputs::Arrow>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Arrow{expr_id: _, file: _, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: ref body}}} => (*body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*expr).clone(), (*file).clone(), (*pat).clone(), (*body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)] by (body, file)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref expr, ref file, ref pat, ref body) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                         Some(((::types::ddlog_std::tuple2((*body).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*expr).clone(), (*file).clone(), (*pat).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(body: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::inputs_Expression as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple3(ref expr, ref file, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref scope = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::inputs::Expression{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some((::types::ddlog_std::tuple4((*expr).clone(), (*file).clone(), (*pat).clone(), (*scope).clone())).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(Some(XFormCollection::FlatMap{
                                                                                                                description: "inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(body: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                                                                fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple4(ref expr, ref file, ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                    let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                                                                    let expr = (*expr).clone();
                                                                                                                    let file = (*file).clone();
                                                                                                                    let scope = (*scope).clone();
                                                                                                                    Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple4(bound_var.clone(), expr.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                        description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(body: ast::ExprId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Expression[(inputs::Expression{.id=(body: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let ::types::ddlog_std::tuple4(ref bound_var, ref expr, ref file, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                            let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                                                                ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()}), implicit: false})).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(None)
                                                                                                                                    }))
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_ArrowParam as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Arrow as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref expr, ref file, ref pat) = match *unsafe {<::types::inputs::ArrowParam>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::ArrowParam{expr_id: ref expr, file: ref file, param: ref pat} => ((*expr).clone(), (*file).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref body = match *unsafe {<::types::inputs::Arrow>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Arrow{expr_id: _, file: _, body: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Right{r: ref body}}} => (*body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*expr).clone(), (*file).clone(), (*pat).clone(), (*body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)] by (body, file)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref expr, ref file, ref pat, ref body) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                         Some(((::types::ddlog_std::tuple2((*body).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*expr).clone(), (*file).clone(), (*pat).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::inputs_Statement as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple3(ref expr, ref file, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some((::types::ddlog_std::tuple4((*expr).clone(), (*file).clone(), (*pat).clone(), (*scope).clone())).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(Some(XFormCollection::FlatMap{
                                                                                                                description: "inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                                                                fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple4(ref expr, ref file, ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                    let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                                                                    let expr = (*expr).clone();
                                                                                                                    let file = (*file).clone();
                                                                                                                    let scope = (*scope).clone();
                                                                                                                    Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple4(bound_var.clone(), expr.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                        description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::ArrowParam[(inputs::ArrowParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::ArrowParam)], inputs::Arrow[(inputs::Arrow{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .body=(ddlog_std::Some{.x=(ddlog_std::Right{.r=(body: ast::StmtId)}: ddlog_std::Either<ast::ExprId,ast::StmtId>)}: ddlog_std::Option<ddlog_std::Either<ast::ExprId,ast::StmtId>>)}: inputs::Arrow)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let ::types::ddlog_std::tuple4(ref bound_var, ref expr, ref file, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                            let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                                                                ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()}), implicit: false})).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(None)
                                                                                                                                    }))
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)].".to_string(),
                                  arr: ( Relations::inputs_InlineFunc as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(name: internment::Intern<string>), .span=(span: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Statement as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref expr, ref file, ref name, ref span, ref body) = match *unsafe {<::types::inputs::InlineFunc>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::InlineFunc{expr_id: ref expr, file: ref file, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: ref name, span: ref span}}, body: ::types::ddlog_std::Option::Some{x: ref body}} => ((*expr).clone(), (*file).clone(), (*name).clone(), (*span).clone(), (*body).clone()),
                                                     _ => return None
                                                 };
                                                 let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()}), implicit: false})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_InlineFuncParam as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_InlineFunc as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref expr, ref file, ref pat) = match *unsafe {<::types::inputs::InlineFuncParam>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::InlineFuncParam{expr_id: ref expr, file: ref file, param: ref pat} => ((*expr).clone(), (*file).clone(), (*pat).clone()),
                                                     _ => return None
                                                 };
                                                 let ref body = match *unsafe {<::types::inputs::InlineFunc>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::InlineFunc{expr_id: _, file: _, name: _, body: ::types::ddlog_std::Option::Some{x: ref body}} => (*body).clone(),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*expr).clone(), (*file).clone(), (*pat).clone(), (*body).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)] by (body, file)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref expr, ref file, ref pat, ref body) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                         Some(((::types::ddlog_std::tuple2((*body).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple3((*expr).clone(), (*file).clone(), (*pat).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Join{
                                                                                        description: "inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::inputs_Statement as RelId,0),
                                                                                        jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                        {
                                                                                            let ::types::ddlog_std::tuple3(ref expr, ref file, ref pat) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>>>::from_ddvalue_ref( __v1 ) };
                                                                                            let ref scope = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                ::types::inputs::Statement{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                                                                _ => return None
                                                                                            };
                                                                                            Some((::types::ddlog_std::tuple4((*expr).clone(), (*file).clone(), (*pat).clone(), (*scope).clone())).into_ddvalue())
                                                                                        }
                                                                                        __f},
                                                                                        next: Box::new(Some(XFormCollection::FlatMap{
                                                                                                                description: "inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                                                                fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple4(ref expr, ref file, ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                    let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                                                                    let expr = (*expr).clone();
                                                                                                                    let file = (*file).clone();
                                                                                                                    let scope = (*scope).clone();
                                                                                                                    Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple4(bound_var.clone(), expr.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                        description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdExpr{.expr=expr}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::InlineFuncParam[(inputs::InlineFuncParam{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .param=(pat: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam)], inputs::InlineFunc[(inputs::InlineFunc{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(body: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc)], inputs::Statement[(inputs::Statement{.id=(body: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                                                                        fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                        {
                                                                                                                                            let ::types::ddlog_std::tuple4(ref bound_var, ref expr, ref file, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                            let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                                                                ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                                                                _ => return None
                                                                                                                                            };
                                                                                                                                            Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdExpr{expr: (*expr).clone()}), implicit: false})).into_ddvalue())
                                                                                                                                        }
                                                                                                                                        __f},
                                                                                                                                        next: Box::new(None)
                                                                                                                                    }))
                                                                                                            }))
                                                                                    })
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=catcher}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Try[(inputs::Try{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(error: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(catcher: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try)], inputs::Statement[(inputs::Statement{.id=(expr: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(error))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var). */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=catcher}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Try[(inputs::Try{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(error: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(catcher: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try)], inputs::Statement[(inputs::Statement{.id=(expr: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(error))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var).".to_string(),
                                  arr: ( Relations::inputs_Try as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::Try[(inputs::Try{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(error: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(catcher: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try)], inputs::Statement[(inputs::Statement{.id=(expr: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Statement as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref stmt, ref file, ref error, ref catcher) = match *unsafe {<::types::inputs::Try>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::Try{stmt_id: ref stmt, file: ref file, body: _, handler: ::types::ast::TryHandler{error: ::types::ddlog_std::Option::Some{x: ref error}, body: ::types::ddlog_std::Option::Some{x: ref catcher}}, finalizer: _} => ((*stmt).clone(), (*file).clone(), (*error).clone(), (*catcher).clone()),
                                                     _ => return None
                                                 };
                                                 let (ref expr, ref scope) = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Statement{id: ref expr, file: _, kind: _, scope: ref scope, span: _} => ((*expr).clone(), (*scope).clone()),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*file).clone(), (*error).clone(), (*catcher).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::FlatMap{
                                                                     description: "inputs::Try[(inputs::Try{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(error: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(catcher: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try)], inputs::Statement[(inputs::Statement{.id=(expr: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(error)))" .to_string(),
                                                                     fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref file, ref error, ref catcher, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::StmtId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                         let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(error);
                                                                         let file = (*file).clone();
                                                                         let catcher = (*catcher).clone();
                                                                         let scope = (*scope).clone();
                                                                         Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple4(bound_var.clone(), file.clone(), catcher.clone(), scope.clone())).into_ddvalue())))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(Some(XFormCollection::FilterMap{
                                                                                             description: "head of NameInScope[(NameInScope{.file=file, .name=name, .scope=scope, .span=(ddlog_std::Some{.x=span}: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=catcher}: ast::AnyId), .implicit=false}: NameInScope)] :- inputs::Try[(inputs::Try{.stmt_id=(stmt: ast::StmtId), .file=(file: ast::FileId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(error: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(catcher: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try)], inputs::Statement[(inputs::Statement{.id=(expr: ast::StmtId), .file=(file: ast::FileId), .kind=(_: ast::StmtKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(error))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var)." .to_string(),
                                                                                             fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                             {
                                                                                                 let ::types::ddlog_std::tuple4(ref bound_var, ref file, ref catcher, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::FileId, ::types::ast::StmtId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                 let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                     ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                     _ => return None
                                                                                                 };
                                                                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), span: (::types::ddlog_std::Option::Some{x: (*span).clone()}), declared_in: (::types::ast::AnyId::AnyIdStmt{stmt: (*catcher).clone()}), implicit: false})).into_ddvalue())
                                                                                             }
                                                                                             __f},
                                                                                             next: Box::new(None)
                                                                                         }))
                                                                 }))
                                         }
                              },
                              /* NameInScope[(NameInScope{.file=file, .name=name, .scope=to, .span=span, .declared_in=declared_in, .implicit=implicit}: NameInScope)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(from: ast::ScopeId), .span=(span: ddlog_std::Option<ast::Span>), .declared_in=(declared_in: ast::AnyId), .implicit=(implicit: bool)}: NameInScope)], ChildScope[(ChildScope{.parent=(from: ast::ScopeId), .child=(to: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                              Rule::ArrangementRule {
                                  description: "NameInScope[(NameInScope{.file=file, .name=name, .scope=to, .span=span, .declared_in=declared_in, .implicit=implicit}: NameInScope)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(from: ast::ScopeId), .span=(span: ddlog_std::Option<ast::Span>), .declared_in=(declared_in: ast::AnyId), .implicit=(implicit: bool)}: NameInScope)], ChildScope[(ChildScope{.parent=(from: ast::ScopeId), .child=(to: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                  arr: ( Relations::NameInScope as RelId, 1),
                                  xform: XFormArrangement::Join{
                                             description: "NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(from: ast::ScopeId), .span=(span: ddlog_std::Option<ast::Span>), .declared_in=(declared_in: ast::AnyId), .implicit=(implicit: bool)}: NameInScope)], ChildScope[(ChildScope{.parent=(from: ast::ScopeId), .child=(to: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::ChildScope as RelId,0),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref file, ref name, ref from, ref span, ref declared_in, ref implicit) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v1) } {
                                                     ::types::NameInScope{file: ref file, name: ref name, scope: ref from, span: ref span, declared_in: ref declared_in, implicit: ref implicit} => ((*file).clone(), (*name).clone(), (*from).clone(), (*span).clone(), (*declared_in).clone(), (*implicit).clone()),
                                                     _ => return None
                                                 };
                                                 let ref to = match *unsafe {<::types::ChildScope>::from_ddvalue_ref(__v2) } {
                                                     ::types::ChildScope{parent: _, child: ref to, file: _} => (*to).clone(),
                                                     _ => return None
                                                 };
                                                 Some(((::types::NameInScope{file: (*file).clone(), name: (*name).clone(), scope: (*to).clone(), span: (*span).clone(), declared_in: (*declared_in).clone(), implicit: (*implicit).clone()})).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(None)
                                         }
                              }],
                          arrangements: vec![
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=false}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: ref _1, scope: ref _2, span: _, declared_in: _, implicit: false} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_1: ast::FileId), .name=(_: internment::Intern<string>), .scope=(_0: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _1, name: _, scope: ref _0, span: _, declared_in: _, implicit: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Set{
                                  name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*antijoin*/"###.to_string(),
                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                  {
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: ref _1, scope: ref _2, span: _, declared_in: _, implicit: _} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                          _ => None
                                      }
                                  }
                                  __f},
                                  distinct: true
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(_: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(_1: ast::AnyId), .implicit=false}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: _, scope: _, span: ::types::ddlog_std::Option::Some{x: _}, declared_in: ref _1, implicit: false} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(_: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=_1, .implicit=false}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: _, scope: _, span: ::types::ddlog_std::Option::Some{x: _}, declared_in: ref _1, implicit: false} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdStmt{.stmt=(_: ast::StmtId)}: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: ref _1, scope: ref _2, span: _, declared_in: ::types::ast::AnyId::AnyIdStmt{stmt: _}, implicit: _} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdClass{.class=(_: ast::ClassId)}: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: ref _1, scope: ref _2, span: _, declared_in: ::types::ast::AnyId::AnyIdClass{class: _}, implicit: _} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_2: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(ast::AnyIdFunc{.func=(_: ast::FuncId)}: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: ref _1, scope: ref _2, span: _, declared_in: ::types::ast::AnyId::AnyIdFunc{func: _}, implicit: _} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: ref _1, scope: _, span: _, declared_in: _, implicit: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: false
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=_0, .name=_2, .scope=_1, .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: ref _2, scope: ref _1, span: _, declared_in: _, implicit: _} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: true
                              },
                              Arrangement::Map{
                                 name: r###"(NameInScope{.file=_0, .name=(_: internment::Intern<string>), .scope=_1, .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope) /*join*/"###.to_string(),
                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                  {
                                      let __cloned = __v.clone();
                                      match unsafe {< ::types::NameInScope>::from_ddvalue(__v) } {
                                          ::types::NameInScope{file: ref _0, name: _, scope: ref _1, span: _, declared_in: _, implicit: _} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                          _ => None
                                      }.map(|x|(x,__cloned))
                                  }
                                  __f},
                                  queryable: true
                              }],
                          change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                      };
    let NoUndef = Relation {
                      name:         "NoUndef".to_string(),
                      input:        false,
                      distinct:     true,
                      caching_mode: CachingMode::Set,
                      key_func:     None,
                      id:           Relations::NoUndef as RelId,
                      rules:        vec![
                          /* NoUndef[(NoUndef{.name=name, .scope=scope, .span=span, .file=file}: NoUndef)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)]. */
                          Rule::ArrangementRule {
                              description: "NoUndef[(NoUndef{.name=name, .scope=scope, .span=span, .file=file}: NoUndef)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)].".to_string(),
                              arr: ( Relations::inputs_NameRef as RelId, 0),
                              xform: XFormArrangement::Join{
                                         description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)]".to_string(),
                                         ffun: None,
                                         arrangement: (Relations::inputs_Expression as RelId,1),
                                         jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                         {
                                             let (ref expr, ref file, ref name) = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v1) } {
                                                 ::types::inputs::NameRef{expr_id: ref expr, file: ref file, value: ref name} => ((*expr).clone(), (*file).clone(), (*name).clone()),
                                                 _ => return None
                                             };
                                             let (ref scope, ref span) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                 ::types::inputs::Expression{id: _, file: _, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                 _ => return None
                                             };
                                             Some((::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue())
                                         }
                                         __f},
                                         next: Box::new(Some(XFormCollection::Arrange {
                                                                 description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)] by (file, name, scope)" .to_string(),
                                                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                 {
                                                                     let ::types::ddlog_std::tuple5(ref expr, ref file, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                     Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                 }
                                                                 __f},
                                                                 next: Box::new(XFormArrangement::Antijoin {
                                                                                    description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)]".to_string(),
                                                                                    ffun: None,
                                                                                    arrangement: (Relations::NameInScope as RelId,2),
                                                                                    next: Box::new(Some(XFormCollection::Arrange {
                                                                                                            description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)] by (expr, file)" .to_string(),
                                                                                                            afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                            {
                                                                                                                let ::types::ddlog_std::tuple5(ref expr, ref file, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                Some(((::types::ddlog_std::tuple2((*expr).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                                                            }
                                                                                                            __f},
                                                                                                            next: Box::new(XFormArrangement::Antijoin {
                                                                                                                               description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)]".to_string(),
                                                                                                                               ffun: None,
                                                                                                                               arrangement: (Relations::WithinTypeofExpr as RelId,0),
                                                                                                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                       description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)] by (expr, file)" .to_string(),
                                                                                                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                       {
                                                                                                                                                           let ::types::ddlog_std::tuple5(ref expr, ref file, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                           Some(((::types::ddlog_std::tuple2((*expr).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*file).clone(), (*name).clone(), (*scope).clone(), (*span).clone())).into_ddvalue()))
                                                                                                                                                       }
                                                                                                                                                       __f},
                                                                                                                                                       next: Box::new(XFormArrangement::Antijoin {
                                                                                                                                                                          description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)]".to_string(),
                                                                                                                                                                          ffun: None,
                                                                                                                                                                          arrangement: (Relations::ChainedWith as RelId,2),
                                                                                                                                                                          next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                                                                                  description: "head of NoUndef[(NoUndef{.name=name, .scope=scope, .span=span, .file=file}: NoUndef)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], not WithinTypeofExpr[(WithinTypeofExpr{.type_of=(_: ast::ExprId), .expr=(expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)], not ChainedWith[(ChainedWith{.object=(_: ast::ExprId), .property=(expr: ast::ExprId), .file=(file: ast::FileId)}: ChainedWith)]." .to_string(),
                                                                                                                                                                                                  fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                                                                                  {
                                                                                                                                                                                                      let ::types::ddlog_std::tuple4(ref file, ref name, ref scope, ref span) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                                                      Some(((::types::NoUndef{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone(), file: (*file).clone()})).into_ddvalue())
                                                                                                                                                                                                  }
                                                                                                                                                                                                  __f},
                                                                                                                                                                                                  next: Box::new(None)
                                                                                                                                                                                              }))
                                                                                                                                                                      })
                                                                                                                                                   }))
                                                                                                                           })
                                                                                                        }))
                                                                                })
                                                             }))
                                     }
                          },
                          /* NoUndef[(NoUndef{.name=name, .scope=scope, .span=span, .file=file}: NoUndef)] :- inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var), not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)]. */
                          Rule::ArrangementRule {
                              description: "NoUndef[(NoUndef{.name=name, .scope=scope, .span=span, .file=file}: NoUndef)] :- inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var), not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)].".to_string(),
                              arr: ( Relations::inputs_Assign as RelId, 0),
                              xform: XFormArrangement::Join{
                                         description: "inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                         ffun: None,
                                         arrangement: (Relations::inputs_Expression as RelId,0),
                                         jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                         {
                                             let (ref expr, ref file, ref pat) = match *unsafe {<::types::inputs::Assign>::from_ddvalue_ref(__v1) } {
                                                 ::types::inputs::Assign{expr_id: ref expr, file: ref file, lhs: ::types::ddlog_std::Option::Some{x: ::types::ddlog_std::Either::Left{l: ref pat}}, rhs: _, op: _} => ((*expr).clone(), (*file).clone(), (*pat).clone()),
                                                 _ => return None
                                             };
                                             let ref scope = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                 ::types::inputs::Expression{id: _, file: _, kind: _, scope: ref scope, span: _} => (*scope).clone(),
                                                 _ => return None
                                             };
                                             Some((::types::ddlog_std::tuple3((*file).clone(), (*pat).clone(), (*scope).clone())).into_ddvalue())
                                         }
                                         __f},
                                         next: Box::new(Some(XFormCollection::FlatMap{
                                                                 description: "inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat)))" .to_string(),
                                                                 fmfun: &{fn __f(__v: DDValue) -> Option<Box<dyn Iterator<Item=DDValue>>>
                                                                 {
                                                                     let ::types::ddlog_std::tuple3(ref file, ref pat, ref scope) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::FileId, ::types::internment::Intern<::types::ast::Pattern>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                     let __flattened = ::types::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat);
                                                                     let file = (*file).clone();
                                                                     let scope = (*scope).clone();
                                                                     Some(Box::new(__flattened.into_iter().map(move |bound_var|(::types::ddlog_std::tuple3(bound_var.clone(), file.clone(), scope.clone())).into_ddvalue())))
                                                                 }
                                                                 __f},
                                                                 next: Box::new(Some(XFormCollection::Arrange {
                                                                                         description: "arrange inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))) by (file, name, scope)" .to_string(),
                                                                                         afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                         {
                                                                                             let ::types::ddlog_std::tuple3(ref bound_var, ref file, ref scope) = *unsafe {<::types::ddlog_std::tuple3<::types::ast::Spanned<::types::internment::Intern<String>>, ::types::ast::FileId, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                             let (ref name, ref span): (::types::internment::Intern<String>, ::types::ast::Span) = match (*bound_var).clone() {
                                                                                                 ::types::ast::Spanned{data: name, span: span} => (name, span),
                                                                                                 _ => return None
                                                                                             };
                                                                                             Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*file).clone(), (*scope).clone(), (*name).clone(), (*span).clone())).into_ddvalue()))
                                                                                         }
                                                                                         __f},
                                                                                         next: Box::new(XFormArrangement::Antijoin {
                                                                                                            description: "inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var), not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)]".to_string(),
                                                                                                            ffun: None,
                                                                                                            arrangement: (Relations::NameInScope as RelId,2),
                                                                                                            next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                                    description: "head of NoUndef[(NoUndef{.name=name, .scope=scope, .span=span, .file=file}: NoUndef)] :- inputs::Assign[(inputs::Assign{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(pat: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)], var bound_var = FlatMap(((ast::bound_vars: function(internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>)(pat))), ((ast::Spanned{.data=(var name: internment::Intern<string>), .span=(var span: ast::Span)}: ast::Spanned<internment::Intern<string>>) = bound_var), not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)]." .to_string(),
                                                                                                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                    {
                                                                                                                                        let ::types::ddlog_std::tuple4(ref file, ref scope, ref name, ref span) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::FileId, ::types::ast::ScopeId, ::types::internment::Intern<String>, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                        Some(((::types::NoUndef{name: (*name).clone(), scope: (*scope).clone(), span: (*span).clone(), file: (*file).clone()})).into_ddvalue())
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
                          ],
                      change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                  };
    let IsExported = Relation {
                         name:         "IsExported".to_string(),
                         input:        false,
                         distinct:     true,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::IsExported as RelId,
                         rules:        vec![
                             /* IsExported[(IsExported{.file=file, .id=(ast::AnyIdFunc{.func=id}: ast::AnyId)}: IsExported)] :- inputs::Function[(inputs::Function{.id=(id: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=true}: inputs::Function)]. */
                             Rule::CollectionRule {
                                 description: "IsExported[(IsExported{.file=file, .id=(ast::AnyIdFunc{.func=id}: ast::AnyId)}: IsExported)] :- inputs::Function[(inputs::Function{.id=(id: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=true}: inputs::Function)].".to_string(),
                                 rel: Relations::inputs_Function as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of IsExported[(IsExported{.file=file, .id=(ast::AnyIdFunc{.func=id}: ast::AnyId)}: IsExported)] :- inputs::Function[(inputs::Function{.id=(id: ast::FuncId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=true}: inputs::Function)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref id, ref file) = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::Function{id: ref id, file: ref file, name: _, scope: _, body: _, exported: true} => ((*id).clone(), (*file).clone()),
                                                         _ => return None
                                                     };
                                                     Some(((::types::IsExported{file: (*file).clone(), id: (::types::ast::AnyId::AnyIdFunc{func: (*id).clone()})})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* IsExported[(IsExported{.file=file, .id=(ast::AnyIdClass{.class=id}: ast::AnyId)}: IsExported)] :- inputs::Class[(inputs::Class{.id=(id: ast::ClassId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=true}: inputs::Class)]. */
                             Rule::CollectionRule {
                                 description: "IsExported[(IsExported{.file=file, .id=(ast::AnyIdClass{.class=id}: ast::AnyId)}: IsExported)] :- inputs::Class[(inputs::Class{.id=(id: ast::ClassId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=true}: inputs::Class)].".to_string(),
                                 rel: Relations::inputs_Class as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of IsExported[(IsExported{.file=file, .id=(ast::AnyIdClass{.class=id}: ast::AnyId)}: IsExported)] :- inputs::Class[(inputs::Class{.id=(id: ast::ClassId), .file=(file: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=true}: inputs::Class)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref id, ref file) = match *unsafe {<::types::inputs::Class>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::Class{id: ref id, file: ref file, name: _, parent: _, elements: _, scope: _, exported: true} => ((*id).clone(), (*file).clone()),
                                                         _ => return None
                                                     };
                                                     Some(((::types::IsExported{file: (*file).clone(), id: (::types::ast::AnyId::AnyIdClass{class: (*id).clone()})})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::VarDecl)]. */
                             Rule::CollectionRule {
                                 description: "IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::VarDecl)].".to_string(),
                                 rel: Relations::inputs_VarDecl as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::VarDecl[(inputs::VarDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::VarDecl)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref id, ref file) = match *unsafe {<::types::inputs::VarDecl>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::VarDecl{stmt_id: ref id, file: ref file, pattern: _, value: _, exported: true} => ((*id).clone(), (*file).clone()),
                                                         _ => return None
                                                     };
                                                     Some(((::types::IsExported{file: (*file).clone(), id: (::types::ast::AnyId::AnyIdStmt{stmt: (*id).clone()})})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::LetDecl)]. */
                             Rule::CollectionRule {
                                 description: "IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::LetDecl)].".to_string(),
                                 rel: Relations::inputs_LetDecl as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::LetDecl[(inputs::LetDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::LetDecl)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref id, ref file) = match *unsafe {<::types::inputs::LetDecl>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::LetDecl{stmt_id: ref id, file: ref file, pattern: _, value: _, exported: true} => ((*id).clone(), (*file).clone()),
                                                         _ => return None
                                                     };
                                                     Some(((::types::IsExported{file: (*file).clone(), id: (::types::ast::AnyId::AnyIdStmt{stmt: (*id).clone()})})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::ConstDecl)]. */
                             Rule::CollectionRule {
                                 description: "IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::ConstDecl)].".to_string(),
                                 rel: Relations::inputs_ConstDecl as RelId,
                                 xform: Some(XFormCollection::FilterMap{
                                                 description: "head of IsExported[(IsExported{.file=file, .id=(ast::AnyIdStmt{.stmt=id}: ast::AnyId)}: IsExported)] :- inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(id: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=true}: inputs::ConstDecl)]." .to_string(),
                                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                 {
                                                     let (ref id, ref file) = match *unsafe {<::types::inputs::ConstDecl>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::ConstDecl{stmt_id: ref id, file: ref file, pattern: _, value: _, exported: true} => ((*id).clone(), (*file).clone()),
                                                         _ => return None
                                                     };
                                                     Some(((::types::IsExported{file: (*file).clone(), id: (::types::ast::AnyId::AnyIdStmt{stmt: (*id).clone()})})).into_ddvalue())
                                                 }
                                                 __f},
                                                 next: Box::new(None)
                                             })
                             },
                             /* IsExported[(IsExported{.file=file, .id=id}: IsExported)] :- inputs::FileExport[(inputs::FileExport{.file=(file: ast::FileId), .export=(ast::NamedExport{.name=(export_name: ddlog_std::Option<ast::Spanned<ast::Name>>), .alias=(export_alias: ddlog_std::Option<ast::Spanned<ast::Name>>)}: ast::ExportKind), .scope=(export_scope: ast::ScopeId)}: inputs::FileExport)], ((ddlog_std::Some{.x=(ast::Spanned{.data=(var name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<internment::Intern<string>>>) = ((utils::or_else: function(ddlog_std::Option<ast::Spanned<ast::Name>>, ddlog_std::Option<ast::Spanned<ast::Name>>):ddlog_std::Option<ast::Spanned<internment::Intern<string>>>)(export_alias, export_name))), NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(export_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(id: ast::AnyId), .implicit=false}: NameInScope)]. */
                             Rule::CollectionRule {
                                 description: "IsExported[(IsExported{.file=file, .id=id}: IsExported)] :- inputs::FileExport[(inputs::FileExport{.file=(file: ast::FileId), .export=(ast::NamedExport{.name=(export_name: ddlog_std::Option<ast::Spanned<ast::Name>>), .alias=(export_alias: ddlog_std::Option<ast::Spanned<ast::Name>>)}: ast::ExportKind), .scope=(export_scope: ast::ScopeId)}: inputs::FileExport)], ((ddlog_std::Some{.x=(ast::Spanned{.data=(var name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<internment::Intern<string>>>) = ((utils::or_else: function(ddlog_std::Option<ast::Spanned<ast::Name>>, ddlog_std::Option<ast::Spanned<ast::Name>>):ddlog_std::Option<ast::Spanned<internment::Intern<string>>>)(export_alias, export_name))), NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(export_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(id: ast::AnyId), .implicit=false}: NameInScope)].".to_string(),
                                 rel: Relations::inputs_FileExport as RelId,
                                 xform: Some(XFormCollection::Arrange {
                                                 description: "arrange inputs::FileExport[(inputs::FileExport{.file=(file: ast::FileId), .export=(ast::NamedExport{.name=(export_name: ddlog_std::Option<ast::Spanned<ast::Name>>), .alias=(export_alias: ddlog_std::Option<ast::Spanned<ast::Name>>)}: ast::ExportKind), .scope=(export_scope: ast::ScopeId)}: inputs::FileExport)] by (file, name, export_scope)" .to_string(),
                                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                 {
                                                     let (ref file, ref export_name, ref export_alias, ref export_scope) = match *unsafe {<::types::inputs::FileExport>::from_ddvalue_ref(&__v) } {
                                                         ::types::inputs::FileExport{file: ref file, export: ::types::ast::ExportKind::NamedExport{name: ref export_name, alias: ref export_alias}, scope: ref export_scope} => ((*file).clone(), (*export_name).clone(), (*export_alias).clone(), (*export_scope).clone()),
                                                         _ => return None
                                                     };
                                                     let ref name: ::types::internment::Intern<String> = match ::types::utils::or_else::<::types::ast::Spanned<::types::ast::Name>>(export_alias, export_name) {
                                                         ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: name, span: _}} => name,
                                                         _ => return None
                                                     };
                                                     Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*export_scope).clone())).into_ddvalue(), ((*file).clone()).into_ddvalue()))
                                                 }
                                                 __f},
                                                 next: Box::new(XFormArrangement::Join{
                                                                    description: "inputs::FileExport[(inputs::FileExport{.file=(file: ast::FileId), .export=(ast::NamedExport{.name=(export_name: ddlog_std::Option<ast::Spanned<ast::Name>>), .alias=(export_alias: ddlog_std::Option<ast::Spanned<ast::Name>>)}: ast::ExportKind), .scope=(export_scope: ast::ScopeId)}: inputs::FileExport)], ((ddlog_std::Some{.x=(ast::Spanned{.data=(var name: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<internment::Intern<string>>>) = ((utils::or_else: function(ddlog_std::Option<ast::Spanned<ast::Name>>, ddlog_std::Option<ast::Spanned<ast::Name>>):ddlog_std::Option<ast::Spanned<internment::Intern<string>>>)(export_alias, export_name))), NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(export_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(id: ast::AnyId), .implicit=false}: NameInScope)]".to_string(),
                                                                    ffun: None,
                                                                    arrangement: (Relations::NameInScope as RelId,0),
                                                                    jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                    {
                                                                        let ref file = *unsafe {<::types::ast::FileId>::from_ddvalue_ref( __v1 ) };
                                                                        let ref id = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v2) } {
                                                                            ::types::NameInScope{file: _, name: _, scope: _, span: _, declared_in: ref id, implicit: _} => (*id).clone(),
                                                                            _ => return None
                                                                        };
                                                                        Some(((::types::IsExported{file: (*file).clone(), id: (*id).clone()})).into_ddvalue())
                                                                    }
                                                                    __f},
                                                                    next: Box::new(None)
                                                                })
                                             })
                             }],
                         arrangements: vec![
                             Arrangement::Set{
                                 name: r###"(IsExported{.file=(_0: ast::FileId), .id=(_1: ast::AnyId)}: IsExported) /*antijoin*/"###.to_string(),
                                 fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                 {
                                     match unsafe {< ::types::IsExported>::from_ddvalue(__v) } {
                                         ::types::IsExported{file: ref _0, id: ref _1} => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                         _ => None
                                     }
                                 }
                                 __f},
                                 distinct: false
                             }],
                         change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                     };
    let TypeofUndef = Relation {
                          name:         "TypeofUndef".to_string(),
                          input:        false,
                          distinct:     true,
                          caching_mode: CachingMode::Set,
                          key_func:     None,
                          id:           Relations::TypeofUndef as RelId,
                          rules:        vec![
                              /* TypeofUndef[(TypeofUndef{.whole_expr=whole_expr, .undefined_expr=undefined_expr, .file=file}: TypeofUndef)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], WithinTypeofExpr[(WithinTypeofExpr{.type_of=(whole_expr: ast::ExprId), .expr=(undefined_expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)]. */
                              Rule::ArrangementRule {
                                  description: "TypeofUndef[(TypeofUndef{.whole_expr=whole_expr, .undefined_expr=undefined_expr, .file=file}: TypeofUndef)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], WithinTypeofExpr[(WithinTypeofExpr{.type_of=(whole_expr: ast::ExprId), .expr=(undefined_expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)].".to_string(),
                                  arr: ( Relations::inputs_NameRef as RelId, 0),
                                  xform: XFormArrangement::Join{
                                             description: "inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)]".to_string(),
                                             ffun: None,
                                             arrangement: (Relations::inputs_Expression as RelId,1),
                                             jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                             {
                                                 let (ref undefined_expr, ref file, ref name) = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v1) } {
                                                     ::types::inputs::NameRef{expr_id: ref undefined_expr, file: ref file, value: ref name} => ((*undefined_expr).clone(), (*file).clone(), (*name).clone()),
                                                     _ => return None
                                                 };
                                                 let (ref scope, ref span) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                     ::types::inputs::Expression{id: _, file: _, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: ref scope, span: ref span} => ((*scope).clone(), (*span).clone()),
                                                     _ => return None
                                                 };
                                                 Some((::types::ddlog_std::tuple4((*undefined_expr).clone(), (*file).clone(), (*name).clone(), (*scope).clone())).into_ddvalue())
                                             }
                                             __f},
                                             next: Box::new(Some(XFormCollection::Arrange {
                                                                     description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)] by (file, name, scope)" .to_string(),
                                                                     afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                     {
                                                                         let ::types::ddlog_std::tuple4(ref undefined_expr, ref file, ref name, ref scope) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                         Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple2((*undefined_expr).clone(), (*file).clone())).into_ddvalue()))
                                                                     }
                                                                     __f},
                                                                     next: Box::new(XFormArrangement::Antijoin {
                                                                                        description: "inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)]".to_string(),
                                                                                        ffun: None,
                                                                                        arrangement: (Relations::NameInScope as RelId,2),
                                                                                        next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)] by (undefined_expr, file)" .to_string(),
                                                                                                                afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                {
                                                                                                                    let ::types::ddlog_std::tuple2(ref undefined_expr, ref file) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::ExprId, ::types::ast::FileId>>::from_ddvalue_ref( &__v ) };
                                                                                                                    Some(((::types::ddlog_std::tuple2((*undefined_expr).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple2((*undefined_expr).clone(), (*file).clone())).into_ddvalue()))
                                                                                                                }
                                                                                                                __f},
                                                                                                                next: Box::new(XFormArrangement::Join{
                                                                                                                                   description: "inputs::NameRef[(inputs::NameRef{.expr_id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(undefined_expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(span: ast::Span)}: inputs::Expression)], not NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(_: ast::AnyId), .implicit=(_: bool)}: NameInScope)], WithinTypeofExpr[(WithinTypeofExpr{.type_of=(whole_expr: ast::ExprId), .expr=(undefined_expr: ast::ExprId), .file=(file: ast::FileId)}: WithinTypeofExpr)]".to_string(),
                                                                                                                                   ffun: None,
                                                                                                                                   arrangement: (Relations::WithinTypeofExpr as RelId,1),
                                                                                                                                   jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                   {
                                                                                                                                       let ::types::ddlog_std::tuple2(ref undefined_expr, ref file) = *unsafe {<::types::ddlog_std::tuple2<::types::ast::ExprId, ::types::ast::FileId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                       let ref whole_expr = match *unsafe {<::types::WithinTypeofExpr>::from_ddvalue_ref(__v2) } {
                                                                                                                                           ::types::WithinTypeofExpr{type_of: ref whole_expr, expr: _, file: _} => (*whole_expr).clone(),
                                                                                                                                           _ => return None
                                                                                                                                       };
                                                                                                                                       Some(((::types::TypeofUndef{whole_expr: (*whole_expr).clone(), undefined_expr: (*undefined_expr).clone(), file: (*file).clone()})).into_ddvalue())
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
    let VariableUsages = Relation {
                             name:         "VariableUsages".to_string(),
                             input:        false,
                             distinct:     true,
                             caching_mode: CachingMode::Set,
                             key_func:     None,
                             id:           Relations::VariableUsages as RelId,
                             rules:        vec![
                                 /* VariableUsages[(VariableUsages{.file=file, .name=name, .scope=scope, .declared_in=declared}: VariableUsages)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=(_: bool)}: NameInScope)], inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]. */
                                 Rule::ArrangementRule {
                                     description: "VariableUsages[(VariableUsages{.file=file, .name=name, .scope=scope, .declared_in=declared}: VariableUsages)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=(_: bool)}: NameInScope)], inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)].".to_string(),
                                     arr: ( Relations::NameInScope as RelId, 8),
                                     xform: XFormArrangement::Join{
                                                description: "NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=(_: bool)}: NameInScope)], inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)]".to_string(),
                                                ffun: None,
                                                arrangement: (Relations::inputs_NameRef as RelId,2),
                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                {
                                                    let (ref file, ref name, ref scope, ref declared) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v1) } {
                                                        ::types::NameInScope{file: ref file, name: ref name, scope: ref scope, span: _, declared_in: ref declared, implicit: _} => ((*file).clone(), (*name).clone(), (*scope).clone(), (*declared).clone()),
                                                        _ => return None
                                                    };
                                                    let ref expr = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v2) } {
                                                        ::types::inputs::NameRef{expr_id: ref expr, file: _, value: _} => (*expr).clone(),
                                                        _ => return None
                                                    };
                                                    Some((::types::ddlog_std::tuple5((*file).clone(), (*name).clone(), (*scope).clone(), (*declared).clone(), (*expr).clone())).into_ddvalue())
                                                }
                                                __f},
                                                next: Box::new(Some(XFormCollection::Arrange {
                                                                        description: "arrange NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=(_: bool)}: NameInScope)], inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)] by (expr, file, scope)" .to_string(),
                                                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                        {
                                                                            let ::types::ddlog_std::tuple5(ref file, ref name, ref scope, ref declared, ref expr) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                            Some(((::types::ddlog_std::tuple3((*expr).clone(), (*file).clone(), (*scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*file).clone(), (*name).clone(), (*scope).clone(), (*declared).clone())).into_ddvalue()))
                                                                        }
                                                                        __f},
                                                                        next: Box::new(XFormArrangement::Semijoin{
                                                                                           description: "NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=(_: bool)}: NameInScope)], inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(scope: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression)]".to_string(),
                                                                                           ffun: None,
                                                                                           arrangement: (Relations::inputs_Expression as RelId,2),
                                                                                           jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                           {
                                                                                               let ::types::ddlog_std::tuple4(ref file, ref name, ref scope, ref declared) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::AnyId>>::from_ddvalue_ref( __v1 ) };
                                                                                               Some(((::types::VariableUsages{file: (*file).clone(), name: (*name).clone(), scope: (*scope).clone(), declared_in: (*declared).clone()})).into_ddvalue())
                                                                                           }
                                                                                           __f},
                                                                                           next: Box::new(None)
                                                                                       })
                                                                    }))
                                            }
                                 }],
                             arrangements: vec![
                                 Arrangement::Set{
                                     name: r###"(VariableUsages{.file=(_0: ast::FileId), .name=(_1: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(_2: ast::AnyId)}: VariableUsages) /*antijoin*/"###.to_string(),
                                     fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                     {
                                         match unsafe {< ::types::VariableUsages>::from_ddvalue(__v) } {
                                             ::types::VariableUsages{file: ref _0, name: ref _1, scope: _, declared_in: ref _2} => Some((::types::ddlog_std::tuple3((*_0).clone(), (*_1).clone(), (*_2).clone())).into_ddvalue()),
                                             _ => None
                                         }
                                     }
                                     __f},
                                     distinct: true
                                 }],
                             change_cb:    Some(sync::Arc::new(sync::Mutex::new(__update_cb.clone())))
                         };
    let UnusedVariables = Relation {
                              name:         "UnusedVariables".to_string(),
                              input:        false,
                              distinct:     true,
                              caching_mode: CachingMode::Set,
                              key_func:     None,
                              id:           Relations::UnusedVariables as RelId,
                              rules:        vec![
                                  /* UnusedVariables[(UnusedVariables{.name=name, .declared=declared, .span=span, .file=file}: UnusedVariables)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=false}: NameInScope)], (not (ast::is_global(declared))), not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)]. */
                                  Rule::ArrangementRule {
                                      description: "UnusedVariables[(UnusedVariables{.name=name, .declared=declared, .span=span, .file=file}: UnusedVariables)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=false}: NameInScope)], (not (ast::is_global(declared))), not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)].".to_string(),
                                      arr: ( Relations::NameInScope as RelId, 3),
                                      xform: XFormArrangement::Antijoin {
                                                 description: "NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=false}: NameInScope)], (not (ast::is_global(declared))), not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)]".to_string(),
                                                 ffun: Some(&{fn __f(__v: &DDValue) -> bool
                                                       {
                                                           let (ref file, ref name, ref span, ref declared) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v) } {
                                                               ::types::NameInScope{file: ref file, name: ref name, scope: _, span: ::types::ddlog_std::Option::Some{x: ref span}, declared_in: ref declared, implicit: false} => ((*file).clone(), (*name).clone(), (*span).clone(), (*declared).clone()),
                                                               _ => return false
                                                           };
                                                           (!::types::ast::is_global(declared))
                                                       }
                                                           __f
                                                       }),
                                                 arrangement: (Relations::IsExported as RelId,0),
                                                 next: Box::new(Some(XFormCollection::Arrange {
                                                                         description: "arrange NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=false}: NameInScope)], (not (ast::is_global(declared))), not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)] by (file, name, declared)" .to_string(),
                                                                         afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                         {
                                                                             let (ref file, ref name, ref span, ref declared) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(&__v) } {
                                                                                 ::types::NameInScope{file: ref file, name: ref name, scope: _, span: ::types::ddlog_std::Option::Some{x: ref span}, declared_in: ref declared, implicit: false} => ((*file).clone(), (*name).clone(), (*span).clone(), (*declared).clone()),
                                                                                 _ => return None
                                                                             };
                                                                             Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*declared).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*file).clone(), (*name).clone(), (*span).clone(), (*declared).clone())).into_ddvalue()))
                                                                         }
                                                                         __f},
                                                                         next: Box::new(XFormArrangement::Antijoin {
                                                                                            description: "NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=false}: NameInScope)], (not (ast::is_global(declared))), not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)]".to_string(),
                                                                                            ffun: None,
                                                                                            arrangement: (Relations::VariableUsages as RelId,0),
                                                                                            next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                    description: "head of UnusedVariables[(UnusedVariables{.name=name, .declared=declared, .span=span, .file=file}: UnusedVariables)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared: ast::AnyId), .implicit=false}: NameInScope)], (not (ast::is_global(declared))), not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)]." .to_string(),
                                                                                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                    {
                                                                                                                        let ::types::ddlog_std::tuple4(ref file, ref name, ref span, ref declared) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::Span, ::types::ast::AnyId>>::from_ddvalue_ref( &__v ) };
                                                                                                                        Some(((::types::UnusedVariables{name: (*name).clone(), declared: (*declared).clone(), span: (*span).clone(), file: (*file).clone()})).into_ddvalue())
                                                                                                                    }
                                                                                                                    __f},
                                                                                                                    next: Box::new(None)
                                                                                                                }))
                                                                                        })
                                                                     }))
                                             }
                                  },
                                  /* UnusedVariables[(UnusedVariables{.name=name, .declared=declared, .span=span, .file=file}: UnusedVariables)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdGlobal{.global=(global: ast::GlobalId)}: ast::AnyId)), .implicit=false}: NameInScope)], not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)]. */
                                  Rule::ArrangementRule {
                                      description: "UnusedVariables[(UnusedVariables{.name=name, .declared=declared, .span=span, .file=file}: UnusedVariables)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdGlobal{.global=(global: ast::GlobalId)}: ast::AnyId)), .implicit=false}: NameInScope)], not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)].".to_string(),
                                      arr: ( Relations::NameInScope as RelId, 4),
                                      xform: XFormArrangement::Antijoin {
                                                 description: "NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdGlobal{.global=(global: ast::GlobalId)}: ast::AnyId)), .implicit=false}: NameInScope)], not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)]".to_string(),
                                                 ffun: None,
                                                 arrangement: (Relations::IsExported as RelId,0),
                                                 next: Box::new(Some(XFormCollection::Arrange {
                                                                         description: "arrange NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdGlobal{.global=(global: ast::GlobalId)}: ast::AnyId)), .implicit=false}: NameInScope)], not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)] by (file, name, declared)" .to_string(),
                                                                         afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                         {
                                                                             let (ref file, ref name, ref span, ref declared, ref global) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(&__v) } {
                                                                                 ::types::NameInScope{file: ref file, name: ref name, scope: _, span: ::types::ddlog_std::Option::Some{x: ref span}, declared_in: ref declared, implicit: false} => match declared {
                                                                                                                                                                                                                                                        ::types::ast::AnyId::AnyIdGlobal{global: ref global} => ((*file).clone(), (*name).clone(), (*span).clone(), (*declared).clone(), (*global).clone()),
                                                                                                                                                                                                                                                        _ => return None
                                                                                                                                                                                                                                                    },
                                                                                 _ => return None
                                                                             };
                                                                             Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*declared).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*file).clone(), (*name).clone(), (*span).clone(), (*declared).clone())).into_ddvalue()))
                                                                         }
                                                                         __f},
                                                                         next: Box::new(XFormArrangement::Antijoin {
                                                                                            description: "NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdGlobal{.global=(global: ast::GlobalId)}: ast::AnyId)), .implicit=false}: NameInScope)], not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)]".to_string(),
                                                                                            ffun: None,
                                                                                            arrangement: (Relations::VariableUsages as RelId,0),
                                                                                            next: Box::new(Some(XFormCollection::FilterMap{
                                                                                                                    description: "head of UnusedVariables[(UnusedVariables{.name=name, .declared=declared, .span=span, .file=file}: UnusedVariables)] :- NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .span=(ddlog_std::Some{.x=(span: ast::Span)}: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdGlobal{.global=(global: ast::GlobalId)}: ast::AnyId)), .implicit=false}: NameInScope)], not IsExported[(IsExported{.file=(file: ast::FileId), .id=(declared: ast::AnyId)}: IsExported)], not VariableUsages[(VariableUsages{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(_: ast::ScopeId), .declared_in=(declared: ast::AnyId)}: VariableUsages)]." .to_string(),
                                                                                                                    fmfun: &{fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                    {
                                                                                                                        let ::types::ddlog_std::tuple4(ref file, ref name, ref span, ref declared) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::Span, ::types::ast::AnyId>>::from_ddvalue_ref( &__v ) };
                                                                                                                        Some(((::types::UnusedVariables{name: (*name).clone(), declared: (*declared).clone(), span: (*span).clone(), file: (*file).clone()})).into_ddvalue())
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
    let __Prefix_0 = Relation {
                         name:         "__Prefix_0".to_string(),
                         input:        false,
                         distinct:     false,
                         caching_mode: CachingMode::Set,
                         key_func:     None,
                         id:           Relations::__Prefix_0 as RelId,
                         rules:        vec![
                             /* __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))] :- __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(object: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(decl: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)]. */
                             Rule::ArrangementRule {
                                 description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))] :- __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(object: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(decl: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)].".to_string(),
                                 arr: ( Relations::__Prefix_1 as RelId, 1),
                                 xform: XFormArrangement::Join{
                                            description: "__Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(object: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)]".to_string(),
                                            ffun: None,
                                            arrangement: (Relations::inputs_NameRef as RelId,0),
                                            jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                            {
                                                let (ref expr, ref file, ref object, ref used_scope, ref used_in) = match *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref(__v1) } {
                                                    ::types::ddlog_std::tuple5(ref expr, ref file, ref object, ref used_scope, ref used_in) => ((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone()),
                                                    _ => return None
                                                };
                                                let ref name = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v2) } {
                                                    ::types::inputs::NameRef{expr_id: _, file: _, value: ref name} => (*name).clone(),
                                                    _ => return None
                                                };
                                                Some((::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue())
                                            }
                                            __f},
                                            next: Box::new(Some(XFormCollection::Arrange {
                                                                    description: "arrange __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(object: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)] by (file, name, used_scope)" .to_string(),
                                                                    afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                    {
                                                                        let ::types::ddlog_std::tuple6(ref expr, ref file, ref object, ref used_scope, ref used_in, ref name) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>>>::from_ddvalue_ref( &__v ) };
                                                                        Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*used_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue()))
                                                                    }
                                                                    __f},
                                                                    next: Box::new(XFormArrangement::Join{
                                                                                       description: "__Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(object: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(decl: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)]".to_string(),
                                                                                       ffun: None,
                                                                                       arrangement: (Relations::NameInScope as RelId,5),
                                                                                       jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                       {
                                                                                           let ::types::ddlog_std::tuple6(ref expr, ref file, ref object, ref used_scope, ref used_in, ref name) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>>>::from_ddvalue_ref( __v1 ) };
                                                                                           let (ref declared, ref decl) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v2) } {
                                                                                               ::types::NameInScope{file: _, name: _, scope: _, span: _, declared_in: ref declared, implicit: _} => match declared {
                                                                                                                                                                                                        ::types::ast::AnyId::AnyIdStmt{stmt: ref decl} => ((*declared).clone(), (*decl).clone()),
                                                                                                                                                                                                        _ => return None
                                                                                                                                                                                                    },
                                                                                               _ => return None
                                                                                           };
                                                                                           Some((::types::ddlog_std::tuple8((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*decl).clone())).into_ddvalue())
                                                                                       }
                                                                                       __f},
                                                                                       next: Box::new(None)
                                                                                   })
                                                                }))
                                        }
                             }],
                         arrangements: vec![
                             Arrangement::Map{
                                name: r###"((_: ast::ExprId), (_1: ast::FileId), (_: ast::ExprId), (_: ast::ScopeId), (_: ast::Span), (_: internment::Intern<string>), (_: ast::AnyId), (_0: ast::StmtId)) /*join*/"###.to_string(),
                                 afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                 {
                                     let __cloned = __v.clone();
                                     match unsafe {< ::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::StmtId>>::from_ddvalue(__v) } {
                                         ::types::ddlog_std::tuple8(_, ref _1, _, _, _, _, _, ref _0) => Some((::types::ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                         _ => None
                                     }.map(|x|(x,__cloned))
                                 }
                                 __f},
                                 queryable: false
                             }],
                         change_cb:    None
                     };
    let UseBeforeDecl = Relation {
                            name:         "UseBeforeDecl".to_string(),
                            input:        false,
                            distinct:     true,
                            caching_mode: CachingMode::Set,
                            key_func:     None,
                            id:           Relations::UseBeforeDecl as RelId,
                            rules:        vec![
                                /* UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Statement)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(declared_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                                Rule::ArrangementRule {
                                    description: "UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Statement)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(declared_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                    arr: ( Relations::inputs_NameRef as RelId, 0),
                                    xform: XFormArrangement::Join{
                                               description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)]".to_string(),
                                               ffun: None,
                                               arrangement: (Relations::inputs_Expression as RelId,1),
                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                               {
                                                   let (ref expr, ref file, ref name) = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v1) } {
                                                       ::types::inputs::NameRef{expr_id: ref expr, file: ref file, value: ref name} => ((*expr).clone(), (*file).clone(), (*name).clone()),
                                                       _ => return None
                                                   };
                                                   let (ref used_scope, ref used_in) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                       ::types::inputs::Expression{id: _, file: _, kind: ::types::ast::ExprKind::ExprNameRef{}, scope: ref used_scope, span: ref used_in} => ((*used_scope).clone(), (*used_in).clone()),
                                                       _ => return None
                                                   };
                                                   Some((::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue())
                                               }
                                               __f},
                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                       description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)] by (file, name, used_scope)" .to_string(),
                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                       {
                                                                           let ::types::ddlog_std::tuple5(ref expr, ref file, ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                           Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*used_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*name).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue()))
                                                                       }
                                                                       __f},
                                                                       next: Box::new(XFormArrangement::Join{
                                                                                          description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)]".to_string(),
                                                                                          ffun: None,
                                                                                          arrangement: (Relations::NameInScope as RelId,5),
                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                          {
                                                                                              let ::types::ddlog_std::tuple5(ref expr, ref file, ref name, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                              let (ref declared, ref stmt) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v2) } {
                                                                                                  ::types::NameInScope{file: _, name: _, scope: _, span: _, declared_in: ref declared, implicit: _} => match declared {
                                                                                                                                                                                                           ::types::ast::AnyId::AnyIdStmt{stmt: ref stmt} => ((*declared).clone(), (*stmt).clone()),
                                                                                                                                                                                                           _ => return None
                                                                                                                                                                                                       },
                                                                                                  _ => return None
                                                                                              };
                                                                                              Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*name).clone(), (*used_scope).clone(), (*used_in).clone(), (*declared).clone(), (*stmt).clone())).into_ddvalue())
                                                                                          }
                                                                                          __f},
                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                  description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)] by (stmt, file)" .to_string(),
                                                                                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                  {
                                                                                                                      let ::types::ddlog_std::tuple7(ref expr, ref file, ref name, ref used_scope, ref used_in, ref declared, ref stmt) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span, ::types::ast::AnyId, ::types::ast::StmtId>>::from_ddvalue_ref( &__v ) };
                                                                                                                      Some(((::types::ddlog_std::tuple2((*stmt).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*name).clone(), (*used_scope).clone(), (*used_in).clone(), (*declared).clone())).into_ddvalue()))
                                                                                                                  }
                                                                                                                  __f},
                                                                                                                  next: Box::new(XFormArrangement::Join{
                                                                                                                                     description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Statement)]".to_string(),
                                                                                                                                     ffun: None,
                                                                                                                                     arrangement: (Relations::inputs_Statement as RelId,2),
                                                                                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                     {
                                                                                                                                         let ::types::ddlog_std::tuple6(ref expr, ref file, ref name, ref used_scope, ref used_in, ref declared) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span, ::types::ast::AnyId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                         let (ref declared_scope, ref declared_in) = match *unsafe {<::types::inputs::Statement>::from_ddvalue_ref(__v2) } {
                                                                                                                                             ::types::inputs::Statement{id: _, file: _, kind: ::types::ast::StmtKind::StmtVarDecl{}, scope: ref declared_scope, span: ref declared_in} => ((*declared_scope).clone(), (*declared_in).clone()),
                                                                                                                                             _ => return None
                                                                                                                                         };
                                                                                                                                         Some((::types::ddlog_std::tuple8((*expr).clone(), (*file).clone(), (*name).clone(), (*used_scope).clone(), (*used_in).clone(), (*declared).clone(), (*declared_scope).clone(), (*declared_in).clone())).into_ddvalue())
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                             description: "arrange inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Statement)] by (used_scope, declared_scope, file)" .to_string(),
                                                                                                                                                             afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                             {
                                                                                                                                                                 let ::types::ddlog_std::tuple8(ref expr, ref file, ref name, ref used_scope, ref used_in, ref declared, ref declared_scope, ref declared_in) = *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::ScopeId, ::types::ast::Span, ::types::ast::AnyId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                 Some(((::types::ddlog_std::tuple3((*used_scope).clone(), (*declared_scope).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*name).clone(), (*used_in).clone(), (*declared).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                             }
                                                                                                                                                             __f},
                                                                                                                                                             next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                description: "inputs::NameRef[(inputs::NameRef{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdStmt{.stmt=(stmt: ast::StmtId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Statement[(inputs::Statement{.id=(stmt: ast::StmtId), .file=(file: ast::FileId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(declared_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Statement)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(declared_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                                                                                                                                                                ffun: None,
                                                                                                                                                                                arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                {
                                                                                                                                                                                    let ::types::ddlog_std::tuple6(ref expr, ref file, ref name, ref used_in, ref declared, ref declared_in) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::internment::Intern<String>, ::types::ast::Span, ::types::ast::AnyId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                    Some(((::types::UseBeforeDecl{name: (*name).clone(), used: (*expr).clone(), used_in: (*used_in).clone(), declared: (*declared).clone(), declared_in: (*declared_in).clone(), file: (*file).clone()})).into_ddvalue())
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
                                },
                                /* UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdClass{.class=(class: ast::ClassId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(decl_scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                                Rule::ArrangementRule {
                                    description: "UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdClass{.class=(class: ast::ClassId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(decl_scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                    arr: ( Relations::__Prefix_1 as RelId, 0),
                                    xform: XFormArrangement::Join{
                                               description: "__Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)]".to_string(),
                                               ffun: None,
                                               arrangement: (Relations::inputs_NameRef as RelId,1),
                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                               {
                                                   let (ref expr, ref file, ref object, ref used_scope, ref used_in) = match *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref(__v1) } {
                                                       ::types::ddlog_std::tuple5(ref expr, ref file, ref object, ref used_scope, ref used_in) => ((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone()),
                                                       _ => return None
                                                   };
                                                   let (ref callee, ref name) = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v2) } {
                                                       ::types::inputs::NameRef{expr_id: ref callee, file: _, value: ref name} => ((*callee).clone(), (*name).clone()),
                                                       _ => return None
                                                   };
                                                   Some((::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue())
                                               }
                                               __f},
                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                       description: "arrange __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)] by (file, name, used_scope)" .to_string(),
                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                       {
                                                                           let ::types::ddlog_std::tuple5(ref expr, ref file, ref used_scope, ref used_in, ref name) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>>>::from_ddvalue_ref( &__v ) };
                                                                           Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*used_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue()))
                                                                       }
                                                                       __f},
                                                                       next: Box::new(XFormArrangement::Join{
                                                                                          description: "__Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdClass{.class=(class: ast::ClassId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)]".to_string(),
                                                                                          ffun: None,
                                                                                          arrangement: (Relations::NameInScope as RelId,6),
                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                          {
                                                                                              let ::types::ddlog_std::tuple5(ref expr, ref file, ref used_scope, ref used_in, ref name) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>>>::from_ddvalue_ref( __v1 ) };
                                                                                              let (ref declared, ref class) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v2) } {
                                                                                                  ::types::NameInScope{file: _, name: _, scope: _, span: _, declared_in: ref declared, implicit: _} => match declared {
                                                                                                                                                                                                           ::types::ast::AnyId::AnyIdClass{class: ref class} => ((*declared).clone(), (*class).clone()),
                                                                                                                                                                                                           _ => return None
                                                                                                                                                                                                       },
                                                                                                  _ => return None
                                                                                              };
                                                                                              Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue())
                                                                                          }
                                                                                          __f},
                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                  description: "arrange __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdClass{.class=(class: ast::ClassId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)] by (class, file)" .to_string(),
                                                                                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                  {
                                                                                                                      let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ClassId>>::from_ddvalue_ref( &__v ) };
                                                                                                                      Some(((::types::ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone())).into_ddvalue()))
                                                                                                                  }
                                                                                                                  __f},
                                                                                                                  next: Box::new(XFormArrangement::Join{
                                                                                                                                     description: "__Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdClass{.class=(class: ast::ClassId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(decl_scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)]".to_string(),
                                                                                                                                     ffun: None,
                                                                                                                                     arrangement: (Relations::inputs_Class as RelId,0),
                                                                                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                     {
                                                                                                                                         let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                         let (ref declared_in, ref decl_scope) = match *unsafe {<::types::inputs::Class>::from_ddvalue_ref(__v2) } {
                                                                                                                                             ::types::inputs::Class{id: _, file: _, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: _, span: ref declared_in}}, parent: _, elements: _, scope: ref decl_scope, exported: _} => ((*declared_in).clone(), (*decl_scope).clone()),
                                                                                                                                             _ => return None
                                                                                                                                         };
                                                                                                                                         Some((::types::ddlog_std::tuple8((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*declared_in).clone(), (*decl_scope).clone())).into_ddvalue())
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                             description: "arrange __Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdClass{.class=(class: ast::ClassId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(decl_scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)] by (used_scope, decl_scope, file)" .to_string(),
                                                                                                                                                             afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                             {
                                                                                                                                                                 let ::types::ddlog_std::tuple8(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref declared_in, ref decl_scope) = *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::Span, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                 Some(((::types::ddlog_std::tuple3((*used_scope).clone(), (*decl_scope).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                             }
                                                                                                                                                             __f},
                                                                                                                                                             next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                description: "__Prefix_1[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span))], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdClass{.class=(class: ast::ClassId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Class[(inputs::Class{.id=(class: ast::ClassId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(decl_scope: ast::ScopeId), .exported=(_: bool)}: inputs::Class)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                                                                                                                                                                ffun: None,
                                                                                                                                                                                arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                {
                                                                                                                                                                                    let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_in, ref name, ref declared, ref declared_in) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                    Some(((::types::UseBeforeDecl{name: (*name).clone(), used: (*expr).clone(), used_in: (*used_in).clone(), declared: (*declared).clone(), declared_in: (*declared_in).clone(), file: (*file).clone()})).into_ddvalue())
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
                                },
                                /* UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdFunc{.func=(func: ast::FuncId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(decl_scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                                Rule::ArrangementRule {
                                    description: "UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdFunc{.func=(func: ast::FuncId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(decl_scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                    arr: ( Relations::inputs_Call as RelId, 0),
                                    xform: XFormArrangement::Join{
                                               description: "inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)]".to_string(),
                                               ffun: None,
                                               arrangement: (Relations::inputs_Expression as RelId,0),
                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                               {
                                                   let (ref expr, ref file, ref callee) = match *unsafe {<::types::inputs::Call>::from_ddvalue_ref(__v1) } {
                                                       ::types::inputs::Call{expr_id: ref expr, file: ref file, callee: ::types::ddlog_std::Option::Some{x: ref callee}, args: _} => ((*expr).clone(), (*file).clone(), (*callee).clone()),
                                                       _ => return None
                                                   };
                                                   let (ref used_scope, ref used_in) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                       ::types::inputs::Expression{id: _, file: _, kind: _, scope: ref used_scope, span: ref used_in} => ((*used_scope).clone(), (*used_in).clone()),
                                                       _ => return None
                                                   };
                                                   Some((::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*callee).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue())
                                               }
                                               __f},
                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                       description: "arrange inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)] by (callee, file)" .to_string(),
                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                       {
                                                                           let ::types::ddlog_std::tuple5(ref expr, ref file, ref callee, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                           Some(((::types::ddlog_std::tuple2((*callee).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple4((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone())).into_ddvalue()))
                                                                       }
                                                                       __f},
                                                                       next: Box::new(XFormArrangement::Join{
                                                                                          description: "inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)]".to_string(),
                                                                                          ffun: None,
                                                                                          arrangement: (Relations::inputs_NameRef as RelId,0),
                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                          {
                                                                                              let ::types::ddlog_std::tuple4(ref expr, ref file, ref used_scope, ref used_in) = *unsafe {<::types::ddlog_std::tuple4<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                              let ref name = match *unsafe {<::types::inputs::NameRef>::from_ddvalue_ref(__v2) } {
                                                                                                  ::types::inputs::NameRef{expr_id: _, file: _, value: ref name} => (*name).clone(),
                                                                                                  _ => return None
                                                                                              };
                                                                                              Some((::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue())
                                                                                          }
                                                                                          __f},
                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                  description: "arrange inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)] by (file, name, used_scope)" .to_string(),
                                                                                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                  {
                                                                                                                      let ::types::ddlog_std::tuple5(ref expr, ref file, ref used_scope, ref used_in, ref name) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>>>::from_ddvalue_ref( &__v ) };
                                                                                                                      Some(((::types::ddlog_std::tuple3((*file).clone(), (*name).clone(), (*used_scope).clone())).into_ddvalue(), (::types::ddlog_std::tuple5((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone())).into_ddvalue()))
                                                                                                                  }
                                                                                                                  __f},
                                                                                                                  next: Box::new(XFormArrangement::Join{
                                                                                                                                     description: "inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdFunc{.func=(func: ast::FuncId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)]".to_string(),
                                                                                                                                     ffun: None,
                                                                                                                                     arrangement: (Relations::NameInScope as RelId,7),
                                                                                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                     {
                                                                                                                                         let ::types::ddlog_std::tuple5(ref expr, ref file, ref used_scope, ref used_in, ref name) = *unsafe {<::types::ddlog_std::tuple5<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                         let (ref declared, ref func) = match *unsafe {<::types::NameInScope>::from_ddvalue_ref(__v2) } {
                                                                                                                                             ::types::NameInScope{file: _, name: _, scope: _, span: _, declared_in: ref declared, implicit: _} => match declared {
                                                                                                                                                                                                                                                      ::types::ast::AnyId::AnyIdFunc{func: ref func} => ((*declared).clone(), (*func).clone()),
                                                                                                                                                                                                                                                      _ => return None
                                                                                                                                                                                                                                                  },
                                                                                                                                             _ => return None
                                                                                                                                         };
                                                                                                                                         Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*func).clone())).into_ddvalue())
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                             description: "arrange inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdFunc{.func=(func: ast::FuncId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)] by (func, file)" .to_string(),
                                                                                                                                                             afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                             {
                                                                                                                                                                 let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref func) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::FuncId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                 Some(((::types::ddlog_std::tuple2((*func).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone())).into_ddvalue()))
                                                                                                                                                             }
                                                                                                                                                             __f},
                                                                                                                                                             next: Box::new(XFormArrangement::Join{
                                                                                                                                                                                description: "inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdFunc{.func=(func: ast::FuncId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(decl_scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)]".to_string(),
                                                                                                                                                                                ffun: None,
                                                                                                                                                                                arrangement: (Relations::inputs_Function as RelId,2),
                                                                                                                                                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                                                                {
                                                                                                                                                                                    let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                    let (ref declared_in, ref decl_scope) = match *unsafe {<::types::inputs::Function>::from_ddvalue_ref(__v2) } {
                                                                                                                                                                                        ::types::inputs::Function{id: _, file: _, name: ::types::ddlog_std::Option::Some{x: ::types::ast::Spanned{data: _, span: ref declared_in}}, scope: ref decl_scope, body: _, exported: _} => ((*declared_in).clone(), (*decl_scope).clone()),
                                                                                                                                                                                        _ => return None
                                                                                                                                                                                    };
                                                                                                                                                                                    Some((::types::ddlog_std::tuple8((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*declared_in).clone(), (*decl_scope).clone())).into_ddvalue())
                                                                                                                                                                                }
                                                                                                                                                                                __f},
                                                                                                                                                                                next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                                                                        description: "arrange inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdFunc{.func=(func: ast::FuncId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(decl_scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)] by (used_scope, decl_scope, file)" .to_string(),
                                                                                                                                                                                                        afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                                                                        {
                                                                                                                                                                                                            let ::types::ddlog_std::tuple8(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref declared_in, ref decl_scope) = *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::Span, ::types::ast::ScopeId>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                                                            Some(((::types::ddlog_std::tuple3((*used_scope).clone(), (*decl_scope).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                                                                        }
                                                                                                                                                                                                        __f},
                                                                                                                                                                                                        next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                                                           description: "inputs::Call[(inputs::Call{.expr_id=(expr: ast::ExprId), .file=(file: ast::FileId), .callee=(ddlog_std::Some{.x=(callee: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call)], inputs::Expression[(inputs::Expression{.id=(expr: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(used_scope: ast::ScopeId), .span=(used_in: ast::Span)}: inputs::Expression)], inputs::NameRef[(inputs::NameRef{.expr_id=(callee: ast::ExprId), .file=(file: ast::FileId), .value=(name: internment::Intern<string>)}: inputs::NameRef)], NameInScope[(NameInScope{.file=(file: ast::FileId), .name=(name: internment::Intern<string>), .scope=(used_scope: ast::ScopeId), .span=(_: ddlog_std::Option<ast::Span>), .declared_in=(declared@ (ast::AnyIdFunc{.func=(func: ast::FuncId)}: ast::AnyId)), .implicit=(_: bool)}: NameInScope)], inputs::Function[(inputs::Function{.id=(func: ast::FuncId), .file=(file: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(declared_in: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(decl_scope: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                                                                                                                                                                                                           ffun: None,
                                                                                                                                                                                                                           arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                                                           jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                                                           {
                                                                                                                                                                                                                               let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_in, ref name, ref declared, ref declared_in) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                                                               Some(((::types::UseBeforeDecl{name: (*name).clone(), used: (*expr).clone(), used_in: (*used_in).clone(), declared: (*declared).clone(), declared_in: (*declared_in).clone(), file: (*file).clone()})).into_ddvalue())
                                                                                                                                                                                                                           }
                                                                                                                                                                                                                           __f},
                                                                                                                                                                                                                           next: Box::new(None)
                                                                                                                                                                                                                       })
                                                                                                                                                                                                    }))
                                                                                                                                                                            })
                                                                                                                                                         }))
                                                                                                                                 })
                                                                                                              }))
                                                                                      })
                                                                   }))
                                           }
                                },
                                /* UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                                Rule::ArrangementRule {
                                    description: "UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                    arr: ( Relations::__Prefix_0 as RelId, 0),
                                    xform: XFormArrangement::Join{
                                               description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)]".to_string(),
                                               ffun: None,
                                               arrangement: (Relations::inputs_VarDecl as RelId,1),
                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                               {
                                                   let (ref expr, ref file, ref object, ref used_scope, ref used_in, ref name, ref declared, ref decl) = match *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::StmtId>>::from_ddvalue_ref(__v1) } {
                                                       ::types::ddlog_std::tuple8(ref expr, ref file, ref object, ref used_scope, ref used_in, ref name, ref declared, ref decl) => ((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*decl).clone()),
                                                       _ => return None
                                                   };
                                                   let ref class = match *unsafe {<::types::inputs::VarDecl>::from_ddvalue_ref(__v2) } {
                                                       ::types::inputs::VarDecl{stmt_id: _, file: _, pattern: _, value: ::types::ddlog_std::Option::Some{x: ref class}, exported: _} => (*class).clone(),
                                                       _ => return None
                                                   };
                                                   Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue())
                                               }
                                               __f},
                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                       description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)] by (class, file)" .to_string(),
                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                       {
                                                                           let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                           Some(((::types::ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue()))
                                                                       }
                                                                       __f},
                                                                       next: Box::new(XFormArrangement::Semijoin{
                                                                                          description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)]".to_string(),
                                                                                          ffun: None,
                                                                                          arrangement: (Relations::inputs_ClassExpr as RelId,0),
                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                          {
                                                                                              let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                              Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue())
                                                                                          }
                                                                                          __f},
                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                  description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)] by (class, file)" .to_string(),
                                                                                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                  {
                                                                                                                      let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                                                                      Some(((::types::ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone())).into_ddvalue()))
                                                                                                                  }
                                                                                                                  __f},
                                                                                                                  next: Box::new(XFormArrangement::Join{
                                                                                                                                     description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)]".to_string(),
                                                                                                                                     ffun: None,
                                                                                                                                     arrangement: (Relations::inputs_Expression as RelId,0),
                                                                                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                     {
                                                                                                                                         let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                         let (ref decl_scope, ref declared_in) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                                                                                                             ::types::inputs::Expression{id: _, file: _, kind: _, scope: ref decl_scope, span: ref declared_in} => ((*decl_scope).clone(), (*declared_in).clone()),
                                                                                                                                             _ => return None
                                                                                                                                         };
                                                                                                                                         Some((::types::ddlog_std::tuple8((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*decl_scope).clone(), (*declared_in).clone())).into_ddvalue())
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                             description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)] by (used_scope, decl_scope, file)" .to_string(),
                                                                                                                                                             afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                             {
                                                                                                                                                                 let ::types::ddlog_std::tuple8(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref decl_scope, ref declared_in) = *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                 Some(((::types::ddlog_std::tuple3((*used_scope).clone(), (*decl_scope).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                             }
                                                                                                                                                             __f},
                                                                                                                                                             next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::VarDecl[(inputs::VarDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                                                                                                                                                                ffun: None,
                                                                                                                                                                                arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                {
                                                                                                                                                                                    let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_in, ref name, ref declared, ref declared_in) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                    Some(((::types::UseBeforeDecl{name: (*name).clone(), used: (*expr).clone(), used_in: (*used_in).clone(), declared: (*declared).clone(), declared_in: (*declared_in).clone(), file: (*file).clone()})).into_ddvalue())
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
                                },
                                /* UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                                Rule::ArrangementRule {
                                    description: "UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                    arr: ( Relations::__Prefix_0 as RelId, 0),
                                    xform: XFormArrangement::Join{
                                               description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)]".to_string(),
                                               ffun: None,
                                               arrangement: (Relations::inputs_LetDecl as RelId,1),
                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                               {
                                                   let (ref expr, ref file, ref object, ref used_scope, ref used_in, ref name, ref declared, ref decl) = match *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::StmtId>>::from_ddvalue_ref(__v1) } {
                                                       ::types::ddlog_std::tuple8(ref expr, ref file, ref object, ref used_scope, ref used_in, ref name, ref declared, ref decl) => ((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*decl).clone()),
                                                       _ => return None
                                                   };
                                                   let ref class = match *unsafe {<::types::inputs::LetDecl>::from_ddvalue_ref(__v2) } {
                                                       ::types::inputs::LetDecl{stmt_id: _, file: _, pattern: _, value: ::types::ddlog_std::Option::Some{x: ref class}, exported: _} => (*class).clone(),
                                                       _ => return None
                                                   };
                                                   Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue())
                                               }
                                               __f},
                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                       description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)] by (class, file)" .to_string(),
                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                       {
                                                                           let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                           Some(((::types::ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue()))
                                                                       }
                                                                       __f},
                                                                       next: Box::new(XFormArrangement::Semijoin{
                                                                                          description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)]".to_string(),
                                                                                          ffun: None,
                                                                                          arrangement: (Relations::inputs_ClassExpr as RelId,0),
                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                          {
                                                                                              let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                              Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue())
                                                                                          }
                                                                                          __f},
                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                  description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)] by (class, file)" .to_string(),
                                                                                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                  {
                                                                                                                      let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                                                                      Some(((::types::ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone())).into_ddvalue()))
                                                                                                                  }
                                                                                                                  __f},
                                                                                                                  next: Box::new(XFormArrangement::Join{
                                                                                                                                     description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)]".to_string(),
                                                                                                                                     ffun: None,
                                                                                                                                     arrangement: (Relations::inputs_Expression as RelId,0),
                                                                                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                     {
                                                                                                                                         let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                         let (ref decl_scope, ref declared_in) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                                                                                                             ::types::inputs::Expression{id: _, file: _, kind: _, scope: ref decl_scope, span: ref declared_in} => ((*decl_scope).clone(), (*declared_in).clone()),
                                                                                                                                             _ => return None
                                                                                                                                         };
                                                                                                                                         Some((::types::ddlog_std::tuple8((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*decl_scope).clone(), (*declared_in).clone())).into_ddvalue())
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                             description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)] by (used_scope, decl_scope, file)" .to_string(),
                                                                                                                                                             afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                             {
                                                                                                                                                                 let ::types::ddlog_std::tuple8(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref decl_scope, ref declared_in) = *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                 Some(((::types::ddlog_std::tuple3((*used_scope).clone(), (*decl_scope).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                             }
                                                                                                                                                             __f},
                                                                                                                                                             next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::LetDecl[(inputs::LetDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                                                                                                                                                                ffun: None,
                                                                                                                                                                                arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                {
                                                                                                                                                                                    let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_in, ref name, ref declared, ref declared_in) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                    Some(((::types::UseBeforeDecl{name: (*name).clone(), used: (*expr).clone(), used_in: (*used_in).clone(), declared: (*declared).clone(), declared_in: (*declared_in).clone(), file: (*file).clone()})).into_ddvalue())
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
                                },
                                /* UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]. */
                                Rule::ArrangementRule {
                                    description: "UseBeforeDecl[(UseBeforeDecl{.name=name, .used=expr, .used_in=used_in, .declared=declared, .declared_in=declared_in, .file=file}: UseBeforeDecl)] :- __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)].".to_string(),
                                    arr: ( Relations::__Prefix_0 as RelId, 0),
                                    xform: XFormArrangement::Join{
                                               description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)]".to_string(),
                                               ffun: None,
                                               arrangement: (Relations::inputs_ConstDecl as RelId,1),
                                               jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                               {
                                                   let (ref expr, ref file, ref object, ref used_scope, ref used_in, ref name, ref declared, ref decl) = match *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ExprId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::StmtId>>::from_ddvalue_ref(__v1) } {
                                                       ::types::ddlog_std::tuple8(ref expr, ref file, ref object, ref used_scope, ref used_in, ref name, ref declared, ref decl) => ((*expr).clone(), (*file).clone(), (*object).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*decl).clone()),
                                                       _ => return None
                                                   };
                                                   let ref class = match *unsafe {<::types::inputs::ConstDecl>::from_ddvalue_ref(__v2) } {
                                                       ::types::inputs::ConstDecl{stmt_id: _, file: _, pattern: _, value: ::types::ddlog_std::Option::Some{x: ref class}, exported: _} => (*class).clone(),
                                                       _ => return None
                                                   };
                                                   Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue())
                                               }
                                               __f},
                                               next: Box::new(Some(XFormCollection::Arrange {
                                                                       description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)] by (class, file)" .to_string(),
                                                                       afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                       {
                                                                           let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                           Some(((::types::ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue()))
                                                                       }
                                                                       __f},
                                                                       next: Box::new(XFormArrangement::Semijoin{
                                                                                          description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)]".to_string(),
                                                                                          ffun: None,
                                                                                          arrangement: (Relations::inputs_ClassExpr as RelId,0),
                                                                                          jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                          {
                                                                                              let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( __v1 ) };
                                                                                              Some((::types::ddlog_std::tuple7((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*class).clone())).into_ddvalue())
                                                                                          }
                                                                                          __f},
                                                                                          next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                  description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)] by (class, file)" .to_string(),
                                                                                                                  afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                  {
                                                                                                                      let ::types::ddlog_std::tuple7(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref class) = *unsafe {<::types::ddlog_std::tuple7<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ExprId>>::from_ddvalue_ref( &__v ) };
                                                                                                                      Some(((::types::ddlog_std::tuple2((*class).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone())).into_ddvalue()))
                                                                                                                  }
                                                                                                                  __f},
                                                                                                                  next: Box::new(XFormArrangement::Join{
                                                                                                                                     description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)]".to_string(),
                                                                                                                                     ffun: None,
                                                                                                                                     arrangement: (Relations::inputs_Expression as RelId,0),
                                                                                                                                     jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,__v2: &DDValue) -> Option<DDValue>
                                                                                                                                     {
                                                                                                                                         let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                         let (ref decl_scope, ref declared_in) = match *unsafe {<::types::inputs::Expression>::from_ddvalue_ref(__v2) } {
                                                                                                                                             ::types::inputs::Expression{id: _, file: _, kind: _, scope: ref decl_scope, span: ref declared_in} => ((*decl_scope).clone(), (*declared_in).clone()),
                                                                                                                                             _ => return None
                                                                                                                                         };
                                                                                                                                         Some((::types::ddlog_std::tuple8((*expr).clone(), (*file).clone(), (*used_scope).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*decl_scope).clone(), (*declared_in).clone())).into_ddvalue())
                                                                                                                                     }
                                                                                                                                     __f},
                                                                                                                                     next: Box::new(Some(XFormCollection::Arrange {
                                                                                                                                                             description: "arrange __Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)] by (used_scope, decl_scope, file)" .to_string(),
                                                                                                                                                             afun: &{fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                                             {
                                                                                                                                                                 let ::types::ddlog_std::tuple8(ref expr, ref file, ref used_scope, ref used_in, ref name, ref declared, ref decl_scope, ref declared_in) = *unsafe {<::types::ddlog_std::tuple8<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::ScopeId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::ScopeId, ::types::ast::Span>>::from_ddvalue_ref( &__v ) };
                                                                                                                                                                 Some(((::types::ddlog_std::tuple3((*used_scope).clone(), (*decl_scope).clone(), (*file).clone())).into_ddvalue(), (::types::ddlog_std::tuple6((*expr).clone(), (*file).clone(), (*used_in).clone(), (*name).clone(), (*declared).clone(), (*declared_in).clone())).into_ddvalue()))
                                                                                                                                                             }
                                                                                                                                                             __f},
                                                                                                                                                             next: Box::new(XFormArrangement::Semijoin{
                                                                                                                                                                                description: "__Prefix_0[((expr: ast::ExprId), (file: ast::FileId), (object: ast::ExprId), (used_scope: ast::ScopeId), (used_in: ast::Span), (name: internment::Intern<string>), (declared: ast::AnyId), (decl: ast::StmtId))], inputs::ConstDecl[(inputs::ConstDecl{.stmt_id=(decl: ast::StmtId), .file=(file: ast::FileId), .pattern=(_: ddlog_std::Option<ast::IPattern>), .value=(ddlog_std::Some{.x=(class: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl)], inputs::ClassExpr[(inputs::ClassExpr{.expr_id=(class: ast::ExprId), .file=(file: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr)], inputs::Expression[(inputs::Expression{.id=(class: ast::ExprId), .file=(file: ast::FileId), .kind=(_: ast::ExprKind), .scope=(decl_scope: ast::ScopeId), .span=(declared_in: ast::Span)}: inputs::Expression)], ChildScope[(ChildScope{.parent=(used_scope: ast::ScopeId), .child=(decl_scope: ast::ScopeId), .file=(file: ast::FileId)}: ChildScope)]".to_string(),
                                                                                                                                                                                ffun: None,
                                                                                                                                                                                arrangement: (Relations::ChildScope as RelId,1),
                                                                                                                                                                                jfun: &{fn __f(_: &DDValue ,__v1: &DDValue,___v2: &()) -> Option<DDValue>
                                                                                                                                                                                {
                                                                                                                                                                                    let ::types::ddlog_std::tuple6(ref expr, ref file, ref used_in, ref name, ref declared, ref declared_in) = *unsafe {<::types::ddlog_std::tuple6<::types::ast::ExprId, ::types::ast::FileId, ::types::ast::Span, ::types::internment::Intern<String>, ::types::ast::AnyId, ::types::ast::Span>>::from_ddvalue_ref( __v1 ) };
                                                                                                                                                                                    Some(((::types::UseBeforeDecl{name: (*name).clone(), used: (*expr).clone(), used_in: (*used_in).clone(), declared: (*declared).clone(), declared_in: (*declared_in).clone(), file: (*file).clone()})).into_ddvalue())
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
                                ],
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
    Program {
        nodes: vec![
            ProgNode::Rel{rel: inputs_Array},
            ProgNode::Rel{rel: inputs_Arrow},
            ProgNode::Rel{rel: inputs_ArrowParam},
            ProgNode::Rel{rel: inputs_Assign},
            ProgNode::Rel{rel: inputs_Await},
            ProgNode::Rel{rel: inputs_BinOp},
            ProgNode::Rel{rel: inputs_BracketAccess},
            ProgNode::Rel{rel: inputs_Break},
            ProgNode::Rel{rel: inputs_Call},
            ProgNode::Rel{rel: inputs_Class},
            ProgNode::Rel{rel: inputs_ClassExpr},
            ProgNode::Rel{rel: inputs_ConstDecl},
            ProgNode::Rel{rel: inputs_Continue},
            ProgNode::Rel{rel: inputs_DoWhile},
            ProgNode::Rel{rel: inputs_DotAccess},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: ChainedWith, distinct: true}]},
            ProgNode::Rel{rel: inputs_EveryScope},
            ProgNode::Rel{rel: inputs_ExprBigInt},
            ProgNode::Rel{rel: inputs_ExprBool},
            ProgNode::Rel{rel: inputs_ExprNumber},
            ProgNode::Rel{rel: inputs_ExprString},
            ProgNode::Rel{rel: inputs_Expression},
            ProgNode::Rel{rel: inputs_File},
            ProgNode::Rel{rel: inputs_FileExport},
            ProgNode::Rel{rel: inputs_For},
            ProgNode::Rel{rel: inputs_ForIn},
            ProgNode::Rel{rel: inputs_Function},
            ProgNode::Rel{rel: inputs_FunctionArg},
            ProgNode::Rel{rel: inputs_If},
            ProgNode::Rel{rel: inputs_ImplicitGlobal},
            ProgNode::Rel{rel: inputs_ImportDecl},
            ProgNode::Rel{rel: inputs_InlineFunc},
            ProgNode::Rel{rel: inputs_InlineFuncParam},
            ProgNode::Rel{rel: inputs_InputScope},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: ChildScope, distinct: true}]},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: FunctionLevelScope, distinct: true}]},
            ProgNode::Rel{rel: inputs_Label},
            ProgNode::Rel{rel: inputs_LetDecl},
            ProgNode::Rel{rel: inputs_NameRef},
            ProgNode::Rel{rel: inputs_New},
            ProgNode::Rel{rel: __Prefix_1},
            ProgNode::Rel{rel: inputs_Property},
            ProgNode::Rel{rel: inputs_Return},
            ProgNode::Rel{rel: inputs_Statement},
            ProgNode::Rel{rel: inputs_Switch},
            ProgNode::Rel{rel: inputs_SwitchCase},
            ProgNode::Rel{rel: inputs_Template},
            ProgNode::Rel{rel: inputs_Ternary},
            ProgNode::Rel{rel: inputs_Throw},
            ProgNode::Rel{rel: inputs_Try},
            ProgNode::Rel{rel: inputs_UnaryOp},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: WithinTypeofExpr, distinct: true}]},
            ProgNode::Rel{rel: inputs_VarDecl},
            ProgNode::SCC{rels: vec![RecursiveRelation{rel: NameInScope, distinct: true}]},
            ProgNode::Rel{rel: NoUndef},
            ProgNode::Rel{rel: IsExported},
            ProgNode::Rel{rel: TypeofUndef},
            ProgNode::Rel{rel: VariableUsages},
            ProgNode::Rel{rel: UnusedVariables},
            ProgNode::Rel{rel: __Prefix_0},
            ProgNode::Rel{rel: UseBeforeDecl},
            ProgNode::Rel{rel: inputs_While},
            ProgNode::Rel{rel: inputs_With},
            ProgNode::Rel{rel: inputs_Yield}
        ],
        init_data: vec![
        ]
    }
}