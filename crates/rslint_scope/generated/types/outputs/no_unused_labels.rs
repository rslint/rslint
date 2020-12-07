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
pub struct LabelUsage {
    pub stmt: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub label_name: crate::ast::Name,
    pub scope: crate::ast::ScopeId
}
impl abomonation::Abomonation for LabelUsage{}
::differential_datalog::decl_struct_from_record!(LabelUsage["outputs::no_unused_labels::LabelUsage"]<>, ["outputs::no_unused_labels::LabelUsage"][4]{[0]stmt["stmt"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]label_name["label_name"]: crate::ast::Name, [3]scope["scope"]: crate::ast::ScopeId});
::differential_datalog::decl_struct_into_record!(LabelUsage, ["outputs::no_unused_labels::LabelUsage"]<>, stmt, file, label_name, scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(LabelUsage, <>, stmt: crate::ast::StmtId, file: crate::ast::FileId, label_name: crate::ast::Name, scope: crate::ast::ScopeId);
impl ::std::fmt::Display for LabelUsage {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::no_unused_labels::LabelUsage{stmt,file,label_name,scope} => {
                __formatter.write_str("outputs::no_unused_labels::LabelUsage{")?;
                ::std::fmt::Debug::fmt(stmt, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label_name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for LabelUsage {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NoUnusedLabels {
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub label_name: crate::ast::Spanned<crate::ast::Name>
}
impl abomonation::Abomonation for NoUnusedLabels{}
::differential_datalog::decl_struct_from_record!(NoUnusedLabels["outputs::no_unused_labels::NoUnusedLabels"]<>, ["outputs::no_unused_labels::NoUnusedLabels"][3]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]label_name["label_name"]: crate::ast::Spanned<crate::ast::Name>});
::differential_datalog::decl_struct_into_record!(NoUnusedLabels, ["outputs::no_unused_labels::NoUnusedLabels"]<>, stmt_id, file, label_name);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NoUnusedLabels, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, label_name: crate::ast::Spanned<crate::ast::Name>);
impl ::std::fmt::Display for NoUnusedLabels {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::no_unused_labels::NoUnusedLabels{stmt_id,file,label_name} => {
                __formatter.write_str("outputs::no_unused_labels::NoUnusedLabels{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label_name, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NoUnusedLabels {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct UsedLabels {
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub label_name: crate::ast::Name
}
impl abomonation::Abomonation for UsedLabels{}
::differential_datalog::decl_struct_from_record!(UsedLabels["outputs::no_unused_labels::UsedLabels"]<>, ["outputs::no_unused_labels::UsedLabels"][3]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]label_name["label_name"]: crate::ast::Name});
::differential_datalog::decl_struct_into_record!(UsedLabels, ["outputs::no_unused_labels::UsedLabels"]<>, stmt_id, file, label_name);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(UsedLabels, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, label_name: crate::ast::Name);
impl ::std::fmt::Display for UsedLabels {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::no_unused_labels::UsedLabels{stmt_id,file,label_name} => {
                __formatter.write_str("outputs::no_unused_labels::UsedLabels{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label_name, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for UsedLabels {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}