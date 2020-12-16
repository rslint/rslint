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
    clippy::unknown_clippy_lints,
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

//use ::serde::de::DeserializeOwned;
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


#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Array {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub elements: ddlog_std::Vec<types__ast::ArrayElement>
}
impl abomonation::Abomonation for Array{}
::differential_datalog::decl_struct_from_record!(Array["inputs::Array"]<>, ["inputs::Array"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]elements["elements"]: ddlog_std::Vec<types__ast::ArrayElement>});
::differential_datalog::decl_struct_into_record!(Array, ["inputs::Array"]<>, expr_id, file, elements);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Array, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, elements: ddlog_std::Vec<types__ast::ArrayElement>);
impl ::std::fmt::Display for Array {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Array{expr_id,file,elements} => {
                __formatter.write_str("inputs::Array{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Arrow {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub body: ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Either<types__ast::ExprId, types__ast::StmtId>, types__ast::ScopeId>>
}
impl abomonation::Abomonation for Arrow{}
::differential_datalog::decl_struct_from_record!(Arrow["inputs::Arrow"]<>, ["inputs::Arrow"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]body["body"]: ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Either<types__ast::ExprId, types__ast::StmtId>, types__ast::ScopeId>>});
::differential_datalog::decl_struct_into_record!(Arrow, ["inputs::Arrow"]<>, expr_id, file, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Arrow, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, body: ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Either<types__ast::ExprId, types__ast::StmtId>, types__ast::ScopeId>>);
impl ::std::fmt::Display for Arrow {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Arrow{expr_id,file,body} => {
                __formatter.write_str("inputs::Arrow{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ArrowParam {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub param: types__ast::IPattern
}
impl abomonation::Abomonation for ArrowParam{}
::differential_datalog::decl_struct_from_record!(ArrowParam["inputs::ArrowParam"]<>, ["inputs::ArrowParam"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]param["param"]: types__ast::IPattern});
::differential_datalog::decl_struct_into_record!(ArrowParam, ["inputs::ArrowParam"]<>, expr_id, file, param);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ArrowParam, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, param: types__ast::IPattern);
impl ::std::fmt::Display for ArrowParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ArrowParam{expr_id,file,param} => {
                __formatter.write_str("inputs::ArrowParam{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Assign {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub lhs: ddlog_std::Option<ddlog_std::Either<types__ast::IPattern, types__ast::ExprId>>,
    pub rhs: ddlog_std::Option<types__ast::ExprId>,
    pub op: ddlog_std::Option<types__ast::AssignOperand>
}
impl abomonation::Abomonation for Assign{}
::differential_datalog::decl_struct_from_record!(Assign["inputs::Assign"]<>, ["inputs::Assign"][5]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]lhs["lhs"]: ddlog_std::Option<ddlog_std::Either<types__ast::IPattern, types__ast::ExprId>>, [3]rhs["rhs"]: ddlog_std::Option<types__ast::ExprId>, [4]op["op"]: ddlog_std::Option<types__ast::AssignOperand>});
::differential_datalog::decl_struct_into_record!(Assign, ["inputs::Assign"]<>, expr_id, file, lhs, rhs, op);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Assign, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, lhs: ddlog_std::Option<ddlog_std::Either<types__ast::IPattern, types__ast::ExprId>>, rhs: ddlog_std::Option<types__ast::ExprId>, op: ddlog_std::Option<types__ast::AssignOperand>);
impl ::std::fmt::Display for Assign {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Assign{expr_id,file,lhs,rhs,op} => {
                __formatter.write_str("inputs::Assign{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Await {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub value: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Await{}
::differential_datalog::decl_struct_from_record!(Await["inputs::Await"]<>, ["inputs::Await"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]value["value"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Await, ["inputs::Await"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Await, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, value: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for Await {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Await{expr_id,file,value} => {
                __formatter.write_str("inputs::Await{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct BinOp {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub op: ddlog_std::Option<types__ast::BinOperand>,
    pub lhs: ddlog_std::Option<types__ast::ExprId>,
    pub rhs: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for BinOp{}
::differential_datalog::decl_struct_from_record!(BinOp["inputs::BinOp"]<>, ["inputs::BinOp"][5]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]op["op"]: ddlog_std::Option<types__ast::BinOperand>, [3]lhs["lhs"]: ddlog_std::Option<types__ast::ExprId>, [4]rhs["rhs"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(BinOp, ["inputs::BinOp"]<>, expr_id, file, op, lhs, rhs);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(BinOp, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, op: ddlog_std::Option<types__ast::BinOperand>, lhs: ddlog_std::Option<types__ast::ExprId>, rhs: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for BinOp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            BinOp{expr_id,file,op,lhs,rhs} => {
                __formatter.write_str("inputs::BinOp{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct BracketAccess {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub object: ddlog_std::Option<types__ast::ExprId>,
    pub prop: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for BracketAccess{}
::differential_datalog::decl_struct_from_record!(BracketAccess["inputs::BracketAccess"]<>, ["inputs::BracketAccess"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]object["object"]: ddlog_std::Option<types__ast::ExprId>, [3]prop["prop"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(BracketAccess, ["inputs::BracketAccess"]<>, expr_id, file, object, prop);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(BracketAccess, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, object: ddlog_std::Option<types__ast::ExprId>, prop: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for BracketAccess {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            BracketAccess{expr_id,file,object,prop} => {
                __formatter.write_str("inputs::BracketAccess{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Break {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub label: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>
}
impl abomonation::Abomonation for Break{}
::differential_datalog::decl_struct_from_record!(Break["inputs::Break"]<>, ["inputs::Break"][3]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]label["label"]: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>});
::differential_datalog::decl_struct_into_record!(Break, ["inputs::Break"]<>, stmt_id, file, label);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Break, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, label: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>);
impl ::std::fmt::Display for Break {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Break{stmt_id,file,label} => {
                __formatter.write_str("inputs::Break{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Call {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub callee: ddlog_std::Option<types__ast::ExprId>,
    pub args: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>
}
impl abomonation::Abomonation for Call{}
::differential_datalog::decl_struct_from_record!(Call["inputs::Call"]<>, ["inputs::Call"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]callee["callee"]: ddlog_std::Option<types__ast::ExprId>, [3]args["args"]: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>});
::differential_datalog::decl_struct_into_record!(Call, ["inputs::Call"]<>, expr_id, file, callee, args);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Call, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, callee: ddlog_std::Option<types__ast::ExprId>, args: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>);
impl ::std::fmt::Display for Call {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Call{expr_id,file,callee,args} => {
                __formatter.write_str("inputs::Call{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Class {
    pub id: types__ast::ClassId,
    pub file: types__ast::FileId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub parent: ddlog_std::Option<types__ast::ExprId>,
    pub elements: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>,
    pub scope: types__ast::ScopeId,
    pub exported: bool
}
impl abomonation::Abomonation for Class{}
::differential_datalog::decl_struct_from_record!(Class["inputs::Class"]<>, ["inputs::Class"][7]{[0]id["id"]: types__ast::ClassId, [1]file["file"]: types__ast::FileId, [2]name["name"]: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, [3]parent["parent"]: ddlog_std::Option<types__ast::ExprId>, [4]elements["elements"]: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>, [5]scope["scope"]: types__ast::ScopeId, [6]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(Class, ["inputs::Class"]<>, id, file, name, parent, elements, scope, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Class, <>, id: types__ast::ClassId, file: types__ast::FileId, name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, parent: ddlog_std::Option<types__ast::ExprId>, elements: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>, scope: types__ast::ScopeId, exported: bool);
impl ::std::fmt::Display for Class {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Class{id,file,name,parent,elements,scope,exported} => {
                __formatter.write_str("inputs::Class{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ClassExpr {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub elements: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>
}
impl abomonation::Abomonation for ClassExpr{}
::differential_datalog::decl_struct_from_record!(ClassExpr["inputs::ClassExpr"]<>, ["inputs::ClassExpr"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]elements["elements"]: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>});
::differential_datalog::decl_struct_into_record!(ClassExpr, ["inputs::ClassExpr"]<>, expr_id, file, elements);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ClassExpr, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, elements: ddlog_std::Option<ddlog_std::Vec<types__ast::IClassElement>>);
impl ::std::fmt::Display for ClassExpr {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ClassExpr{expr_id,file,elements} => {
                __formatter.write_str("inputs::ClassExpr{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ConstDecl {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub pattern: ddlog_std::Option<types__ast::IPattern>,
    pub value: ddlog_std::Option<types__ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for ConstDecl{}
::differential_datalog::decl_struct_from_record!(ConstDecl["inputs::ConstDecl"]<>, ["inputs::ConstDecl"][5]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]pattern["pattern"]: ddlog_std::Option<types__ast::IPattern>, [3]value["value"]: ddlog_std::Option<types__ast::ExprId>, [4]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(ConstDecl, ["inputs::ConstDecl"]<>, stmt_id, file, pattern, value, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ConstDecl, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, pattern: ddlog_std::Option<types__ast::IPattern>, value: ddlog_std::Option<types__ast::ExprId>, exported: bool);
impl ::std::fmt::Display for ConstDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ConstDecl{stmt_id,file,pattern,value,exported} => {
                __formatter.write_str("inputs::ConstDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Continue {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub label: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>
}
impl abomonation::Abomonation for Continue{}
::differential_datalog::decl_struct_from_record!(Continue["inputs::Continue"]<>, ["inputs::Continue"][3]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]label["label"]: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>});
::differential_datalog::decl_struct_into_record!(Continue, ["inputs::Continue"]<>, stmt_id, file, label);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Continue, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, label: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>);
impl ::std::fmt::Display for Continue {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Continue{stmt_id,file,label} => {
                __formatter.write_str("inputs::Continue{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct DoWhile {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub body: ddlog_std::Option<types__ast::StmtId>,
    pub cond: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for DoWhile{}
::differential_datalog::decl_struct_from_record!(DoWhile["inputs::DoWhile"]<>, ["inputs::DoWhile"][4]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]body["body"]: ddlog_std::Option<types__ast::StmtId>, [3]cond["cond"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(DoWhile, ["inputs::DoWhile"]<>, stmt_id, file, body, cond);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(DoWhile, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, body: ddlog_std::Option<types__ast::StmtId>, cond: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for DoWhile {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            DoWhile{stmt_id,file,body,cond} => {
                __formatter.write_str("inputs::DoWhile{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct DotAccess {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub object: ddlog_std::Option<types__ast::ExprId>,
    pub prop: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>
}
impl abomonation::Abomonation for DotAccess{}
::differential_datalog::decl_struct_from_record!(DotAccess["inputs::DotAccess"]<>, ["inputs::DotAccess"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]object["object"]: ddlog_std::Option<types__ast::ExprId>, [3]prop["prop"]: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>});
::differential_datalog::decl_struct_into_record!(DotAccess, ["inputs::DotAccess"]<>, expr_id, file, object, prop);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(DotAccess, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, object: ddlog_std::Option<types__ast::ExprId>, prop: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>);
impl ::std::fmt::Display for DotAccess {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            DotAccess{expr_id,file,object,prop} => {
                __formatter.write_str("inputs::DotAccess{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct EveryScope {
    pub scope: types__ast::ScopeId,
    pub file: types__ast::FileId
}
impl abomonation::Abomonation for EveryScope{}
::differential_datalog::decl_struct_from_record!(EveryScope["inputs::EveryScope"]<>, ["inputs::EveryScope"][2]{[0]scope["scope"]: types__ast::ScopeId, [1]file["file"]: types__ast::FileId});
::differential_datalog::decl_struct_into_record!(EveryScope, ["inputs::EveryScope"]<>, scope, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(EveryScope, <>, scope: types__ast::ScopeId, file: types__ast::FileId);
impl ::std::fmt::Display for EveryScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            EveryScope{scope,file} => {
                __formatter.write_str("inputs::EveryScope{")?;
                ::std::fmt::Debug::fmt(scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ExprBigInt {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub value: ::ddlog_bigint::Int
}
impl abomonation::Abomonation for ExprBigInt{}
::differential_datalog::decl_struct_from_record!(ExprBigInt["inputs::ExprBigInt"]<>, ["inputs::ExprBigInt"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]value["value"]: ::ddlog_bigint::Int});
::differential_datalog::decl_struct_into_record!(ExprBigInt, ["inputs::ExprBigInt"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprBigInt, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, value: ::ddlog_bigint::Int);
impl ::std::fmt::Display for ExprBigInt {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprBigInt{expr_id,file,value} => {
                __formatter.write_str("inputs::ExprBigInt{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ExprBool {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub value: bool
}
impl abomonation::Abomonation for ExprBool{}
::differential_datalog::decl_struct_from_record!(ExprBool["inputs::ExprBool"]<>, ["inputs::ExprBool"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]value["value"]: bool});
::differential_datalog::decl_struct_into_record!(ExprBool, ["inputs::ExprBool"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprBool, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, value: bool);
impl ::std::fmt::Display for ExprBool {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprBool{expr_id,file,value} => {
                __formatter.write_str("inputs::ExprBool{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ExprNumber {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub value: ::ordered_float::OrderedFloat<f64>
}
impl abomonation::Abomonation for ExprNumber{}
::differential_datalog::decl_struct_from_record!(ExprNumber["inputs::ExprNumber"]<>, ["inputs::ExprNumber"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]value["value"]: ::ordered_float::OrderedFloat<f64>});
::differential_datalog::decl_struct_into_record!(ExprNumber, ["inputs::ExprNumber"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprNumber, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, value: ::ordered_float::OrderedFloat<f64>);
impl ::std::fmt::Display for ExprNumber {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprNumber{expr_id,file,value} => {
                __formatter.write_str("inputs::ExprNumber{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ExprString {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub value: internment::istring
}
impl abomonation::Abomonation for ExprString{}
::differential_datalog::decl_struct_from_record!(ExprString["inputs::ExprString"]<>, ["inputs::ExprString"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]value["value"]: internment::istring});
::differential_datalog::decl_struct_into_record!(ExprString, ["inputs::ExprString"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprString, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, value: internment::istring);
impl ::std::fmt::Display for ExprString {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprString{expr_id,file,value} => {
                __formatter.write_str("inputs::ExprString{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Expression {
    pub id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub kind: types__ast::ExprKind,
    pub scope: types__ast::ScopeId,
    pub span: types__ast::Span
}
impl abomonation::Abomonation for Expression{}
::differential_datalog::decl_struct_from_record!(Expression["inputs::Expression"]<>, ["inputs::Expression"][5]{[0]id["id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]kind["kind"]: types__ast::ExprKind, [3]scope["scope"]: types__ast::ScopeId, [4]span["span"]: types__ast::Span});
::differential_datalog::decl_struct_into_record!(Expression, ["inputs::Expression"]<>, id, file, kind, scope, span);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Expression, <>, id: types__ast::ExprId, file: types__ast::FileId, kind: types__ast::ExprKind, scope: types__ast::ScopeId, span: types__ast::Span);
impl ::std::fmt::Display for Expression {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Expression{id,file,kind,scope,span} => {
                __formatter.write_str("inputs::Expression{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct File {
    pub id: types__ast::FileId,
    pub kind: types__ast::FileKind,
    pub top_level_scope: types__ast::ScopeId,
    pub config: types__config::Config
}
impl abomonation::Abomonation for File{}
::differential_datalog::decl_struct_from_record!(File["inputs::File"]<>, ["inputs::File"][4]{[0]id["id"]: types__ast::FileId, [1]kind["kind"]: types__ast::FileKind, [2]top_level_scope["top_level_scope"]: types__ast::ScopeId, [3]config["config"]: types__config::Config});
::differential_datalog::decl_struct_into_record!(File, ["inputs::File"]<>, id, kind, top_level_scope, config);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(File, <>, id: types__ast::FileId, kind: types__ast::FileKind, top_level_scope: types__ast::ScopeId, config: types__config::Config);
impl ::std::fmt::Display for File {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            File{id,kind,top_level_scope,config} => {
                __formatter.write_str("inputs::File{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(top_level_scope, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(config, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct FileExport {
    pub file: types__ast::FileId,
    pub export: types__ast::ExportKind,
    pub scope: types__ast::ScopeId
}
impl abomonation::Abomonation for FileExport{}
::differential_datalog::decl_struct_from_record!(FileExport["inputs::FileExport"]<>, ["inputs::FileExport"][3]{[0]file["file"]: types__ast::FileId, [1]export["export"]: types__ast::ExportKind, [2]scope["scope"]: types__ast::ScopeId});
::differential_datalog::decl_struct_into_record!(FileExport, ["inputs::FileExport"]<>, file, export, scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FileExport, <>, file: types__ast::FileId, export: types__ast::ExportKind, scope: types__ast::ScopeId);
impl ::std::fmt::Display for FileExport {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FileExport{file,export,scope} => {
                __formatter.write_str("inputs::FileExport{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str(",")?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct For {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub init: ddlog_std::Option<types__ast::ForInit>,
    pub test: ddlog_std::Option<types__ast::ExprId>,
    pub update: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for For{}
::differential_datalog::decl_struct_from_record!(For["inputs::For"]<>, ["inputs::For"][6]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]init["init"]: ddlog_std::Option<types__ast::ForInit>, [3]test["test"]: ddlog_std::Option<types__ast::ExprId>, [4]update["update"]: ddlog_std::Option<types__ast::ExprId>, [5]body["body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(For, ["inputs::For"]<>, stmt_id, file, init, test, update, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(For, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, init: ddlog_std::Option<types__ast::ForInit>, test: ddlog_std::Option<types__ast::ExprId>, update: ddlog_std::Option<types__ast::ExprId>, body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for For {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            For{stmt_id,file,init,test,update,body} => {
                __formatter.write_str("inputs::For{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ForIn {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub elem: ddlog_std::Option<types__ast::ForInit>,
    pub collection: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for ForIn{}
::differential_datalog::decl_struct_from_record!(ForIn["inputs::ForIn"]<>, ["inputs::ForIn"][5]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]elem["elem"]: ddlog_std::Option<types__ast::ForInit>, [3]collection["collection"]: ddlog_std::Option<types__ast::ExprId>, [4]body["body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(ForIn, ["inputs::ForIn"]<>, stmt_id, file, elem, collection, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ForIn, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, elem: ddlog_std::Option<types__ast::ForInit>, collection: ddlog_std::Option<types__ast::ExprId>, body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for ForIn {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ForIn{stmt_id,file,elem,collection,body} => {
                __formatter.write_str("inputs::ForIn{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ForOf {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub awaited: bool,
    pub elem: ddlog_std::Option<types__ast::ForInit>,
    pub collection: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for ForOf{}
::differential_datalog::decl_struct_from_record!(ForOf["inputs::ForOf"]<>, ["inputs::ForOf"][6]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]awaited["awaited"]: bool, [3]elem["elem"]: ddlog_std::Option<types__ast::ForInit>, [4]collection["collection"]: ddlog_std::Option<types__ast::ExprId>, [5]body["body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(ForOf, ["inputs::ForOf"]<>, stmt_id, file, awaited, elem, collection, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ForOf, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, awaited: bool, elem: ddlog_std::Option<types__ast::ForInit>, collection: ddlog_std::Option<types__ast::ExprId>, body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for ForOf {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ForOf{stmt_id,file,awaited,elem,collection,body} => {
                __formatter.write_str("inputs::ForOf{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Function {
    pub id: types__ast::FuncId,
    pub file: types__ast::FileId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub scope: types__ast::ScopeId,
    pub body: types__ast::ScopeId,
    pub exported: bool
}
impl abomonation::Abomonation for Function{}
::differential_datalog::decl_struct_from_record!(Function["inputs::Function"]<>, ["inputs::Function"][6]{[0]id["id"]: types__ast::FuncId, [1]file["file"]: types__ast::FileId, [2]name["name"]: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, [3]scope["scope"]: types__ast::ScopeId, [4]body["body"]: types__ast::ScopeId, [5]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(Function, ["inputs::Function"]<>, id, file, name, scope, body, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Function, <>, id: types__ast::FuncId, file: types__ast::FileId, name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, scope: types__ast::ScopeId, body: types__ast::ScopeId, exported: bool);
impl ::std::fmt::Display for Function {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Function{id,file,name,scope,body,exported} => {
                __formatter.write_str("inputs::Function{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct FunctionArg {
    pub parent_func: types__ast::FuncId,
    pub file: types__ast::FileId,
    pub pattern: types__ast::IPattern,
    pub implicit: bool
}
impl abomonation::Abomonation for FunctionArg{}
::differential_datalog::decl_struct_from_record!(FunctionArg["inputs::FunctionArg"]<>, ["inputs::FunctionArg"][4]{[0]parent_func["parent_func"]: types__ast::FuncId, [1]file["file"]: types__ast::FileId, [2]pattern["pattern"]: types__ast::IPattern, [3]implicit["implicit"]: bool});
::differential_datalog::decl_struct_into_record!(FunctionArg, ["inputs::FunctionArg"]<>, parent_func, file, pattern, implicit);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FunctionArg, <>, parent_func: types__ast::FuncId, file: types__ast::FileId, pattern: types__ast::IPattern, implicit: bool);
impl ::std::fmt::Display for FunctionArg {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FunctionArg{parent_func,file,pattern,implicit} => {
                __formatter.write_str("inputs::FunctionArg{")?;
                ::std::fmt::Debug::fmt(parent_func, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct If {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub cond: ddlog_std::Option<types__ast::ExprId>,
    pub if_body: ddlog_std::Option<types__ast::StmtId>,
    pub else_body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for If{}
::differential_datalog::decl_struct_from_record!(If["inputs::If"]<>, ["inputs::If"][5]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]cond["cond"]: ddlog_std::Option<types__ast::ExprId>, [3]if_body["if_body"]: ddlog_std::Option<types__ast::StmtId>, [4]else_body["else_body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(If, ["inputs::If"]<>, stmt_id, file, cond, if_body, else_body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(If, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, cond: ddlog_std::Option<types__ast::ExprId>, if_body: ddlog_std::Option<types__ast::StmtId>, else_body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for If {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            If{stmt_id,file,cond,if_body,else_body} => {
                __formatter.write_str("inputs::If{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ImplicitGlobal {
    pub id: types__ast::GlobalId,
    pub name: types__ast::Name,
    pub privileges: types__ast::GlobalPriv
}
impl abomonation::Abomonation for ImplicitGlobal{}
::differential_datalog::decl_struct_from_record!(ImplicitGlobal["inputs::ImplicitGlobal"]<>, ["inputs::ImplicitGlobal"][3]{[0]id["id"]: types__ast::GlobalId, [1]name["name"]: types__ast::Name, [2]privileges["privileges"]: types__ast::GlobalPriv});
::differential_datalog::decl_struct_into_record!(ImplicitGlobal, ["inputs::ImplicitGlobal"]<>, id, name, privileges);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ImplicitGlobal, <>, id: types__ast::GlobalId, name: types__ast::Name, privileges: types__ast::GlobalPriv);
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ImportDecl {
    pub id: types__ast::ImportId,
    pub file: types__ast::FileId,
    pub clause: types__ast::ImportClause
}
impl abomonation::Abomonation for ImportDecl{}
::differential_datalog::decl_struct_from_record!(ImportDecl["inputs::ImportDecl"]<>, ["inputs::ImportDecl"][3]{[0]id["id"]: types__ast::ImportId, [1]file["file"]: types__ast::FileId, [2]clause["clause"]: types__ast::ImportClause});
::differential_datalog::decl_struct_into_record!(ImportDecl, ["inputs::ImportDecl"]<>, id, file, clause);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ImportDecl, <>, id: types__ast::ImportId, file: types__ast::FileId, clause: types__ast::ImportClause);
impl ::std::fmt::Display for ImportDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImportDecl{id,file,clause} => {
                __formatter.write_str("inputs::ImportDecl{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct InlineFunc {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for InlineFunc{}
::differential_datalog::decl_struct_from_record!(InlineFunc["inputs::InlineFunc"]<>, ["inputs::InlineFunc"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]name["name"]: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, [3]body["body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(InlineFunc, ["inputs::InlineFunc"]<>, expr_id, file, name, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(InlineFunc, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for InlineFunc {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            InlineFunc{expr_id,file,name,body} => {
                __formatter.write_str("inputs::InlineFunc{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct InlineFuncParam {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub param: types__ast::IPattern
}
impl abomonation::Abomonation for InlineFuncParam{}
::differential_datalog::decl_struct_from_record!(InlineFuncParam["inputs::InlineFuncParam"]<>, ["inputs::InlineFuncParam"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]param["param"]: types__ast::IPattern});
::differential_datalog::decl_struct_into_record!(InlineFuncParam, ["inputs::InlineFuncParam"]<>, expr_id, file, param);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(InlineFuncParam, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, param: types__ast::IPattern);
impl ::std::fmt::Display for InlineFuncParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            InlineFuncParam{expr_id,file,param} => {
                __formatter.write_str("inputs::InlineFuncParam{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct InputScope {
    pub parent: types__ast::ScopeId,
    pub child: types__ast::ScopeId,
    pub file: types__ast::FileId
}
impl abomonation::Abomonation for InputScope{}
::differential_datalog::decl_struct_from_record!(InputScope["inputs::InputScope"]<>, ["inputs::InputScope"][3]{[0]parent["parent"]: types__ast::ScopeId, [1]child["child"]: types__ast::ScopeId, [2]file["file"]: types__ast::FileId});
::differential_datalog::decl_struct_into_record!(InputScope, ["inputs::InputScope"]<>, parent, child, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(InputScope, <>, parent: types__ast::ScopeId, child: types__ast::ScopeId, file: types__ast::FileId);
impl ::std::fmt::Display for InputScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            InputScope{parent,child,file} => {
                __formatter.write_str("inputs::InputScope{")?;
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
impl ::std::fmt::Debug for InputScope {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Label {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>,
    pub body: ddlog_std::Option<types__ast::StmtId>,
    pub body_scope: types__ast::ScopeId
}
impl abomonation::Abomonation for Label{}
::differential_datalog::decl_struct_from_record!(Label["inputs::Label"]<>, ["inputs::Label"][5]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]name["name"]: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, [3]body["body"]: ddlog_std::Option<types__ast::StmtId>, [4]body_scope["body_scope"]: types__ast::ScopeId});
::differential_datalog::decl_struct_into_record!(Label, ["inputs::Label"]<>, stmt_id, file, name, body, body_scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Label, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, name: ddlog_std::Option<types__ast::Spanned<types__ast::Name>>, body: ddlog_std::Option<types__ast::StmtId>, body_scope: types__ast::ScopeId);
impl ::std::fmt::Display for Label {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Label{stmt_id,file,name,body,body_scope} => {
                __formatter.write_str("inputs::Label{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct LetDecl {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub pattern: ddlog_std::Option<types__ast::IPattern>,
    pub value: ddlog_std::Option<types__ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for LetDecl{}
::differential_datalog::decl_struct_from_record!(LetDecl["inputs::LetDecl"]<>, ["inputs::LetDecl"][5]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]pattern["pattern"]: ddlog_std::Option<types__ast::IPattern>, [3]value["value"]: ddlog_std::Option<types__ast::ExprId>, [4]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(LetDecl, ["inputs::LetDecl"]<>, stmt_id, file, pattern, value, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(LetDecl, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, pattern: ddlog_std::Option<types__ast::IPattern>, value: ddlog_std::Option<types__ast::ExprId>, exported: bool);
impl ::std::fmt::Display for LetDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            LetDecl{stmt_id,file,pattern,value,exported} => {
                __formatter.write_str("inputs::LetDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NameRef {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub value: types__ast::Name
}
impl abomonation::Abomonation for NameRef{}
::differential_datalog::decl_struct_from_record!(NameRef["inputs::NameRef"]<>, ["inputs::NameRef"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]value["value"]: types__ast::Name});
::differential_datalog::decl_struct_into_record!(NameRef, ["inputs::NameRef"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameRef, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, value: types__ast::Name);
impl ::std::fmt::Display for NameRef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NameRef{expr_id,file,value} => {
                __formatter.write_str("inputs::NameRef{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct New {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub object: ddlog_std::Option<types__ast::ExprId>,
    pub args: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>
}
impl abomonation::Abomonation for New{}
::differential_datalog::decl_struct_from_record!(New["inputs::New"]<>, ["inputs::New"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]object["object"]: ddlog_std::Option<types__ast::ExprId>, [3]args["args"]: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>});
::differential_datalog::decl_struct_into_record!(New, ["inputs::New"]<>, expr_id, file, object, args);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(New, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, object: ddlog_std::Option<types__ast::ExprId>, args: ddlog_std::Option<ddlog_std::Vec<types__ast::ExprId>>);
impl ::std::fmt::Display for New {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            New{expr_id,file,object,args} => {
                __formatter.write_str("inputs::New{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Property {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub key: ddlog_std::Option<types__ast::PropertyKey>,
    pub val: ddlog_std::Option<types__ast::PropertyVal>
}
impl abomonation::Abomonation for Property{}
::differential_datalog::decl_struct_from_record!(Property["inputs::Property"]<>, ["inputs::Property"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]key["key"]: ddlog_std::Option<types__ast::PropertyKey>, [3]val["val"]: ddlog_std::Option<types__ast::PropertyVal>});
::differential_datalog::decl_struct_into_record!(Property, ["inputs::Property"]<>, expr_id, file, key, val);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Property, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, key: ddlog_std::Option<types__ast::PropertyKey>, val: ddlog_std::Option<types__ast::PropertyVal>);
impl ::std::fmt::Display for Property {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Property{expr_id,file,key,val} => {
                __formatter.write_str("inputs::Property{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Return {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub value: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Return{}
::differential_datalog::decl_struct_from_record!(Return["inputs::Return"]<>, ["inputs::Return"][3]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]value["value"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Return, ["inputs::Return"]<>, stmt_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Return, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, value: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for Return {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Return{stmt_id,file,value} => {
                __formatter.write_str("inputs::Return{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Statement {
    pub id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub kind: types__ast::StmtKind,
    pub scope: types__ast::ScopeId,
    pub span: types__ast::Span
}
impl abomonation::Abomonation for Statement{}
::differential_datalog::decl_struct_from_record!(Statement["inputs::Statement"]<>, ["inputs::Statement"][5]{[0]id["id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]kind["kind"]: types__ast::StmtKind, [3]scope["scope"]: types__ast::ScopeId, [4]span["span"]: types__ast::Span});
::differential_datalog::decl_struct_into_record!(Statement, ["inputs::Statement"]<>, id, file, kind, scope, span);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Statement, <>, id: types__ast::StmtId, file: types__ast::FileId, kind: types__ast::StmtKind, scope: types__ast::ScopeId, span: types__ast::Span);
impl ::std::fmt::Display for Statement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Statement{id,file,kind,scope,span} => {
                __formatter.write_str("inputs::Statement{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Switch {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub test: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Switch{}
::differential_datalog::decl_struct_from_record!(Switch["inputs::Switch"]<>, ["inputs::Switch"][3]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]test["test"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Switch, ["inputs::Switch"]<>, stmt_id, file, test);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Switch, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, test: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for Switch {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Switch{stmt_id,file,test} => {
                __formatter.write_str("inputs::Switch{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct SwitchCase {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub case: types__ast::SwitchClause,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for SwitchCase{}
::differential_datalog::decl_struct_from_record!(SwitchCase["inputs::SwitchCase"]<>, ["inputs::SwitchCase"][4]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]case["case"]: types__ast::SwitchClause, [3]body["body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(SwitchCase, ["inputs::SwitchCase"]<>, stmt_id, file, case, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(SwitchCase, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, case: types__ast::SwitchClause, body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for SwitchCase {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            SwitchCase{stmt_id,file,case,body} => {
                __formatter.write_str("inputs::SwitchCase{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Template {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub tag: ddlog_std::Option<types__ast::ExprId>,
    pub elements: ddlog_std::Vec<types__ast::ExprId>
}
impl abomonation::Abomonation for Template{}
::differential_datalog::decl_struct_from_record!(Template["inputs::Template"]<>, ["inputs::Template"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]tag["tag"]: ddlog_std::Option<types__ast::ExprId>, [3]elements["elements"]: ddlog_std::Vec<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Template, ["inputs::Template"]<>, expr_id, file, tag, elements);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Template, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, tag: ddlog_std::Option<types__ast::ExprId>, elements: ddlog_std::Vec<types__ast::ExprId>);
impl ::std::fmt::Display for Template {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Template{expr_id,file,tag,elements} => {
                __formatter.write_str("inputs::Template{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Ternary {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub test: ddlog_std::Option<types__ast::ExprId>,
    pub true_val: ddlog_std::Option<types__ast::ExprId>,
    pub false_val: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Ternary{}
::differential_datalog::decl_struct_from_record!(Ternary["inputs::Ternary"]<>, ["inputs::Ternary"][5]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]test["test"]: ddlog_std::Option<types__ast::ExprId>, [3]true_val["true_val"]: ddlog_std::Option<types__ast::ExprId>, [4]false_val["false_val"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Ternary, ["inputs::Ternary"]<>, expr_id, file, test, true_val, false_val);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Ternary, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, test: ddlog_std::Option<types__ast::ExprId>, true_val: ddlog_std::Option<types__ast::ExprId>, false_val: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for Ternary {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Ternary{expr_id,file,test,true_val,false_val} => {
                __formatter.write_str("inputs::Ternary{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Throw {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub exception: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Throw{}
::differential_datalog::decl_struct_from_record!(Throw["inputs::Throw"]<>, ["inputs::Throw"][3]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]exception["exception"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Throw, ["inputs::Throw"]<>, stmt_id, file, exception);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Throw, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, exception: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for Throw {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Throw{stmt_id,file,exception} => {
                __formatter.write_str("inputs::Throw{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Try {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub body: ddlog_std::Option<types__ast::StmtId>,
    pub handler: types__ast::TryHandler,
    pub finalizer: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for Try{}
::differential_datalog::decl_struct_from_record!(Try["inputs::Try"]<>, ["inputs::Try"][5]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]body["body"]: ddlog_std::Option<types__ast::StmtId>, [3]handler["handler"]: types__ast::TryHandler, [4]finalizer["finalizer"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(Try, ["inputs::Try"]<>, stmt_id, file, body, handler, finalizer);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Try, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, body: ddlog_std::Option<types__ast::StmtId>, handler: types__ast::TryHandler, finalizer: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for Try {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Try{stmt_id,file,body,handler,finalizer} => {
                __formatter.write_str("inputs::Try{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct UnaryOp {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub op: ddlog_std::Option<types__ast::UnaryOperand>,
    pub expr: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for UnaryOp{}
::differential_datalog::decl_struct_from_record!(UnaryOp["inputs::UnaryOp"]<>, ["inputs::UnaryOp"][4]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]op["op"]: ddlog_std::Option<types__ast::UnaryOperand>, [3]expr["expr"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(UnaryOp, ["inputs::UnaryOp"]<>, expr_id, file, op, expr);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(UnaryOp, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, op: ddlog_std::Option<types__ast::UnaryOperand>, expr: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for UnaryOp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            UnaryOp{expr_id,file,op,expr} => {
                __formatter.write_str("inputs::UnaryOp{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct UserGlobal {
    pub id: types__ast::GlobalId,
    pub file: types__ast::FileId,
    pub name: types__ast::Name,
    pub privileges: types__ast::GlobalPriv
}
impl abomonation::Abomonation for UserGlobal{}
::differential_datalog::decl_struct_from_record!(UserGlobal["inputs::UserGlobal"]<>, ["inputs::UserGlobal"][4]{[0]id["id"]: types__ast::GlobalId, [1]file["file"]: types__ast::FileId, [2]name["name"]: types__ast::Name, [3]privileges["privileges"]: types__ast::GlobalPriv});
::differential_datalog::decl_struct_into_record!(UserGlobal, ["inputs::UserGlobal"]<>, id, file, name, privileges);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(UserGlobal, <>, id: types__ast::GlobalId, file: types__ast::FileId, name: types__ast::Name, privileges: types__ast::GlobalPriv);
impl ::std::fmt::Display for UserGlobal {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            UserGlobal{id,file,name,privileges} => {
                __formatter.write_str("inputs::UserGlobal{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct VarDecl {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub pattern: ddlog_std::Option<types__ast::IPattern>,
    pub value: ddlog_std::Option<types__ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for VarDecl{}
::differential_datalog::decl_struct_from_record!(VarDecl["inputs::VarDecl"]<>, ["inputs::VarDecl"][5]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]pattern["pattern"]: ddlog_std::Option<types__ast::IPattern>, [3]value["value"]: ddlog_std::Option<types__ast::ExprId>, [4]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(VarDecl, ["inputs::VarDecl"]<>, stmt_id, file, pattern, value, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(VarDecl, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, pattern: ddlog_std::Option<types__ast::IPattern>, value: ddlog_std::Option<types__ast::ExprId>, exported: bool);
impl ::std::fmt::Display for VarDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            VarDecl{stmt_id,file,pattern,value,exported} => {
                __formatter.write_str("inputs::VarDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct While {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub cond: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for While{}
::differential_datalog::decl_struct_from_record!(While["inputs::While"]<>, ["inputs::While"][4]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]cond["cond"]: ddlog_std::Option<types__ast::ExprId>, [3]body["body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(While, ["inputs::While"]<>, stmt_id, file, cond, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(While, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, cond: ddlog_std::Option<types__ast::ExprId>, body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for While {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            While{stmt_id,file,cond,body} => {
                __formatter.write_str("inputs::While{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct With {
    pub stmt_id: types__ast::StmtId,
    pub file: types__ast::FileId,
    pub cond: ddlog_std::Option<types__ast::ExprId>,
    pub body: ddlog_std::Option<types__ast::StmtId>
}
impl abomonation::Abomonation for With{}
::differential_datalog::decl_struct_from_record!(With["inputs::With"]<>, ["inputs::With"][4]{[0]stmt_id["stmt_id"]: types__ast::StmtId, [1]file["file"]: types__ast::FileId, [2]cond["cond"]: ddlog_std::Option<types__ast::ExprId>, [3]body["body"]: ddlog_std::Option<types__ast::StmtId>});
::differential_datalog::decl_struct_into_record!(With, ["inputs::With"]<>, stmt_id, file, cond, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(With, <>, stmt_id: types__ast::StmtId, file: types__ast::FileId, cond: ddlog_std::Option<types__ast::ExprId>, body: ddlog_std::Option<types__ast::StmtId>);
impl ::std::fmt::Display for With {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            With{stmt_id,file,cond,body} => {
                __formatter.write_str("inputs::With{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Yield {
    pub expr_id: types__ast::ExprId,
    pub file: types__ast::FileId,
    pub value: ddlog_std::Option<types__ast::ExprId>
}
impl abomonation::Abomonation for Yield{}
::differential_datalog::decl_struct_from_record!(Yield["inputs::Yield"]<>, ["inputs::Yield"][3]{[0]expr_id["expr_id"]: types__ast::ExprId, [1]file["file"]: types__ast::FileId, [2]value["value"]: ddlog_std::Option<types__ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Yield, ["inputs::Yield"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Yield, <>, expr_id: types__ast::ExprId, file: types__ast::FileId, value: ddlog_std::Option<types__ast::ExprId>);
impl ::std::fmt::Display for Yield {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Yield{expr_id,file,value} => {
                __formatter.write_str("inputs::Yield{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
    (ddlog_std::tuple2(c.id.clone(), c.file.clone())).into_ddvalue()
}
pub fn __Key_inputs_Expression(__key: &DDValue) -> DDValue {
    let ref e = *{<Expression>::from_ddvalue_ref(__key) };
    (ddlog_std::tuple2(e.id.clone(), e.file.clone())).into_ddvalue()
}
pub fn __Key_inputs_File(__key: &DDValue) -> DDValue {
    let ref f = *{<File>::from_ddvalue_ref(__key) };
    (f.id.clone()).into_ddvalue()
}
pub fn __Key_inputs_Function(__key: &DDValue) -> DDValue {
    let ref f = *{<Function>::from_ddvalue_ref(__key) };
    (ddlog_std::tuple2(f.id.clone(), f.file.clone())).into_ddvalue()
}
pub fn __Key_inputs_Statement(__key: &DDValue) -> DDValue {
    let ref stmt = *{<Statement>::from_ddvalue_ref(__key) };
    (ddlog_std::tuple2(stmt.id.clone(), stmt.file.clone())).into_ddvalue()
}
pub static __Arng_inputs_Arrow_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Arrow{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .body=(ddlog_std::Some{.x=((_: ddlog_std::Either<ast::ExprId,ast::StmtId>), (_: ast::ScopeId))}: ddlog_std::Option<(ddlog_std::Either<ast::ExprId,ast::StmtId>, ast::ScopeId)>)}: inputs::Arrow) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Arrow>::from_ddvalue(__v) {
                                                                                                                               Arrow{expr_id: ref _0, file: ref _1, body: ddlog_std::Option::Some{x: ddlog_std::tuple2(_, _)}} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_ArrowParam_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::ArrowParam{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .param=(_: internment::Intern<ast::Pattern>)}: inputs::ArrowParam) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < ArrowParam>::from_ddvalue(__v) {
                                                                                                                                    ArrowParam{expr_id: ref _0, file: ref _1, param: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Assign_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                       name: std::borrow::Cow::from(r###"(inputs::Assign{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign) /*join*/"###),
                                                                                                                        afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                        {
                                                                                                                            let __cloned = __v.clone();
                                                                                                                            match < Assign>::from_ddvalue(__v) {
                                                                                                                                Assign{expr_id: ref _0, file: ref _1, lhs: ddlog_std::Option::Some{x: ddlog_std::Either::Left{l: _}}, rhs: _, op: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                _ => None
                                                                                                                            }.map(|x|(x,__cloned))
                                                                                                                        }
                                                                                                                        __f},
                                                                                                                        queryable: false
                                                                                                                    });
pub static __Arng_inputs_Assign_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                       name: std::borrow::Cow::from(r###"(inputs::Assign{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .lhs=(ddlog_std::Some{.x=(ddlog_std::Left{.l=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Either<internment::Intern<ast::Pattern>,ast::ExprId>)}: ddlog_std::Option<ddlog_std::Either<ast::IPattern,ast::ExprId>>), .rhs=(_: ddlog_std::Option<ast::ExprId>), .op=(_: ddlog_std::Option<ast::AssignOperand>)}: inputs::Assign) /*join*/"###),
                                                                                                                        afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                        {
                                                                                                                            let __cloned = __v.clone();
                                                                                                                            match < Assign>::from_ddvalue(__v) {
                                                                                                                                Assign{expr_id: _, file: ref _0, lhs: ddlog_std::Option::Some{x: ddlog_std::Either::Left{l: _}}, rhs: _, op: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                _ => None
                                                                                                                            }.map(|x|(x,__cloned))
                                                                                                                        }
                                                                                                                        __f},
                                                                                                                        queryable: false
                                                                                                                    });
pub static __Arng_inputs_BracketAccess_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                              name: std::borrow::Cow::from(r###"(inputs::BracketAccess{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .object=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::BracketAccess) /*join*/"###),
                                                                                                                               afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                               {
                                                                                                                                   let __cloned = __v.clone();
                                                                                                                                   match < BracketAccess>::from_ddvalue(__v) {
                                                                                                                                       BracketAccess{expr_id: _, file: ref _0, object: ddlog_std::Option::Some{x: _}, prop: ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                       _ => None
                                                                                                                                   }.map(|x|(x,__cloned))
                                                                                                                               }
                                                                                                                               __f},
                                                                                                                               queryable: false
                                                                                                                           });
pub static __Arng_inputs_Break_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Break{.stmt_id=(_: ast::StmtId), .file=(_0: ast::FileId), .label=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::Break) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Break>::from_ddvalue(__v) {
                                                                                                                               Break{stmt_id: _, file: ref _0, label: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_Call_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::Call{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .callee=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::Call) /*join*/"###),
                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                      {
                                                                                                                          let __cloned = __v.clone();
                                                                                                                          match < Call>::from_ddvalue(__v) {
                                                                                                                              Call{expr_id: _, file: ref _0, callee: ddlog_std::Option::Some{x: _}, args: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                              _ => None
                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                      }
                                                                                                                      __f},
                                                                                                                      queryable: false
                                                                                                                  });
pub static __Arng_inputs_Class_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Class{.id=(_0: ast::ClassId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Class) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Class>::from_ddvalue(__v) {
                                                                                                                               Class{id: ref _0, file: ref _1, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, parent: _, elements: _, scope: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_Class_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Class{.id=(_: ast::ClassId), .file=(_0: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_1: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .parent=(_: ddlog_std::Option<ast::ExprId>), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>), .scope=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Class) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Class>::from_ddvalue(__v) {
                                                                                                                               Class{id: _, file: ref _0, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: ref _1, span: _}}, parent: _, elements: _, scope: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_ClassExpr_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::ClassExpr{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .elements=(_: ddlog_std::Option<ddlog_std::Vec<ast::IClassElement>>)}: inputs::ClassExpr) /*semijoin*/"###),
                                                                                                                           fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                           {
                                                                                                                               match < ClassExpr>::from_ddvalue(__v) {
                                                                                                                                   ClassExpr{expr_id: ref _0, file: ref _1, elements: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           distinct: false
                                                                                                                       });
pub static __Arng_inputs_ConstDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::ConstDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::ConstDecl) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < ConstDecl>::from_ddvalue(__v) {
                                                                                                                                   ConstDecl{stmt_id: ref _0, file: ref _1, pattern: ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_Continue_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Continue{.stmt_id=(_: ast::StmtId), .file=(_0: ast::FileId), .label=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::Continue) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Continue>::from_ddvalue(__v) {
                                                                                                                                  Continue{stmt_id: _, file: ref _0, label: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_DotAccess_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::DotAccess{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .object=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .prop=(_: ddlog_std::Option<ast::Spanned<ast::Name>>)}: inputs::DotAccess) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < DotAccess>::from_ddvalue(__v) {
                                                                                                                                   DotAccess{expr_id: _, file: ref _0, object: ddlog_std::Option::Some{x: _}, prop: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_EveryScope_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::EveryScope{.scope=(_: ast::ScopeId), .file=_0}: inputs::EveryScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < EveryScope>::from_ddvalue(__v) {
                                                                                                                                    EveryScope{scope: _, file: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_Expression_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, file: ref _1, kind: _, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(ast::ExprNameRef{}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, file: ref _1, kind: types__ast::ExprKind::ExprNameRef{}, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(ast::ExprGrouping{.inner=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, file: ref _1, kind: types__ast::ExprKind::ExprGrouping{inner: ddlog_std::Option::Some{x: _}}, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_0: ast::ExprId), .file=(_1: ast::FileId), .kind=(ast::ExprSequence{.exprs=(_: ddlog_std::Vec<ast::ExprId>)}: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, file: ref _1, kind: types__ast::ExprKind::ExprSequence{exprs: _}, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_Expression_4 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_: ast::ExprId), .file=_0, .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: _, file: ref _0, kind: _, scope: _, span: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_Expression_5 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=_0, .file=_1, .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: ref _0, file: ref _1, kind: _, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_Expression_6 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::Expression{.id=(_: ast::ExprId), .file=_1, .kind=(_: ast::ExprKind), .scope=(_: ast::ScopeId), .span=_0}: inputs::Expression) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < Expression>::from_ddvalue(__v) {
                                                                                                                                    Expression{id: _, file: ref _1, kind: _, scope: _, span: ref _0} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_File_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::File{.id=(_0: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(_: config::Config)}: inputs::File) /*join*/"###),
                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                      {
                                                                                                                          let __cloned = __v.clone();
                                                                                                                          match < File>::from_ddvalue(__v) {
                                                                                                                              File{id: ref _0, kind: _, top_level_scope: _, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                              _ => None
                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                      }
                                                                                                                      __f},
                                                                                                                      queryable: false
                                                                                                                  });
pub static __Arng_inputs_File_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::File{.id=(_: ast::FileId), .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(_: config::Config)}: inputs::File) /*join*/"###),
                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                      {
                                                                                                                          let __cloned = __v.clone();
                                                                                                                          match < File>::from_ddvalue(__v) {
                                                                                                                              File{id: _, kind: _, top_level_scope: _, config: _} => Some((()).into_ddvalue()),
                                                                                                                              _ => None
                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                      }
                                                                                                                      __f},
                                                                                                                      queryable: false
                                                                                                                  });
pub static __Arng_inputs_File_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::File{.id=_0, .kind=(_: ast::FileKind), .top_level_scope=(_: ast::ScopeId), .config=(_: config::Config)}: inputs::File) /*join*/"###),
                                                                                                                      afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                      {
                                                                                                                          let __cloned = __v.clone();
                                                                                                                          match < File>::from_ddvalue(__v) {
                                                                                                                              File{id: ref _0, kind: _, top_level_scope: _, config: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                              _ => None
                                                                                                                          }.map(|x|(x,__cloned))
                                                                                                                      }
                                                                                                                      __f},
                                                                                                                      queryable: true
                                                                                                                  });
pub static __Arng_inputs_Function_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Function{.id=(_0: ast::FuncId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Function>::from_ddvalue(__v) {
                                                                                                                                  Function{id: ref _0, file: ref _1, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, scope: _, body: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_Function_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Function{.id=(_: ast::FuncId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_0: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Function>::from_ddvalue(__v) {
                                                                                                                                  Function{id: _, file: ref _1, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, scope: ref _0, body: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_Function_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                         name: std::borrow::Cow::from(r###"(inputs::Function{.id=(_0: ast::FuncId), .file=(_1: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .scope=(_: ast::ScopeId), .body=(_: ast::ScopeId), .exported=(_: bool)}: inputs::Function) /*join*/"###),
                                                                                                                          afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                          {
                                                                                                                              let __cloned = __v.clone();
                                                                                                                              match < Function>::from_ddvalue(__v) {
                                                                                                                                  Function{id: ref _0, file: ref _1, name: _, scope: _, body: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                  _ => None
                                                                                                                              }.map(|x|(x,__cloned))
                                                                                                                          }
                                                                                                                          __f},
                                                                                                                          queryable: false
                                                                                                                      });
pub static __Arng_inputs_FunctionArg_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                            name: std::borrow::Cow::from(r###"(inputs::FunctionArg{.parent_func=(_0: ast::FuncId), .file=(_1: ast::FileId), .pattern=(_: internment::Intern<ast::Pattern>), .implicit=(_: bool)}: inputs::FunctionArg) /*join*/"###),
                                                                                                                             afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                             {
                                                                                                                                 let __cloned = __v.clone();
                                                                                                                                 match < FunctionArg>::from_ddvalue(__v) {
                                                                                                                                     FunctionArg{parent_func: ref _0, file: ref _1, pattern: _, implicit: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                     _ => None
                                                                                                                                 }.map(|x|(x,__cloned))
                                                                                                                             }
                                                                                                                             __f},
                                                                                                                             queryable: false
                                                                                                                         });
pub static __Arng_inputs_ImplicitGlobal_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                               name: std::borrow::Cow::from(r###"(inputs::ImplicitGlobal{.id=(_: ast::GlobalId), .name=(_: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::ImplicitGlobal) /*join*/"###),
                                                                                                                                afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                {
                                                                                                                                    let __cloned = __v.clone();
                                                                                                                                    match < ImplicitGlobal>::from_ddvalue(__v) {
                                                                                                                                        ImplicitGlobal{id: _, name: _, privileges: _} => Some((()).into_ddvalue()),
                                                                                                                                        _ => None
                                                                                                                                    }.map(|x|(x,__cloned))
                                                                                                                                }
                                                                                                                                __f},
                                                                                                                                queryable: false
                                                                                                                            });
pub static __Arng_inputs_ImportDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::ImportDecl{.id=(_: ast::ImportId), .file=(_0: ast::FileId), .clause=(_: ast::ImportClause)}: inputs::ImportDecl) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < ImportDecl>::from_ddvalue(__v) {
                                                                                                                                    ImportDecl{id: _, file: ref _0, clause: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFunc_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InlineFunc{.expr_id=(_: ast::ExprId), .file=(_1: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_0: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InlineFunc>::from_ddvalue(__v) {
                                                                                                                                    InlineFunc{expr_id: _, file: ref _1, name: _, body: ddlog_std::Option::Some{x: ref _0}} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFunc_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InlineFunc{.expr_id=(_: ast::ExprId), .file=(_1: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_0: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InlineFunc>::from_ddvalue(__v) {
                                                                                                                                    InlineFunc{expr_id: _, file: ref _1, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, body: ddlog_std::Option::Some{x: ref _0}} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFunc_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InlineFunc{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .name=(_: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(ddlog_std::Some{.x=(_: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: inputs::InlineFunc) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InlineFunc>::from_ddvalue(__v) {
                                                                                                                                    InlineFunc{expr_id: ref _0, file: ref _1, name: _, body: ddlog_std::Option::Some{x: _}} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InlineFuncParam_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                                name: std::borrow::Cow::from(r###"(inputs::InlineFuncParam{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .param=(_: internment::Intern<ast::Pattern>)}: inputs::InlineFuncParam) /*join*/"###),
                                                                                                                                 afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                                 {
                                                                                                                                     let __cloned = __v.clone();
                                                                                                                                     match < InlineFuncParam>::from_ddvalue(__v) {
                                                                                                                                         InlineFuncParam{expr_id: ref _0, file: ref _1, param: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                         _ => None
                                                                                                                                     }.map(|x|(x,__cloned))
                                                                                                                                 }
                                                                                                                                 __f},
                                                                                                                                 queryable: false
                                                                                                                             });
pub static __Arng_inputs_InputScope_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=(_: ast::ScopeId), .child=(_0: ast::ScopeId), .file=(_1: ast::FileId)}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: _, child: ref _0, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InputScope_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=(_0: ast::ScopeId), .child=(_: ast::ScopeId), .file=(_1: ast::FileId)}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: ref _0, child: _, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InputScope_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=(_: ast::ScopeId), .child=(_: ast::ScopeId), .file=(_0: ast::FileId)}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: _, child: _, file: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_InputScope_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=(_: ast::ScopeId), .child=_0, .file=_1}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: _, child: ref _0, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_InputScope_4 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=(_: ast::ScopeId), .child=(_: ast::ScopeId), .file=_0}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: _, child: _, file: ref _0} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_InputScope_5 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::InputScope{.parent=_0, .child=(_: ast::ScopeId), .file=_1}: inputs::InputScope) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < InputScope>::from_ddvalue(__v) {
                                                                                                                                    InputScope{parent: ref _0, child: _, file: ref _1} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: true
                                                                                                                        });
pub static __Arng_inputs_Label_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Label{.stmt_id=(_: ast::StmtId), .file=(_0: ast::FileId), .name=(ddlog_std::Some{.x=(_: ast::Spanned<ast::Name>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>), .body_scope=(_: ast::ScopeId)}: inputs::Label) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Label>::from_ddvalue(__v) {
                                                                                                                               Label{stmt_id: _, file: ref _0, name: ddlog_std::Option::Some{x: _}, body: _, body_scope: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_Label_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                      name: std::borrow::Cow::from(r###"(inputs::Label{.stmt_id=(_: ast::StmtId), .file=(_0: ast::FileId), .name=(ddlog_std::Some{.x=(ast::Spanned{.data=(_: internment::Intern<string>), .span=(_: ast::Span)}: ast::Spanned<internment::Intern<string>>)}: ddlog_std::Option<ast::Spanned<ast::Name>>), .body=(_: ddlog_std::Option<ast::StmtId>), .body_scope=(_: ast::ScopeId)}: inputs::Label) /*join*/"###),
                                                                                                                       afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                       {
                                                                                                                           let __cloned = __v.clone();
                                                                                                                           match < Label>::from_ddvalue(__v) {
                                                                                                                               Label{stmt_id: _, file: ref _0, name: ddlog_std::Option::Some{x: types__ast::Spanned{data: _, span: _}}, body: _, body_scope: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                               _ => None
                                                                                                                           }.map(|x|(x,__cloned))
                                                                                                                       }
                                                                                                                       __f},
                                                                                                                       queryable: false
                                                                                                                   });
pub static __Arng_inputs_LetDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::LetDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::LetDecl) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < LetDecl>::from_ddvalue(__v) {
                                                                                                                                 LetDecl{stmt_id: ref _0, file: ref _1, pattern: ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_NameRef_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::NameRef{.expr_id=(_0: ast::ExprId), .file=(_1: ast::FileId), .value=(_: internment::Intern<string>)}: inputs::NameRef) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < NameRef>::from_ddvalue(__v) {
                                                                                                                                 NameRef{expr_id: ref _0, file: ref _1, value: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_NameRef_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::NameRef{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .value=(_: internment::Intern<string>)}: inputs::NameRef) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < NameRef>::from_ddvalue(__v) {
                                                                                                                                 NameRef{expr_id: _, file: ref _0, value: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_New_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"(inputs::New{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .object=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < New>::from_ddvalue(__v) {
                                                                                                                             New{expr_id: _, file: ref _0, object: ddlog_std::Option::Some{x: _}, args: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng_inputs_New_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Set{
                                                                                                                     name: std::borrow::Cow::from(r###"(inputs::New{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .object=(ddlog_std::Some{.x=(_1: ast::ExprId)}: ddlog_std::Option<ast::ExprId>), .args=(_: ddlog_std::Option<ddlog_std::Vec<ast::ExprId>>)}: inputs::New) /*antijoin*/"###),
                                                                                                                     fmfun: {fn __f(__v: DDValue) -> Option<DDValue>
                                                                                                                     {
                                                                                                                         match < New>::from_ddvalue(__v) {
                                                                                                                             New{expr_id: _, file: ref _0, object: ddlog_std::Option::Some{x: ref _1}, args: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     distinct: true
                                                                                                                 });
pub static __Arng_inputs_Statement_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::Statement{.id=(_0: ast::StmtId), .file=(_1: ast::FileId), .kind=(_: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < Statement>::from_ddvalue(__v) {
                                                                                                                                   Statement{id: ref _0, file: ref _1, kind: _, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_Statement_1 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::Statement{.id=(_0: ast::StmtId), .file=(_1: ast::FileId), .kind=(ast::StmtVarDecl{}: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < Statement>::from_ddvalue(__v) {
                                                                                                                                   Statement{id: ref _0, file: ref _1, kind: types__ast::StmtKind::StmtVarDecl{}, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: false
                                                                                                                       });
pub static __Arng_inputs_Statement_2 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::Statement{.id=_0, .file=_1, .kind=(_: ast::StmtKind), .scope=(_: ast::ScopeId), .span=(_: ast::Span)}: inputs::Statement) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < Statement>::from_ddvalue(__v) {
                                                                                                                                   Statement{id: ref _0, file: ref _1, kind: _, scope: _, span: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: true
                                                                                                                       });
pub static __Arng_inputs_Statement_3 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                          name: std::borrow::Cow::from(r###"(inputs::Statement{.id=(_: ast::StmtId), .file=_1, .kind=(_: ast::StmtKind), .scope=(_: ast::ScopeId), .span=_0}: inputs::Statement) /*join*/"###),
                                                                                                                           afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                           {
                                                                                                                               let __cloned = __v.clone();
                                                                                                                               match < Statement>::from_ddvalue(__v) {
                                                                                                                                   Statement{id: _, file: ref _1, kind: _, scope: _, span: ref _0} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                   _ => None
                                                                                                                               }.map(|x|(x,__cloned))
                                                                                                                           }
                                                                                                                           __f},
                                                                                                                           queryable: true
                                                                                                                       });
pub static __Arng_inputs_Try_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                    name: std::borrow::Cow::from(r###"(inputs::Try{.stmt_id=(_: ast::StmtId), .file=(_1: ast::FileId), .body=(_: ddlog_std::Option<ast::StmtId>), .handler=(ast::TryHandler{.error=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .body=(ddlog_std::Some{.x=(_0: ast::StmtId)}: ddlog_std::Option<ast::StmtId>)}: ast::TryHandler), .finalizer=(_: ddlog_std::Option<ast::StmtId>)}: inputs::Try) /*join*/"###),
                                                                                                                     afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                     {
                                                                                                                         let __cloned = __v.clone();
                                                                                                                         match < Try>::from_ddvalue(__v) {
                                                                                                                             Try{stmt_id: _, file: ref _1, body: _, handler: types__ast::TryHandler{error: ddlog_std::Option::Some{x: _}, body: ddlog_std::Option::Some{x: ref _0}}, finalizer: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                             _ => None
                                                                                                                         }.map(|x|(x,__cloned))
                                                                                                                     }
                                                                                                                     __f},
                                                                                                                     queryable: false
                                                                                                                 });
pub static __Arng_inputs_UnaryOp_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::UnaryOp{.expr_id=(_: ast::ExprId), .file=(_0: ast::FileId), .op=(ddlog_std::Some{.x=(ast::UnaryTypeof{}: ast::UnaryOperand)}: ddlog_std::Option<ast::UnaryOperand>), .expr=(ddlog_std::Some{.x=(_: ast::ExprId)}: ddlog_std::Option<ast::ExprId>)}: inputs::UnaryOp) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < UnaryOp>::from_ddvalue(__v) {
                                                                                                                                 UnaryOp{expr_id: _, file: ref _0, op: ddlog_std::Option::Some{x: types__ast::UnaryOperand::UnaryTypeof{}}, expr: ddlog_std::Option::Some{x: _}} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });
pub static __Arng_inputs_UserGlobal_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                           name: std::borrow::Cow::from(r###"(inputs::UserGlobal{.id=(_: ast::GlobalId), .file=(_0: ast::FileId), .name=(_: internment::Intern<string>), .privileges=(_: ast::GlobalPriv)}: inputs::UserGlobal) /*join*/"###),
                                                                                                                            afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                            {
                                                                                                                                let __cloned = __v.clone();
                                                                                                                                match < UserGlobal>::from_ddvalue(__v) {
                                                                                                                                    UserGlobal{id: _, file: ref _0, name: _, privileges: _} => Some(((*_0).clone()).into_ddvalue()),
                                                                                                                                    _ => None
                                                                                                                                }.map(|x|(x,__cloned))
                                                                                                                            }
                                                                                                                            __f},
                                                                                                                            queryable: false
                                                                                                                        });
pub static __Arng_inputs_VarDecl_0 : ::once_cell::sync::Lazy<program::Arrangement> = ::once_cell::sync::Lazy::new(|| program::Arrangement::Map{
                                                                                                                        name: std::borrow::Cow::from(r###"(inputs::VarDecl{.stmt_id=(_0: ast::StmtId), .file=(_1: ast::FileId), .pattern=(ddlog_std::Some{.x=(_: internment::Intern<ast::Pattern>)}: ddlog_std::Option<ast::IPattern>), .value=(_: ddlog_std::Option<ast::ExprId>), .exported=(_: bool)}: inputs::VarDecl) /*join*/"###),
                                                                                                                         afun: {fn __f(__v: DDValue) -> Option<(DDValue,DDValue)>
                                                                                                                         {
                                                                                                                             let __cloned = __v.clone();
                                                                                                                             match < VarDecl>::from_ddvalue(__v) {
                                                                                                                                 VarDecl{stmt_id: ref _0, file: ref _1, pattern: ddlog_std::Option::Some{x: _}, value: _, exported: _} => Some((ddlog_std::tuple2((*_0).clone(), (*_1).clone())).into_ddvalue()),
                                                                                                                                 _ => None
                                                                                                                             }.map(|x|(x,__cloned))
                                                                                                                         }
                                                                                                                         __f},
                                                                                                                         queryable: false
                                                                                                                     });