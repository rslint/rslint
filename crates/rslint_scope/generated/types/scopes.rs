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
pub struct FunctionLevelScope {
    pub scope: crate::ast::ScopeId,
    pub nearest: crate::ast::ScopeId,
    pub file: crate::ast::FileId,
    pub id: crate::ast::AnyId
}
impl abomonation::Abomonation for FunctionLevelScope{}
::differential_datalog::decl_struct_from_record!(FunctionLevelScope["scopes::FunctionLevelScope"]<>, ["scopes::FunctionLevelScope"][4]{[0]scope["scope"]: crate::ast::ScopeId, [1]nearest["nearest"]: crate::ast::ScopeId, [2]file["file"]: crate::ast::FileId, [3]id["id"]: crate::ast::AnyId});
::differential_datalog::decl_struct_into_record!(FunctionLevelScope, ["scopes::FunctionLevelScope"]<>, scope, nearest, file, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FunctionLevelScope, <>, scope: crate::ast::ScopeId, nearest: crate::ast::ScopeId, file: crate::ast::FileId, id: crate::ast::AnyId);
impl ::std::fmt::Display for FunctionLevelScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::scopes::FunctionLevelScope{scope,nearest,file,id} => {
                __formatter.write_str("scopes::FunctionLevelScope{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(nearest, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FunctionLevelScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct IsHoistable {
    pub id: crate::ast::AnyId,
    pub file: crate::ast::FileId,
    pub hoistable: bool
}
impl abomonation::Abomonation for IsHoistable{}
::differential_datalog::decl_struct_from_record!(IsHoistable["scopes::IsHoistable"]<>, ["scopes::IsHoistable"][3]{[0]id["id"]: crate::ast::AnyId, [1]file["file"]: crate::ast::FileId, [2]hoistable["hoistable"]: bool});
::differential_datalog::decl_struct_into_record!(IsHoistable, ["scopes::IsHoistable"]<>, id, file, hoistable);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(IsHoistable, <>, id: crate::ast::AnyId, file: crate::ast::FileId, hoistable: bool);
impl ::std::fmt::Display for IsHoistable {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::scopes::IsHoistable{id,file,hoistable} => {
                __formatter.write_str("scopes::IsHoistable{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(hoistable, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for IsHoistable {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NeedsScopeChildren {
    pub scope: crate::ast::ScopeId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for NeedsScopeChildren{}
::differential_datalog::decl_struct_from_record!(NeedsScopeChildren["scopes::NeedsScopeChildren"]<>, ["scopes::NeedsScopeChildren"][2]{[0]scope["scope"]: crate::ast::ScopeId, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(NeedsScopeChildren, ["scopes::NeedsScopeChildren"]<>, scope, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NeedsScopeChildren, <>, scope: crate::ast::ScopeId, file: crate::ast::FileId);
impl ::std::fmt::Display for NeedsScopeChildren {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::scopes::NeedsScopeChildren{scope,file} => {
                __formatter.write_str("scopes::NeedsScopeChildren{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NeedsScopeChildren {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NeedsScopeParents {
    pub scope: crate::ast::ScopeId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for NeedsScopeParents{}
::differential_datalog::decl_struct_from_record!(NeedsScopeParents["scopes::NeedsScopeParents"]<>, ["scopes::NeedsScopeParents"][2]{[0]scope["scope"]: crate::ast::ScopeId, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(NeedsScopeParents, ["scopes::NeedsScopeParents"]<>, scope, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NeedsScopeParents, <>, scope: crate::ast::ScopeId, file: crate::ast::FileId);
impl ::std::fmt::Display for NeedsScopeParents {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::scopes::NeedsScopeParents{scope,file} => {
                __formatter.write_str("scopes::NeedsScopeParents{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NeedsScopeParents {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ScopeFamily {
    pub parent: crate::ast::ScopeId,
    pub child: crate::ast::ScopeId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for ScopeFamily{}
::differential_datalog::decl_struct_from_record!(ScopeFamily["scopes::ScopeFamily"]<>, ["scopes::ScopeFamily"][3]{[0]parent["parent"]: crate::ast::ScopeId, [1]child["child"]: crate::ast::ScopeId, [2]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(ScopeFamily, ["scopes::ScopeFamily"]<>, parent, child, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ScopeFamily, <>, parent: crate::ast::ScopeId, child: crate::ast::ScopeId, file: crate::ast::FileId);
impl ::std::fmt::Display for ScopeFamily {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::scopes::ScopeFamily{parent,child,file} => {
                __formatter.write_str("scopes::ScopeFamily{")?;
                ::std::fmt::Debug::fmt(parent, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(child, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ScopeFamily {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ScopeOfId {
    pub id: crate::ast::AnyId,
    pub file: crate::ast::FileId,
    pub scope: crate::ast::ScopeId
}
impl abomonation::Abomonation for ScopeOfId{}
::differential_datalog::decl_struct_from_record!(ScopeOfId["scopes::ScopeOfId"]<>, ["scopes::ScopeOfId"][3]{[0]id["id"]: crate::ast::AnyId, [1]file["file"]: crate::ast::FileId, [2]scope["scope"]: crate::ast::ScopeId});
::differential_datalog::decl_struct_into_record!(ScopeOfId, ["scopes::ScopeOfId"]<>, id, file, scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ScopeOfId, <>, id: crate::ast::AnyId, file: crate::ast::FileId, scope: crate::ast::ScopeId);
impl ::std::fmt::Display for ScopeOfId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::scopes::ScopeOfId{id,file,scope} => {
                __formatter.write_str("scopes::ScopeOfId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ScopeOfId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}