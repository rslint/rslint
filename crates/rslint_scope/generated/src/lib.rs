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
use std::{any::TypeId, sync};

use ordered_float::*;

use differential_dataflow::collection;
use timely::communication;
use timely::dataflow::scopes;
use timely::worker;

use differential_datalog::ddval::*;
use differential_datalog::program;
use differential_datalog::record;
use differential_datalog::record::FromRecord;
use differential_datalog::record::IntoRecord;
use differential_datalog::record::RelIdentifier;
use differential_datalog::record::UpdCmd;
use differential_datalog::DDlogConvert;
use num_traits::cast::FromPrimitive;
use num_traits::identities::One;
use once_cell::sync::Lazy;

use fnv::FnvHashMap;

pub mod api;
pub mod ovsdb_api;
pub mod update_handler;

use crate::api::updcmd2upd;

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
    fn relid2name(relId: program::RelId) -> Option<&'static str> {
        relid2name(relId)
    }

    fn indexid2name(idxId: program::IdxId) -> Option<&'static str> {
        indexid2name(idxId)
    }

    fn updcmd2upd(upd_cmd: &UpdCmd) -> ::std::result::Result<program::Update<DDValue>, String> {
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
pub struct UpdateSerializer(program::Update<DDValue>);

impl From<program::Update<DDValue>> for UpdateSerializer {
    fn from(u: program::Update<DDValue>) -> Self {
        UpdateSerializer(u)
    }
}
impl From<UpdateSerializer> for program::Update<DDValue> {
    fn from(u: UpdateSerializer) -> Self {
        u.0
    }
}

impl Serialize for UpdateSerializer {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tup = serializer.serialize_tuple(3)?;
        match &self.0 {
            program::Update::Insert { relid, v } => {
                tup.serialize_element(&true)?;
                tup.serialize_element(relid)?;
                tup.serialize_element(v)?;
            }
            program::Update::DeleteValue { relid, v } => {
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
                        let relid = seq.next_element::<program::RelId>()?.ok_or_else(|| <A::Error as ::serde::de::Error>::custom("Missing relation id"))?;
                        match relid {
                            $(
                                $rel => {
                                    let v = seq.next_element::<$typ>()?.ok_or_else(|| <A::Error as ::serde::de::Error>::custom("Missing value"))?.into_ddvalue();
                                    if polarity {
                                        Ok(UpdateSerializer(program::Update::Insert{relid, v}))
                                    } else {
                                        Ok(UpdateSerializer(program::Update::DeleteValue{relid, v}))
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

#[cfg(feature = "flatbuf")]
pub mod flatbuf_generated;

impl TryFrom<&RelIdentifier> for Relations {
    type Error = ();

    fn try_from(rel_id: &RelIdentifier) -> ::std::result::Result<Self, ()> {
        match rel_id {
            RelIdentifier::RelName(rname) => Relations::try_from(rname.as_ref()),
            RelIdentifier::RelId(id) => Relations::try_from(*id),
        }
    }
}


pub mod typedefs
{
    pub mod ast
    {
        pub use ::types__ast::UnaryOperand;
        pub use ::types__ast::TryHandler;
        pub use ::types__ast::SwitchClause;
        pub use ::types__ast::StmtKind;
        pub use ::types__ast::StmtId;
        pub use ::types__ast::Spanned;
        pub use ::types__ast::Span;
        pub use ::types__ast::ScopeId;
        pub use ::types__ast::PropertyVal;
        pub use ::types__ast::PropertyKey;
        pub use ::types__ast::Pattern;
        pub use ::types__ast::OneOf;
        pub use ::types__ast::ObjectPatternProp;
        pub use ::types__ast::NamedImport;
        pub use ::types__ast::Name;
        pub use ::types__ast::LitKind;
        pub use ::types__ast::JSFlavor;
        pub use ::types__ast::ImportId;
        pub use ::types__ast::ImportClause;
        pub use ::types__ast::ImplicitGlobalId;
        pub use ::types__ast::IPattern;
        pub use ::types__ast::IObjectPatternProp;
        pub use ::types__ast::IClassElement;
        pub use ::types__ast::GlobalPriv;
        pub use ::types__ast::GlobalId;
        pub use ::types__ast::FuncParam;
        pub use ::types__ast::FuncId;
        pub use ::types__ast::ForInit;
        pub use ::types__ast::FileKind;
        pub use ::types__ast::FileId;
        pub use ::types__ast::ExprKind;
        pub use ::types__ast::ExprId;
        pub use ::types__ast::ExportKind;
        pub use ::types__ast::ClassId;
        pub use ::types__ast::ClassElement;
        pub use ::types__ast::BinOperand;
        pub use ::types__ast::AssignOperand;
        pub use ::types__ast::ArrayElement;
        pub use ::types__ast::AnyId;
        pub use ::types__ast::to_string_ast_Span___Stringval;
        pub use ::types__ast::to_string_ast_AnyId___Stringval;
        pub use ::types__ast::to_string_ast_ScopeId___Stringval;
        pub use ::types__ast::method_comps_ast_ClassElement_ddlog_std_Option____Tuple2__ddlog_std_Vec__ast_FuncParam_ast_StmtId;
        pub use ::types__ast::method_comps_ast_PropertyVal_ddlog_std_Option____Tuple2__ddlog_std_Vec__ast_FuncParam_ast_StmtId;
        pub use ::types__ast::is_variable_decl;
        pub use ::types__ast::is_global;
        pub use ::types__ast::is_function;
        pub use ::types__ast::is_expr;
        pub use ::types__ast::free_variables;
        pub use ::types__ast::free_variable;
        pub use ::types__ast::file;
        pub use ::types__ast::bound_vars_ast_FuncParam_ddlog_std_Vec____Tuple2__ast_Spanned__internment_Intern____Stringval___Boolval;
        pub use ::types__ast::bound_vars_internment_Intern__ast_ObjectPatternProp_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval;
        pub use ::types__ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval;
        pub use ::types__ast::body_ast_ClassElement_ddlog_std_Option__ast_StmtId;
        pub use ::types__ast::body_ast_PropertyVal_ddlog_std_Option__ast_StmtId;
        pub use ::types__ast::any_id;
    }
    pub mod config
    {
        pub use ::types__config::NoUseBeforeDefConfig;
        pub use ::types__config::NoUnusedVarsConfig;
        pub use ::types__config::NoUnusedLabelsConfig;
        pub use ::types__config::NoUndefConfig;
        pub use ::types__config::NoTypeofUndefConfig;
        pub use ::types__config::NoShadowHoisting;
        pub use ::types__config::NoShadowConfig;
        pub use ::types__config::IgnoreArgs;
        pub use ::types__config::EnableNoUseBeforeDef;
        pub use ::types__config::EnableNoUnusedVars;
        pub use ::types__config::EnableNoUnusedLabels;
        pub use ::types__config::EnableNoUndef;
        pub use ::types__config::EnableNoTypeofUndef;
        pub use ::types__config::EnableNoShadow;
        pub use ::types__config::CaughtErrors;
        pub use ::types__config::ignored_patterns;
        pub use ::types__config::hoisting_never;
        pub use ::types__config::hoisting_functions;
        pub use ::types__config::hoisting_enabled;
        pub use ::types__config::hoisting_always;
    }
    pub mod ddlog_std
    {
        pub use ::ddlog_std::s8;
        pub use ::ddlog_std::s64;
        pub use ::ddlog_std::s32;
        pub use ::ddlog_std::s16;
        pub use ::ddlog_std::s128;
        pub use ::ddlog_std::Vec;
        pub use ::ddlog_std::Set;
        pub use ::ddlog_std::Result;
        pub use ::ddlog_std::Ref;
        pub use ::ddlog_std::Option;
        pub use ::ddlog_std::Map;
        pub use ::ddlog_std::Group;
        pub use ::ddlog_std::Either;
        pub use ::ddlog_std::DDlogGroup;
        pub use ::ddlog_std::DDWeight;
        pub use ::ddlog_std::DDNestedTS;
        pub use ::ddlog_std::DDIteration;
        pub use ::ddlog_std::DDEpoch;
        pub use ::ddlog_std::zip;
        pub use ::ddlog_std::vec_zip;
        pub use ::ddlog_std::vec_with_length;
        pub use ::ddlog_std::vec_with_capacity;
        pub use ::ddlog_std::vec_update_nth;
        pub use ::ddlog_std::vec_truncate;
        pub use ::ddlog_std::vec_to_set;
        pub use ::ddlog_std::vec_swap_nth;
        pub use ::ddlog_std::vec_sort_imm;
        pub use ::ddlog_std::vec_sort;
        pub use ::ddlog_std::vec_singleton;
        pub use ::ddlog_std::vec_resize;
        pub use ::ddlog_std::vec_push_imm;
        pub use ::ddlog_std::vec_push;
        pub use ::ddlog_std::vec_pop;
        pub use ::ddlog_std::vec_nth;
        pub use ::ddlog_std::vec_len;
        pub use ::ddlog_std::vec_is_empty;
        pub use ::ddlog_std::vec_empty;
        pub use ::ddlog_std::vec_contains;
        pub use ::ddlog_std::vec_append;
        pub use ::ddlog_std::update_nth;
        pub use ::ddlog_std::unzip;
        pub use ::ddlog_std::unwrap_or_default_ddlog_std_Result__V_E_V;
        pub use ::ddlog_std::unwrap_or_default_ddlog_std_Option__A_A;
        pub use ::ddlog_std::unwrap_or_ddlog_std_Result__V_E_V_V;
        pub use ::ddlog_std::unwrap_or_ddlog_std_Option__A_A_A;
        pub use ::ddlog_std::unions;
        pub use ::ddlog_std::union_ddlog_std_Vec__ddlog_std_Set__X_ddlog_std_Set__X;
        pub use ::ddlog_std::union_ddlog_std_Set__X_ddlog_std_Set__X_ddlog_std_Set__X;
        pub use ::ddlog_std::union_ddlog_std_Map__K_V_ddlog_std_Map__K_V_ddlog_std_Map__K_V;
        pub use ::ddlog_std::union_ddlog_std_Group__K_ddlog_std_Ref__ddlog_std_Set__A_ddlog_std_Ref__ddlog_std_Set__A;
        pub use ::ddlog_std::union_ddlog_std_Group__K_ddlog_std_Set__A_ddlog_std_Set__A;
        pub use ::ddlog_std::u8_pow32;
        pub use ::ddlog_std::u64_pow32;
        pub use ::ddlog_std::u32_pow32;
        pub use ::ddlog_std::u16_pow32;
        pub use ::ddlog_std::u128_pow32;
        pub use ::ddlog_std::truncate;
        pub use ::ddlog_std::trim;
        pub use ::ddlog_std::to_vec_ddlog_std_Set__A_ddlog_std_Vec__A;
        pub use ::ddlog_std::to_vec_ddlog_std_Group__K_V_ddlog_std_Vec__V;
        pub use ::ddlog_std::to_vec_ddlog_std_Option__X_ddlog_std_Vec__X;
        pub use ::ddlog_std::to_uppercase;
        pub use ::ddlog_std::to_string___Stringval___Stringval;
        pub use ::ddlog_std::to_string___Bitval128___Stringval;
        pub use ::ddlog_std::to_string___Bitval64___Stringval;
        pub use ::ddlog_std::to_string___Bitval32___Stringval;
        pub use ::ddlog_std::to_string___Bitval16___Stringval;
        pub use ::ddlog_std::to_string___Bitval8___Stringval;
        pub use ::ddlog_std::to_string___Signedval128___Stringval;
        pub use ::ddlog_std::to_string___Signedval64___Stringval;
        pub use ::ddlog_std::to_string___Signedval32___Stringval;
        pub use ::ddlog_std::to_string___Signedval16___Stringval;
        pub use ::ddlog_std::to_string___Signedval8___Stringval;
        pub use ::ddlog_std::to_string___Doubleval___Stringval;
        pub use ::ddlog_std::to_string___Floatval___Stringval;
        pub use ::ddlog_std::to_string___Intval___Stringval;
        pub use ::ddlog_std::to_string___Boolval___Stringval;
        pub use ::ddlog_std::to_string_ddlog_std_DDNestedTS___Stringval;
        pub use ::ddlog_std::to_setmap;
        pub use ::ddlog_std::to_set_ddlog_std_Vec__A_ddlog_std_Set__A;
        pub use ::ddlog_std::to_set_ddlog_std_Group__K_V_ddlog_std_Set__V;
        pub use ::ddlog_std::to_set_ddlog_std_Option__X_ddlog_std_Set__X;
        pub use ::ddlog_std::to_map_ddlog_std_Vec____Tuple2__K_V_ddlog_std_Map__K_V;
        pub use ::ddlog_std::to_map_ddlog_std_Group__K1___Tuple2__K2_V_ddlog_std_Map__K2_V;
        pub use ::ddlog_std::to_lowercase;
        pub use ::ddlog_std::to_bytes;
        pub use ::ddlog_std::swap_nth;
        pub use ::ddlog_std::substr;
        pub use ::ddlog_std::string_trim;
        pub use ::ddlog_std::string_to_uppercase;
        pub use ::ddlog_std::string_to_lowercase;
        pub use ::ddlog_std::string_to_bytes;
        pub use ::ddlog_std::string_substr;
        pub use ::ddlog_std::string_starts_with;
        pub use ::ddlog_std::string_split;
        pub use ::ddlog_std::string_reverse;
        pub use ::ddlog_std::string_replace;
        pub use ::ddlog_std::string_len;
        pub use ::ddlog_std::string_join;
        pub use ::ddlog_std::string_ends_with;
        pub use ::ddlog_std::string_contains;
        pub use ::ddlog_std::str_to_lower;
        pub use ::ddlog_std::starts_with;
        pub use ::ddlog_std::split;
        pub use ::ddlog_std::sort_imm;
        pub use ::ddlog_std::sort;
        pub use ::ddlog_std::size_ddlog_std_Set__X___Bitval64;
        pub use ::ddlog_std::size_ddlog_std_Map__K_V___Bitval64;
        pub use ::ddlog_std::size_ddlog_std_Group__K_V___Bitval64;
        pub use ::ddlog_std::setref_unions;
        pub use ::ddlog_std::set_unions;
        pub use ::ddlog_std::set_union;
        pub use ::ddlog_std::set_to_vec;
        pub use ::ddlog_std::set_size;
        pub use ::ddlog_std::set_singleton;
        pub use ::ddlog_std::set_nth;
        pub use ::ddlog_std::set_is_empty;
        pub use ::ddlog_std::set_intersection;
        pub use ::ddlog_std::set_insert_imm;
        pub use ::ddlog_std::set_insert;
        pub use ::ddlog_std::set_empty;
        pub use ::ddlog_std::set_difference;
        pub use ::ddlog_std::set_contains;
        pub use ::ddlog_std::s8_pow32;
        pub use ::ddlog_std::s64_pow32;
        pub use ::ddlog_std::s32_pow32;
        pub use ::ddlog_std::s16_pow32;
        pub use ::ddlog_std::s128_pow32;
        pub use ::ddlog_std::reverse;
        pub use ::ddlog_std::result_unwrap_or_default;
        pub use ::ddlog_std::resize;
        pub use ::ddlog_std::replace;
        pub use ::ddlog_std::remove;
        pub use ::ddlog_std::ref_new;
        pub use ::ddlog_std::range_vec;
        pub use ::ddlog_std::push_imm;
        pub use ::ddlog_std::push;
        pub use ::ddlog_std::pow32___Intval___Bitval32___Intval;
        pub use ::ddlog_std::pow32___Signedval128___Bitval32___Signedval128;
        pub use ::ddlog_std::pow32___Signedval64___Bitval32___Signedval64;
        pub use ::ddlog_std::pow32___Signedval32___Bitval32___Signedval32;
        pub use ::ddlog_std::pow32___Signedval16___Bitval32___Signedval16;
        pub use ::ddlog_std::pow32___Signedval8___Bitval32___Signedval8;
        pub use ::ddlog_std::pow32___Bitval128___Bitval32___Bitval128;
        pub use ::ddlog_std::pow32___Bitval64___Bitval32___Bitval64;
        pub use ::ddlog_std::pow32___Bitval32___Bitval32___Bitval32;
        pub use ::ddlog_std::pow32___Bitval16___Bitval32___Bitval16;
        pub use ::ddlog_std::pow32___Bitval8___Bitval32___Bitval8;
        pub use ::ddlog_std::pop;
        pub use ::ddlog_std::parse_dec_u64;
        pub use ::ddlog_std::parse_dec_i64;
        pub use ::ddlog_std::option_unwrap_or_default;
        pub use ::ddlog_std::ok_or_else;
        pub use ::ddlog_std::ok_or;
        pub use ::ddlog_std::ntohs;
        pub use ::ddlog_std::ntohl;
        pub use ::ddlog_std::nth_ddlog_std_Set__X___Bitval64_ddlog_std_Option__X;
        pub use ::ddlog_std::nth_ddlog_std_Vec__X___Bitval64_ddlog_std_Option__X;
        pub use ::ddlog_std::nth_ddlog_std_Group__K_V___Bitval64_ddlog_std_Option__V;
        pub use ::ddlog_std::min_ddlog_std_Group__K_V_V;
        pub use ::ddlog_std::min_A_A_A;
        pub use ::ddlog_std::max_ddlog_std_Group__K_V_V;
        pub use ::ddlog_std::max_A_A_A;
        pub use ::ddlog_std::map_union;
        pub use ::ddlog_std::map_size;
        pub use ::ddlog_std::map_singleton;
        pub use ::ddlog_std::map_remove;
        pub use ::ddlog_std::map_keys;
        pub use ::ddlog_std::map_is_empty;
        pub use ::ddlog_std::map_insert_imm;
        pub use ::ddlog_std::map_insert;
        pub use ::ddlog_std::map_get;
        pub use ::ddlog_std::map_err;
        pub use ::ddlog_std::map_empty;
        pub use ::ddlog_std::map_contains_key;
        pub use ::ddlog_std::map_ddlog_std_Result__V1_E___Closureimm_V1_ret_V2_ddlog_std_Result__V2_E;
        pub use ::ddlog_std::map_ddlog_std_Option__A___Closureimm_A_ret_B_ddlog_std_Option__B;
        pub use ::ddlog_std::len_ddlog_std_Vec__X___Bitval64;
        pub use ::ddlog_std::len___Stringval___Bitval64;
        pub use ::ddlog_std::keys;
        pub use ::ddlog_std::key;
        pub use ::ddlog_std::join;
        pub use ::ddlog_std::is_some;
        pub use ::ddlog_std::is_ok;
        pub use ::ddlog_std::is_none;
        pub use ::ddlog_std::is_err;
        pub use ::ddlog_std::is_empty_ddlog_std_Set__X___Boolval;
        pub use ::ddlog_std::is_empty_ddlog_std_Map__K_V___Boolval;
        pub use ::ddlog_std::is_empty_ddlog_std_Vec__X___Boolval;
        pub use ::ddlog_std::intersection;
        pub use ::ddlog_std::insert_imm_ddlog_std_Set__X_X_ddlog_std_Set__X;
        pub use ::ddlog_std::insert_imm_ddlog_std_Map__K_V_K_V_ddlog_std_Map__K_V;
        pub use ::ddlog_std::insert_ddlog_std_Set__X_X___Tuple0__;
        pub use ::ddlog_std::insert_ddlog_std_Map__K_V_K_V___Tuple0__;
        pub use ::ddlog_std::htons;
        pub use ::ddlog_std::htonl;
        pub use ::ddlog_std::hex;
        pub use ::ddlog_std::hash64;
        pub use ::ddlog_std::hash128;
        pub use ::ddlog_std::group_unzip;
        pub use ::ddlog_std::group_to_vec;
        pub use ::ddlog_std::group_to_setmap;
        pub use ::ddlog_std::group_to_set;
        pub use ::ddlog_std::group_to_map;
        pub use ::ddlog_std::group_sum;
        pub use ::ddlog_std::group_setref_unions;
        pub use ::ddlog_std::group_set_unions;
        pub use ::ddlog_std::group_nth;
        pub use ::ddlog_std::group_min;
        pub use ::ddlog_std::group_max;
        pub use ::ddlog_std::group_key;
        pub use ::ddlog_std::group_first;
        pub use ::ddlog_std::group_count;
        pub use ::ddlog_std::get;
        pub use ::ddlog_std::first;
        pub use ::ddlog_std::ends_with;
        pub use ::ddlog_std::difference;
        pub use ::ddlog_std::deref;
        pub use ::ddlog_std::default;
        pub use ::ddlog_std::count;
        pub use ::ddlog_std::contains_key;
        pub use ::ddlog_std::contains_ddlog_std_Set__X_X___Boolval;
        pub use ::ddlog_std::contains_ddlog_std_Vec__X_X___Boolval;
        pub use ::ddlog_std::contains___Stringval___Stringval___Boolval;
        pub use ::ddlog_std::bigint_pow32;
        pub use ::ddlog_std::append;
        pub use ::ddlog_std::and_then;
        pub use ::ddlog_std::__builtin_2string;
    }
    pub mod debug
    {
        pub use ::debug::DDlogOpId;
        pub use ::debug::debug_split_group;
        pub use ::debug::debug_event_join;
        pub use ::debug::debug_event;
    }
    pub mod group
    {
        pub use ::types__group::map;
        pub use ::types__group::fold;
        pub use ::types__group::flatmap;
        pub use ::types__group::find;
        pub use ::types__group::filter_map;
        pub use ::types__group::filter;
        pub use ::types__group::count;
        pub use ::types__group::arg_min;
        pub use ::types__group::arg_max;
        pub use ::types__group::any;
        pub use ::types__group::all;
    }
    pub mod inputs
    {
        pub use ::types__inputs::Yield;
        pub use ::types__inputs::With;
        pub use ::types__inputs::While;
        pub use ::types__inputs::VarDecl;
        pub use ::types__inputs::UserGlobal;
        pub use ::types__inputs::UnaryOp;
        pub use ::types__inputs::Try;
        pub use ::types__inputs::Throw;
        pub use ::types__inputs::Ternary;
        pub use ::types__inputs::Template;
        pub use ::types__inputs::SwitchCase;
        pub use ::types__inputs::Switch;
        pub use ::types__inputs::Statement;
        pub use ::types__inputs::Return;
        pub use ::types__inputs::Property;
        pub use ::types__inputs::New;
        pub use ::types__inputs::NameRef;
        pub use ::types__inputs::LetDecl;
        pub use ::types__inputs::Label;
        pub use ::types__inputs::InputScope;
        pub use ::types__inputs::InlineFuncParam;
        pub use ::types__inputs::InlineFunc;
        pub use ::types__inputs::ImportDecl;
        pub use ::types__inputs::ImplicitGlobal;
        pub use ::types__inputs::If;
        pub use ::types__inputs::FunctionArg;
        pub use ::types__inputs::Function;
        pub use ::types__inputs::ForOf;
        pub use ::types__inputs::ForIn;
        pub use ::types__inputs::For;
        pub use ::types__inputs::FileExport;
        pub use ::types__inputs::File;
        pub use ::types__inputs::Expression;
        pub use ::types__inputs::ExprString;
        pub use ::types__inputs::ExprNumber;
        pub use ::types__inputs::ExprBool;
        pub use ::types__inputs::ExprBigInt;
        pub use ::types__inputs::EveryScope;
        pub use ::types__inputs::DotAccess;
        pub use ::types__inputs::DoWhile;
        pub use ::types__inputs::Continue;
        pub use ::types__inputs::ConstDecl;
        pub use ::types__inputs::ClassExpr;
        pub use ::types__inputs::Class;
        pub use ::types__inputs::Call;
        pub use ::types__inputs::Break;
        pub use ::types__inputs::BracketAccess;
        pub use ::types__inputs::BinOp;
        pub use ::types__inputs::Await;
        pub use ::types__inputs::Assign;
        pub use ::types__inputs::ArrowParam;
        pub use ::types__inputs::Arrow;
        pub use ::types__inputs::Array;
    }
    pub mod internment
    {
        pub use ::internment::istring;
        pub use ::internment::Intern;
        pub use ::internment::trim;
        pub use ::internment::to_uppercase;
        pub use ::internment::to_string;
        pub use ::internment::to_lowercase;
        pub use ::internment::to_bytes;
        pub use ::internment::substr;
        pub use ::internment::starts_with;
        pub use ::internment::split;
        pub use ::internment::reverse;
        pub use ::internment::replace;
        pub use ::internment::len;
        pub use ::internment::join;
        pub use ::internment::ival;
        pub use ::internment::istring_trim;
        pub use ::internment::istring_to_uppercase;
        pub use ::internment::istring_to_lowercase;
        pub use ::internment::istring_to_bytes;
        pub use ::internment::istring_substr;
        pub use ::internment::istring_starts_with;
        pub use ::internment::istring_split;
        pub use ::internment::istring_reverse;
        pub use ::internment::istring_replace;
        pub use ::internment::istring_len;
        pub use ::internment::istring_join;
        pub use ::internment::istring_ends_with;
        pub use ::internment::istring_contains;
        pub use ::internment::intern;
        pub use ::internment::ends_with;
        pub use ::internment::contains;
    }
    pub mod is_exported
    {
        pub use ::types::is_exported::IsExported;
    }
    pub mod name_in_scope
    {
        pub use ::types::name_in_scope::ScopeOfDeclName;
        pub use ::types::name_in_scope::NameOrigin;
        pub use ::types::name_in_scope::NameOccursInScope;
        pub use ::types::name_in_scope::NameInScope;
    }
    pub mod outputs
    {
        pub mod no_shadow
        {
            pub use ::types::outputs::no_shadow::ScopeOfDecl;
            pub use ::types::outputs::no_shadow::NoShadow;
            pub use ::types::outputs::no_shadow::DeclarationInDescendent;
        }
        pub mod no_typeof_undef
        {
            pub use ::types::outputs::no_typeof_undef::WithinTypeofExpr;
            pub use ::types::outputs::no_typeof_undef::NoTypeofUndef;
            pub use ::types::outputs::no_typeof_undef::NeedsWithinTypeofExpr;
        }
        pub mod no_undef
        {
            pub use ::types::outputs::no_undef::NoUndef;
            pub use ::types::outputs::no_undef::ChainedWith;
        }
        pub mod no_unused_labels
        {
            pub use ::types__outputs__no_unused_labels::UsedLabels;
            pub use ::types__outputs__no_unused_labels::NoUnusedLabels;
            pub use ::types__outputs__no_unused_labels::LabelUsage;
        }
        pub mod no_use_before_def
        {
            pub use ::types::outputs::no_use_before_def::NoUseBeforeDef;
        }
        pub mod unused_vars
        {
            pub use ::types::outputs::unused_vars::UnusedVariables;
            pub use ::types::outputs::unused_vars::FunctionBodyScope;
        }
    }
    pub mod regex
    {
        pub use ::types__regex::RegexSet;
        pub use ::types__regex::Regex;
        pub use ::types__regex::try_regex_set;
        pub use ::types__regex::try_regex;
        pub use ::types__regex::regex_set_match;
        pub use ::types__regex::regex_set;
        pub use ::types__regex::regex_match;
        pub use ::types__regex::regex_first_match;
        pub use ::types__regex::regex_all_matches;
        pub use ::types__regex::regex;
    }
    pub mod scopes
    {
        pub use ::types__scopes::ScopeOfId;
        pub use ::types__scopes::ScopeFamily;
        pub use ::types__scopes::NeedsScopeParents;
        pub use ::types__scopes::NeedsScopeChildren;
        pub use ::types__scopes::IsHoistable;
        pub use ::types__scopes::FunctionLevelScope;
    }
    pub mod utils
    {
        pub use ::types__utils::or_else;
        pub use ::types__utils::debug;
        pub use ::types__utils::dbg;
    }
    pub mod var_decls
    {
        pub use ::types::var_decls::VariableMeta;
        pub use ::types::var_decls::VariableDeclarations;
        pub use ::types::var_decls::DeclarationScope;
        pub use ::types::var_decls::unhoisted_scope;
        pub use ::types::var_decls::is_unhoistable;
        pub use ::types::var_decls::is_hoistable;
        pub use ::types::var_decls::hoisted_scope;
    }
    pub mod variable_decl
    {
        pub use ::types__variable_decl::VariableDeclKind;
        pub use ::types__variable_decl::VariableDecl;
    }
    pub mod vec
    {
        pub use ::types__vec::vec_sort_by;
        pub use ::types__vec::vec_arg_min;
        pub use ::types__vec::vec_arg_max;
        pub use ::types__vec::sort_by;
        pub use ::types__vec::retain;
        pub use ::types__vec::map;
        pub use ::types__vec::last;
        pub use ::types__vec::fold;
        pub use ::types__vec::flatmap;
        pub use ::types__vec::first;
        pub use ::types__vec::find;
        pub use ::types__vec::filter_map;
        pub use ::types__vec::filter;
        pub use ::types__vec::count;
        pub use ::types__vec::arg_min;
        pub use ::types__vec::arg_max;
        pub use ::types__vec::any;
        pub use ::types__vec::all;
    }
}
decl_update_deserializer!(UpdateSerializer,(1, types__config::EnableNoShadow), (2, types__config::EnableNoTypeofUndef), (3, types__config::EnableNoUndef), (4, types__config::EnableNoUnusedLabels), (5, types__config::EnableNoUnusedVars), (6, types__config::EnableNoUseBeforeDef), (7, types__inputs::Array), (8, types__inputs::Arrow), (9, types__inputs::ArrowParam), (10, types__inputs::Assign), (11, types__inputs::Await), (12, types__inputs::BinOp), (13, types__inputs::BracketAccess), (14, types__inputs::Break), (15, types__inputs::Call), (16, types__inputs::Class), (17, types__inputs::ClassExpr), (18, types__inputs::ConstDecl), (19, types__inputs::Continue), (20, types__inputs::DoWhile), (21, types__inputs::DotAccess), (22, types__inputs::EveryScope), (23, types__inputs::ExprBigInt), (24, types__inputs::ExprBool), (25, types__inputs::ExprNumber), (26, types__inputs::ExprString), (27, types__inputs::Expression), (28, types__inputs::File), (29, types__inputs::FileExport), (30, types__inputs::For), (31, types__inputs::ForIn), (32, types__inputs::ForOf), (33, types__inputs::Function), (34, types__inputs::FunctionArg), (35, types__inputs::If), (36, types__inputs::ImplicitGlobal), (37, types__inputs::ImportDecl), (38, types__inputs::InlineFunc), (39, types__inputs::InlineFuncParam), (40, types__inputs::InputScope), (41, types__inputs::Label), (42, types__inputs::LetDecl), (43, types__inputs::NameRef), (44, types__inputs::New), (45, types__inputs::Property), (46, types__inputs::Return), (47, types__inputs::Statement), (48, types__inputs::Switch), (49, types__inputs::SwitchCase), (50, types__inputs::Template), (51, types__inputs::Ternary), (52, types__inputs::Throw), (53, types__inputs::Try), (54, types__inputs::UnaryOp), (55, types__inputs::UserGlobal), (56, types__inputs::VarDecl), (57, types__inputs::While), (58, types__inputs::With), (59, types__inputs::Yield), (65, types::outputs::no_shadow::NoShadow), (68, types::outputs::no_typeof_undef::NoTypeofUndef), (71, types::outputs::no_undef::NoUndef), (73, types__outputs__no_unused_labels::NoUnusedLabels), (74, types__outputs__no_unused_labels::UsedLabels), (76, types::outputs::no_use_before_def::NoUseBeforeDef), (78, types::outputs::unused_vars::UnusedVariables));
impl TryFrom<&str> for Relations {
    type Error = ();
    fn try_from(rname: &str) -> ::std::result::Result<Self, ()> {
         match rname {
        "__Prefix_0" => Ok(Relations::__Prefix_0),
        "config::EnableNoShadow" => Ok(Relations::config_EnableNoShadow),
        "config::EnableNoTypeofUndef" => Ok(Relations::config_EnableNoTypeofUndef),
        "config::EnableNoUndef" => Ok(Relations::config_EnableNoUndef),
        "config::EnableNoUnusedLabels" => Ok(Relations::config_EnableNoUnusedLabels),
        "config::EnableNoUnusedVars" => Ok(Relations::config_EnableNoUnusedVars),
        "config::EnableNoUseBeforeDef" => Ok(Relations::config_EnableNoUseBeforeDef),
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
        "inputs::ForOf" => Ok(Relations::inputs_ForOf),
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
        "inputs::UserGlobal" => Ok(Relations::inputs_UserGlobal),
        "inputs::VarDecl" => Ok(Relations::inputs_VarDecl),
        "inputs::While" => Ok(Relations::inputs_While),
        "inputs::With" => Ok(Relations::inputs_With),
        "inputs::Yield" => Ok(Relations::inputs_Yield),
        "is_exported::IsExported" => Ok(Relations::is_exported_IsExported),
        "name_in_scope::NameInScope" => Ok(Relations::name_in_scope_NameInScope),
        "name_in_scope::NameOccursInScope" => Ok(Relations::name_in_scope_NameOccursInScope),
        "name_in_scope::ScopeOfDeclName" => Ok(Relations::name_in_scope_ScopeOfDeclName),
        "outputs::no_shadow::DeclarationInDescendent" => Ok(Relations::outputs_no_shadow_DeclarationInDescendent),
        "outputs::no_shadow::NoShadow" => Ok(Relations::outputs_no_shadow_NoShadow),
        "outputs::no_shadow::ScopeOfDecl" => Ok(Relations::outputs_no_shadow_ScopeOfDecl),
        "outputs::no_typeof_undef::NeedsWithinTypeofExpr" => Ok(Relations::outputs_no_typeof_undef_NeedsWithinTypeofExpr),
        "outputs::no_typeof_undef::NoTypeofUndef" => Ok(Relations::outputs_no_typeof_undef_NoTypeofUndef),
        "outputs::no_typeof_undef::WithinTypeofExpr" => Ok(Relations::outputs_no_typeof_undef_WithinTypeofExpr),
        "outputs::no_undef::ChainedWith" => Ok(Relations::outputs_no_undef_ChainedWith),
        "outputs::no_undef::NoUndef" => Ok(Relations::outputs_no_undef_NoUndef),
        "outputs::no_unused_labels::LabelUsage" => Ok(Relations::outputs_no_unused_labels_LabelUsage),
        "outputs::no_unused_labels::NoUnusedLabels" => Ok(Relations::outputs_no_unused_labels_NoUnusedLabels),
        "outputs::no_unused_labels::UsedLabels" => Ok(Relations::outputs_no_unused_labels_UsedLabels),
        "outputs::no_unused_labels::__Prefix_1" => Ok(Relations::outputs_no_unused_labels___Prefix_1),
        "outputs::no_use_before_def::NoUseBeforeDef" => Ok(Relations::outputs_no_use_before_def_NoUseBeforeDef),
        "outputs::unused_vars::FunctionBodyScope" => Ok(Relations::outputs_unused_vars_FunctionBodyScope),
        "outputs::unused_vars::UnusedVariables" => Ok(Relations::outputs_unused_vars_UnusedVariables),
        "scopes::FunctionLevelScope" => Ok(Relations::scopes_FunctionLevelScope),
        "scopes::IsHoistable" => Ok(Relations::scopes_IsHoistable),
        "scopes::NeedsScopeChildren" => Ok(Relations::scopes_NeedsScopeChildren),
        "scopes::NeedsScopeParents" => Ok(Relations::scopes_NeedsScopeParents),
        "scopes::ScopeFamily" => Ok(Relations::scopes_ScopeFamily),
        "scopes::ScopeOfId" => Ok(Relations::scopes_ScopeOfId),
        "var_decls::VariableDeclarations" => Ok(Relations::var_decls_VariableDeclarations),
        "variable_decl::VariableDecl" => Ok(Relations::variable_decl_VariableDecl),
             _  => Err(())
         }
    }
}
impl Relations {
    pub fn is_output(&self) -> bool {
        match self {
        Relations::outputs_no_shadow_NoShadow => true,
        Relations::outputs_no_typeof_undef_NoTypeofUndef => true,
        Relations::outputs_no_undef_NoUndef => true,
        Relations::outputs_no_unused_labels_NoUnusedLabels => true,
        Relations::outputs_no_unused_labels_UsedLabels => true,
        Relations::outputs_no_use_before_def_NoUseBeforeDef => true,
        Relations::outputs_unused_vars_UnusedVariables => true,
            _  => false
        }
    }
}
impl Relations {
    pub fn is_input(&self) -> bool {
        match self {
        Relations::config_EnableNoShadow => true,
        Relations::config_EnableNoTypeofUndef => true,
        Relations::config_EnableNoUndef => true,
        Relations::config_EnableNoUnusedLabels => true,
        Relations::config_EnableNoUnusedVars => true,
        Relations::config_EnableNoUseBeforeDef => true,
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
        Relations::inputs_ForOf => true,
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
        Relations::inputs_UserGlobal => true,
        Relations::inputs_VarDecl => true,
        Relations::inputs_While => true,
        Relations::inputs_With => true,
        Relations::inputs_Yield => true,
            _  => false
        }
    }
}
impl Relations {
    pub fn type_id(&self) -> ::std::any::TypeId {
        match self {
            Relations::__Prefix_0 => ::std::any::TypeId::of::<ddlog_std::tuple5<types__ast::FileId, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span>>(),
            Relations::config_EnableNoShadow => ::std::any::TypeId::of::<types__config::EnableNoShadow>(),
            Relations::config_EnableNoTypeofUndef => ::std::any::TypeId::of::<types__config::EnableNoTypeofUndef>(),
            Relations::config_EnableNoUndef => ::std::any::TypeId::of::<types__config::EnableNoUndef>(),
            Relations::config_EnableNoUnusedLabels => ::std::any::TypeId::of::<types__config::EnableNoUnusedLabels>(),
            Relations::config_EnableNoUnusedVars => ::std::any::TypeId::of::<types__config::EnableNoUnusedVars>(),
            Relations::config_EnableNoUseBeforeDef => ::std::any::TypeId::of::<types__config::EnableNoUseBeforeDef>(),
            Relations::inputs_Array => ::std::any::TypeId::of::<types__inputs::Array>(),
            Relations::inputs_Arrow => ::std::any::TypeId::of::<types__inputs::Arrow>(),
            Relations::inputs_ArrowParam => ::std::any::TypeId::of::<types__inputs::ArrowParam>(),
            Relations::inputs_Assign => ::std::any::TypeId::of::<types__inputs::Assign>(),
            Relations::inputs_Await => ::std::any::TypeId::of::<types__inputs::Await>(),
            Relations::inputs_BinOp => ::std::any::TypeId::of::<types__inputs::BinOp>(),
            Relations::inputs_BracketAccess => ::std::any::TypeId::of::<types__inputs::BracketAccess>(),
            Relations::inputs_Break => ::std::any::TypeId::of::<types__inputs::Break>(),
            Relations::inputs_Call => ::std::any::TypeId::of::<types__inputs::Call>(),
            Relations::inputs_Class => ::std::any::TypeId::of::<types__inputs::Class>(),
            Relations::inputs_ClassExpr => ::std::any::TypeId::of::<types__inputs::ClassExpr>(),
            Relations::inputs_ConstDecl => ::std::any::TypeId::of::<types__inputs::ConstDecl>(),
            Relations::inputs_Continue => ::std::any::TypeId::of::<types__inputs::Continue>(),
            Relations::inputs_DoWhile => ::std::any::TypeId::of::<types__inputs::DoWhile>(),
            Relations::inputs_DotAccess => ::std::any::TypeId::of::<types__inputs::DotAccess>(),
            Relations::inputs_EveryScope => ::std::any::TypeId::of::<types__inputs::EveryScope>(),
            Relations::inputs_ExprBigInt => ::std::any::TypeId::of::<types__inputs::ExprBigInt>(),
            Relations::inputs_ExprBool => ::std::any::TypeId::of::<types__inputs::ExprBool>(),
            Relations::inputs_ExprNumber => ::std::any::TypeId::of::<types__inputs::ExprNumber>(),
            Relations::inputs_ExprString => ::std::any::TypeId::of::<types__inputs::ExprString>(),
            Relations::inputs_Expression => ::std::any::TypeId::of::<types__inputs::Expression>(),
            Relations::inputs_File => ::std::any::TypeId::of::<types__inputs::File>(),
            Relations::inputs_FileExport => ::std::any::TypeId::of::<types__inputs::FileExport>(),
            Relations::inputs_For => ::std::any::TypeId::of::<types__inputs::For>(),
            Relations::inputs_ForIn => ::std::any::TypeId::of::<types__inputs::ForIn>(),
            Relations::inputs_ForOf => ::std::any::TypeId::of::<types__inputs::ForOf>(),
            Relations::inputs_Function => ::std::any::TypeId::of::<types__inputs::Function>(),
            Relations::inputs_FunctionArg => ::std::any::TypeId::of::<types__inputs::FunctionArg>(),
            Relations::inputs_If => ::std::any::TypeId::of::<types__inputs::If>(),
            Relations::inputs_ImplicitGlobal => ::std::any::TypeId::of::<types__inputs::ImplicitGlobal>(),
            Relations::inputs_ImportDecl => ::std::any::TypeId::of::<types__inputs::ImportDecl>(),
            Relations::inputs_InlineFunc => ::std::any::TypeId::of::<types__inputs::InlineFunc>(),
            Relations::inputs_InlineFuncParam => ::std::any::TypeId::of::<types__inputs::InlineFuncParam>(),
            Relations::inputs_InputScope => ::std::any::TypeId::of::<types__inputs::InputScope>(),
            Relations::inputs_Label => ::std::any::TypeId::of::<types__inputs::Label>(),
            Relations::inputs_LetDecl => ::std::any::TypeId::of::<types__inputs::LetDecl>(),
            Relations::inputs_NameRef => ::std::any::TypeId::of::<types__inputs::NameRef>(),
            Relations::inputs_New => ::std::any::TypeId::of::<types__inputs::New>(),
            Relations::inputs_Property => ::std::any::TypeId::of::<types__inputs::Property>(),
            Relations::inputs_Return => ::std::any::TypeId::of::<types__inputs::Return>(),
            Relations::inputs_Statement => ::std::any::TypeId::of::<types__inputs::Statement>(),
            Relations::inputs_Switch => ::std::any::TypeId::of::<types__inputs::Switch>(),
            Relations::inputs_SwitchCase => ::std::any::TypeId::of::<types__inputs::SwitchCase>(),
            Relations::inputs_Template => ::std::any::TypeId::of::<types__inputs::Template>(),
            Relations::inputs_Ternary => ::std::any::TypeId::of::<types__inputs::Ternary>(),
            Relations::inputs_Throw => ::std::any::TypeId::of::<types__inputs::Throw>(),
            Relations::inputs_Try => ::std::any::TypeId::of::<types__inputs::Try>(),
            Relations::inputs_UnaryOp => ::std::any::TypeId::of::<types__inputs::UnaryOp>(),
            Relations::inputs_UserGlobal => ::std::any::TypeId::of::<types__inputs::UserGlobal>(),
            Relations::inputs_VarDecl => ::std::any::TypeId::of::<types__inputs::VarDecl>(),
            Relations::inputs_While => ::std::any::TypeId::of::<types__inputs::While>(),
            Relations::inputs_With => ::std::any::TypeId::of::<types__inputs::With>(),
            Relations::inputs_Yield => ::std::any::TypeId::of::<types__inputs::Yield>(),
            Relations::is_exported_IsExported => ::std::any::TypeId::of::<types::is_exported::IsExported>(),
            Relations::name_in_scope_NameInScope => ::std::any::TypeId::of::<types::name_in_scope::NameInScope>(),
            Relations::name_in_scope_NameOccursInScope => ::std::any::TypeId::of::<types::name_in_scope::NameOccursInScope>(),
            Relations::name_in_scope_ScopeOfDeclName => ::std::any::TypeId::of::<types::name_in_scope::ScopeOfDeclName>(),
            Relations::outputs_no_shadow_DeclarationInDescendent => ::std::any::TypeId::of::<types::outputs::no_shadow::DeclarationInDescendent>(),
            Relations::outputs_no_shadow_NoShadow => ::std::any::TypeId::of::<types::outputs::no_shadow::NoShadow>(),
            Relations::outputs_no_shadow_ScopeOfDecl => ::std::any::TypeId::of::<types::outputs::no_shadow::ScopeOfDecl>(),
            Relations::outputs_no_typeof_undef_NeedsWithinTypeofExpr => ::std::any::TypeId::of::<types::outputs::no_typeof_undef::NeedsWithinTypeofExpr>(),
            Relations::outputs_no_typeof_undef_NoTypeofUndef => ::std::any::TypeId::of::<types::outputs::no_typeof_undef::NoTypeofUndef>(),
            Relations::outputs_no_typeof_undef_WithinTypeofExpr => ::std::any::TypeId::of::<types::outputs::no_typeof_undef::WithinTypeofExpr>(),
            Relations::outputs_no_undef_ChainedWith => ::std::any::TypeId::of::<types::outputs::no_undef::ChainedWith>(),
            Relations::outputs_no_undef_NoUndef => ::std::any::TypeId::of::<types::outputs::no_undef::NoUndef>(),
            Relations::outputs_no_unused_labels_LabelUsage => ::std::any::TypeId::of::<types__outputs__no_unused_labels::LabelUsage>(),
            Relations::outputs_no_unused_labels_NoUnusedLabels => ::std::any::TypeId::of::<types__outputs__no_unused_labels::NoUnusedLabels>(),
            Relations::outputs_no_unused_labels_UsedLabels => ::std::any::TypeId::of::<types__outputs__no_unused_labels::UsedLabels>(),
            Relations::outputs_no_unused_labels___Prefix_1 => ::std::any::TypeId::of::<ddlog_std::tuple4<types__ast::FileId, types__ast::StmtId, types__ast::Spanned<types__ast::Name>, types__ast::ScopeId>>(),
            Relations::outputs_no_use_before_def_NoUseBeforeDef => ::std::any::TypeId::of::<types::outputs::no_use_before_def::NoUseBeforeDef>(),
            Relations::outputs_unused_vars_FunctionBodyScope => ::std::any::TypeId::of::<types::outputs::unused_vars::FunctionBodyScope>(),
            Relations::outputs_unused_vars_UnusedVariables => ::std::any::TypeId::of::<types::outputs::unused_vars::UnusedVariables>(),
            Relations::scopes_FunctionLevelScope => ::std::any::TypeId::of::<types__scopes::FunctionLevelScope>(),
            Relations::scopes_IsHoistable => ::std::any::TypeId::of::<types__scopes::IsHoistable>(),
            Relations::scopes_NeedsScopeChildren => ::std::any::TypeId::of::<types__scopes::NeedsScopeChildren>(),
            Relations::scopes_NeedsScopeParents => ::std::any::TypeId::of::<types__scopes::NeedsScopeParents>(),
            Relations::scopes_ScopeFamily => ::std::any::TypeId::of::<types__scopes::ScopeFamily>(),
            Relations::scopes_ScopeOfId => ::std::any::TypeId::of::<types__scopes::ScopeOfId>(),
            Relations::var_decls_VariableDeclarations => ::std::any::TypeId::of::<types::var_decls::VariableDeclarations>(),
            Relations::variable_decl_VariableDecl => ::std::any::TypeId::of::<types__variable_decl::VariableDecl>(),
        }
    }
}
impl TryFrom<program::RelId> for Relations {
    type Error = ();
    fn try_from(rid: program::RelId) -> ::std::result::Result<Self, ()> {
         match rid {
        0 => Ok(Relations::__Prefix_0),
        1 => Ok(Relations::config_EnableNoShadow),
        2 => Ok(Relations::config_EnableNoTypeofUndef),
        3 => Ok(Relations::config_EnableNoUndef),
        4 => Ok(Relations::config_EnableNoUnusedLabels),
        5 => Ok(Relations::config_EnableNoUnusedVars),
        6 => Ok(Relations::config_EnableNoUseBeforeDef),
        7 => Ok(Relations::inputs_Array),
        8 => Ok(Relations::inputs_Arrow),
        9 => Ok(Relations::inputs_ArrowParam),
        10 => Ok(Relations::inputs_Assign),
        11 => Ok(Relations::inputs_Await),
        12 => Ok(Relations::inputs_BinOp),
        13 => Ok(Relations::inputs_BracketAccess),
        14 => Ok(Relations::inputs_Break),
        15 => Ok(Relations::inputs_Call),
        16 => Ok(Relations::inputs_Class),
        17 => Ok(Relations::inputs_ClassExpr),
        18 => Ok(Relations::inputs_ConstDecl),
        19 => Ok(Relations::inputs_Continue),
        20 => Ok(Relations::inputs_DoWhile),
        21 => Ok(Relations::inputs_DotAccess),
        22 => Ok(Relations::inputs_EveryScope),
        23 => Ok(Relations::inputs_ExprBigInt),
        24 => Ok(Relations::inputs_ExprBool),
        25 => Ok(Relations::inputs_ExprNumber),
        26 => Ok(Relations::inputs_ExprString),
        27 => Ok(Relations::inputs_Expression),
        28 => Ok(Relations::inputs_File),
        29 => Ok(Relations::inputs_FileExport),
        30 => Ok(Relations::inputs_For),
        31 => Ok(Relations::inputs_ForIn),
        32 => Ok(Relations::inputs_ForOf),
        33 => Ok(Relations::inputs_Function),
        34 => Ok(Relations::inputs_FunctionArg),
        35 => Ok(Relations::inputs_If),
        36 => Ok(Relations::inputs_ImplicitGlobal),
        37 => Ok(Relations::inputs_ImportDecl),
        38 => Ok(Relations::inputs_InlineFunc),
        39 => Ok(Relations::inputs_InlineFuncParam),
        40 => Ok(Relations::inputs_InputScope),
        41 => Ok(Relations::inputs_Label),
        42 => Ok(Relations::inputs_LetDecl),
        43 => Ok(Relations::inputs_NameRef),
        44 => Ok(Relations::inputs_New),
        45 => Ok(Relations::inputs_Property),
        46 => Ok(Relations::inputs_Return),
        47 => Ok(Relations::inputs_Statement),
        48 => Ok(Relations::inputs_Switch),
        49 => Ok(Relations::inputs_SwitchCase),
        50 => Ok(Relations::inputs_Template),
        51 => Ok(Relations::inputs_Ternary),
        52 => Ok(Relations::inputs_Throw),
        53 => Ok(Relations::inputs_Try),
        54 => Ok(Relations::inputs_UnaryOp),
        55 => Ok(Relations::inputs_UserGlobal),
        56 => Ok(Relations::inputs_VarDecl),
        57 => Ok(Relations::inputs_While),
        58 => Ok(Relations::inputs_With),
        59 => Ok(Relations::inputs_Yield),
        60 => Ok(Relations::is_exported_IsExported),
        61 => Ok(Relations::name_in_scope_NameInScope),
        62 => Ok(Relations::name_in_scope_NameOccursInScope),
        63 => Ok(Relations::name_in_scope_ScopeOfDeclName),
        64 => Ok(Relations::outputs_no_shadow_DeclarationInDescendent),
        65 => Ok(Relations::outputs_no_shadow_NoShadow),
        66 => Ok(Relations::outputs_no_shadow_ScopeOfDecl),
        67 => Ok(Relations::outputs_no_typeof_undef_NeedsWithinTypeofExpr),
        68 => Ok(Relations::outputs_no_typeof_undef_NoTypeofUndef),
        69 => Ok(Relations::outputs_no_typeof_undef_WithinTypeofExpr),
        70 => Ok(Relations::outputs_no_undef_ChainedWith),
        71 => Ok(Relations::outputs_no_undef_NoUndef),
        72 => Ok(Relations::outputs_no_unused_labels_LabelUsage),
        73 => Ok(Relations::outputs_no_unused_labels_NoUnusedLabels),
        74 => Ok(Relations::outputs_no_unused_labels_UsedLabels),
        75 => Ok(Relations::outputs_no_unused_labels___Prefix_1),
        76 => Ok(Relations::outputs_no_use_before_def_NoUseBeforeDef),
        77 => Ok(Relations::outputs_unused_vars_FunctionBodyScope),
        78 => Ok(Relations::outputs_unused_vars_UnusedVariables),
        79 => Ok(Relations::scopes_FunctionLevelScope),
        80 => Ok(Relations::scopes_IsHoistable),
        81 => Ok(Relations::scopes_NeedsScopeChildren),
        82 => Ok(Relations::scopes_NeedsScopeParents),
        83 => Ok(Relations::scopes_ScopeFamily),
        84 => Ok(Relations::scopes_ScopeOfId),
        85 => Ok(Relations::var_decls_VariableDeclarations),
        86 => Ok(Relations::variable_decl_VariableDecl),
             _  => Err(())
         }
    }
}
pub fn relid2name(rid: program::RelId) -> Option<&'static str> {
   match rid {
        0 => Some(&"__Prefix_0"),
        1 => Some(&"config::EnableNoShadow"),
        2 => Some(&"config::EnableNoTypeofUndef"),
        3 => Some(&"config::EnableNoUndef"),
        4 => Some(&"config::EnableNoUnusedLabels"),
        5 => Some(&"config::EnableNoUnusedVars"),
        6 => Some(&"config::EnableNoUseBeforeDef"),
        7 => Some(&"inputs::Array"),
        8 => Some(&"inputs::Arrow"),
        9 => Some(&"inputs::ArrowParam"),
        10 => Some(&"inputs::Assign"),
        11 => Some(&"inputs::Await"),
        12 => Some(&"inputs::BinOp"),
        13 => Some(&"inputs::BracketAccess"),
        14 => Some(&"inputs::Break"),
        15 => Some(&"inputs::Call"),
        16 => Some(&"inputs::Class"),
        17 => Some(&"inputs::ClassExpr"),
        18 => Some(&"inputs::ConstDecl"),
        19 => Some(&"inputs::Continue"),
        20 => Some(&"inputs::DoWhile"),
        21 => Some(&"inputs::DotAccess"),
        22 => Some(&"inputs::EveryScope"),
        23 => Some(&"inputs::ExprBigInt"),
        24 => Some(&"inputs::ExprBool"),
        25 => Some(&"inputs::ExprNumber"),
        26 => Some(&"inputs::ExprString"),
        27 => Some(&"inputs::Expression"),
        28 => Some(&"inputs::File"),
        29 => Some(&"inputs::FileExport"),
        30 => Some(&"inputs::For"),
        31 => Some(&"inputs::ForIn"),
        32 => Some(&"inputs::ForOf"),
        33 => Some(&"inputs::Function"),
        34 => Some(&"inputs::FunctionArg"),
        35 => Some(&"inputs::If"),
        36 => Some(&"inputs::ImplicitGlobal"),
        37 => Some(&"inputs::ImportDecl"),
        38 => Some(&"inputs::InlineFunc"),
        39 => Some(&"inputs::InlineFuncParam"),
        40 => Some(&"inputs::InputScope"),
        41 => Some(&"inputs::Label"),
        42 => Some(&"inputs::LetDecl"),
        43 => Some(&"inputs::NameRef"),
        44 => Some(&"inputs::New"),
        45 => Some(&"inputs::Property"),
        46 => Some(&"inputs::Return"),
        47 => Some(&"inputs::Statement"),
        48 => Some(&"inputs::Switch"),
        49 => Some(&"inputs::SwitchCase"),
        50 => Some(&"inputs::Template"),
        51 => Some(&"inputs::Ternary"),
        52 => Some(&"inputs::Throw"),
        53 => Some(&"inputs::Try"),
        54 => Some(&"inputs::UnaryOp"),
        55 => Some(&"inputs::UserGlobal"),
        56 => Some(&"inputs::VarDecl"),
        57 => Some(&"inputs::While"),
        58 => Some(&"inputs::With"),
        59 => Some(&"inputs::Yield"),
        60 => Some(&"is_exported::IsExported"),
        61 => Some(&"name_in_scope::NameInScope"),
        62 => Some(&"name_in_scope::NameOccursInScope"),
        63 => Some(&"name_in_scope::ScopeOfDeclName"),
        64 => Some(&"outputs::no_shadow::DeclarationInDescendent"),
        65 => Some(&"outputs::no_shadow::NoShadow"),
        66 => Some(&"outputs::no_shadow::ScopeOfDecl"),
        67 => Some(&"outputs::no_typeof_undef::NeedsWithinTypeofExpr"),
        68 => Some(&"outputs::no_typeof_undef::NoTypeofUndef"),
        69 => Some(&"outputs::no_typeof_undef::WithinTypeofExpr"),
        70 => Some(&"outputs::no_undef::ChainedWith"),
        71 => Some(&"outputs::no_undef::NoUndef"),
        72 => Some(&"outputs::no_unused_labels::LabelUsage"),
        73 => Some(&"outputs::no_unused_labels::NoUnusedLabels"),
        74 => Some(&"outputs::no_unused_labels::UsedLabels"),
        75 => Some(&"outputs::no_unused_labels::__Prefix_1"),
        76 => Some(&"outputs::no_use_before_def::NoUseBeforeDef"),
        77 => Some(&"outputs::unused_vars::FunctionBodyScope"),
        78 => Some(&"outputs::unused_vars::UnusedVariables"),
        79 => Some(&"scopes::FunctionLevelScope"),
        80 => Some(&"scopes::IsHoistable"),
        81 => Some(&"scopes::NeedsScopeChildren"),
        82 => Some(&"scopes::NeedsScopeParents"),
        83 => Some(&"scopes::ScopeFamily"),
        84 => Some(&"scopes::ScopeOfId"),
        85 => Some(&"var_decls::VariableDeclarations"),
        86 => Some(&"variable_decl::VariableDecl"),
       _  => None
   }
}
#[cfg(feature = "c_api")]
pub fn relid2cname(rid: program::RelId) -> Option<&'static ::std::ffi::CStr> {
    RELIDMAPC.get(&rid).copied()
}   /// A map of `RelId`s to their name as an `&'static str`
pub static RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(87, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::__Prefix_0, "__Prefix_0");
        map.insert(Relations::config_EnableNoShadow, "config::EnableNoShadow");
        map.insert(Relations::config_EnableNoTypeofUndef, "config::EnableNoTypeofUndef");
        map.insert(Relations::config_EnableNoUndef, "config::EnableNoUndef");
        map.insert(Relations::config_EnableNoUnusedLabels, "config::EnableNoUnusedLabels");
        map.insert(Relations::config_EnableNoUnusedVars, "config::EnableNoUnusedVars");
        map.insert(Relations::config_EnableNoUseBeforeDef, "config::EnableNoUseBeforeDef");
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
        map.insert(Relations::inputs_ForOf, "inputs::ForOf");
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
        map.insert(Relations::inputs_UserGlobal, "inputs::UserGlobal");
        map.insert(Relations::inputs_VarDecl, "inputs::VarDecl");
        map.insert(Relations::inputs_While, "inputs::While");
        map.insert(Relations::inputs_With, "inputs::With");
        map.insert(Relations::inputs_Yield, "inputs::Yield");
        map.insert(Relations::is_exported_IsExported, "is_exported::IsExported");
        map.insert(Relations::name_in_scope_NameInScope, "name_in_scope::NameInScope");
        map.insert(Relations::name_in_scope_NameOccursInScope, "name_in_scope::NameOccursInScope");
        map.insert(Relations::name_in_scope_ScopeOfDeclName, "name_in_scope::ScopeOfDeclName");
        map.insert(Relations::outputs_no_shadow_DeclarationInDescendent, "outputs::no_shadow::DeclarationInDescendent");
        map.insert(Relations::outputs_no_shadow_NoShadow, "outputs::no_shadow::NoShadow");
        map.insert(Relations::outputs_no_shadow_ScopeOfDecl, "outputs::no_shadow::ScopeOfDecl");
        map.insert(Relations::outputs_no_typeof_undef_NeedsWithinTypeofExpr, "outputs::no_typeof_undef::NeedsWithinTypeofExpr");
        map.insert(Relations::outputs_no_typeof_undef_NoTypeofUndef, "outputs::no_typeof_undef::NoTypeofUndef");
        map.insert(Relations::outputs_no_typeof_undef_WithinTypeofExpr, "outputs::no_typeof_undef::WithinTypeofExpr");
        map.insert(Relations::outputs_no_undef_ChainedWith, "outputs::no_undef::ChainedWith");
        map.insert(Relations::outputs_no_undef_NoUndef, "outputs::no_undef::NoUndef");
        map.insert(Relations::outputs_no_unused_labels_LabelUsage, "outputs::no_unused_labels::LabelUsage");
        map.insert(Relations::outputs_no_unused_labels_NoUnusedLabels, "outputs::no_unused_labels::NoUnusedLabels");
        map.insert(Relations::outputs_no_unused_labels_UsedLabels, "outputs::no_unused_labels::UsedLabels");
        map.insert(Relations::outputs_no_unused_labels___Prefix_1, "outputs::no_unused_labels::__Prefix_1");
        map.insert(Relations::outputs_no_use_before_def_NoUseBeforeDef, "outputs::no_use_before_def::NoUseBeforeDef");
        map.insert(Relations::outputs_unused_vars_FunctionBodyScope, "outputs::unused_vars::FunctionBodyScope");
        map.insert(Relations::outputs_unused_vars_UnusedVariables, "outputs::unused_vars::UnusedVariables");
        map.insert(Relations::scopes_FunctionLevelScope, "scopes::FunctionLevelScope");
        map.insert(Relations::scopes_IsHoistable, "scopes::IsHoistable");
        map.insert(Relations::scopes_NeedsScopeChildren, "scopes::NeedsScopeChildren");
        map.insert(Relations::scopes_NeedsScopeParents, "scopes::NeedsScopeParents");
        map.insert(Relations::scopes_ScopeFamily, "scopes::ScopeFamily");
        map.insert(Relations::scopes_ScopeOfId, "scopes::ScopeOfId");
        map.insert(Relations::var_decls_VariableDeclarations, "var_decls::VariableDeclarations");
        map.insert(Relations::variable_decl_VariableDecl, "variable_decl::VariableDecl");
        map
    });
    /// A map of `RelId`s to their name as an `&'static CStr`
#[cfg(feature = "c_api")]
pub static RELIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<program::RelId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(87, ::fnv::FnvBuildHasher::default());
        map.insert(0, ::std::ffi::CStr::from_bytes_with_nul(b"__Prefix_0\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(1, ::std::ffi::CStr::from_bytes_with_nul(b"config::EnableNoShadow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(2, ::std::ffi::CStr::from_bytes_with_nul(b"config::EnableNoTypeofUndef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(3, ::std::ffi::CStr::from_bytes_with_nul(b"config::EnableNoUndef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(4, ::std::ffi::CStr::from_bytes_with_nul(b"config::EnableNoUnusedLabels\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(5, ::std::ffi::CStr::from_bytes_with_nul(b"config::EnableNoUnusedVars\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(6, ::std::ffi::CStr::from_bytes_with_nul(b"config::EnableNoUseBeforeDef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(7, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Array\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(8, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Arrow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(9, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ArrowParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(10, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Assign\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(11, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Await\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(12, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::BinOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(13, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::BracketAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(14, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Break\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(15, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Call\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(16, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Class\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(17, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ClassExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(18, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ConstDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(19, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Continue\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(20, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::DoWhile\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(21, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::DotAccess\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(22, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::EveryScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(23, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprBigInt\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(24, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprBool\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(25, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprNumber\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(26, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExprString\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(27, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Expression\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(28, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::File\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(29, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::FileExport\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(30, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::For\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(31, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ForIn\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(32, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ForOf\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(33, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Function\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(34, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::FunctionArg\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(35, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::If\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(36, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ImplicitGlobal\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(37, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ImportDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(38, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InlineFunc\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(39, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InlineFuncParam\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(40, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::InputScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(41, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Label\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(42, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::LetDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(43, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::NameRef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(44, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::New\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(45, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Property\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(46, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Return\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(47, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Statement\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(48, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Switch\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(49, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::SwitchCase\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(50, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Template\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(51, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Ternary\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(52, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Throw\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(53, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Try\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(54, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::UnaryOp\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(55, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::UserGlobal\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(56, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::VarDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(57, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::While\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(58, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::With\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(59, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::Yield\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(60, ::std::ffi::CStr::from_bytes_with_nul(b"is_exported::IsExported\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(61, ::std::ffi::CStr::from_bytes_with_nul(b"name_in_scope::NameInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(62, ::std::ffi::CStr::from_bytes_with_nul(b"name_in_scope::NameOccursInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(63, ::std::ffi::CStr::from_bytes_with_nul(b"name_in_scope::ScopeOfDeclName\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(64, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_shadow::DeclarationInDescendent\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(65, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_shadow::NoShadow\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(66, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_shadow::ScopeOfDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(67, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_typeof_undef::NeedsWithinTypeofExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(68, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_typeof_undef::NoTypeofUndef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(69, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_typeof_undef::WithinTypeofExpr\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(70, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_undef::ChainedWith\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(71, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_undef::NoUndef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(72, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_unused_labels::LabelUsage\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(73, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_unused_labels::NoUnusedLabels\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(74, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_unused_labels::UsedLabels\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(75, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_unused_labels::__Prefix_1\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(76, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::no_use_before_def::NoUseBeforeDef\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(77, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::unused_vars::FunctionBodyScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(78, ::std::ffi::CStr::from_bytes_with_nul(b"outputs::unused_vars::UnusedVariables\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(79, ::std::ffi::CStr::from_bytes_with_nul(b"scopes::FunctionLevelScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(80, ::std::ffi::CStr::from_bytes_with_nul(b"scopes::IsHoistable\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(81, ::std::ffi::CStr::from_bytes_with_nul(b"scopes::NeedsScopeChildren\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(82, ::std::ffi::CStr::from_bytes_with_nul(b"scopes::NeedsScopeParents\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(83, ::std::ffi::CStr::from_bytes_with_nul(b"scopes::ScopeFamily\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(84, ::std::ffi::CStr::from_bytes_with_nul(b"scopes::ScopeOfId\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(85, ::std::ffi::CStr::from_bytes_with_nul(b"var_decls::VariableDeclarations\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(86, ::std::ffi::CStr::from_bytes_with_nul(b"variable_decl::VariableDecl\0").expect("Unreachable: A null byte was specifically inserted"));
        map
    });
    /// A map of input `Relations`s to their name as an `&'static str`
pub static INPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(59, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::config_EnableNoShadow, "config::EnableNoShadow");
        map.insert(Relations::config_EnableNoTypeofUndef, "config::EnableNoTypeofUndef");
        map.insert(Relations::config_EnableNoUndef, "config::EnableNoUndef");
        map.insert(Relations::config_EnableNoUnusedLabels, "config::EnableNoUnusedLabels");
        map.insert(Relations::config_EnableNoUnusedVars, "config::EnableNoUnusedVars");
        map.insert(Relations::config_EnableNoUseBeforeDef, "config::EnableNoUseBeforeDef");
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
        map.insert(Relations::inputs_ForOf, "inputs::ForOf");
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
        map.insert(Relations::inputs_UserGlobal, "inputs::UserGlobal");
        map.insert(Relations::inputs_VarDecl, "inputs::VarDecl");
        map.insert(Relations::inputs_While, "inputs::While");
        map.insert(Relations::inputs_With, "inputs::With");
        map.insert(Relations::inputs_Yield, "inputs::Yield");
        map
    });
    /// A map of output `Relations`s to their name as an `&'static str`
pub static OUTPUT_RELIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Relations, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(7, ::fnv::FnvBuildHasher::default());
        map.insert(Relations::outputs_no_shadow_NoShadow, "outputs::no_shadow::NoShadow");
        map.insert(Relations::outputs_no_typeof_undef_NoTypeofUndef, "outputs::no_typeof_undef::NoTypeofUndef");
        map.insert(Relations::outputs_no_undef_NoUndef, "outputs::no_undef::NoUndef");
        map.insert(Relations::outputs_no_unused_labels_NoUnusedLabels, "outputs::no_unused_labels::NoUnusedLabels");
        map.insert(Relations::outputs_no_unused_labels_UsedLabels, "outputs::no_unused_labels::UsedLabels");
        map.insert(Relations::outputs_no_use_before_def_NoUseBeforeDef, "outputs::no_use_before_def::NoUseBeforeDef");
        map.insert(Relations::outputs_unused_vars_UnusedVariables, "outputs::unused_vars::UnusedVariables");
        map
    });
impl TryFrom<&str> for Indexes {
    type Error = ();
    fn try_from(iname: &str) -> ::std::result::Result<Self, ()> {
         match iname {
        "inputs::ExpressionById" => Ok(Indexes::inputs_ExpressionById),
        "inputs::StatementById" => Ok(Indexes::inputs_StatementById),
        "name_in_scope::Index_VariableInScope" => Ok(Indexes::name_in_scope_Index_VariableInScope),
        "name_in_scope::Index_VariablesForScope" => Ok(Indexes::name_in_scope_Index_VariablesForScope),
        "scopes::ScopeFamilyByParent" => Ok(Indexes::scopes_ScopeFamilyByParent),
             _  => Err(())
         }
    }
}
impl TryFrom<program::IdxId> for Indexes {
    type Error = ();
    fn try_from(iid: program::IdxId) -> ::core::result::Result<Self, ()> {
         match iid {
        0 => Ok(Indexes::inputs_ExpressionById),
        1 => Ok(Indexes::inputs_StatementById),
        2 => Ok(Indexes::name_in_scope_Index_VariableInScope),
        3 => Ok(Indexes::name_in_scope_Index_VariablesForScope),
        4 => Ok(Indexes::scopes_ScopeFamilyByParent),
             _  => Err(())
         }
    }
}
pub fn indexid2name(iid: program::IdxId) -> Option<&'static str> {
   match iid {
        0 => Some(&"inputs::ExpressionById"),
        1 => Some(&"inputs::StatementById"),
        2 => Some(&"name_in_scope::Index_VariableInScope"),
        3 => Some(&"name_in_scope::Index_VariablesForScope"),
        4 => Some(&"scopes::ScopeFamilyByParent"),
       _  => None
   }
}
#[cfg(feature = "c_api")]
pub fn indexid2cname(iid: program::IdxId) -> Option<&'static ::std::ffi::CStr> {
    IDXIDMAPC.get(&iid).copied()
}   /// A map of `Indexes` to their name as an `&'static str`
pub static IDXIDMAP: ::once_cell::sync::Lazy<::fnv::FnvHashMap<Indexes, &'static str>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(5, ::fnv::FnvBuildHasher::default());
        map.insert(Indexes::inputs_ExpressionById, "inputs::ExpressionById");
        map.insert(Indexes::inputs_StatementById, "inputs::StatementById");
        map.insert(Indexes::name_in_scope_Index_VariableInScope, "name_in_scope::Index_VariableInScope");
        map.insert(Indexes::name_in_scope_Index_VariablesForScope, "name_in_scope::Index_VariablesForScope");
        map.insert(Indexes::scopes_ScopeFamilyByParent, "scopes::ScopeFamilyByParent");
        map
    });
    /// A map of `IdxId`s to their name as an `&'static CStr`
#[cfg(feature = "c_api")]
pub static IDXIDMAPC: ::once_cell::sync::Lazy<::fnv::FnvHashMap<program::IdxId, &'static ::std::ffi::CStr>> =
    ::once_cell::sync::Lazy::new(|| {
        let mut map = ::fnv::FnvHashMap::with_capacity_and_hasher(5, ::fnv::FnvBuildHasher::default());
        map.insert(0, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::ExpressionById\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(1, ::std::ffi::CStr::from_bytes_with_nul(b"inputs::StatementById\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(2, ::std::ffi::CStr::from_bytes_with_nul(b"name_in_scope::Index_VariableInScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(3, ::std::ffi::CStr::from_bytes_with_nul(b"name_in_scope::Index_VariablesForScope\0").expect("Unreachable: A null byte was specifically inserted"));
        map.insert(4, ::std::ffi::CStr::from_bytes_with_nul(b"scopes::ScopeFamilyByParent\0").expect("Unreachable: A null byte was specifically inserted"));
        map
    });
pub fn relval_from_record(rel: Relations, _rec: &differential_datalog::record::Record) -> ::std::result::Result<DDValue, String> {
    match rel {
        Relations::__Prefix_0 => {
            Ok(<ddlog_std::tuple5<types__ast::FileId, types__ast::ExprId, types__ast::ExprId, types__ast::ScopeId, types__ast::Span>>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoShadow => {
            Ok(<types__config::EnableNoShadow>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoTypeofUndef => {
            Ok(<types__config::EnableNoTypeofUndef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUndef => {
            Ok(<types__config::EnableNoUndef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUnusedLabels => {
            Ok(<types__config::EnableNoUnusedLabels>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUnusedVars => {
            Ok(<types__config::EnableNoUnusedVars>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUseBeforeDef => {
            Ok(<types__config::EnableNoUseBeforeDef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Array => {
            Ok(<types__inputs::Array>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Arrow => {
            Ok(<types__inputs::Arrow>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ArrowParam => {
            Ok(<types__inputs::ArrowParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Assign => {
            Ok(<types__inputs::Assign>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Await => {
            Ok(<types__inputs::Await>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_BinOp => {
            Ok(<types__inputs::BinOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_BracketAccess => {
            Ok(<types__inputs::BracketAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Break => {
            Ok(<types__inputs::Break>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Call => {
            Ok(<types__inputs::Call>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Class => {
            Ok(<types__inputs::Class>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ClassExpr => {
            Ok(<types__inputs::ClassExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ConstDecl => {
            Ok(<types__inputs::ConstDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Continue => {
            Ok(<types__inputs::Continue>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_DoWhile => {
            Ok(<types__inputs::DoWhile>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_DotAccess => {
            Ok(<types__inputs::DotAccess>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_EveryScope => {
            Ok(<types__inputs::EveryScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprBigInt => {
            Ok(<types__inputs::ExprBigInt>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprBool => {
            Ok(<types__inputs::ExprBool>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprNumber => {
            Ok(<types__inputs::ExprNumber>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ExprString => {
            Ok(<types__inputs::ExprString>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Expression => {
            Ok(<types__inputs::Expression>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_File => {
            Ok(<types__inputs::File>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_FileExport => {
            Ok(<types__inputs::FileExport>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_For => {
            Ok(<types__inputs::For>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ForIn => {
            Ok(<types__inputs::ForIn>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ForOf => {
            Ok(<types__inputs::ForOf>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Function => {
            Ok(<types__inputs::Function>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_FunctionArg => {
            Ok(<types__inputs::FunctionArg>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_If => {
            Ok(<types__inputs::If>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ImplicitGlobal => {
            Ok(<types__inputs::ImplicitGlobal>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_ImportDecl => {
            Ok(<types__inputs::ImportDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_InlineFunc => {
            Ok(<types__inputs::InlineFunc>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_InlineFuncParam => {
            Ok(<types__inputs::InlineFuncParam>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_InputScope => {
            Ok(<types__inputs::InputScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Label => {
            Ok(<types__inputs::Label>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_LetDecl => {
            Ok(<types__inputs::LetDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_NameRef => {
            Ok(<types__inputs::NameRef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_New => {
            Ok(<types__inputs::New>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Property => {
            Ok(<types__inputs::Property>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Return => {
            Ok(<types__inputs::Return>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Statement => {
            Ok(<types__inputs::Statement>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Switch => {
            Ok(<types__inputs::Switch>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_SwitchCase => {
            Ok(<types__inputs::SwitchCase>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Template => {
            Ok(<types__inputs::Template>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Ternary => {
            Ok(<types__inputs::Ternary>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Throw => {
            Ok(<types__inputs::Throw>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Try => {
            Ok(<types__inputs::Try>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_UnaryOp => {
            Ok(<types__inputs::UnaryOp>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_UserGlobal => {
            Ok(<types__inputs::UserGlobal>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_VarDecl => {
            Ok(<types__inputs::VarDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_While => {
            Ok(<types__inputs::While>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_With => {
            Ok(<types__inputs::With>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Yield => {
            Ok(<types__inputs::Yield>::from_record(_rec)?.into_ddvalue())
        },
        Relations::is_exported_IsExported => {
            Ok(<types::is_exported::IsExported>::from_record(_rec)?.into_ddvalue())
        },
        Relations::name_in_scope_NameInScope => {
            Ok(<types::name_in_scope::NameInScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::name_in_scope_NameOccursInScope => {
            Ok(<types::name_in_scope::NameOccursInScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::name_in_scope_ScopeOfDeclName => {
            Ok(<types::name_in_scope::ScopeOfDeclName>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_shadow_DeclarationInDescendent => {
            Ok(<types::outputs::no_shadow::DeclarationInDescendent>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_shadow_NoShadow => {
            Ok(<types::outputs::no_shadow::NoShadow>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_shadow_ScopeOfDecl => {
            Ok(<types::outputs::no_shadow::ScopeOfDecl>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_typeof_undef_NeedsWithinTypeofExpr => {
            Ok(<types::outputs::no_typeof_undef::NeedsWithinTypeofExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_typeof_undef_NoTypeofUndef => {
            Ok(<types::outputs::no_typeof_undef::NoTypeofUndef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_typeof_undef_WithinTypeofExpr => {
            Ok(<types::outputs::no_typeof_undef::WithinTypeofExpr>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_undef_ChainedWith => {
            Ok(<types::outputs::no_undef::ChainedWith>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_undef_NoUndef => {
            Ok(<types::outputs::no_undef::NoUndef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_unused_labels_LabelUsage => {
            Ok(<types__outputs__no_unused_labels::LabelUsage>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_unused_labels_NoUnusedLabels => {
            Ok(<types__outputs__no_unused_labels::NoUnusedLabels>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_unused_labels_UsedLabels => {
            Ok(<types__outputs__no_unused_labels::UsedLabels>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_unused_labels___Prefix_1 => {
            Ok(<ddlog_std::tuple4<types__ast::FileId, types__ast::StmtId, types__ast::Spanned<internment::Intern<String>>, types__ast::ScopeId>>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_no_use_before_def_NoUseBeforeDef => {
            Ok(<types::outputs::no_use_before_def::NoUseBeforeDef>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_unused_vars_FunctionBodyScope => {
            Ok(<types::outputs::unused_vars::FunctionBodyScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::outputs_unused_vars_UnusedVariables => {
            Ok(<types::outputs::unused_vars::UnusedVariables>::from_record(_rec)?.into_ddvalue())
        },
        Relations::scopes_FunctionLevelScope => {
            Ok(<types__scopes::FunctionLevelScope>::from_record(_rec)?.into_ddvalue())
        },
        Relations::scopes_IsHoistable => {
            Ok(<types__scopes::IsHoistable>::from_record(_rec)?.into_ddvalue())
        },
        Relations::scopes_NeedsScopeChildren => {
            Ok(<types__scopes::NeedsScopeChildren>::from_record(_rec)?.into_ddvalue())
        },
        Relations::scopes_NeedsScopeParents => {
            Ok(<types__scopes::NeedsScopeParents>::from_record(_rec)?.into_ddvalue())
        },
        Relations::scopes_ScopeFamily => {
            Ok(<types__scopes::ScopeFamily>::from_record(_rec)?.into_ddvalue())
        },
        Relations::scopes_ScopeOfId => {
            Ok(<types__scopes::ScopeOfId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::var_decls_VariableDeclarations => {
            Ok(<types::var_decls::VariableDeclarations>::from_record(_rec)?.into_ddvalue())
        },
        Relations::variable_decl_VariableDecl => {
            Ok(<types__variable_decl::VariableDecl>::from_record(_rec)?.into_ddvalue())
        }
    }
}
pub fn relkey_from_record(rel: Relations, _rec: &differential_datalog::record::Record) -> ::std::result::Result<DDValue, String> {
    match rel {
        Relations::config_EnableNoShadow => {
            Ok(<types__ast::FileId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoTypeofUndef => {
            Ok(<types__ast::FileId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUndef => {
            Ok(<types__ast::FileId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUnusedLabels => {
            Ok(<types__ast::FileId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUnusedVars => {
            Ok(<types__ast::FileId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::config_EnableNoUseBeforeDef => {
            Ok(<types__ast::FileId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Class => {
            Ok(<types__ast::ClassId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Expression => {
            Ok(<types__ast::ExprId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_File => {
            Ok(<types__ast::FileId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Function => {
            Ok(<types__ast::FuncId>::from_record(_rec)?.into_ddvalue())
        },
        Relations::inputs_Statement => {
            Ok(<types__ast::StmtId>::from_record(_rec)?.into_ddvalue())
        }
        _ => Err(format!("relation {:?} does not have a primary key", rel))
    }
}
pub fn idxkey_from_record(idx: Indexes, _rec: &differential_datalog::record::Record) -> ::std::result::Result<DDValue, String> {
    match idx {
        Indexes::inputs_ExpressionById => {
            Ok(<types__ast::ExprId>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::inputs_StatementById => {
            Ok(<types__ast::StmtId>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::name_in_scope_Index_VariableInScope => {
            Ok(<ddlog_std::tuple2<types__ast::ScopeId, internment::Intern<String>>>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::name_in_scope_Index_VariablesForScope => {
            Ok(<types__ast::ScopeId>::from_record(_rec)?.into_ddvalue())
        },
        Indexes::scopes_ScopeFamilyByParent => {
            Ok(<types__ast::ScopeId>::from_record(_rec)?.into_ddvalue())
        }
    }
}
pub fn indexes2arrid(idx: Indexes) -> program::ArrId {
    match idx {
        Indexes::inputs_ExpressionById => ( 27, 4),
        Indexes::inputs_StatementById => ( 47, 2),
        Indexes::name_in_scope_Index_VariableInScope => ( 61, 7),
        Indexes::name_in_scope_Index_VariablesForScope => ( 61, 8),
        Indexes::scopes_ScopeFamilyByParent => ( 83, 2),
    }
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Relations {
    __Prefix_0 = 0,
    config_EnableNoShadow = 1,
    config_EnableNoTypeofUndef = 2,
    config_EnableNoUndef = 3,
    config_EnableNoUnusedLabels = 4,
    config_EnableNoUnusedVars = 5,
    config_EnableNoUseBeforeDef = 6,
    inputs_Array = 7,
    inputs_Arrow = 8,
    inputs_ArrowParam = 9,
    inputs_Assign = 10,
    inputs_Await = 11,
    inputs_BinOp = 12,
    inputs_BracketAccess = 13,
    inputs_Break = 14,
    inputs_Call = 15,
    inputs_Class = 16,
    inputs_ClassExpr = 17,
    inputs_ConstDecl = 18,
    inputs_Continue = 19,
    inputs_DoWhile = 20,
    inputs_DotAccess = 21,
    inputs_EveryScope = 22,
    inputs_ExprBigInt = 23,
    inputs_ExprBool = 24,
    inputs_ExprNumber = 25,
    inputs_ExprString = 26,
    inputs_Expression = 27,
    inputs_File = 28,
    inputs_FileExport = 29,
    inputs_For = 30,
    inputs_ForIn = 31,
    inputs_ForOf = 32,
    inputs_Function = 33,
    inputs_FunctionArg = 34,
    inputs_If = 35,
    inputs_ImplicitGlobal = 36,
    inputs_ImportDecl = 37,
    inputs_InlineFunc = 38,
    inputs_InlineFuncParam = 39,
    inputs_InputScope = 40,
    inputs_Label = 41,
    inputs_LetDecl = 42,
    inputs_NameRef = 43,
    inputs_New = 44,
    inputs_Property = 45,
    inputs_Return = 46,
    inputs_Statement = 47,
    inputs_Switch = 48,
    inputs_SwitchCase = 49,
    inputs_Template = 50,
    inputs_Ternary = 51,
    inputs_Throw = 52,
    inputs_Try = 53,
    inputs_UnaryOp = 54,
    inputs_UserGlobal = 55,
    inputs_VarDecl = 56,
    inputs_While = 57,
    inputs_With = 58,
    inputs_Yield = 59,
    is_exported_IsExported = 60,
    name_in_scope_NameInScope = 61,
    name_in_scope_NameOccursInScope = 62,
    name_in_scope_ScopeOfDeclName = 63,
    outputs_no_shadow_DeclarationInDescendent = 64,
    outputs_no_shadow_NoShadow = 65,
    outputs_no_shadow_ScopeOfDecl = 66,
    outputs_no_typeof_undef_NeedsWithinTypeofExpr = 67,
    outputs_no_typeof_undef_NoTypeofUndef = 68,
    outputs_no_typeof_undef_WithinTypeofExpr = 69,
    outputs_no_undef_ChainedWith = 70,
    outputs_no_undef_NoUndef = 71,
    outputs_no_unused_labels_LabelUsage = 72,
    outputs_no_unused_labels_NoUnusedLabels = 73,
    outputs_no_unused_labels_UsedLabels = 74,
    outputs_no_unused_labels___Prefix_1 = 75,
    outputs_no_use_before_def_NoUseBeforeDef = 76,
    outputs_unused_vars_FunctionBodyScope = 77,
    outputs_unused_vars_UnusedVariables = 78,
    scopes_FunctionLevelScope = 79,
    scopes_IsHoistable = 80,
    scopes_NeedsScopeChildren = 81,
    scopes_NeedsScopeParents = 82,
    scopes_ScopeFamily = 83,
    scopes_ScopeOfId = 84,
    var_decls_VariableDeclarations = 85,
    variable_decl_VariableDecl = 86
}
#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub enum Indexes {
    inputs_ExpressionById = 0,
    inputs_StatementById = 1,
    name_in_scope_Index_VariableInScope = 2,
    name_in_scope_Index_VariablesForScope = 3,
    scopes_ScopeFamilyByParent = 4
}
pub fn prog(__update_cb: std::sync::Arc<dyn program::RelationCallback>) -> program::Program {
    let config_EnableNoShadow = program::Relation {
                                    name:         std::borrow::Cow::from("config::EnableNoShadow"),
                                    input:        true,
                                    distinct:     false,
                                    caching_mode: program::CachingMode::Set,
                                    key_func:     Some(types__config::__Key_config_EnableNoShadow),
                                    id:           1,
                                    rules:        vec![
                                    ],
                                    arrangements: vec![
                                        types__config::__Arng_config_EnableNoShadow_0.clone()
                                    ],
                                    change_cb:    None
                                };
    let config_EnableNoTypeofUndef = program::Relation {
                                         name:         std::borrow::Cow::from("config::EnableNoTypeofUndef"),
                                         input:        true,
                                         distinct:     false,
                                         caching_mode: program::CachingMode::Set,
                                         key_func:     Some(types__config::__Key_config_EnableNoTypeofUndef),
                                         id:           2,
                                         rules:        vec![
                                         ],
                                         arrangements: vec![
                                             types__config::__Arng_config_EnableNoTypeofUndef_0.clone()
                                         ],
                                         change_cb:    None
                                     };
    let config_EnableNoUndef = program::Relation {
                                   name:         std::borrow::Cow::from("config::EnableNoUndef"),
                                   input:        true,
                                   distinct:     false,
                                   caching_mode: program::CachingMode::Set,
                                   key_func:     Some(types__config::__Key_config_EnableNoUndef),
                                   id:           3,
                                   rules:        vec![
                                   ],
                                   arrangements: vec![
                                       types__config::__Arng_config_EnableNoUndef_0.clone()
                                   ],
                                   change_cb:    None
                               };
    let outputs_no_typeof_undef_NeedsWithinTypeofExpr = program::Relation {
                                                            name:         std::borrow::Cow::from("outputs::no_typeof_undef::NeedsWithinTypeofExpr"),
                                                            input:        false,
                                                            distinct:     false,
                                                            caching_mode: program::CachingMode::Set,
                                                            key_func:     None,
                                                            id:           67,
                                                            rules:        vec![
                                                                types::outputs::no_typeof_undef::__Rule_outputs_no_typeof_undef_NeedsWithinTypeofExpr_0.clone(),
                                                                types::outputs::no_typeof_undef::__Rule_outputs_no_typeof_undef_NeedsWithinTypeofExpr_1.clone()
                                                            ],
                                                            arrangements: vec![
                                                                types::outputs::no_typeof_undef::__Arng_outputs_no_typeof_undef_NeedsWithinTypeofExpr_0.clone()
                                                            ],
                                                            change_cb:    None
                                                        };
    let config_EnableNoUnusedLabels = program::Relation {
                                          name:         std::borrow::Cow::from("config::EnableNoUnusedLabels"),
                                          input:        true,
                                          distinct:     false,
                                          caching_mode: program::CachingMode::Set,
                                          key_func:     Some(types__config::__Key_config_EnableNoUnusedLabels),
                                          id:           4,
                                          rules:        vec![
                                          ],
                                          arrangements: vec![
                                              types__config::__Arng_config_EnableNoUnusedLabels_0.clone()
                                          ],
                                          change_cb:    None
                                      };
    let config_EnableNoUnusedVars = program::Relation {
                                        name:         std::borrow::Cow::from("config::EnableNoUnusedVars"),
                                        input:        true,
                                        distinct:     false,
                                        caching_mode: program::CachingMode::Set,
                                        key_func:     Some(types__config::__Key_config_EnableNoUnusedVars),
                                        id:           5,
                                        rules:        vec![
                                        ],
                                        arrangements: vec![
                                            types__config::__Arng_config_EnableNoUnusedVars_0.clone(),
                                            types__config::__Arng_config_EnableNoUnusedVars_1.clone()
                                        ],
                                        change_cb:    None
                                    };
    let config_EnableNoUseBeforeDef = program::Relation {
                                          name:         std::borrow::Cow::from("config::EnableNoUseBeforeDef"),
                                          input:        true,
                                          distinct:     false,
                                          caching_mode: program::CachingMode::Set,
                                          key_func:     Some(types__config::__Key_config_EnableNoUseBeforeDef),
                                          id:           6,
                                          rules:        vec![
                                          ],
                                          arrangements: vec![
                                              types__config::__Arng_config_EnableNoUseBeforeDef_0.clone()
                                          ],
                                          change_cb:    None
                                      };
    let inputs_Array = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Array"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           7,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let inputs_Arrow = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Arrow"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           8,
                           rules:        vec![
                           ],
                           arrangements: vec![
                               types__inputs::__Arng_inputs_Arrow_0.clone(),
                               types__inputs::__Arng_inputs_Arrow_1.clone()
                           ],
                           change_cb:    None
                       };
    let inputs_ArrowParam = program::Relation {
                                name:         std::borrow::Cow::from("inputs::ArrowParam"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           9,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                    types__inputs::__Arng_inputs_ArrowParam_0.clone()
                                ],
                                change_cb:    None
                            };
    let inputs_Assign = program::Relation {
                            name:         std::borrow::Cow::from("inputs::Assign"),
                            input:        true,
                            distinct:     false,
                            caching_mode: program::CachingMode::Set,
                            key_func:     None,
                            id:           10,
                            rules:        vec![
                            ],
                            arrangements: vec![
                                types__inputs::__Arng_inputs_Assign_0.clone(),
                                types__inputs::__Arng_inputs_Assign_1.clone()
                            ],
                            change_cb:    None
                        };
    let inputs_Await = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Await"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           11,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let inputs_BinOp = program::Relation {
                           name:         std::borrow::Cow::from("inputs::BinOp"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           12,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let inputs_BracketAccess = program::Relation {
                                   name:         std::borrow::Cow::from("inputs::BracketAccess"),
                                   input:        true,
                                   distinct:     false,
                                   caching_mode: program::CachingMode::Set,
                                   key_func:     None,
                                   id:           13,
                                   rules:        vec![
                                   ],
                                   arrangements: vec![
                                       types__inputs::__Arng_inputs_BracketAccess_0.clone()
                                   ],
                                   change_cb:    None
                               };
    let inputs_Break = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Break"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           14,
                           rules:        vec![
                           ],
                           arrangements: vec![
                               types__inputs::__Arng_inputs_Break_0.clone()
                           ],
                           change_cb:    None
                       };
    let inputs_Call = program::Relation {
                          name:         std::borrow::Cow::from("inputs::Call"),
                          input:        true,
                          distinct:     false,
                          caching_mode: program::CachingMode::Set,
                          key_func:     None,
                          id:           15,
                          rules:        vec![
                          ],
                          arrangements: vec![
                              types__inputs::__Arng_inputs_Call_0.clone()
                          ],
                          change_cb:    None
                      };
    let inputs_Class = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Class"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     Some(types__inputs::__Key_inputs_Class),
                           id:           16,
                           rules:        vec![
                           ],
                           arrangements: vec![
                               types__inputs::__Arng_inputs_Class_0.clone(),
                               types__inputs::__Arng_inputs_Class_1.clone()
                           ],
                           change_cb:    None
                       };
    let inputs_ClassExpr = program::Relation {
                               name:         std::borrow::Cow::from("inputs::ClassExpr"),
                               input:        true,
                               distinct:     false,
                               caching_mode: program::CachingMode::Set,
                               key_func:     None,
                               id:           17,
                               rules:        vec![
                               ],
                               arrangements: vec![
                                   types__inputs::__Arng_inputs_ClassExpr_0.clone()
                               ],
                               change_cb:    None
                           };
    let inputs_ConstDecl = program::Relation {
                               name:         std::borrow::Cow::from("inputs::ConstDecl"),
                               input:        true,
                               distinct:     false,
                               caching_mode: program::CachingMode::Set,
                               key_func:     None,
                               id:           18,
                               rules:        vec![
                               ],
                               arrangements: vec![
                                   types__inputs::__Arng_inputs_ConstDecl_0.clone()
                               ],
                               change_cb:    None
                           };
    let inputs_Continue = program::Relation {
                              name:         std::borrow::Cow::from("inputs::Continue"),
                              input:        true,
                              distinct:     false,
                              caching_mode: program::CachingMode::Set,
                              key_func:     None,
                              id:           19,
                              rules:        vec![
                              ],
                              arrangements: vec![
                                  types__inputs::__Arng_inputs_Continue_0.clone()
                              ],
                              change_cb:    None
                          };
    let inputs_DoWhile = program::Relation {
                             name:         std::borrow::Cow::from("inputs::DoWhile"),
                             input:        true,
                             distinct:     false,
                             caching_mode: program::CachingMode::Set,
                             key_func:     None,
                             id:           20,
                             rules:        vec![
                             ],
                             arrangements: vec![
                             ],
                             change_cb:    None
                         };
    let inputs_DotAccess = program::Relation {
                               name:         std::borrow::Cow::from("inputs::DotAccess"),
                               input:        true,
                               distinct:     false,
                               caching_mode: program::CachingMode::Set,
                               key_func:     None,
                               id:           21,
                               rules:        vec![
                               ],
                               arrangements: vec![
                                   types__inputs::__Arng_inputs_DotAccess_0.clone()
                               ],
                               change_cb:    None
                           };
    let outputs_no_undef_ChainedWith = program::Relation {
                                           name:         std::borrow::Cow::from("outputs::no_undef::ChainedWith"),
                                           input:        false,
                                           distinct:     false,
                                           caching_mode: program::CachingMode::Set,
                                           key_func:     None,
                                           id:           70,
                                           rules:        vec![
                                               types::outputs::no_undef::__Rule_outputs_no_undef_ChainedWith_0.clone(),
                                               types::outputs::no_undef::__Rule_outputs_no_undef_ChainedWith_1.clone(),
                                               types::outputs::no_undef::__Rule_outputs_no_undef_ChainedWith_2.clone()
                                           ],
                                           arrangements: vec![
                                               types::outputs::no_undef::__Arng_outputs_no_undef_ChainedWith_0.clone(),
                                               types::outputs::no_undef::__Arng_outputs_no_undef_ChainedWith_1.clone(),
                                               types::outputs::no_undef::__Arng_outputs_no_undef_ChainedWith_2.clone()
                                           ],
                                           change_cb:    None
                                       };
    let inputs_EveryScope = program::Relation {
                                name:         std::borrow::Cow::from("inputs::EveryScope"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           22,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                ],
                                change_cb:    None
                            };
    let inputs_ExprBigInt = program::Relation {
                                name:         std::borrow::Cow::from("inputs::ExprBigInt"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           23,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                ],
                                change_cb:    None
                            };
    let inputs_ExprBool = program::Relation {
                              name:         std::borrow::Cow::from("inputs::ExprBool"),
                              input:        true,
                              distinct:     false,
                              caching_mode: program::CachingMode::Set,
                              key_func:     None,
                              id:           24,
                              rules:        vec![
                              ],
                              arrangements: vec![
                              ],
                              change_cb:    None
                          };
    let inputs_ExprNumber = program::Relation {
                                name:         std::borrow::Cow::from("inputs::ExprNumber"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           25,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                ],
                                change_cb:    None
                            };
    let inputs_ExprString = program::Relation {
                                name:         std::borrow::Cow::from("inputs::ExprString"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           26,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                ],
                                change_cb:    None
                            };
    let inputs_Expression = program::Relation {
                                name:         std::borrow::Cow::from("inputs::Expression"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     Some(types__inputs::__Key_inputs_Expression),
                                id:           27,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                    types__inputs::__Arng_inputs_Expression_0.clone(),
                                    types__inputs::__Arng_inputs_Expression_1.clone(),
                                    types__inputs::__Arng_inputs_Expression_2.clone(),
                                    types__inputs::__Arng_inputs_Expression_3.clone(),
                                    types__inputs::__Arng_inputs_Expression_4.clone()
                                ],
                                change_cb:    None
                            };
    let inputs_File = program::Relation {
                          name:         std::borrow::Cow::from("inputs::File"),
                          input:        true,
                          distinct:     false,
                          caching_mode: program::CachingMode::Set,
                          key_func:     Some(types__inputs::__Key_inputs_File),
                          id:           28,
                          rules:        vec![
                          ],
                          arrangements: vec![
                              types__inputs::__Arng_inputs_File_0.clone(),
                              types__inputs::__Arng_inputs_File_1.clone()
                          ],
                          change_cb:    None
                      };
    let inputs_FileExport = program::Relation {
                                name:         std::borrow::Cow::from("inputs::FileExport"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           29,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                ],
                                change_cb:    None
                            };
    let inputs_For = program::Relation {
                         name:         std::borrow::Cow::from("inputs::For"),
                         input:        true,
                         distinct:     false,
                         caching_mode: program::CachingMode::Set,
                         key_func:     None,
                         id:           30,
                         rules:        vec![
                         ],
                         arrangements: vec![
                         ],
                         change_cb:    None
                     };
    let inputs_ForIn = program::Relation {
                           name:         std::borrow::Cow::from("inputs::ForIn"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           31,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let inputs_ForOf = program::Relation {
                           name:         std::borrow::Cow::from("inputs::ForOf"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           32,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let inputs_Function = program::Relation {
                              name:         std::borrow::Cow::from("inputs::Function"),
                              input:        true,
                              distinct:     false,
                              caching_mode: program::CachingMode::Set,
                              key_func:     Some(types__inputs::__Key_inputs_Function),
                              id:           33,
                              rules:        vec![
                              ],
                              arrangements: vec![
                                  types__inputs::__Arng_inputs_Function_0.clone(),
                                  types__inputs::__Arng_inputs_Function_1.clone(),
                                  types__inputs::__Arng_inputs_Function_2.clone(),
                                  types__inputs::__Arng_inputs_Function_3.clone()
                              ],
                              change_cb:    None
                          };
    let inputs_FunctionArg = program::Relation {
                                 name:         std::borrow::Cow::from("inputs::FunctionArg"),
                                 input:        true,
                                 distinct:     false,
                                 caching_mode: program::CachingMode::Set,
                                 key_func:     None,
                                 id:           34,
                                 rules:        vec![
                                 ],
                                 arrangements: vec![
                                     types__inputs::__Arng_inputs_FunctionArg_0.clone()
                                 ],
                                 change_cb:    None
                             };
    let inputs_If = program::Relation {
                        name:         std::borrow::Cow::from("inputs::If"),
                        input:        true,
                        distinct:     false,
                        caching_mode: program::CachingMode::Set,
                        key_func:     None,
                        id:           35,
                        rules:        vec![
                        ],
                        arrangements: vec![
                        ],
                        change_cb:    None
                    };
    let inputs_ImplicitGlobal = program::Relation {
                                    name:         std::borrow::Cow::from("inputs::ImplicitGlobal"),
                                    input:        true,
                                    distinct:     false,
                                    caching_mode: program::CachingMode::Set,
                                    key_func:     None,
                                    id:           36,
                                    rules:        vec![
                                    ],
                                    arrangements: vec![
                                        types__inputs::__Arng_inputs_ImplicitGlobal_0.clone()
                                    ],
                                    change_cb:    None
                                };
    let inputs_ImportDecl = program::Relation {
                                name:         std::borrow::Cow::from("inputs::ImportDecl"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           37,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                    types__inputs::__Arng_inputs_ImportDecl_0.clone()
                                ],
                                change_cb:    None
                            };
    let inputs_InlineFunc = program::Relation {
                                name:         std::borrow::Cow::from("inputs::InlineFunc"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           38,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                    types__inputs::__Arng_inputs_InlineFunc_0.clone(),
                                    types__inputs::__Arng_inputs_InlineFunc_1.clone(),
                                    types__inputs::__Arng_inputs_InlineFunc_2.clone(),
                                    types__inputs::__Arng_inputs_InlineFunc_3.clone()
                                ],
                                change_cb:    None
                            };
    let inputs_InlineFuncParam = program::Relation {
                                     name:         std::borrow::Cow::from("inputs::InlineFuncParam"),
                                     input:        true,
                                     distinct:     false,
                                     caching_mode: program::CachingMode::Set,
                                     key_func:     None,
                                     id:           39,
                                     rules:        vec![
                                     ],
                                     arrangements: vec![
                                         types__inputs::__Arng_inputs_InlineFuncParam_0.clone()
                                     ],
                                     change_cb:    None
                                 };
    let inputs_InputScope = program::Relation {
                                name:         std::borrow::Cow::from("inputs::InputScope"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           40,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                    types__inputs::__Arng_inputs_InputScope_0.clone(),
                                    types__inputs::__Arng_inputs_InputScope_1.clone()
                                ],
                                change_cb:    None
                            };
    let inputs_Label = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Label"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           41,
                           rules:        vec![
                           ],
                           arrangements: vec![
                               types__inputs::__Arng_inputs_Label_0.clone(),
                               types__inputs::__Arng_inputs_Label_1.clone()
                           ],
                           change_cb:    None
                       };
    let outputs_no_unused_labels___Prefix_1 = program::Relation {
                                                  name:         std::borrow::Cow::from("outputs::no_unused_labels::__Prefix_1"),
                                                  input:        false,
                                                  distinct:     false,
                                                  caching_mode: program::CachingMode::Set,
                                                  key_func:     None,
                                                  id:           75,
                                                  rules:        vec![
                                                      types__outputs__no_unused_labels::__Rule_outputs_no_unused_labels___Prefix_1_0.clone()
                                                  ],
                                                  arrangements: vec![
                                                      types__outputs__no_unused_labels::__Arng_outputs_no_unused_labels___Prefix_1_0.clone(),
                                                      types__outputs__no_unused_labels::__Arng_outputs_no_unused_labels___Prefix_1_1.clone()
                                                  ],
                                                  change_cb:    None
                                              };
    let inputs_LetDecl = program::Relation {
                             name:         std::borrow::Cow::from("inputs::LetDecl"),
                             input:        true,
                             distinct:     false,
                             caching_mode: program::CachingMode::Set,
                             key_func:     None,
                             id:           42,
                             rules:        vec![
                             ],
                             arrangements: vec![
                                 types__inputs::__Arng_inputs_LetDecl_0.clone()
                             ],
                             change_cb:    None
                         };
    let inputs_NameRef = program::Relation {
                             name:         std::borrow::Cow::from("inputs::NameRef"),
                             input:        true,
                             distinct:     false,
                             caching_mode: program::CachingMode::Set,
                             key_func:     None,
                             id:           43,
                             rules:        vec![
                             ],
                             arrangements: vec![
                                 types__inputs::__Arng_inputs_NameRef_0.clone(),
                                 types__inputs::__Arng_inputs_NameRef_1.clone(),
                                 types__inputs::__Arng_inputs_NameRef_2.clone()
                             ],
                             change_cb:    None
                         };
    let name_in_scope_NameOccursInScope = program::Relation {
                                              name:         std::borrow::Cow::from("name_in_scope::NameOccursInScope"),
                                              input:        false,
                                              distinct:     false,
                                              caching_mode: program::CachingMode::Set,
                                              key_func:     None,
                                              id:           62,
                                              rules:        vec![
                                                  types::name_in_scope::__Rule_name_in_scope_NameOccursInScope_0.clone(),
                                                  types::name_in_scope::__Rule_name_in_scope_NameOccursInScope_1.clone(),
                                                  types::name_in_scope::__Rule_name_in_scope_NameOccursInScope_2.clone(),
                                                  types::name_in_scope::__Rule_name_in_scope_NameOccursInScope_3.clone()
                                              ],
                                              arrangements: vec![
                                                  types::name_in_scope::__Arng_name_in_scope_NameOccursInScope_0.clone(),
                                                  types::name_in_scope::__Arng_name_in_scope_NameOccursInScope_1.clone(),
                                                  types::name_in_scope::__Arng_name_in_scope_NameOccursInScope_2.clone()
                                              ],
                                              change_cb:    None
                                          };
    let inputs_New = program::Relation {
                         name:         std::borrow::Cow::from("inputs::New"),
                         input:        true,
                         distinct:     false,
                         caching_mode: program::CachingMode::Set,
                         key_func:     None,
                         id:           44,
                         rules:        vec![
                         ],
                         arrangements: vec![
                             types__inputs::__Arng_inputs_New_0.clone(),
                             types__inputs::__Arng_inputs_New_1.clone()
                         ],
                         change_cb:    None
                     };
    let __Prefix_0 = program::Relation {
                         name:         std::borrow::Cow::from("__Prefix_0"),
                         input:        false,
                         distinct:     false,
                         caching_mode: program::CachingMode::Set,
                         key_func:     None,
                         id:           0,
                         rules:        vec![
                             types::__Rule___Prefix_0_0.clone()
                         ],
                         arrangements: vec![
                             types::__Arng___Prefix_0_0.clone(),
                             types::__Arng___Prefix_0_1.clone()
                         ],
                         change_cb:    None
                     };
    let inputs_Property = program::Relation {
                              name:         std::borrow::Cow::from("inputs::Property"),
                              input:        true,
                              distinct:     false,
                              caching_mode: program::CachingMode::Set,
                              key_func:     None,
                              id:           45,
                              rules:        vec![
                              ],
                              arrangements: vec![
                              ],
                              change_cb:    None
                          };
    let inputs_Return = program::Relation {
                            name:         std::borrow::Cow::from("inputs::Return"),
                            input:        true,
                            distinct:     false,
                            caching_mode: program::CachingMode::Set,
                            key_func:     None,
                            id:           46,
                            rules:        vec![
                            ],
                            arrangements: vec![
                            ],
                            change_cb:    None
                        };
    let inputs_Statement = program::Relation {
                               name:         std::borrow::Cow::from("inputs::Statement"),
                               input:        true,
                               distinct:     false,
                               caching_mode: program::CachingMode::Set,
                               key_func:     Some(types__inputs::__Key_inputs_Statement),
                               id:           47,
                               rules:        vec![
                               ],
                               arrangements: vec![
                                   types__inputs::__Arng_inputs_Statement_0.clone(),
                                   types__inputs::__Arng_inputs_Statement_1.clone(),
                                   types__inputs::__Arng_inputs_Statement_2.clone()
                               ],
                               change_cb:    None
                           };
    let outputs_no_unused_labels_LabelUsage = program::Relation {
                                                  name:         std::borrow::Cow::from("outputs::no_unused_labels::LabelUsage"),
                                                  input:        false,
                                                  distinct:     false,
                                                  caching_mode: program::CachingMode::Set,
                                                  key_func:     None,
                                                  id:           72,
                                                  rules:        vec![
                                                      types__outputs__no_unused_labels::__Rule_outputs_no_unused_labels_LabelUsage_0.clone(),
                                                      types__outputs__no_unused_labels::__Rule_outputs_no_unused_labels_LabelUsage_1.clone()
                                                  ],
                                                  arrangements: vec![
                                                      types__outputs__no_unused_labels::__Arng_outputs_no_unused_labels_LabelUsage_0.clone(),
                                                      types__outputs__no_unused_labels::__Arng_outputs_no_unused_labels_LabelUsage_1.clone()
                                                  ],
                                                  change_cb:    None
                                              };
    let scopes_NeedsScopeChildren = program::Relation {
                                        name:         std::borrow::Cow::from("scopes::NeedsScopeChildren"),
                                        input:        false,
                                        distinct:     false,
                                        caching_mode: program::CachingMode::Set,
                                        key_func:     None,
                                        id:           81,
                                        rules:        vec![
                                            types__outputs__no_unused_labels::__Rule_scopes_NeedsScopeChildren_0.clone()
                                        ],
                                        arrangements: vec![
                                            types__scopes::__Arng_scopes_NeedsScopeChildren_0.clone()
                                        ],
                                        change_cb:    None
                                    };
    let outputs_unused_vars_FunctionBodyScope = program::Relation {
                                                    name:         std::borrow::Cow::from("outputs::unused_vars::FunctionBodyScope"),
                                                    input:        false,
                                                    distinct:     false,
                                                    caching_mode: program::CachingMode::Set,
                                                    key_func:     None,
                                                    id:           77,
                                                    rules:        vec![
                                                        types::outputs::unused_vars::__Rule_outputs_unused_vars_FunctionBodyScope_0.clone(),
                                                        types::outputs::unused_vars::__Rule_outputs_unused_vars_FunctionBodyScope_1.clone(),
                                                        types::outputs::unused_vars::__Rule_outputs_unused_vars_FunctionBodyScope_2.clone()
                                                    ],
                                                    arrangements: vec![
                                                        types::outputs::unused_vars::__Arng_outputs_unused_vars_FunctionBodyScope_0.clone()
                                                    ],
                                                    change_cb:    None
                                                };
    let scopes_FunctionLevelScope = program::Relation {
                                        name:         std::borrow::Cow::from("scopes::FunctionLevelScope"),
                                        input:        false,
                                        distinct:     false,
                                        caching_mode: program::CachingMode::Set,
                                        key_func:     None,
                                        id:           79,
                                        rules:        vec![
                                            types__scopes::__Rule_scopes_FunctionLevelScope_0.clone(),
                                            types__scopes::__Rule_scopes_FunctionLevelScope_1.clone(),
                                            types__scopes::__Rule_scopes_FunctionLevelScope_2.clone(),
                                            types__scopes::__Rule_scopes_FunctionLevelScope_3.clone(),
                                            types__scopes::__Rule_scopes_FunctionLevelScope_4.clone(),
                                            types__scopes::__Rule_scopes_FunctionLevelScope_5.clone(),
                                            types__scopes::__Rule_scopes_FunctionLevelScope_6.clone(),
                                            types__scopes::__Rule_scopes_FunctionLevelScope_7.clone()
                                        ],
                                        arrangements: vec![
                                            types__scopes::__Arng_scopes_FunctionLevelScope_0.clone()
                                        ],
                                        change_cb:    None
                                    };
    let scopes_ScopeOfId = program::Relation {
                               name:         std::borrow::Cow::from("scopes::ScopeOfId"),
                               input:        false,
                               distinct:     false,
                               caching_mode: program::CachingMode::Set,
                               key_func:     None,
                               id:           84,
                               rules:        vec![
                                   types__scopes::__Rule_scopes_ScopeOfId_0.clone(),
                                   types__scopes::__Rule_scopes_ScopeOfId_1.clone(),
                                   types__scopes::__Rule_scopes_ScopeOfId_2.clone(),
                                   types__scopes::__Rule_scopes_ScopeOfId_3.clone(),
                                   types__scopes::__Rule_scopes_ScopeOfId_4.clone(),
                                   types__scopes::__Rule_scopes_ScopeOfId_5.clone()
                               ],
                               arrangements: vec![
                               ],
                               change_cb:    None
                           };
    let inputs_Switch = program::Relation {
                            name:         std::borrow::Cow::from("inputs::Switch"),
                            input:        true,
                            distinct:     false,
                            caching_mode: program::CachingMode::Set,
                            key_func:     None,
                            id:           48,
                            rules:        vec![
                            ],
                            arrangements: vec![
                            ],
                            change_cb:    None
                        };
    let inputs_SwitchCase = program::Relation {
                                name:         std::borrow::Cow::from("inputs::SwitchCase"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           49,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                ],
                                change_cb:    None
                            };
    let inputs_Template = program::Relation {
                              name:         std::borrow::Cow::from("inputs::Template"),
                              input:        true,
                              distinct:     false,
                              caching_mode: program::CachingMode::Set,
                              key_func:     None,
                              id:           50,
                              rules:        vec![
                              ],
                              arrangements: vec![
                              ],
                              change_cb:    None
                          };
    let inputs_Ternary = program::Relation {
                             name:         std::borrow::Cow::from("inputs::Ternary"),
                             input:        true,
                             distinct:     false,
                             caching_mode: program::CachingMode::Set,
                             key_func:     None,
                             id:           51,
                             rules:        vec![
                             ],
                             arrangements: vec![
                             ],
                             change_cb:    None
                         };
    let inputs_Throw = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Throw"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           52,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let inputs_Try = program::Relation {
                         name:         std::borrow::Cow::from("inputs::Try"),
                         input:        true,
                         distinct:     false,
                         caching_mode: program::CachingMode::Set,
                         key_func:     None,
                         id:           53,
                         rules:        vec![
                         ],
                         arrangements: vec![
                             types__inputs::__Arng_inputs_Try_0.clone()
                         ],
                         change_cb:    None
                     };
    let inputs_UnaryOp = program::Relation {
                             name:         std::borrow::Cow::from("inputs::UnaryOp"),
                             input:        true,
                             distinct:     false,
                             caching_mode: program::CachingMode::Set,
                             key_func:     None,
                             id:           54,
                             rules:        vec![
                             ],
                             arrangements: vec![
                                 types__inputs::__Arng_inputs_UnaryOp_0.clone()
                             ],
                             change_cb:    None
                         };
    let outputs_no_typeof_undef_WithinTypeofExpr = program::Relation {
                                                       name:         std::borrow::Cow::from("outputs::no_typeof_undef::WithinTypeofExpr"),
                                                       input:        false,
                                                       distinct:     false,
                                                       caching_mode: program::CachingMode::Set,
                                                       key_func:     None,
                                                       id:           69,
                                                       rules:        vec![
                                                           types::outputs::no_typeof_undef::__Rule_outputs_no_typeof_undef_WithinTypeofExpr_0.clone(),
                                                           types::outputs::no_typeof_undef::__Rule_outputs_no_typeof_undef_WithinTypeofExpr_1.clone(),
                                                           types::outputs::no_typeof_undef::__Rule_outputs_no_typeof_undef_WithinTypeofExpr_2.clone()
                                                       ],
                                                       arrangements: vec![
                                                           types::outputs::no_typeof_undef::__Arng_outputs_no_typeof_undef_WithinTypeofExpr_0.clone(),
                                                           types::outputs::no_typeof_undef::__Arng_outputs_no_typeof_undef_WithinTypeofExpr_1.clone()
                                                       ],
                                                       change_cb:    None
                                                   };
    let inputs_UserGlobal = program::Relation {
                                name:         std::borrow::Cow::from("inputs::UserGlobal"),
                                input:        true,
                                distinct:     false,
                                caching_mode: program::CachingMode::Set,
                                key_func:     None,
                                id:           55,
                                rules:        vec![
                                ],
                                arrangements: vec![
                                    types__inputs::__Arng_inputs_UserGlobal_0.clone()
                                ],
                                change_cb:    None
                            };
    let inputs_VarDecl = program::Relation {
                             name:         std::borrow::Cow::from("inputs::VarDecl"),
                             input:        true,
                             distinct:     false,
                             caching_mode: program::CachingMode::Set,
                             key_func:     None,
                             id:           56,
                             rules:        vec![
                             ],
                             arrangements: vec![
                                 types__inputs::__Arng_inputs_VarDecl_0.clone()
                             ],
                             change_cb:    None
                         };
    let var_decls_VariableDeclarations = program::Relation {
                                             name:         std::borrow::Cow::from("var_decls::VariableDeclarations"),
                                             input:        false,
                                             distinct:     false,
                                             caching_mode: program::CachingMode::Set,
                                             key_func:     None,
                                             id:           85,
                                             rules:        vec![
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_0.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_1.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_2.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_3.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_4.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_5.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_6.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_7.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_8.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_9.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_10.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_11.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_12.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_13.clone(),
                                                 types::var_decls::__Rule_var_decls_VariableDeclarations_14.clone()
                                             ],
                                             arrangements: vec![
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_0.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_1.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_2.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_3.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_4.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_5.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_6.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_7.clone(),
                                                 types::var_decls::__Arng_var_decls_VariableDeclarations_8.clone()
                                             ],
                                             change_cb:    None
                                         };
    let outputs_no_shadow_ScopeOfDecl = program::Relation {
                                            name:         std::borrow::Cow::from("outputs::no_shadow::ScopeOfDecl"),
                                            input:        false,
                                            distinct:     false,
                                            caching_mode: program::CachingMode::Set,
                                            key_func:     None,
                                            id:           66,
                                            rules:        vec![
                                                types::outputs::no_shadow::__Rule_outputs_no_shadow_ScopeOfDecl_0.clone(),
                                                types::outputs::no_shadow::__Rule_outputs_no_shadow_ScopeOfDecl_1.clone()
                                            ],
                                            arrangements: vec![
                                                types::outputs::no_shadow::__Arng_outputs_no_shadow_ScopeOfDecl_0.clone()
                                            ],
                                            change_cb:    None
                                        };
    let outputs_no_shadow_DeclarationInDescendent = program::Relation {
                                                        name:         std::borrow::Cow::from("outputs::no_shadow::DeclarationInDescendent"),
                                                        input:        false,
                                                        distinct:     false,
                                                        caching_mode: program::CachingMode::Set,
                                                        key_func:     None,
                                                        id:           64,
                                                        rules:        vec![
                                                            types::outputs::no_shadow::__Rule_outputs_no_shadow_DeclarationInDescendent_0.clone(),
                                                            types::outputs::no_shadow::__Rule_outputs_no_shadow_DeclarationInDescendent_1.clone()
                                                        ],
                                                        arrangements: vec![
                                                            types::outputs::no_shadow::__Arng_outputs_no_shadow_DeclarationInDescendent_0.clone(),
                                                            types::outputs::no_shadow::__Arng_outputs_no_shadow_DeclarationInDescendent_1.clone()
                                                        ],
                                                        change_cb:    None
                                                    };
    let outputs_no_shadow_NoShadow = program::Relation {
                                         name:         std::borrow::Cow::from("outputs::no_shadow::NoShadow"),
                                         input:        false,
                                         distinct:     true,
                                         caching_mode: program::CachingMode::Set,
                                         key_func:     None,
                                         id:           65,
                                         rules:        vec![
                                             types::outputs::no_shadow::__Rule_outputs_no_shadow_NoShadow_0.clone()
                                         ],
                                         arrangements: vec![
                                         ],
                                         change_cb:    Some(std::sync::Arc::clone(&__update_cb))
                                     };
    let name_in_scope_ScopeOfDeclName = program::Relation {
                                            name:         std::borrow::Cow::from("name_in_scope::ScopeOfDeclName"),
                                            input:        false,
                                            distinct:     false,
                                            caching_mode: program::CachingMode::Set,
                                            key_func:     None,
                                            id:           63,
                                            rules:        vec![
                                                types::name_in_scope::__Rule_name_in_scope_ScopeOfDeclName_0.clone(),
                                                types::name_in_scope::__Rule_name_in_scope_ScopeOfDeclName_1.clone()
                                            ],
                                            arrangements: vec![
                                                types::name_in_scope::__Arng_name_in_scope_ScopeOfDeclName_0.clone()
                                            ],
                                            change_cb:    None
                                        };
    let name_in_scope_NameInScope = program::Relation {
                                        name:         std::borrow::Cow::from("name_in_scope::NameInScope"),
                                        input:        false,
                                        distinct:     false,
                                        caching_mode: program::CachingMode::Set,
                                        key_func:     None,
                                        id:           61,
                                        rules:        vec![
                                            types::name_in_scope::__Rule_name_in_scope_NameInScope_0.clone(),
                                            types::name_in_scope::__Rule_name_in_scope_NameInScope_1.clone()
                                        ],
                                        arrangements: vec![
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_0.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_1.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_2.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_3.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_4.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_5.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_6.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_7.clone(),
                                            types::name_in_scope::__Arng_name_in_scope_NameInScope_8.clone()
                                        ],
                                        change_cb:    None
                                    };
    let outputs_no_typeof_undef_NoTypeofUndef = program::Relation {
                                                    name:         std::borrow::Cow::from("outputs::no_typeof_undef::NoTypeofUndef"),
                                                    input:        false,
                                                    distinct:     true,
                                                    caching_mode: program::CachingMode::Set,
                                                    key_func:     None,
                                                    id:           68,
                                                    rules:        vec![
                                                        types::outputs::no_typeof_undef::__Rule_outputs_no_typeof_undef_NoTypeofUndef_0.clone()
                                                    ],
                                                    arrangements: vec![
                                                    ],
                                                    change_cb:    Some(std::sync::Arc::clone(&__update_cb))
                                                };
    let outputs_no_undef_NoUndef = program::Relation {
                                       name:         std::borrow::Cow::from("outputs::no_undef::NoUndef"),
                                       input:        false,
                                       distinct:     true,
                                       caching_mode: program::CachingMode::Set,
                                       key_func:     None,
                                       id:           71,
                                       rules:        vec![
                                           types::outputs::no_undef::__Rule_outputs_no_undef_NoUndef_0.clone(),
                                           types::outputs::no_undef::__Rule_outputs_no_undef_NoUndef_1.clone()
                                       ],
                                       arrangements: vec![
                                       ],
                                       change_cb:    Some(std::sync::Arc::clone(&__update_cb))
                                   };
    let is_exported_IsExported = program::Relation {
                                     name:         std::borrow::Cow::from("is_exported::IsExported"),
                                     input:        false,
                                     distinct:     false,
                                     caching_mode: program::CachingMode::Set,
                                     key_func:     None,
                                     id:           60,
                                     rules:        vec![
                                         types::is_exported::__Rule_is_exported_IsExported_0.clone(),
                                         types::is_exported::__Rule_is_exported_IsExported_1.clone(),
                                         types::is_exported::__Rule_is_exported_IsExported_2.clone(),
                                         types::is_exported::__Rule_is_exported_IsExported_3.clone(),
                                         types::is_exported::__Rule_is_exported_IsExported_4.clone(),
                                         types::is_exported::__Rule_is_exported_IsExported_5.clone()
                                     ],
                                     arrangements: vec![
                                         types::is_exported::__Arng_is_exported_IsExported_0.clone()
                                     ],
                                     change_cb:    None
                                 };
    let outputs_unused_vars_UnusedVariables = program::Relation {
                                                  name:         std::borrow::Cow::from("outputs::unused_vars::UnusedVariables"),
                                                  input:        false,
                                                  distinct:     true,
                                                  caching_mode: program::CachingMode::Set,
                                                  key_func:     None,
                                                  id:           78,
                                                  rules:        vec![
                                                      types::outputs::unused_vars::__Rule_outputs_unused_vars_UnusedVariables_0.clone(),
                                                      types::outputs::unused_vars::__Rule_outputs_unused_vars_UnusedVariables_1.clone(),
                                                      types::outputs::unused_vars::__Rule_outputs_unused_vars_UnusedVariables_2.clone()
                                                  ],
                                                  arrangements: vec![
                                                  ],
                                                  change_cb:    Some(std::sync::Arc::clone(&__update_cb))
                                              };
    let variable_decl_VariableDecl = program::Relation {
                                         name:         std::borrow::Cow::from("variable_decl::VariableDecl"),
                                         input:        false,
                                         distinct:     false,
                                         caching_mode: program::CachingMode::Set,
                                         key_func:     None,
                                         id:           86,
                                         rules:        vec![
                                             types__variable_decl::__Rule_variable_decl_VariableDecl_0.clone(),
                                             types__variable_decl::__Rule_variable_decl_VariableDecl_1.clone(),
                                             types__variable_decl::__Rule_variable_decl_VariableDecl_2.clone()
                                         ],
                                         arrangements: vec![
                                             types__variable_decl::__Arng_variable_decl_VariableDecl_0.clone()
                                         ],
                                         change_cb:    None
                                     };
    let outputs_no_use_before_def_NoUseBeforeDef = program::Relation {
                                                       name:         std::borrow::Cow::from("outputs::no_use_before_def::NoUseBeforeDef"),
                                                       input:        false,
                                                       distinct:     true,
                                                       caching_mode: program::CachingMode::Set,
                                                       key_func:     None,
                                                       id:           76,
                                                       rules:        vec![
                                                           types::outputs::no_use_before_def::__Rule_outputs_no_use_before_def_NoUseBeforeDef_0.clone(),
                                                           types::outputs::no_use_before_def::__Rule_outputs_no_use_before_def_NoUseBeforeDef_1.clone(),
                                                           types::outputs::no_use_before_def::__Rule_outputs_no_use_before_def_NoUseBeforeDef_2.clone(),
                                                           types::outputs::no_use_before_def::__Rule_outputs_no_use_before_def_NoUseBeforeDef_3.clone()
                                                       ],
                                                       arrangements: vec![
                                                       ],
                                                       change_cb:    Some(std::sync::Arc::clone(&__update_cb))
                                                   };
    let scopes_IsHoistable = program::Relation {
                                 name:         std::borrow::Cow::from("scopes::IsHoistable"),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: program::CachingMode::Set,
                                 key_func:     None,
                                 id:           80,
                                 rules:        vec![
                                     types__scopes::__Rule_scopes_IsHoistable_0.clone(),
                                     types__scopes::__Rule_scopes_IsHoistable_1.clone()
                                 ],
                                 arrangements: vec![
                                 ],
                                 change_cb:    None
                             };
    let inputs_While = program::Relation {
                           name:         std::borrow::Cow::from("inputs::While"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           57,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let inputs_With = program::Relation {
                          name:         std::borrow::Cow::from("inputs::With"),
                          input:        true,
                          distinct:     false,
                          caching_mode: program::CachingMode::Set,
                          key_func:     None,
                          id:           58,
                          rules:        vec![
                          ],
                          arrangements: vec![
                          ],
                          change_cb:    None
                      };
    let inputs_Yield = program::Relation {
                           name:         std::borrow::Cow::from("inputs::Yield"),
                           input:        true,
                           distinct:     false,
                           caching_mode: program::CachingMode::Set,
                           key_func:     None,
                           id:           59,
                           rules:        vec![
                           ],
                           arrangements: vec![
                           ],
                           change_cb:    None
                       };
    let scopes_NeedsScopeParents = program::Relation {
                                       name:         std::borrow::Cow::from("scopes::NeedsScopeParents"),
                                       input:        false,
                                       distinct:     false,
                                       caching_mode: program::CachingMode::Set,
                                       key_func:     None,
                                       id:           82,
                                       rules:        vec![
                                       ],
                                       arrangements: vec![
                                           types__scopes::__Arng_scopes_NeedsScopeParents_0.clone()
                                       ],
                                       change_cb:    None
                                   };
    let scopes_ScopeFamily = program::Relation {
                                 name:         std::borrow::Cow::from("scopes::ScopeFamily"),
                                 input:        false,
                                 distinct:     false,
                                 caching_mode: program::CachingMode::Set,
                                 key_func:     None,
                                 id:           83,
                                 rules:        vec![
                                     types__scopes::__Rule_scopes_ScopeFamily_0.clone(),
                                     types__scopes::__Rule_scopes_ScopeFamily_1.clone(),
                                     types__scopes::__Rule_scopes_ScopeFamily_2.clone()
                                 ],
                                 arrangements: vec![
                                     types__scopes::__Arng_scopes_ScopeFamily_0.clone(),
                                     types__scopes::__Arng_scopes_ScopeFamily_1.clone(),
                                     types__scopes::__Arng_scopes_ScopeFamily_2.clone()
                                 ],
                                 change_cb:    None
                             };
    let outputs_no_unused_labels_UsedLabels = program::Relation {
                                                  name:         std::borrow::Cow::from("outputs::no_unused_labels::UsedLabels"),
                                                  input:        false,
                                                  distinct:     true,
                                                  caching_mode: program::CachingMode::Set,
                                                  key_func:     None,
                                                  id:           74,
                                                  rules:        vec![
                                                      types__outputs__no_unused_labels::__Rule_outputs_no_unused_labels_UsedLabels_0.clone(),
                                                      types__outputs__no_unused_labels::__Rule_outputs_no_unused_labels_UsedLabels_1.clone()
                                                  ],
                                                  arrangements: vec![
                                                      types__outputs__no_unused_labels::__Arng_outputs_no_unused_labels_UsedLabels_0.clone()
                                                  ],
                                                  change_cb:    Some(std::sync::Arc::clone(&__update_cb))
                                              };
    let outputs_no_unused_labels_NoUnusedLabels = program::Relation {
                                                      name:         std::borrow::Cow::from("outputs::no_unused_labels::NoUnusedLabels"),
                                                      input:        false,
                                                      distinct:     true,
                                                      caching_mode: program::CachingMode::Set,
                                                      key_func:     None,
                                                      id:           73,
                                                      rules:        vec![
                                                          types__outputs__no_unused_labels::__Rule_outputs_no_unused_labels_NoUnusedLabels_0.clone()
                                                      ],
                                                      arrangements: vec![
                                                      ],
                                                      change_cb:    Some(std::sync::Arc::clone(&__update_cb))
                                                  };
    let nodes: std::vec::Vec<program::ProgNode> = vec![
            program::ProgNode::Rel{rel: config_EnableNoShadow},
            program::ProgNode::Rel{rel: config_EnableNoTypeofUndef},
            program::ProgNode::Rel{rel: config_EnableNoUndef},
            program::ProgNode::Rel{rel: outputs_no_typeof_undef_NeedsWithinTypeofExpr},
            program::ProgNode::Rel{rel: config_EnableNoUnusedLabels},
            program::ProgNode::Rel{rel: config_EnableNoUnusedVars},
            program::ProgNode::Rel{rel: config_EnableNoUseBeforeDef},
            program::ProgNode::Rel{rel: inputs_Array},
            program::ProgNode::Rel{rel: inputs_Arrow},
            program::ProgNode::Rel{rel: inputs_ArrowParam},
            program::ProgNode::Rel{rel: inputs_Assign},
            program::ProgNode::Rel{rel: inputs_Await},
            program::ProgNode::Rel{rel: inputs_BinOp},
            program::ProgNode::Rel{rel: inputs_BracketAccess},
            program::ProgNode::Rel{rel: inputs_Break},
            program::ProgNode::Rel{rel: inputs_Call},
            program::ProgNode::Rel{rel: inputs_Class},
            program::ProgNode::Rel{rel: inputs_ClassExpr},
            program::ProgNode::Rel{rel: inputs_ConstDecl},
            program::ProgNode::Rel{rel: inputs_Continue},
            program::ProgNode::Rel{rel: inputs_DoWhile},
            program::ProgNode::Rel{rel: inputs_DotAccess},
            program::ProgNode::SCC{rels: vec![program::RecursiveRelation{rel: outputs_no_undef_ChainedWith, distinct: true}]},
            program::ProgNode::Rel{rel: inputs_EveryScope},
            program::ProgNode::Rel{rel: inputs_ExprBigInt},
            program::ProgNode::Rel{rel: inputs_ExprBool},
            program::ProgNode::Rel{rel: inputs_ExprNumber},
            program::ProgNode::Rel{rel: inputs_ExprString},
            program::ProgNode::Rel{rel: inputs_Expression},
            program::ProgNode::Rel{rel: inputs_File},
            program::ProgNode::Rel{rel: inputs_FileExport},
            program::ProgNode::Rel{rel: inputs_For},
            program::ProgNode::Rel{rel: inputs_ForIn},
            program::ProgNode::Rel{rel: inputs_ForOf},
            program::ProgNode::Rel{rel: inputs_Function},
            program::ProgNode::Rel{rel: inputs_FunctionArg},
            program::ProgNode::Rel{rel: inputs_If},
            program::ProgNode::Rel{rel: inputs_ImplicitGlobal},
            program::ProgNode::Rel{rel: inputs_ImportDecl},
            program::ProgNode::Rel{rel: inputs_InlineFunc},
            program::ProgNode::Rel{rel: inputs_InlineFuncParam},
            program::ProgNode::Rel{rel: inputs_InputScope},
            program::ProgNode::Rel{rel: inputs_Label},
            program::ProgNode::Rel{rel: outputs_no_unused_labels___Prefix_1},
            program::ProgNode::Rel{rel: inputs_LetDecl},
            program::ProgNode::Rel{rel: inputs_NameRef},
            program::ProgNode::SCC{rels: vec![program::RecursiveRelation{rel: name_in_scope_NameOccursInScope, distinct: true}]},
            program::ProgNode::Rel{rel: inputs_New},
            program::ProgNode::Rel{rel: __Prefix_0},
            program::ProgNode::Rel{rel: inputs_Property},
            program::ProgNode::Rel{rel: inputs_Return},
            program::ProgNode::Rel{rel: inputs_Statement},
            program::ProgNode::Rel{rel: outputs_no_unused_labels_LabelUsage},
            program::ProgNode::Rel{rel: scopes_NeedsScopeChildren},
            program::ProgNode::Rel{rel: outputs_unused_vars_FunctionBodyScope},
            program::ProgNode::SCC{rels: vec![program::RecursiveRelation{rel: scopes_FunctionLevelScope, distinct: true}]},
            program::ProgNode::Rel{rel: scopes_ScopeOfId},
            program::ProgNode::Rel{rel: inputs_Switch},
            program::ProgNode::Rel{rel: inputs_SwitchCase},
            program::ProgNode::Rel{rel: inputs_Template},
            program::ProgNode::Rel{rel: inputs_Ternary},
            program::ProgNode::Rel{rel: inputs_Throw},
            program::ProgNode::Rel{rel: inputs_Try},
            program::ProgNode::Rel{rel: inputs_UnaryOp},
            program::ProgNode::SCC{rels: vec![program::RecursiveRelation{rel: outputs_no_typeof_undef_WithinTypeofExpr, distinct: true}]},
            program::ProgNode::Rel{rel: inputs_UserGlobal},
            program::ProgNode::Rel{rel: inputs_VarDecl},
            program::ProgNode::Rel{rel: var_decls_VariableDeclarations},
            program::ProgNode::Rel{rel: outputs_no_shadow_ScopeOfDecl},
            program::ProgNode::SCC{rels: vec![program::RecursiveRelation{rel: outputs_no_shadow_DeclarationInDescendent, distinct: true}]},
            program::ProgNode::Rel{rel: outputs_no_shadow_NoShadow},
            program::ProgNode::Rel{rel: name_in_scope_ScopeOfDeclName},
            program::ProgNode::SCC{rels: vec![program::RecursiveRelation{rel: name_in_scope_NameInScope, distinct: true}]},
            program::ProgNode::Rel{rel: outputs_no_typeof_undef_NoTypeofUndef},
            program::ProgNode::Rel{rel: outputs_no_undef_NoUndef},
            program::ProgNode::Rel{rel: is_exported_IsExported},
            program::ProgNode::Rel{rel: outputs_unused_vars_UnusedVariables},
            program::ProgNode::Rel{rel: variable_decl_VariableDecl},
            program::ProgNode::Rel{rel: outputs_no_use_before_def_NoUseBeforeDef},
            program::ProgNode::Rel{rel: scopes_IsHoistable},
            program::ProgNode::Rel{rel: inputs_While},
            program::ProgNode::Rel{rel: inputs_With},
            program::ProgNode::Rel{rel: inputs_Yield},
            program::ProgNode::Rel{rel: scopes_NeedsScopeParents},
            program::ProgNode::SCC{rels: vec![program::RecursiveRelation{rel: scopes_ScopeFamily, distinct: true}]},
            program::ProgNode::Rel{rel: outputs_no_unused_labels_UsedLabels},
            program::ProgNode::Rel{rel: outputs_no_unused_labels_NoUnusedLabels}
    ];
    let init_data: std::vec::Vec<(program::RelId, DDValue)> = vec![];
    program::Program {
        nodes,
        init_data,
    }
}