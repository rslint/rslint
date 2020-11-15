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
pub struct NameInScope {
    pub file: crate::ast::FileId,
    pub name: crate::ast::Name,
    pub scope: crate::ast::ScopeId,
    pub span: crate::ddlog_std::Option<crate::ast::Span>,
    pub declared_in: crate::ast::AnyId,
    pub implicit: bool,
    pub is_arg: bool,
    pub origin: crate::name_in_scope::NameOrigin
}
impl abomonation::Abomonation for NameInScope{}
::differential_datalog::decl_struct_from_record!(NameInScope["name_in_scope::NameInScope"]<>, ["name_in_scope::NameInScope"][8]{[0]file["file"]: crate::ast::FileId, [1]name["name"]: crate::ast::Name, [2]scope["scope"]: crate::ast::ScopeId, [3]span["span"]: crate::ddlog_std::Option<crate::ast::Span>, [4]declared_in["declared_in"]: crate::ast::AnyId, [5]implicit["implicit"]: bool, [6]is_arg["is_arg"]: bool, [7]origin["origin"]: crate::name_in_scope::NameOrigin});
::differential_datalog::decl_struct_into_record!(NameInScope, ["name_in_scope::NameInScope"]<>, file, name, scope, span, declared_in, implicit, is_arg, origin);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameInScope, <>, file: crate::ast::FileId, name: crate::ast::Name, scope: crate::ast::ScopeId, span: crate::ddlog_std::Option<crate::ast::Span>, declared_in: crate::ast::AnyId, implicit: bool, is_arg: bool, origin: crate::name_in_scope::NameOrigin);
impl ::std::fmt::Display for NameInScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::name_in_scope::NameInScope{file,name,scope,span,declared_in,implicit,is_arg,origin} => {
                __formatter.write_str("name_in_scope::NameInScope{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared_in, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(implicit, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(is_arg, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(origin, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameInScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum NameOrigin {
    AutoGlobal,
    Imported,
    UserDefined {
        scope: crate::ast::ScopeId
    }
}
impl abomonation::Abomonation for NameOrigin{}
::differential_datalog::decl_enum_from_record!(NameOrigin["name_in_scope::NameOrigin"]<>, AutoGlobal["name_in_scope::AutoGlobal"][0]{}, Imported["name_in_scope::Imported"][0]{}, UserDefined["name_in_scope::UserDefined"][1]{[0]scope["scope"]: crate::ast::ScopeId});
::differential_datalog::decl_enum_into_record!(NameOrigin<>, AutoGlobal["name_in_scope::AutoGlobal"]{}, Imported["name_in_scope::Imported"]{}, UserDefined["name_in_scope::UserDefined"]{scope});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(NameOrigin<>, AutoGlobal{}, Imported{}, UserDefined{scope: crate::ast::ScopeId});
impl ::std::fmt::Display for NameOrigin {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::name_in_scope::NameOrigin::AutoGlobal{} => {
                __formatter.write_str("name_in_scope::AutoGlobal{")?;
                __formatter.write_str("}")
            },
            crate::name_in_scope::NameOrigin::Imported{} => {
                __formatter.write_str("name_in_scope::Imported{")?;
                __formatter.write_str("}")
            },
            crate::name_in_scope::NameOrigin::UserDefined{scope} => {
                __formatter.write_str("name_in_scope::UserDefined{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameOrigin {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for NameOrigin {
    fn default() -> Self {
        crate::name_in_scope::NameOrigin::AutoGlobal{}
    }
}