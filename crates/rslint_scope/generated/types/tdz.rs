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
pub struct ClosestLexicalScope {
    pub id: crate::ast::AnyId,
    pub file: crate::ast::FileId,
    pub lexical_scope: crate::ast::ScopeId
}
impl abomonation::Abomonation for ClosestLexicalScope{}
::differential_datalog::decl_struct_from_record!(ClosestLexicalScope["tdz::ClosestLexicalScope"]<>, ["tdz::ClosestLexicalScope"][3]{[0]id["id"]: crate::ast::AnyId, [1]file["file"]: crate::ast::FileId, [2]lexical_scope["lexical_scope"]: crate::ast::ScopeId});
::differential_datalog::decl_struct_into_record!(ClosestLexicalScope, ["tdz::ClosestLexicalScope"]<>, id, file, lexical_scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ClosestLexicalScope, <>, id: crate::ast::AnyId, file: crate::ast::FileId, lexical_scope: crate::ast::ScopeId);
impl ::std::fmt::Display for ClosestLexicalScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::tdz::ClosestLexicalScope{id,file,lexical_scope} => {
                __formatter.write_str("tdz::ClosestLexicalScope{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(lexical_scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ClosestLexicalScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct CurrentTdzStatus {
    pub id: crate::ast::AnyId,
    pub file: crate::ast::FileId,
    pub scope: crate::ast::ScopeId,
    pub status: crate::tdz::TdzStatus
}
impl abomonation::Abomonation for CurrentTdzStatus{}
::differential_datalog::decl_struct_from_record!(CurrentTdzStatus["tdz::CurrentTdzStatus"]<>, ["tdz::CurrentTdzStatus"][4]{[0]id["id"]: crate::ast::AnyId, [1]file["file"]: crate::ast::FileId, [2]scope["scope"]: crate::ast::ScopeId, [3]status["status"]: crate::tdz::TdzStatus});
::differential_datalog::decl_struct_into_record!(CurrentTdzStatus, ["tdz::CurrentTdzStatus"]<>, id, file, scope, status);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(CurrentTdzStatus, <>, id: crate::ast::AnyId, file: crate::ast::FileId, scope: crate::ast::ScopeId, status: crate::tdz::TdzStatus);
impl ::std::fmt::Display for CurrentTdzStatus {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::tdz::CurrentTdzStatus{id,file,scope,status} => {
                __formatter.write_str("tdz::CurrentTdzStatus{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(status, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for CurrentTdzStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct LexicalScope {
    pub scope: crate::ast::ScopeId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for LexicalScope{}
::differential_datalog::decl_struct_from_record!(LexicalScope["tdz::LexicalScope"]<>, ["tdz::LexicalScope"][2]{[0]scope["scope"]: crate::ast::ScopeId, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(LexicalScope, ["tdz::LexicalScope"]<>, scope, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(LexicalScope, <>, scope: crate::ast::ScopeId, file: crate::ast::FileId);
impl ::std::fmt::Display for LexicalScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::tdz::LexicalScope{scope,file} => {
                __formatter.write_str("tdz::LexicalScope{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for LexicalScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum TdzStatus {
    TdzInitalized,
    TdzUninitialized
}
impl abomonation::Abomonation for TdzStatus{}
::differential_datalog::decl_enum_from_record!(TdzStatus["tdz::TdzStatus"]<>, TdzInitalized["tdz::TdzInitalized"][0]{}, TdzUninitialized["tdz::TdzUninitialized"][0]{});
::differential_datalog::decl_enum_into_record!(TdzStatus<>, TdzInitalized["tdz::TdzInitalized"]{}, TdzUninitialized["tdz::TdzUninitialized"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(TdzStatus<>, TdzInitalized{}, TdzUninitialized{});
impl ::std::fmt::Display for TdzStatus {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::tdz::TdzStatus::TdzInitalized{} => {
                __formatter.write_str("tdz::TdzInitalized{")?;
                __formatter.write_str("}")
            },
            crate::tdz::TdzStatus::TdzUninitialized{} => {
                __formatter.write_str("tdz::TdzUninitialized{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for TdzStatus {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for TdzStatus {
    fn default() -> Self {
        crate::tdz::TdzStatus::TdzInitalized{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct TdzTarget {
    pub id: crate::ast::AnyId,
    pub file: crate::ast::FileId,
    pub declaration_scope: crate::ast::ScopeId
}
impl abomonation::Abomonation for TdzTarget{}
::differential_datalog::decl_struct_from_record!(TdzTarget["tdz::TdzTarget"]<>, ["tdz::TdzTarget"][3]{[0]id["id"]: crate::ast::AnyId, [1]file["file"]: crate::ast::FileId, [2]declaration_scope["declaration_scope"]: crate::ast::ScopeId});
::differential_datalog::decl_struct_into_record!(TdzTarget, ["tdz::TdzTarget"]<>, id, file, declaration_scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(TdzTarget, <>, id: crate::ast::AnyId, file: crate::ast::FileId, declaration_scope: crate::ast::ScopeId);
impl ::std::fmt::Display for TdzTarget {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::tdz::TdzTarget{id,file,declaration_scope} => {
                __formatter.write_str("tdz::TdzTarget{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(declaration_scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for TdzTarget {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}