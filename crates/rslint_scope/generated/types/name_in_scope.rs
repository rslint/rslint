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
    pub declared: crate::ast::AnyId
}
impl abomonation::Abomonation for NameInScope{}
::differential_datalog::decl_struct_from_record!(NameInScope["name_in_scope::NameInScope"]<>, ["name_in_scope::NameInScope"][4]{[0]file["file"]: crate::ast::FileId, [1]name["name"]: crate::ast::Name, [2]scope["scope"]: crate::ast::ScopeId, [3]declared["declared"]: crate::ast::AnyId});
::differential_datalog::decl_struct_into_record!(NameInScope, ["name_in_scope::NameInScope"]<>, file, name, scope, declared);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameInScope, <>, file: crate::ast::FileId, name: crate::ast::Name, scope: crate::ast::ScopeId, declared: crate::ast::AnyId);
impl ::std::fmt::Display for NameInScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::name_in_scope::NameInScope{file,name,scope,declared} => {
                __formatter.write_str("name_in_scope::NameInScope{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declared, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NameVisibleInScope {
    pub name: crate::ast::Name,
    pub scope: crate::ast::ScopeId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for NameVisibleInScope{}
::differential_datalog::decl_struct_from_record!(NameVisibleInScope["name_in_scope::NameVisibleInScope"]<>, ["name_in_scope::NameVisibleInScope"][3]{[0]name["name"]: crate::ast::Name, [1]scope["scope"]: crate::ast::ScopeId, [2]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(NameVisibleInScope, ["name_in_scope::NameVisibleInScope"]<>, name, scope, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameVisibleInScope, <>, name: crate::ast::Name, scope: crate::ast::ScopeId, file: crate::ast::FileId);
impl ::std::fmt::Display for NameVisibleInScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::name_in_scope::NameVisibleInScope{name,scope,file} => {
                __formatter.write_str("name_in_scope::NameVisibleInScope{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameVisibleInScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}