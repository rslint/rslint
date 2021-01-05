#![allow(
    path_statements,
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
    clippy::match_single_binding,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::needless_lifetimes,
    clippy::borrowed_box,
    clippy::map_clone,
    clippy::toplevel_ref_arg,
    clippy::double_parens,
    clippy::collapsible_if,
    clippy::clone_on_copy,
    clippy::unused_unit,
    clippy::deref_addrof,
    clippy::clone_on_copy,
    clippy::needless_return,
    clippy::op_ref,
    clippy::match_like_matches_macro,
    clippy::comparison_chain,
    clippy::len_zero,
    clippy::extra_unused_lifetimes
)]

use ::num::One;
use ::std::ops::Deref;

use ::differential_dataflow::collection;
use ::timely::communication;
use ::timely::dataflow::scopes;
use ::timely::worker;

use ::ddlog_derive::{FromRecord, IntoRecord, Mutator};
use ::differential_datalog::ddval::DDValue;
use ::differential_datalog::ddval::DDValConvert;
use ::differential_datalog::program;
use ::differential_datalog::program::TupleTS;
use ::differential_datalog::program::XFormArrangement;
use ::differential_datalog::program::XFormCollection;
use ::differential_datalog::program::Weight;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;


// `usize` and `isize` are builtin Rust types; we therefore declare an alias to DDlog's `usize` and
// `isize`.
pub type std_usize = u64;
pub type std_isize = i64;


#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Array")]
pub struct Array {
    pub expr_id: types__ast::ExprId,
    pub elements: ddlog_std::Vec<types__ast::ArrayElement>
}
impl abomonation::Abomonation for Array{}
impl ::std::fmt::Display for Array {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Array{expr_id,elements} => {
                __formatter.write_str("inputs::Array{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(elements, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Array {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Arrow")]
pub struct Arrow {
    pub expr_id: types__ast::ExprId,
    pub body: ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Either<types__ast::ExprId, types__ast::StmtId>, types__ast::ScopeId>>
}
impl abomonation::Abomonation for Arrow{}
impl ::std::fmt::Display for Arrow {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Arrow{expr_id,body} => {
                __formatter.write_str("inputs::Arrow{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Arrow {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ArrowParam")]
pub struct ArrowParam {
    pub expr_id: types__ast::ExprId,
    pub param: types__ast::IPattern
}
impl abomonation::Abomonation for ArrowParam{}
impl ::std::fmt::Display for ArrowParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ArrowParam{expr_id,param} => {
                __formatter.write_str("inputs::ArrowParam{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(param, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ArrowParam {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Assign")]
pub struct Assign {
    pub expr_id: types__ast::ExprId,
    pub lhs: ddlog_std::Option<ddlog_std::Either<types__ast::IPattern, types__ast::ExprId>>,
    pub rhs: ddlog_std::Option<types__ast::ExprId>,
    pub op: ddlog_std::Option<types__ast::AssignOperand>
}
impl abomonation::Abomonation for Assign{}
impl ::std::fmt::Display for Assign {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Assign{expr_id,lhs,rhs,op} => {
                __formatter.write_str("inputs::Assign{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(lhs, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(rhs, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(op, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Assign {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Await")]
pub struct Await {
    pub expr_id: types__ast::ExprId,
    pub value: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Await{}
impl ::std::fmt::Display for Await {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Await{expr_id,value} => {
                __formatter.write_str("inputs::Await{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Await {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::BinOp")]
pub struct BinOp {
    pub expr_id: types__ast::ExprId,
    pub op: ddlog_std::Option<types__ast::BinOperand>,
    pub lhs: ddlog_std::Option<types__ast::ExprId>,
    pub rhs: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for BinOp{}
impl ::std::fmt::Display for BinOp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            BinOp{expr_id,op,lhs,rhs} => {
                __formatter.write_str("inputs::BinOp{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(op, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(lhs, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(rhs, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for BinOp {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::BracketAccess")]
pub struct BracketAccess {
    pub expr_id: types__ast::ExprId,
    pub object: ddlog_std::Option<types__ast::ExprId>,
    pub prop: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for BracketAccess{}
impl ::std::fmt::Display for BracketAccess {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            BracketAccess{expr_id,object,prop} => {
                __formatter.write_str("inputs::BracketAccess{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(object, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(prop, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for BracketAccess {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Break")]
pub struct Break {
    pub stmt_id: types__ast::StmtId,
    pub label: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>
}
impl abomonation::Abomonation for Break{}
impl ::std::fmt::Display for Break {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Break{stmt_id,label} => {
                __formatter.write_str("inputs::Break{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Break {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Call")]
pub struct Call {
    pub expr_id: types__ast::ExprId,
    pub callee: ddlog_std::Option<types__ast::ExprId>,
    pub args: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>
}
impl abomonation::Abomonation for Call{}
impl ::std::fmt::Display for Call {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Call{expr_id,callee,args} => {
                __formatter.write_str("inputs::Call{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(callee, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(args, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Call {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Class")]
pub struct Class {
    pub id: types__ast::ClassId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub parent: ddlog_std::Option<types__ast::ExprId>,
    pub elements: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>,
    pub scope: types__ast::ScopeId,
    pub exported: bool
}
impl abomonation::Abomonation for Class{}
impl ::std::fmt::Display for Class {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Class{id,name,parent,elements,scope,exported} => {
                __formatter.write_str("inputs::Class{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(parent, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(elements, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(exported, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Class {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ClassExpr")]
pub struct ClassExpr {
    pub expr_id: types__ast::ExprId,
    pub elements: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>
}
impl abomonation::Abomonation for ClassExpr{}
impl ::std::fmt::Display for ClassExpr {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ClassExpr{expr_id,elements} => {
                __formatter.write_str("inputs::ClassExpr{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(elements, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ClassExpr {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ConstDecl")]
pub struct ConstDecl {
    pub stmt_id: types__ast::StmtId,
    pub pattern: ddlog_std::Option<types__ast::IPattern>,
    pub value: ddlog_std::Option<types__ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for ConstDecl{}
impl ::std::fmt::Display for ConstDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ConstDecl{stmt_id,pattern,value,exported} => {
                __formatter.write_str("inputs::ConstDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(pattern, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(exported, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ConstDecl {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Continue")]
pub struct Continue {
    pub stmt_id: types__ast::StmtId,
    pub label: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>
}
impl abomonation::Abomonation for Continue{}
impl ::std::fmt::Display for Continue {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Continue{stmt_id,label} => {
                __formatter.write_str("inputs::Continue{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(label, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Continue {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::DoWhile")]
pub struct DoWhile {
    pub stmt_id: types__ast::StmtId,
    pub body: ddlog_std::Option<types__ast::StmtId>,
    pub cond: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for DoWhile{}
impl ::std::fmt::Display for DoWhile {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            DoWhile{stmt_id,body,cond} => {
                __formatter.write_str("inputs::DoWhile{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(cond, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for DoWhile {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::DotAccess")]
pub struct DotAccess {
    pub expr_id: types__ast::ExprId,
    pub object: ddlog_std::Option<types__ast::ExprId>,
    pub prop: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>
}
impl abomonation::Abomonation for DotAccess{}
impl ::std::fmt::Display for DotAccess {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            DotAccess{expr_id,object,prop} => {
                __formatter.write_str("inputs::DotAccess{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(object, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(prop, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for DotAccess {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::EveryScope")]
pub struct EveryScope {
    pub scope: types__ast::ScopeId
}
impl abomonation::Abomonation for EveryScope{}
impl ::std::fmt::Display for EveryScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EveryScope{scope} => {
                __formatter.write_str("inputs::EveryScope{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for EveryScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ExprBigInt")]
pub struct ExprBigInt {
    pub expr_id: types__ast::ExprId,
    pub value: ::ddlog_bigint::Int
}
impl abomonation::Abomonation for ExprBigInt{}
impl ::std::fmt::Display for ExprBigInt {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprBigInt{expr_id,value} => {
                __formatter.write_str("inputs::ExprBigInt{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ExprBigInt {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ExprBool")]
pub struct ExprBool {
    pub expr_id: types__ast::ExprId,
    pub value: bool
}
impl abomonation::Abomonation for ExprBool{}
impl ::std::fmt::Display for ExprBool {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprBool{expr_id,value} => {
                __formatter.write_str("inputs::ExprBool{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ExprBool {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ExprNumber")]
pub struct ExprNumber {
    pub expr_id: types__ast::ExprId,
    pub value: ::ordered_float::OrderedFloat<f64>
}
impl abomonation::Abomonation for ExprNumber{}
impl ::std::fmt::Display for ExprNumber {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprNumber{expr_id,value} => {
                __formatter.write_str("inputs::ExprNumber{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ExprNumber {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ExprString")]
pub struct ExprString {
    pub expr_id: types__ast::ExprId,
    pub value: internment::istring
}
impl abomonation::Abomonation for ExprString{}
impl ::std::fmt::Display for ExprString {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprString{expr_id,value} => {
                __formatter.write_str("inputs::ExprString{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ExprString {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Expression")]
pub struct Expression {
    pub id: types__ast::ExprId,
    pub kind: types__ast::ExprKind,
    pub scope: types__ast::ScopeId,
    pub span: types__ast::Span
}
impl abomonation::Abomonation for Expression{}
impl ::std::fmt::Display for Expression {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Expression{id,kind,scope,span} => {
                __formatter.write_str("inputs::Expression{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Expression {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::File")]
pub struct File {
    pub id: types__ast::FileId,
    pub kind: types__ast::FileKind,
    pub top_level_scope: types__ast::ScopeId
}
impl abomonation::Abomonation for File{}
impl ::std::fmt::Display for File {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            File{id,kind,top_level_scope} => {
                __formatter.write_str("inputs::File{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(top_level_scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for File {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::FileExport")]
pub struct FileExport {
    pub export: types__ast::ExportKind,
    pub scope: types__ast::ScopeId
}
impl abomonation::Abomonation for FileExport{}
impl ::std::fmt::Display for FileExport {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FileExport{export,scope} => {
                __formatter.write_str("inputs::FileExport{")?;
                ::std::fmt::Debug::fmt(export, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FileExport {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::For")]
pub struct For {
    pub stmt_id: types__ast::StmtId,
    pub init: ddlog_std::Option<types__ast::ForInit>,
    pub test: ddlog_std::Option<types__ast::ExprId>,
    pub update: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for For{}
impl ::std::fmt::Display for For {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            For{stmt_id,init,test,update,body} => {
                __formatter.write_str("inputs::For{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(init, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(test, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(update, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for For {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ForIn")]
pub struct ForIn {
    pub stmt_id: types__ast::StmtId,
    pub elem: ddlog_std::Option<types__ast::ForInit>,
    pub collection: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for ForIn{}
impl ::std::fmt::Display for ForIn {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ForIn{stmt_id,elem,collection,body} => {
                __formatter.write_str("inputs::ForIn{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(elem, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(collection, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ForIn {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ForOf")]
pub struct ForOf {
    pub stmt_id: types__ast::StmtId,
    pub awaited: bool,
    pub elem: ddlog_std::Option<types__ast::ForInit>,
    pub collection: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for ForOf{}
impl ::std::fmt::Display for ForOf {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ForOf{stmt_id,awaited,elem,collection,body} => {
                __formatter.write_str("inputs::ForOf{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(awaited, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(elem, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(collection, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ForOf {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Function")]
pub struct Function {
    pub id: types__ast::FuncId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub scope: types__ast::ScopeId,
    pub body: types__ast::ScopeId,
    pub exported: bool
}
impl abomonation::Abomonation for Function{}
impl ::std::fmt::Display for Function {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Function{id,name,scope,body,exported} => {
                __formatter.write_str("inputs::Function{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(exported, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Function {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::FunctionArg")]
pub struct FunctionArg {
    pub parent_func: types__ast::FuncId,
    pub pattern: types__ast::IPattern,
    pub implicit: bool
}
impl abomonation::Abomonation for FunctionArg{}
impl ::std::fmt::Display for FunctionArg {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FunctionArg{parent_func,pattern,implicit} => {
                __formatter.write_str("inputs::FunctionArg{")?;
                ::std::fmt::Debug::fmt(parent_func, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(pattern, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(implicit, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FunctionArg {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::If")]
pub struct If {
    pub stmt_id: types__ast::StmtId,
    pub cond: ddlog_std::Option<types__ast::ExprId>,
    pub if_body: ddlog_std::Option<types__ast::StmtId>,
    pub else_body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for If{}
impl ::std::fmt::Display for If {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            If{stmt_id,cond,if_body,else_body} => {
                __formatter.write_str("inputs::If{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(cond, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(if_body, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(else_body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for If {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ImplicitGlobal")]
pub struct ImplicitGlobal {
    pub id: types__ast::GlobalId,
    pub name: types__ast::Name,
    pub privileges: types__ast::GlobalPriv
}
impl abomonation::Abomonation for ImplicitGlobal{}
impl ::std::fmt::Display for ImplicitGlobal {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImplicitGlobal{id,name,privileges} => {
                __formatter.write_str("inputs::ImplicitGlobal{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(privileges, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ImplicitGlobal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::ImportDecl")]
pub struct ImportDecl {
    pub id: types__ast::ImportId,
    pub clause: types__ast::ImportClause
}
impl abomonation::Abomonation for ImportDecl{}
impl ::std::fmt::Display for ImportDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImportDecl{id,clause} => {
                __formatter.write_str("inputs::ImportDecl{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(clause, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ImportDecl {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::InlineFunc")]
pub struct InlineFunc {
    pub expr_id: types__ast::ExprId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for InlineFunc{}
impl ::std::fmt::Display for InlineFunc {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            InlineFunc{expr_id,name,body} => {
                __formatter.write_str("inputs::InlineFunc{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for InlineFunc {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::InlineFuncParam")]
pub struct InlineFuncParam {
    pub expr_id: types__ast::ExprId,
    pub param: types__ast::IPattern
}
impl abomonation::Abomonation for InlineFuncParam{}
impl ::std::fmt::Display for InlineFuncParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            InlineFuncParam{expr_id,param} => {
                __formatter.write_str("inputs::InlineFuncParam{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(param, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for InlineFuncParam {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::InputScope")]
pub struct InputScope {
    pub parent: types__ast::ScopeId,
    pub child: types__ast::ScopeId
}
impl abomonation::Abomonation for InputScope{}
impl ::std::fmt::Display for InputScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            InputScope{parent,child} => {
                __formatter.write_str("inputs::InputScope{")?;
                ::std::fmt::Debug::fmt(parent, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(child, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for InputScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Label")]
pub struct Label {
    pub stmt_id: types__ast::StmtId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub body: ddlog_std::Option<types__ast::StmtId>,
    pub body_scope: types__ast::ScopeId
}
impl abomonation::Abomonation for Label{}
impl ::std::fmt::Display for Label {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Label{stmt_id,name,body,body_scope} => {
                __formatter.write_str("inputs::Label{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body_scope, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Label {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::LetDecl")]
pub struct LetDecl {
    pub stmt_id: types__ast::StmtId,
    pub pattern: ddlog_std::Option<types__ast::IPattern>,
    pub value: ddlog_std::Option<types__ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for LetDecl{}
impl ::std::fmt::Display for LetDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            LetDecl{stmt_id,pattern,value,exported} => {
                __formatter.write_str("inputs::LetDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(pattern, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(exported, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for LetDecl {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::NameRef")]
pub struct NameRef {
    pub expr_id: types__ast::ExprId,
    pub value: types__ast::Name
}
impl abomonation::Abomonation for NameRef{}
impl ::std::fmt::Display for NameRef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameRef{expr_id,value} => {
                __formatter.write_str("inputs::NameRef{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NameRef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::New")]
pub struct New {
    pub expr_id: types__ast::ExprId,
    pub object: ddlog_std::Option<types__ast::ExprId>,
    pub args: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>
}
impl abomonation::Abomonation for New{}
impl ::std::fmt::Display for New {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            New{expr_id,object,args} => {
                __formatter.write_str("inputs::New{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(object, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(args, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for New {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Property")]
pub struct Property {
    pub expr_id: types__ast::ExprId,
    pub key: ddlog_std::Option<types__ast::PropertyKey>,
    pub val: ddlog_std::Option<types__ast::PropertyVal>
}
impl abomonation::Abomonation for Property{}
impl ::std::fmt::Display for Property {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Property{expr_id,key,val} => {
                __formatter.write_str("inputs::Property{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(val, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Property {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Return")]
pub struct Return {
    pub stmt_id: types__ast::StmtId,
    pub value: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Return{}
impl ::std::fmt::Display for Return {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Return{stmt_id,value} => {
                __formatter.write_str("inputs::Return{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Return {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Statement")]
pub struct Statement {
    pub id: types__ast::StmtId,
    pub kind: types__ast::StmtKind,
    pub scope: types__ast::ScopeId,
    pub span: types__ast::Span
}
impl abomonation::Abomonation for Statement{}
impl ::std::fmt::Display for Statement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Statement{id,kind,scope,span} => {
                __formatter.write_str("inputs::Statement{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Statement {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Switch")]
pub struct Switch {
    pub stmt_id: types__ast::StmtId,
    pub test: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Switch{}
impl ::std::fmt::Display for Switch {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Switch{stmt_id,test} => {
                __formatter.write_str("inputs::Switch{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(test, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Switch {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::SwitchCase")]
pub struct SwitchCase {
    pub stmt_id: types__ast::StmtId,
    pub case: types__ast::SwitchClause,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for SwitchCase{}
impl ::std::fmt::Display for SwitchCase {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            SwitchCase{stmt_id,case,body} => {
                __formatter.write_str("inputs::SwitchCase{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(case, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for SwitchCase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Template")]
pub struct Template {
    pub expr_id: types__ast::ExprId,
    pub tag: ddlog_std::Option<types__ast::ExprId>,
    pub elements: ddlog_std::Vec<types__ast::ExprId>
}
impl abomonation::Abomonation for Template{}
impl ::std::fmt::Display for Template {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Template{expr_id,tag,elements} => {
                __formatter.write_str("inputs::Template{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(tag, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(elements, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Template {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Ternary")]
pub struct Ternary {
    pub expr_id: types__ast::ExprId,
    pub test: ddlog_std::Option<types__ast::ExprId>,
    pub true_val: ddlog_std::Option<types__ast::ExprId>,
    pub false_val: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Ternary{}
impl ::std::fmt::Display for Ternary {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Ternary{expr_id,test,true_val,false_val} => {
                __formatter.write_str("inputs::Ternary{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(test, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(true_val, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(false_val, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Ternary {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Throw")]
pub struct Throw {
    pub stmt_id: types__ast::StmtId,
    pub exception: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Throw{}
impl ::std::fmt::Display for Throw {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Throw{stmt_id,exception} => {
                __formatter.write_str("inputs::Throw{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(exception, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Throw {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Try")]
pub struct Try {
    pub stmt_id: types__ast::StmtId,
    pub body: ddlog_std::Option<types__ast::StmtId>,
    pub handler: types__ast::TryHandler,
    pub finalizer: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for Try{}
impl ::std::fmt::Display for Try {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Try{stmt_id,body,handler,finalizer} => {
                __formatter.write_str("inputs::Try{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(handler, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(finalizer, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Try {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::UnaryOp")]
pub struct UnaryOp {
    pub expr_id: types__ast::ExprId,
    pub op: ddlog_std::Option<types__ast::UnaryOperand>,
    pub expr: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for UnaryOp{}
impl ::std::fmt::Display for UnaryOp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            UnaryOp{expr_id,op,expr} => {
                __formatter.write_str("inputs::UnaryOp{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(op, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for UnaryOp {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::UserGlobal")]
pub struct UserGlobal {
    pub id: types__ast::GlobalId,
    pub name: types__ast::Name,
    pub privileges: types__ast::GlobalPriv
}
impl abomonation::Abomonation for UserGlobal{}
impl ::std::fmt::Display for UserGlobal {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            UserGlobal{id,name,privileges} => {
                __formatter.write_str("inputs::UserGlobal{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(privileges, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for UserGlobal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::VarDecl")]
pub struct VarDecl {
    pub stmt_id: types__ast::StmtId,
    pub pattern: ddlog_std::Option<types__ast::IPattern>,
    pub value: ddlog_std::Option<types__ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for VarDecl{}
impl ::std::fmt::Display for VarDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            VarDecl{stmt_id,pattern,value,exported} => {
                __formatter.write_str("inputs::VarDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(pattern, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(exported, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for VarDecl {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::While")]
pub struct While {
    pub stmt_id: types__ast::StmtId,
    pub cond: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for While{}
impl ::std::fmt::Display for While {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            While{stmt_id,cond,body} => {
                __formatter.write_str("inputs::While{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(cond, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for While {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::With")]
pub struct With {
    pub stmt_id: types__ast::StmtId,
    pub cond: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for With{}
impl ::std::fmt::Display for With {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            With{stmt_id,cond,body} => {
                __formatter.write_str("inputs::With{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(cond, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for With {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "inputs::Yield")]
pub struct Yield {
    pub expr_id: types__ast::ExprId,
    pub value: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Yield{}
impl ::std::fmt::Display for Yield {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Yield{expr_id,value} => {
                __formatter.write_str("inputs::Yield{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Yield {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub fn __Key_inputs_Class(__key: &DDValue) -> DDValue {
    let ref c = *{<Class>::from_ddvalue_ref(__key) };
    (c.id.clone()).into_ddvalue()
}
pub fn __Key_inputs_Expression(__key: &DDValue) -> DDValue {
    let ref e = *{<Expression>::from_ddvalue_ref(__key) };
    (e.id.clone()).into_ddvalue()
}
pub fn __Key_inputs_File(__key: &DDValue) -> DDValue {
    let ref f = *{<File>::from_ddvalue_ref(__key) };
    (f.id.clone()).into_ddvalue()
}
pub fn __Key_inputs_Function(__key: &DDValue) -> DDValue {
    let ref f = *{<Function>::from_ddvalue_ref(__key) };
    (f.id.clone()).into_ddvalue()
}
pub fn __Key_inputs_Statement(__key: &DDValue) -> DDValue {
    let ref stmt = *{<Statement>::from_ddvalue_ref(__key) };
    (stmt.id.clone()).into_ddvalue()
}
pub static __Arng_inputs_Arrow_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Arrow{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .body=(ddlog_std::Some{.x=((_: ddlog_std::Either<ast::ExprId,ast::StmtId>), (_: ast::ScopeId))}: ddlog_std::Option<(ddlog_std::Either<ast::ExprId,ast::StmtId>, ast::ScopeId)>)}: inputs::Arrow) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Arrow>::from_ddvalue(__v) {
                                                                                                                               Arrow{expr_id: types__ast::ExprId{id: _, file: ref _0}, body: ddlog_std::Option::Some{x: ddlog_std::tuple2(_, _)}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_Arrow_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Arrow{.expr_id=(_0: ast::ExprId), .body=(ddlog_std::Some{.x=((_: ddlog_std::Either<ast::ExprId,ast::StmtId>), (_: ast::ScopeId))}: ddlog_std::Option<(ddlog_std::Either<ast::ExprId,ast::StmtId>, ast::ScopeId)>)}: inputs::Arrow) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Arrow>::from_ddvalue(__v) {
                                                                                                                               Arrow{expr_id: ref _0, body: ddlog_std::Option::Some{x: ddlog_std::tuple2(_, _)}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_ArrowParam_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::ArrowParam{.expr_id=(_0: ast::ExprId), .param=(_: internment::Intern<ast::Pattern>)}: inputs::ArrowParam) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < ArrowParam>::from_ddvalue(__v) {
                                                                                                                                    ArrowParam{expr_id: ref _0, param: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Assign_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                       name: std::borrow::Cow::from(r###"(inputs::Assign{.expr_id=(_0: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign) /*join*/"###),
                                                                                                                        afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                        {
                                                                                                                            let __cloned = __v.clone();
                                                                                                                            match < Assign>::from_ddvalue(__v) {
                                                                                                                                Assign{expr_id: ref _0, lhs: ddlog_std::Option::Some{x: ddlog_std::Either::Left{l: _}}, rhs: _, op: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                _ => None
                                                                                                                            }.map(|x|(x,__cloned))
                                                                                                                        }
                                                                                                                        __f},
                                                                                                                        queryable: false
                                                                                                                    });
pub static __Arng_inputs_Assign_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                       name: std::borrow::Cow::from(r###"(inputs::Assign{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign) /*join*/"###),
                                                                                                                        afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                        {
                                                                                                                            let __cloned = __v.clone();
                                                                                                                            match < Assign>::from_ddvalue(__v) {
                                                                                                                                Assign{expr_id: types__ast::ExprId{id: _, file: ref _0}, lhs: ddlog_std::Option::Some{x: ddlog_std::Either::Left{l: _}}, rhs: _, op: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                _ => None
                                                                                                                            }.map(|x|(x,__cloned))
                                                                                                                        }
                                                                                                                        __f},
                                                                                                                        queryable: false
                                                                                                                    });
pub static __Arng_inputs_BracketAccess_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                              name: std::borrow::Cow::from(r###"(inputs::BracketAccess{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .object=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess) /*join*/"###),
                                                                                                                               afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                               {
                                                                                                                                   let __cloned = __v.clone();
                                                                                                                                   match < BracketAccess>::from_ddvalue(__v) {
                                                                                                                                       BracketAccess{expr_id: types__ast::ExprId{id: _, file: ref _0}, object: ddlog_std::Option::Some{x: _}, prop: ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                       _ => None
                                                                                                                                   }.map(|x|(x,__cloned))
                                                                                                                               }
                                                                                                                               __f},
                                                                                                                               queryable: false
                                                                                                                           });
pub static __Arng_inputs_Break_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Break{.stmt_id=(ast::StmtId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::StmtId), .label=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::Break) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Break>::from_ddvalue(__v) {
                                                                                                                               Break{stmt_id: types__ast::StmtId{id: _, file: ref _0}, label: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_Call_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::Call{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .callee=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call) /*join*/"###),
                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                      {
                                                                                                                          let __cloned = __v.clone();
                                                                                                                          match < Call>::from_ddvalue(__v) {
                                                                                                                              Call{expr_id: types__ast::ExprId{id: _, file: ref _0}, callee: ddlog_std::Option::Some{x: _}, args: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                              _ => None
                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                      }
                                                                                                                      __f},
                                                                                                                      queryable: false
                                                                                                                  });
pub static __Arng_inputs_Class_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Class{.id=(_0: ast::ClassId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Class) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Class>::from_ddvalue(__v) {
                                                                                                                               Class{id: ref _0, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, parent: _, elements: _, scope: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_Class_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Class{.id=(_: ast::ClassId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_0: ast::ScopeId), .exported=(_: bool)}: inputs::Class) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Class>::from_ddvalue(__v) {
                                                                                                                               Class{id: _, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, parent: _, elements: _, scope: ref _0, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_ClassExpr_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::ClassExpr{.expr_id=(_0: ast::ExprId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr) /*semijoin*/"###),
                                                                                                                           fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                           {
                                                                                                                               match < ClassExpr>::from_ddvalue(__v) {
                                                                                                                                   ClassExpr{expr_id: ref _0, elements: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           distinct: false
                                                                                                                       });
pub static __Arng_inputs_ConstDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::ConstDecl{.stmt_id=(_0: ast::StmtId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < ConstDecl>::from_ddvalue(__v) {
                                                                                                                                   ConstDecl{stmt_id: ref _0, pattern: ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_Continue_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Continue{.stmt_id=(ast::StmtId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::StmtId), .label=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::Continue) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Continue>::from_ddvalue(__v) {
                                                                                                                                  Continue{stmt_id: types__ast::StmtId{id: _, file: ref _0}, label: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_DotAccess_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::DotAccess{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .object=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < DotAccess>::from_ddvalue(__v) {
                                                                                                                                   DotAccess{expr_id: types__ast::ExprId{id: _, file: ref _0}, object: ddlog_std::Option::Some{x: _}, prop: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_Expression_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, kind: types__ast::ExprKind::ExprGrouping{inner: ddlog_std::Option::Some{x: _}}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(ast::ExprSequence{.exprs=(_: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, kind: types__ast::ExprKind::ExprSequence{exprs: _}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, kind: types__ast::ExprKind::ExprNameRef{}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_4 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=_0, .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_File_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::File{.id=(_0: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId)}: inputs::File) /*join*/"###),
                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                      {
                                                                                                                          let __cloned = __v.clone();
                                                                                                                          match < File>::from_ddvalue(__v) {
                                                                                                                              File{id: ref _0, kind: _, top_level_scope: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                              _ => None
                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                      }
                                                                                                                      __f},
                                                                                                                      queryable: false
                                                                                                                  });
pub static __Arng_inputs_Function_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Function{.id=(_0: ast::FuncId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Function>::from_ddvalue(__v) {
                                                                                                                                  Function{id: ref _0, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, scope: _, body: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_Function_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Function{.id=(ast::FuncId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Function>::from_ddvalue(__v) {
                                                                                                                                  Function{id: types__ast::FuncId{id: _, file: ref _0}, name: _, scope: _, body: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_Function_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Function{.id=(_: ast::FuncId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_0: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Function>::from_ddvalue(__v) {
                                                                                                                                  Function{id: _, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, scope: ref _0, body: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_Function_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Function{.id=(_0: ast::FuncId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Function>::from_ddvalue(__v) {
                                                                                                                                  Function{id: ref _0, name: _, scope: _, body: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_FunctionArg_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                            name: std::borrow::Cow::from(r###"(inputs::FunctionArg{.parent_func=(_0: ast::FuncId), .pattern=(_: internment::Intern<ast::Pattern>), .implicit=(_: bool)}: inputs::FunctionArg) /*join*/"###),
                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                             {
                                                                                                                                 let __cloned = __v.clone();
                                                                                                                                 match < FunctionArg>::from_ddvalue(__v) {
                                                                                                                                     FunctionArg{parent_func: ref _0, pattern: _, implicit: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                     _ => None
                                                                                                                                 }.map(|x|(x,__cloned))
                                                                                                                             }
                                                                                                                             __f},
                                                                                                                             queryable: false
                                                                                                                         });
pub static __Arng_inputs_ImplicitGlobal_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                                name: std::borrow::Cow::from(r###"(inputs::ImplicitGlobal{.id=(ast::GlobalId{.id=(_: bit<32>), .file=(ddlog_std::None{}: ddlog_std::Option<ast::FileId>)}: ast::GlobalId), .name=(_0: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::ImplicitGlobal) /*antijoin*/"###),
                                                                                                                                fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                                {
                                                                                                                                    match < ImplicitGlobal>::from_ddvalue(__v) {
                                                                                                                                        ImplicitGlobal{id: types__ast::GlobalId{id: _, file: ddlog_std::Option::None{}}, name: ref _0, privileges: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                        _ => None
                                                                                                                                    }
                                                                                                                                }
                                                                                                                                __f},
                                                                                                                                distinct: true
                                                                                                                            });
pub static __Arng_inputs_ImportDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::ImportDecl{.id=(ast::ImportId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ImportId), .clause=(_: ast::ImportClause)}: inputs::ImportDecl) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < ImportDecl>::from_ddvalue(__v) {
                                                                                                                                    ImportDecl{id: types__ast::ImportId{id: _, file: ref _0}, clause: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFunc_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InlineFunc{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InlineFunc>::from_ddvalue(__v) {
                                                                                                                                    InlineFunc{expr_id: types__ast::ExprId{id: _, file: ref _0}, name: _, body: ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFunc_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InlineFunc{.expr_id=(_: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_0: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InlineFunc>::from_ddvalue(__v) {
                                                                                                                                    InlineFunc{expr_id: _, name: _, body: ddlog_std::Option::Some{x: ref _0}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFunc_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InlineFunc{.expr_id=(_: ast::ExprId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_0: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InlineFunc>::from_ddvalue(__v) {
                                                                                                                                    InlineFunc{expr_id: _, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, body: ddlog_std::Option::Some{x: ref _0}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFunc_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InlineFunc{.expr_id=(_0: ast::ExprId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InlineFunc>::from_ddvalue(__v) {
                                                                                                                                    InlineFunc{expr_id: ref _0, name: _, body: ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFuncParam_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                name: std::borrow::Cow::from(r###"(inputs::InlineFuncParam{.expr_id=(_0: ast::ExprId), .param=(_: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam) /*join*/"###),
                                                                                                                                 afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                 {
                                                                                                                                     let __cloned = __v.clone();
                                                                                                                                     match < InlineFuncParam>::from_ddvalue(__v) {
                                                                                                                                         InlineFuncParam{expr_id: ref _0, param: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                         _ => None
                                                                                                                                     }.map(|x|(x,__cloned))
                                                                                                                                 }
                                                                                                                                 __f},
                                                                                                                                 queryable: false
                                                                                                                             });
pub static __Arng_inputs_InputScope_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=(_: ast::ScopeId), .child=(_0: ast::ScopeId)}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: _, child: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InputScope_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=(_0: ast::ScopeId), .child=(_: ast::ScopeId)}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: ref _0, child: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Label_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Label{.stmt_id=(ast::StmtId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::StmtId), .name=(ddlog_std::Some{.x=(_: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>), .body_scope=(_: ast::ScopeId)}: inputs::Label) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Label>::from_ddvalue(__v) {
                                                                                                                               Label{stmt_id: types__ast::StmtId{id: _, file: ref _0}, name: ddlog_std::Option::Some{x: _}, body: _, body_scope: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_Label_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Label{.stmt_id=(ast::StmtId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::StmtId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>), .body_scope=(_: ast::ScopeId)}: inputs::Label) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Label>::from_ddvalue(__v) {
                                                                                                                               Label{stmt_id: types__ast::StmtId{id: _, file: ref _0}, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, body: _, body_scope: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_LetDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::LetDecl{.stmt_id=(_0: ast::StmtId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < LetDecl>::from_ddvalue(__v) {
                                                                                                                                 LetDecl{stmt_id: ref _0, pattern: ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_NameRef_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::NameRef{.expr_id=(_0: ast::ExprId), .value=(_: internment::Intern<string>)}: inputs::NameRef) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < NameRef>::from_ddvalue(__v) {
                                                                                                                                 NameRef{expr_id: ref _0, value: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_NameRef_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::NameRef{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .value=(_: internment::Intern<string>)}: inputs::NameRef) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < NameRef>::from_ddvalue(__v) {
                                                                                                                                 NameRef{expr_id: types__ast::ExprId{id: _, file: ref _0}, value: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_New_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"(inputs::New{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .object=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < New>::from_ddvalue(__v) {
                                                                                                                             New{expr_id: types__ast::ExprId{id: _, file: ref _0}, object: ddlog_std::Option::Some{x: _}, args: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng_inputs_New_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::New{.expr_id=(_: ast::ExprId), .object=(ddlog_std::Some{.x=(_0: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New) /*antijoin*/"###),
                                                                                                                     fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                     {
                                                                                                                         match < New>::from_ddvalue(__v) {
                                                                                                                             New{expr_id: _, object: ddlog_std::Option::Some{x: ref _0}, args: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     distinct: true
                                                                                                                 });
pub static __Arng_inputs_Statement_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::Statement{.id=(_0: ast::StmtId), .kind=(_: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < Statement>::from_ddvalue(__v) {
                                                                                                                                   Statement{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_Statement_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::Statement{.id=(_0: ast::StmtId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < Statement>::from_ddvalue(__v) {
                                                                                                                                   Statement{id: ref _0, kind: types__ast::StmtKind::StmtVarDecl{}, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_Statement_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::Statement{.id=_0, .kind=(_: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < Statement>::from_ddvalue(__v) {
                                                                                                                                   Statement{id: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: true
                                                                                                                       });
pub static __Arng_inputs_Try_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"(inputs::Try{.stmt_id=(_: ast::StmtId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(_0: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < Try>::from_ddvalue(__v) {
                                                                                                                             Try{stmt_id: _, body: _, handler: types__ast::TryHandler{error: ddlog_std::Option::Some{x: _}, body: ddlog_std::Option::Some{x: ref _0}}, finalizer: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng_inputs_UnaryOp_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::UnaryOp{.expr_id=(ast::ExprId{.id=(_: bit<32>), .file=(_0: ast::FileId)}: ast::ExprId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < UnaryOp>::from_ddvalue(__v) {
                                                                                                                                 UnaryOp{expr_id: types__ast::ExprId{id: _, file: ref _0}, op: ddlog_std::Option::Some{x: types__ast::UnaryOperand::UnaryTypeof{}}, expr: ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_UserGlobal_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                            name: std::borrow::Cow::from(r###"(inputs::UserGlobal{.id=(ast::GlobalId{.id=(_: bit<32>), .file=(ddlog_std::Some{.x=(_0: ast::FileId)}: ddlog_std::Option<ast::FileId>)}: ast::GlobalId), .name=(_1: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::UserGlobal) /*antijoin*/"###),
                                                                                                                            fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                            {
                                                                                                                                match < UserGlobal>::from_ddvalue(__v) {
                                                                                                                                    UserGlobal{id: types__ast::GlobalId{id: _, file: ddlog_std::Option::Some{x: ref _0}}, name: ref _1, privileges: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            distinct: true
                                                                                                                        });
pub static __Arng_inputs_VarDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::VarDecl{.stmt_id=(_0: ast::StmtId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < VarDecl>::from_ddvalue(__v) {
                                                                                                                                 VarDecl{stmt_id: ref _0, pattern: ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });