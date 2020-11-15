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
pub struct ChainedWith {
    pub object: crate::ast::ExprId,
    pub property: crate::ast::ExprId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for ChainedWith{}
::differential_datalog::decl_struct_from_record!(ChainedWith["outputs::no_undef::ChainedWith"]<>, ["outputs::no_undef::ChainedWith"][3]{[0]object["object"]: crate::ast::ExprId, [1]property["property"]: crate::ast::ExprId, [2]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(ChainedWith, ["outputs::no_undef::ChainedWith"]<>, object, property, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ChainedWith, <>, object: crate::ast::ExprId, property: crate::ast::ExprId, file: crate::ast::FileId);
impl ::std::fmt::Display for ChainedWith {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::no_undef::ChainedWith{object,property,file} => {
                __formatter.write_str("outputs::no_undef::ChainedWith{")?;
                ::std::fmt::Debug::fmt(object, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(property, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ChainedWith {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NoUndef {
    pub name: crate::ast::Name,
    pub scope: crate::ast::ScopeId,
    pub span: crate::ast::Span,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for NoUndef{}
::differential_datalog::decl_struct_from_record!(NoUndef["outputs::no_undef::NoUndef"]<>, ["outputs::no_undef::NoUndef"][4]{[0]name["name"]: crate::ast::Name, [1]scope["scope"]: crate::ast::ScopeId, [2]span["span"]: crate::ast::Span, [3]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(NoUndef, ["outputs::no_undef::NoUndef"]<>, name, scope, span, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NoUndef, <>, name: crate::ast::Name, scope: crate::ast::ScopeId, span: crate::ast::Span, file: crate::ast::FileId);
impl ::std::fmt::Display for NoUndef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::outputs::no_undef::NoUndef{name,scope,span,file} => {
                __formatter.write_str("outputs::no_undef::NoUndef{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NoUndef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}