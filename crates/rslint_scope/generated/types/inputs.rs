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
pub struct Array {
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub elements: crate::ddlog_std::Vec<crate::ast::ArrayElement>
}
impl abomonation::Abomonation for Array{}
::differential_datalog::decl_struct_from_record!(Array["inputs::Array"]<>, ["inputs::Array"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]elements["elements"]: crate::ddlog_std::Vec<crate::ast::ArrayElement>});
::differential_datalog::decl_struct_into_record!(Array, ["inputs::Array"]<>, expr_id, file, elements);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Array, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, elements: crate::ddlog_std::Vec<crate::ast::ArrayElement>);
impl ::std::fmt::Display for Array {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Array{expr_id,file,elements} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub body: crate::ddlog_std::Option<crate::ddlog_std::tuple2<crate::ddlog_std::Either<crate::ast::ExprId, crate::ast::StmtId>, crate::ast::ScopeId>>
}
impl abomonation::Abomonation for Arrow{}
::differential_datalog::decl_struct_from_record!(Arrow["inputs::Arrow"]<>, ["inputs::Arrow"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]body["body"]: crate::ddlog_std::Option<crate::ddlog_std::tuple2<crate::ddlog_std::Either<crate::ast::ExprId, crate::ast::StmtId>, crate::ast::ScopeId>>});
::differential_datalog::decl_struct_into_record!(Arrow, ["inputs::Arrow"]<>, expr_id, file, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Arrow, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, body: crate::ddlog_std::Option<crate::ddlog_std::tuple2<crate::ddlog_std::Either<crate::ast::ExprId, crate::ast::StmtId>, crate::ast::ScopeId>>);
impl ::std::fmt::Display for Arrow {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Arrow{expr_id,file,body} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub param: crate::ast::IPattern
}
impl abomonation::Abomonation for ArrowParam{}
::differential_datalog::decl_struct_from_record!(ArrowParam["inputs::ArrowParam"]<>, ["inputs::ArrowParam"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]param["param"]: crate::ast::IPattern});
::differential_datalog::decl_struct_into_record!(ArrowParam, ["inputs::ArrowParam"]<>, expr_id, file, param);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ArrowParam, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, param: crate::ast::IPattern);
impl ::std::fmt::Display for ArrowParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ArrowParam{expr_id,file,param} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub lhs: crate::ddlog_std::Option<crate::ddlog_std::Either<crate::ast::IPattern, crate::ast::ExprId>>,
    pub rhs: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub op: crate::ddlog_std::Option<crate::ast::AssignOperand>
}
impl abomonation::Abomonation for Assign{}
::differential_datalog::decl_struct_from_record!(Assign["inputs::Assign"]<>, ["inputs::Assign"][5]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]lhs["lhs"]: crate::ddlog_std::Option<crate::ddlog_std::Either<crate::ast::IPattern, crate::ast::ExprId>>, [3]rhs["rhs"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]op["op"]: crate::ddlog_std::Option<crate::ast::AssignOperand>});
::differential_datalog::decl_struct_into_record!(Assign, ["inputs::Assign"]<>, expr_id, file, lhs, rhs, op);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Assign, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, lhs: crate::ddlog_std::Option<crate::ddlog_std::Either<crate::ast::IPattern, crate::ast::ExprId>>, rhs: crate::ddlog_std::Option<crate::ast::ExprId>, op: crate::ddlog_std::Option<crate::ast::AssignOperand>);
impl ::std::fmt::Display for Assign {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Assign{expr_id,file,lhs,rhs,op} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub value: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for Await{}
::differential_datalog::decl_struct_from_record!(Await["inputs::Await"]<>, ["inputs::Await"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Await, ["inputs::Await"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Await, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, value: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for Await {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Await{expr_id,file,value} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub op: crate::ddlog_std::Option<crate::ast::BinOperand>,
    pub lhs: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub rhs: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for BinOp{}
::differential_datalog::decl_struct_from_record!(BinOp["inputs::BinOp"]<>, ["inputs::BinOp"][5]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]op["op"]: crate::ddlog_std::Option<crate::ast::BinOperand>, [3]lhs["lhs"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]rhs["rhs"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(BinOp, ["inputs::BinOp"]<>, expr_id, file, op, lhs, rhs);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(BinOp, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, op: crate::ddlog_std::Option<crate::ast::BinOperand>, lhs: crate::ddlog_std::Option<crate::ast::ExprId>, rhs: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for BinOp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::BinOp{expr_id,file,op,lhs,rhs} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub object: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub prop: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for BracketAccess{}
::differential_datalog::decl_struct_from_record!(BracketAccess["inputs::BracketAccess"]<>, ["inputs::BracketAccess"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]object["object"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]prop["prop"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(BracketAccess, ["inputs::BracketAccess"]<>, expr_id, file, object, prop);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(BracketAccess, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, object: crate::ddlog_std::Option<crate::ast::ExprId>, prop: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for BracketAccess {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::BracketAccess{expr_id,file,object,prop} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub label: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
}
impl abomonation::Abomonation for Break{}
::differential_datalog::decl_struct_from_record!(Break["inputs::Break"]<>, ["inputs::Break"][3]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]label["label"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
::differential_datalog::decl_struct_into_record!(Break, ["inputs::Break"]<>, stmt_id, file, label);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Break, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, label: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>);
impl ::std::fmt::Display for Break {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Break{stmt_id,file,label} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub callee: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub args: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>
}
impl abomonation::Abomonation for Call{}
::differential_datalog::decl_struct_from_record!(Call["inputs::Call"]<>, ["inputs::Call"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]callee["callee"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]args["args"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>});
::differential_datalog::decl_struct_into_record!(Call, ["inputs::Call"]<>, expr_id, file, callee, args);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Call, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, callee: crate::ddlog_std::Option<crate::ast::ExprId>, args: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>);
impl ::std::fmt::Display for Call {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Call{expr_id,file,callee,args} => {
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
    pub id: crate::ast::ClassId,
    pub file: crate::ast::FileId,
    pub name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>,
    pub parent: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub elements: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IClassElement>>,
    pub scope: crate::ast::ScopeId,
    pub exported: bool
}
impl abomonation::Abomonation for Class{}
::differential_datalog::decl_struct_from_record!(Class["inputs::Class"]<>, ["inputs::Class"][7]{[0]id["id"]: crate::ast::ClassId, [1]file["file"]: crate::ast::FileId, [2]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, [3]parent["parent"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]elements["elements"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IClassElement>>, [5]scope["scope"]: crate::ast::ScopeId, [6]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(Class, ["inputs::Class"]<>, id, file, name, parent, elements, scope, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Class, <>, id: crate::ast::ClassId, file: crate::ast::FileId, name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, parent: crate::ddlog_std::Option<crate::ast::ExprId>, elements: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IClassElement>>, scope: crate::ast::ScopeId, exported: bool);
impl ::std::fmt::Display for Class {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Class{id,file,name,parent,elements,scope,exported} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub elements: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IClassElement>>
}
impl abomonation::Abomonation for ClassExpr{}
::differential_datalog::decl_struct_from_record!(ClassExpr["inputs::ClassExpr"]<>, ["inputs::ClassExpr"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]elements["elements"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IClassElement>>});
::differential_datalog::decl_struct_into_record!(ClassExpr, ["inputs::ClassExpr"]<>, expr_id, file, elements);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ClassExpr, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, elements: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IClassElement>>);
impl ::std::fmt::Display for ClassExpr {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ClassExpr{expr_id,file,elements} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub pattern: crate::ddlog_std::Option<crate::ast::IPattern>,
    pub value: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for ConstDecl{}
::differential_datalog::decl_struct_from_record!(ConstDecl["inputs::ConstDecl"]<>, ["inputs::ConstDecl"][5]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]pattern["pattern"]: crate::ddlog_std::Option<crate::ast::IPattern>, [3]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(ConstDecl, ["inputs::ConstDecl"]<>, stmt_id, file, pattern, value, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ConstDecl, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, pattern: crate::ddlog_std::Option<crate::ast::IPattern>, value: crate::ddlog_std::Option<crate::ast::ExprId>, exported: bool);
impl ::std::fmt::Display for ConstDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ConstDecl{stmt_id,file,pattern,value,exported} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub label: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
}
impl abomonation::Abomonation for Continue{}
::differential_datalog::decl_struct_from_record!(Continue["inputs::Continue"]<>, ["inputs::Continue"][3]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]label["label"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
::differential_datalog::decl_struct_into_record!(Continue, ["inputs::Continue"]<>, stmt_id, file, label);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Continue, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, label: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>);
impl ::std::fmt::Display for Continue {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Continue{stmt_id,file,label} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>,
    pub cond: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for DoWhile{}
::differential_datalog::decl_struct_from_record!(DoWhile["inputs::DoWhile"]<>, ["inputs::DoWhile"][4]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>, [3]cond["cond"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(DoWhile, ["inputs::DoWhile"]<>, stmt_id, file, body, cond);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(DoWhile, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, body: crate::ddlog_std::Option<crate::ast::StmtId>, cond: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for DoWhile {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::DoWhile{stmt_id,file,body,cond} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub object: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub prop: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
}
impl abomonation::Abomonation for DotAccess{}
::differential_datalog::decl_struct_from_record!(DotAccess["inputs::DotAccess"]<>, ["inputs::DotAccess"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]object["object"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]prop["prop"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
::differential_datalog::decl_struct_into_record!(DotAccess, ["inputs::DotAccess"]<>, expr_id, file, object, prop);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(DotAccess, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, object: crate::ddlog_std::Option<crate::ast::ExprId>, prop: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>);
impl ::std::fmt::Display for DotAccess {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::DotAccess{expr_id,file,object,prop} => {
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
    pub scope: crate::ast::ScopeId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for EveryScope{}
::differential_datalog::decl_struct_from_record!(EveryScope["inputs::EveryScope"]<>, ["inputs::EveryScope"][2]{[0]scope["scope"]: crate::ast::ScopeId, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(EveryScope, ["inputs::EveryScope"]<>, scope, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(EveryScope, <>, scope: crate::ast::ScopeId, file: crate::ast::FileId);
impl ::std::fmt::Display for EveryScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::EveryScope{scope,file} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub value: ::differential_datalog::int::Int
}
impl abomonation::Abomonation for ExprBigInt{}
::differential_datalog::decl_struct_from_record!(ExprBigInt["inputs::ExprBigInt"]<>, ["inputs::ExprBigInt"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: ::differential_datalog::int::Int});
::differential_datalog::decl_struct_into_record!(ExprBigInt, ["inputs::ExprBigInt"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprBigInt, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, value: ::differential_datalog::int::Int);
impl ::std::fmt::Display for ExprBigInt {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ExprBigInt{expr_id,file,value} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub value: bool
}
impl abomonation::Abomonation for ExprBool{}
::differential_datalog::decl_struct_from_record!(ExprBool["inputs::ExprBool"]<>, ["inputs::ExprBool"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: bool});
::differential_datalog::decl_struct_into_record!(ExprBool, ["inputs::ExprBool"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprBool, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, value: bool);
impl ::std::fmt::Display for ExprBool {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ExprBool{expr_id,file,value} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub value: ::ordered_float::OrderedFloat<f64>
}
impl abomonation::Abomonation for ExprNumber{}
::differential_datalog::decl_struct_from_record!(ExprNumber["inputs::ExprNumber"]<>, ["inputs::ExprNumber"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: ::ordered_float::OrderedFloat<f64>});
::differential_datalog::decl_struct_into_record!(ExprNumber, ["inputs::ExprNumber"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprNumber, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, value: ::ordered_float::OrderedFloat<f64>);
impl ::std::fmt::Display for ExprNumber {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ExprNumber{expr_id,file,value} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub value: crate::internment::istring
}
impl abomonation::Abomonation for ExprString{}
::differential_datalog::decl_struct_from_record!(ExprString["inputs::ExprString"]<>, ["inputs::ExprString"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: crate::internment::istring});
::differential_datalog::decl_struct_into_record!(ExprString, ["inputs::ExprString"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprString, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, value: crate::internment::istring);
impl ::std::fmt::Display for ExprString {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ExprString{expr_id,file,value} => {
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
    pub id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub kind: crate::ast::ExprKind,
    pub scope: crate::ast::ScopeId,
    pub span: crate::ast::Span
}
impl abomonation::Abomonation for Expression{}
::differential_datalog::decl_struct_from_record!(Expression["inputs::Expression"]<>, ["inputs::Expression"][5]{[0]id["id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]kind["kind"]: crate::ast::ExprKind, [3]scope["scope"]: crate::ast::ScopeId, [4]span["span"]: crate::ast::Span});
::differential_datalog::decl_struct_into_record!(Expression, ["inputs::Expression"]<>, id, file, kind, scope, span);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Expression, <>, id: crate::ast::ExprId, file: crate::ast::FileId, kind: crate::ast::ExprKind, scope: crate::ast::ScopeId, span: crate::ast::Span);
impl ::std::fmt::Display for Expression {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Expression{id,file,kind,scope,span} => {
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
    pub id: crate::ast::FileId,
    pub kind: crate::ast::FileKind,
    pub top_level_scope: crate::ast::ScopeId,
    pub config: crate::config::Config
}
impl abomonation::Abomonation for File{}
::differential_datalog::decl_struct_from_record!(File["inputs::File"]<>, ["inputs::File"][4]{[0]id["id"]: crate::ast::FileId, [1]kind["kind"]: crate::ast::FileKind, [2]top_level_scope["top_level_scope"]: crate::ast::ScopeId, [3]config["config"]: crate::config::Config});
::differential_datalog::decl_struct_into_record!(File, ["inputs::File"]<>, id, kind, top_level_scope, config);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(File, <>, id: crate::ast::FileId, kind: crate::ast::FileKind, top_level_scope: crate::ast::ScopeId, config: crate::config::Config);
impl ::std::fmt::Display for File {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::File{id,kind,top_level_scope,config} => {
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
    pub file: crate::ast::FileId,
    pub export: crate::ast::ExportKind,
    pub scope: crate::ast::ScopeId
}
impl abomonation::Abomonation for FileExport{}
::differential_datalog::decl_struct_from_record!(FileExport["inputs::FileExport"]<>, ["inputs::FileExport"][3]{[0]file["file"]: crate::ast::FileId, [1]export["export"]: crate::ast::ExportKind, [2]scope["scope"]: crate::ast::ScopeId});
::differential_datalog::decl_struct_into_record!(FileExport, ["inputs::FileExport"]<>, file, export, scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FileExport, <>, file: crate::ast::FileId, export: crate::ast::ExportKind, scope: crate::ast::ScopeId);
impl ::std::fmt::Display for FileExport {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::FileExport{file,export,scope} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub init: crate::ddlog_std::Option<crate::ast::ForInit>,
    pub test: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub update: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for For{}
::differential_datalog::decl_struct_from_record!(For["inputs::For"]<>, ["inputs::For"][6]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]init["init"]: crate::ddlog_std::Option<crate::ast::ForInit>, [3]test["test"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]update["update"]: crate::ddlog_std::Option<crate::ast::ExprId>, [5]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(For, ["inputs::For"]<>, stmt_id, file, init, test, update, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(For, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, init: crate::ddlog_std::Option<crate::ast::ForInit>, test: crate::ddlog_std::Option<crate::ast::ExprId>, update: crate::ddlog_std::Option<crate::ast::ExprId>, body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for For {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::For{stmt_id,file,init,test,update,body} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub elem: crate::ddlog_std::Option<crate::ast::ForInit>,
    pub collection: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for ForIn{}
::differential_datalog::decl_struct_from_record!(ForIn["inputs::ForIn"]<>, ["inputs::ForIn"][5]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]elem["elem"]: crate::ddlog_std::Option<crate::ast::ForInit>, [3]collection["collection"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(ForIn, ["inputs::ForIn"]<>, stmt_id, file, elem, collection, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ForIn, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, elem: crate::ddlog_std::Option<crate::ast::ForInit>, collection: crate::ddlog_std::Option<crate::ast::ExprId>, body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for ForIn {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ForIn{stmt_id,file,elem,collection,body} => {
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
pub struct Function {
    pub id: crate::ast::FuncId,
    pub file: crate::ast::FileId,
    pub name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>,
    pub scope: crate::ast::ScopeId,
    pub body: crate::ast::ScopeId,
    pub exported: bool
}
impl abomonation::Abomonation for Function{}
::differential_datalog::decl_struct_from_record!(Function["inputs::Function"]<>, ["inputs::Function"][6]{[0]id["id"]: crate::ast::FuncId, [1]file["file"]: crate::ast::FileId, [2]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, [3]scope["scope"]: crate::ast::ScopeId, [4]body["body"]: crate::ast::ScopeId, [5]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(Function, ["inputs::Function"]<>, id, file, name, scope, body, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Function, <>, id: crate::ast::FuncId, file: crate::ast::FileId, name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, scope: crate::ast::ScopeId, body: crate::ast::ScopeId, exported: bool);
impl ::std::fmt::Display for Function {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Function{id,file,name,scope,body,exported} => {
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
    pub parent_func: crate::ast::FuncId,
    pub file: crate::ast::FileId,
    pub pattern: crate::ast::IPattern,
    pub implicit: bool
}
impl abomonation::Abomonation for FunctionArg{}
::differential_datalog::decl_struct_from_record!(FunctionArg["inputs::FunctionArg"]<>, ["inputs::FunctionArg"][4]{[0]parent_func["parent_func"]: crate::ast::FuncId, [1]file["file"]: crate::ast::FileId, [2]pattern["pattern"]: crate::ast::IPattern, [3]implicit["implicit"]: bool});
::differential_datalog::decl_struct_into_record!(FunctionArg, ["inputs::FunctionArg"]<>, parent_func, file, pattern, implicit);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FunctionArg, <>, parent_func: crate::ast::FuncId, file: crate::ast::FileId, pattern: crate::ast::IPattern, implicit: bool);
impl ::std::fmt::Display for FunctionArg {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::FunctionArg{parent_func,file,pattern,implicit} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub cond: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub if_body: crate::ddlog_std::Option<crate::ast::StmtId>,
    pub else_body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for If{}
::differential_datalog::decl_struct_from_record!(If["inputs::If"]<>, ["inputs::If"][5]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]cond["cond"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]if_body["if_body"]: crate::ddlog_std::Option<crate::ast::StmtId>, [4]else_body["else_body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(If, ["inputs::If"]<>, stmt_id, file, cond, if_body, else_body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(If, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, cond: crate::ddlog_std::Option<crate::ast::ExprId>, if_body: crate::ddlog_std::Option<crate::ast::StmtId>, else_body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for If {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::If{stmt_id,file,cond,if_body,else_body} => {
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
    pub id: crate::ast::GlobalId,
    pub file: crate::ast::FileId,
    pub name: crate::ast::Name,
    pub privileges: crate::ast::GlobalPriv
}
impl abomonation::Abomonation for ImplicitGlobal{}
::differential_datalog::decl_struct_from_record!(ImplicitGlobal["inputs::ImplicitGlobal"]<>, ["inputs::ImplicitGlobal"][4]{[0]id["id"]: crate::ast::GlobalId, [1]file["file"]: crate::ast::FileId, [2]name["name"]: crate::ast::Name, [3]privileges["privileges"]: crate::ast::GlobalPriv});
::differential_datalog::decl_struct_into_record!(ImplicitGlobal, ["inputs::ImplicitGlobal"]<>, id, file, name, privileges);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ImplicitGlobal, <>, id: crate::ast::GlobalId, file: crate::ast::FileId, name: crate::ast::Name, privileges: crate::ast::GlobalPriv);
impl ::std::fmt::Display for ImplicitGlobal {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ImplicitGlobal{id,file,name,privileges} => {
                __formatter.write_str("inputs::ImplicitGlobal{")?;
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
impl ::std::fmt::Debug for ImplicitGlobal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ImportDecl {
    pub id: crate::ast::ImportId,
    pub file: crate::ast::FileId,
    pub clause: crate::ast::ImportClause
}
impl abomonation::Abomonation for ImportDecl{}
::differential_datalog::decl_struct_from_record!(ImportDecl["inputs::ImportDecl"]<>, ["inputs::ImportDecl"][3]{[0]id["id"]: crate::ast::ImportId, [1]file["file"]: crate::ast::FileId, [2]clause["clause"]: crate::ast::ImportClause});
::differential_datalog::decl_struct_into_record!(ImportDecl, ["inputs::ImportDecl"]<>, id, file, clause);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ImportDecl, <>, id: crate::ast::ImportId, file: crate::ast::FileId, clause: crate::ast::ImportClause);
impl ::std::fmt::Display for ImportDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::ImportDecl{id,file,clause} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for InlineFunc{}
::differential_datalog::decl_struct_from_record!(InlineFunc["inputs::InlineFunc"]<>, ["inputs::InlineFunc"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, [3]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(InlineFunc, ["inputs::InlineFunc"]<>, expr_id, file, name, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(InlineFunc, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for InlineFunc {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::InlineFunc{expr_id,file,name,body} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub param: crate::ast::IPattern
}
impl abomonation::Abomonation for InlineFuncParam{}
::differential_datalog::decl_struct_from_record!(InlineFuncParam["inputs::InlineFuncParam"]<>, ["inputs::InlineFuncParam"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]param["param"]: crate::ast::IPattern});
::differential_datalog::decl_struct_into_record!(InlineFuncParam, ["inputs::InlineFuncParam"]<>, expr_id, file, param);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(InlineFuncParam, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, param: crate::ast::IPattern);
impl ::std::fmt::Display for InlineFuncParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::InlineFuncParam{expr_id,file,param} => {
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
    pub parent: crate::ast::ScopeId,
    pub child: crate::ast::ScopeId,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for InputScope{}
::differential_datalog::decl_struct_from_record!(InputScope["inputs::InputScope"]<>, ["inputs::InputScope"][3]{[0]parent["parent"]: crate::ast::ScopeId, [1]child["child"]: crate::ast::ScopeId, [2]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(InputScope, ["inputs::InputScope"]<>, parent, child, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(InputScope, <>, parent: crate::ast::ScopeId, child: crate::ast::ScopeId, file: crate::ast::FileId);
impl ::std::fmt::Display for InputScope {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::InputScope{parent,child,file} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>,
    pub body_scope: crate::ast::ScopeId
}
impl abomonation::Abomonation for Label{}
::differential_datalog::decl_struct_from_record!(Label["inputs::Label"]<>, ["inputs::Label"][5]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, [3]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>, [4]body_scope["body_scope"]: crate::ast::ScopeId});
::differential_datalog::decl_struct_into_record!(Label, ["inputs::Label"]<>, stmt_id, file, name, body, body_scope);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Label, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, body: crate::ddlog_std::Option<crate::ast::StmtId>, body_scope: crate::ast::ScopeId);
impl ::std::fmt::Display for Label {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Label{stmt_id,file,name,body,body_scope} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub pattern: crate::ddlog_std::Option<crate::ast::IPattern>,
    pub value: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for LetDecl{}
::differential_datalog::decl_struct_from_record!(LetDecl["inputs::LetDecl"]<>, ["inputs::LetDecl"][5]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]pattern["pattern"]: crate::ddlog_std::Option<crate::ast::IPattern>, [3]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(LetDecl, ["inputs::LetDecl"]<>, stmt_id, file, pattern, value, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(LetDecl, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, pattern: crate::ddlog_std::Option<crate::ast::IPattern>, value: crate::ddlog_std::Option<crate::ast::ExprId>, exported: bool);
impl ::std::fmt::Display for LetDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::LetDecl{stmt_id,file,pattern,value,exported} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub value: crate::ast::Name
}
impl abomonation::Abomonation for NameRef{}
::differential_datalog::decl_struct_from_record!(NameRef["inputs::NameRef"]<>, ["inputs::NameRef"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: crate::ast::Name});
::differential_datalog::decl_struct_into_record!(NameRef, ["inputs::NameRef"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NameRef, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, value: crate::ast::Name);
impl ::std::fmt::Display for NameRef {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::NameRef{expr_id,file,value} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub object: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub args: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>
}
impl abomonation::Abomonation for New{}
::differential_datalog::decl_struct_from_record!(New["inputs::New"]<>, ["inputs::New"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]object["object"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]args["args"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>});
::differential_datalog::decl_struct_into_record!(New, ["inputs::New"]<>, expr_id, file, object, args);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(New, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, object: crate::ddlog_std::Option<crate::ast::ExprId>, args: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>);
impl ::std::fmt::Display for New {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::New{expr_id,file,object,args} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub key: crate::ddlog_std::Option<crate::ast::PropertyKey>,
    pub val: crate::ddlog_std::Option<crate::ast::PropertyVal>
}
impl abomonation::Abomonation for Property{}
::differential_datalog::decl_struct_from_record!(Property["inputs::Property"]<>, ["inputs::Property"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]key["key"]: crate::ddlog_std::Option<crate::ast::PropertyKey>, [3]val["val"]: crate::ddlog_std::Option<crate::ast::PropertyVal>});
::differential_datalog::decl_struct_into_record!(Property, ["inputs::Property"]<>, expr_id, file, key, val);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Property, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, key: crate::ddlog_std::Option<crate::ast::PropertyKey>, val: crate::ddlog_std::Option<crate::ast::PropertyVal>);
impl ::std::fmt::Display for Property {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Property{expr_id,file,key,val} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub value: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for Return{}
::differential_datalog::decl_struct_from_record!(Return["inputs::Return"]<>, ["inputs::Return"][3]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Return, ["inputs::Return"]<>, stmt_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Return, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, value: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for Return {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Return{stmt_id,file,value} => {
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
    pub id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub kind: crate::ast::StmtKind,
    pub scope: crate::ast::ScopeId,
    pub span: crate::ast::Span
}
impl abomonation::Abomonation for Statement{}
::differential_datalog::decl_struct_from_record!(Statement["inputs::Statement"]<>, ["inputs::Statement"][5]{[0]id["id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]kind["kind"]: crate::ast::StmtKind, [3]scope["scope"]: crate::ast::ScopeId, [4]span["span"]: crate::ast::Span});
::differential_datalog::decl_struct_into_record!(Statement, ["inputs::Statement"]<>, id, file, kind, scope, span);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Statement, <>, id: crate::ast::StmtId, file: crate::ast::FileId, kind: crate::ast::StmtKind, scope: crate::ast::ScopeId, span: crate::ast::Span);
impl ::std::fmt::Display for Statement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Statement{id,file,kind,scope,span} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub test: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for Switch{}
::differential_datalog::decl_struct_from_record!(Switch["inputs::Switch"]<>, ["inputs::Switch"][3]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]test["test"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Switch, ["inputs::Switch"]<>, stmt_id, file, test);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Switch, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, test: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for Switch {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Switch{stmt_id,file,test} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub case: crate::ast::SwitchClause,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for SwitchCase{}
::differential_datalog::decl_struct_from_record!(SwitchCase["inputs::SwitchCase"]<>, ["inputs::SwitchCase"][4]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]case["case"]: crate::ast::SwitchClause, [3]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(SwitchCase, ["inputs::SwitchCase"]<>, stmt_id, file, case, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(SwitchCase, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, case: crate::ast::SwitchClause, body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for SwitchCase {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::SwitchCase{stmt_id,file,case,body} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub tag: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub elements: crate::ddlog_std::Vec<crate::ast::ExprId>
}
impl abomonation::Abomonation for Template{}
::differential_datalog::decl_struct_from_record!(Template["inputs::Template"]<>, ["inputs::Template"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]tag["tag"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]elements["elements"]: crate::ddlog_std::Vec<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Template, ["inputs::Template"]<>, expr_id, file, tag, elements);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Template, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, tag: crate::ddlog_std::Option<crate::ast::ExprId>, elements: crate::ddlog_std::Vec<crate::ast::ExprId>);
impl ::std::fmt::Display for Template {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Template{expr_id,file,tag,elements} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub test: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub true_val: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub false_val: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for Ternary{}
::differential_datalog::decl_struct_from_record!(Ternary["inputs::Ternary"]<>, ["inputs::Ternary"][5]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]test["test"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]true_val["true_val"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]false_val["false_val"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Ternary, ["inputs::Ternary"]<>, expr_id, file, test, true_val, false_val);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Ternary, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, test: crate::ddlog_std::Option<crate::ast::ExprId>, true_val: crate::ddlog_std::Option<crate::ast::ExprId>, false_val: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for Ternary {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Ternary{expr_id,file,test,true_val,false_val} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub exception: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for Throw{}
::differential_datalog::decl_struct_from_record!(Throw["inputs::Throw"]<>, ["inputs::Throw"][3]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]exception["exception"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Throw, ["inputs::Throw"]<>, stmt_id, file, exception);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Throw, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, exception: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for Throw {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Throw{stmt_id,file,exception} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>,
    pub handler: crate::ast::TryHandler,
    pub finalizer: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for Try{}
::differential_datalog::decl_struct_from_record!(Try["inputs::Try"]<>, ["inputs::Try"][5]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>, [3]handler["handler"]: crate::ast::TryHandler, [4]finalizer["finalizer"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(Try, ["inputs::Try"]<>, stmt_id, file, body, handler, finalizer);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Try, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, body: crate::ddlog_std::Option<crate::ast::StmtId>, handler: crate::ast::TryHandler, finalizer: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for Try {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Try{stmt_id,file,body,handler,finalizer} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub op: crate::ddlog_std::Option<crate::ast::UnaryOperand>,
    pub expr: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for UnaryOp{}
::differential_datalog::decl_struct_from_record!(UnaryOp["inputs::UnaryOp"]<>, ["inputs::UnaryOp"][4]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]op["op"]: crate::ddlog_std::Option<crate::ast::UnaryOperand>, [3]expr["expr"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(UnaryOp, ["inputs::UnaryOp"]<>, expr_id, file, op, expr);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(UnaryOp, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, op: crate::ddlog_std::Option<crate::ast::UnaryOperand>, expr: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for UnaryOp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::UnaryOp{expr_id,file,op,expr} => {
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
pub struct VarDecl {
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub pattern: crate::ddlog_std::Option<crate::ast::IPattern>,
    pub value: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub exported: bool
}
impl abomonation::Abomonation for VarDecl{}
::differential_datalog::decl_struct_from_record!(VarDecl["inputs::VarDecl"]<>, ["inputs::VarDecl"][5]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]pattern["pattern"]: crate::ddlog_std::Option<crate::ast::IPattern>, [3]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>, [4]exported["exported"]: bool});
::differential_datalog::decl_struct_into_record!(VarDecl, ["inputs::VarDecl"]<>, stmt_id, file, pattern, value, exported);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(VarDecl, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, pattern: crate::ddlog_std::Option<crate::ast::IPattern>, value: crate::ddlog_std::Option<crate::ast::ExprId>, exported: bool);
impl ::std::fmt::Display for VarDecl {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::VarDecl{stmt_id,file,pattern,value,exported} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub cond: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for While{}
::differential_datalog::decl_struct_from_record!(While["inputs::While"]<>, ["inputs::While"][4]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]cond["cond"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(While, ["inputs::While"]<>, stmt_id, file, cond, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(While, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, cond: crate::ddlog_std::Option<crate::ast::ExprId>, body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for While {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::While{stmt_id,file,cond,body} => {
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
    pub stmt_id: crate::ast::StmtId,
    pub file: crate::ast::FileId,
    pub cond: crate::ddlog_std::Option<crate::ast::ExprId>,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for With{}
::differential_datalog::decl_struct_from_record!(With["inputs::With"]<>, ["inputs::With"][4]{[0]stmt_id["stmt_id"]: crate::ast::StmtId, [1]file["file"]: crate::ast::FileId, [2]cond["cond"]: crate::ddlog_std::Option<crate::ast::ExprId>, [3]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(With, ["inputs::With"]<>, stmt_id, file, cond, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(With, <>, stmt_id: crate::ast::StmtId, file: crate::ast::FileId, cond: crate::ddlog_std::Option<crate::ast::ExprId>, body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for With {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::With{stmt_id,file,cond,body} => {
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
    pub expr_id: crate::ast::ExprId,
    pub file: crate::ast::FileId,
    pub value: crate::ddlog_std::Option<crate::ast::ExprId>
}
impl abomonation::Abomonation for Yield{}
::differential_datalog::decl_struct_from_record!(Yield["inputs::Yield"]<>, ["inputs::Yield"][3]{[0]expr_id["expr_id"]: crate::ast::ExprId, [1]file["file"]: crate::ast::FileId, [2]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_struct_into_record!(Yield, ["inputs::Yield"]<>, expr_id, file, value);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Yield, <>, expr_id: crate::ast::ExprId, file: crate::ast::FileId, value: crate::ddlog_std::Option<crate::ast::ExprId>);
impl ::std::fmt::Display for Yield {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::inputs::Yield{expr_id,file,value} => {
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