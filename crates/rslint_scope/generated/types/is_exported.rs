#![allow(
    path_statements,
    //unused_imports,
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
    clippy::match_single_binding
)]

// Required for #[derive(Serialize, Deserialize)].
use ::serde::Deserialize;
use ::serde::Serialize;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;

use crate::string_append_str;
use crate::string_append;
use crate::std_usize;
use crate::closure;

//
// use crate::ddlog_std;

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct IsExported {
    pub file: crate::ast::FileId,
    pub id: crate::ast::AnyId
}
impl abomonation::Abomonation for IsExported{}
::differential_datalog::decl_struct_from_record!(IsExported["is_exported::IsExported"]<>, ["is_exported::IsExported"][2]{[0]file["file"]: crate::ast::FileId, [1]id["id"]: crate::ast::AnyId});
::differential_datalog::decl_struct_into_record!(IsExported, ["is_exported::IsExported"]<>, file, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(IsExported, <>, file: crate::ast::FileId, id: crate::ast::AnyId);
impl ::std::fmt::Display for IsExported {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::is_exported::IsExported{file,id} => {
                __formatter.write_str("is_exported::IsExported{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for IsExported {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}