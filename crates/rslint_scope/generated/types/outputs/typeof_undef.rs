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
pub struct TypeofUndef {
    pub whole_expr: crate::ast::ExprId,
    pub undefined_expr: crate::ast::ExprId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for TypeofUndef{}
::differential_datalog::decl_struct_from_record!(TypeofUndef["outputs::typeof_undef::TypeofUndef"]<>, ["outputs::typeof_undef::TypeofUndef"][3]{[0]whole_expr["whole_expr"]: crate::ast::ExprId, [1]undefined_expr["undefined_expr"]: crate::ast::ExprId, [2]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(TypeofUndef, ["outputs::typeof_undef::TypeofUndef"]<>, whole_expr, undefined_expr, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(TypeofUndef, <>, whole_expr: crate::ast::ExprId, undefined_expr: crate::ast::ExprId, file: crate::ast::FileId);
impl ::std::fmt::Display for TypeofUndef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::typeof_undef::TypeofUndef{whole_expr,undefined_expr,file} => {
                __formatter.write_str("outputs::typeof_undef::TypeofUndef{")?;
                ::std::fmt::Debug::fmt(whole_expr, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(undefined_expr, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for TypeofUndef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct WithinTypeofExpr {
    pub type_of: crate::ast::ExprId,
    pub expr: crate::ast::ExprId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for WithinTypeofExpr{}
::differential_datalog::decl_struct_from_record!(WithinTypeofExpr["outputs::typeof_undef::WithinTypeofExpr"]<>, ["outputs::typeof_undef::WithinTypeofExpr"][3]{[0]type_of["type_of"]: crate::ast::ExprId, [1]expr["expr"]: crate::ast::ExprId, [2]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(WithinTypeofExpr, ["outputs::typeof_undef::WithinTypeofExpr"]<>, type_of, expr, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(WithinTypeofExpr, <>, type_of: crate::ast::ExprId, expr: crate::ast::ExprId, file: crate::ast::FileId);
impl ::std::fmt::Display for WithinTypeofExpr {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::typeof_undef::WithinTypeofExpr{type_of,expr,file} => {
                __formatter.write_str("outputs::typeof_undef::WithinTypeofExpr{")?;
                ::std::fmt::Debug::fmt(type_of, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for WithinTypeofExpr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}