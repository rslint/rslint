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

use super::internment::Intern;
use rslint_parser::{
    ast::{AssignOp as AstAssignOp, BinOp as AstBinOp, UnaryOp as AstUnaryOp},
    TextRange,
};
use std::{
    cell::Cell,
    ops::{Add, AddAssign, Range},
};

impl From<&str> for Intern<String> {
    fn from(string: &str) -> Self {
        Self::new(string.to_owned())
    }
}

impl From<String> for Intern<String> {
    fn from(string: String) -> Self {
        Self::new(string)
    }
}

impl<T> Spanned<T> {
    /// Create a new `Spanned`
    pub fn new<S>(data: T, span: S) -> Self
    where
        S: Into<Span>,
    {
        Self {
            data,
            span: span.into(),
        }
    }
}

impl Span {
    /// Create a new `Span`
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

/// Implement the `Span` trait for ddlog `Span`s
impl rslint_errors::Span for Span {
    fn as_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

/// Allow converting a `TextRange` into a ddlog `Span`
impl From<TextRange> for Span {
    fn from(range: TextRange) -> Self {
        Self {
            start: range.start().into(),
            end: range.end().into(),
        }
    }
}

impl From<Range<u32>> for Span {
    fn from(range: Range<u32>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self {
            start: range.start as u32,
            end: range.end as u32,
        }
    }
}

impl Copy for Span {}
impl Copy for AnyId {}

impl FileId {
    /// Creates a new file id from the given value
    pub const fn new(id: u32) -> Self {
        Self { id }
    }
}

impl Copy for FileId {}

macro_rules! impl_id_traits {
    ($($ty:ty),* $(,)?) => {
        /// A convenience trait to allow easily incrementing ids during ast->ddlog translation
        pub trait Increment {
            type Inner;

            /// Increments the id by one, returning the value *before* it was incremented
            fn inc(&self) -> Self::Inner;
        }

        $(
            impl $ty {
                /// Creates a new id from the given value
                pub const fn new(id: u32, file: FileId) -> Self {
                    Self { id, file }
                }
            }

            impl Increment for Cell<$ty> {
                type Inner = $ty;

                fn inc(&self) -> Self::Inner {
                    let old = self.get();
                    self.set(old + 1);
                    old
                }
            }

            impl Add for $ty {
                type Output = Self;

                fn add(self, other: Self) -> Self {
                    debug_assert_eq!(self.file, other.file);

                    Self {
                        id: self.id + other.id,
                        file: self.file,
                    }
                }
            }

            impl Add<u32> for $ty {
                type Output = Self;

                fn add(self, other: u32) -> Self {
                    Self {
                        id: self.id + other,
                        file: self.file,
                    }
                }
            }

            impl AddAssign for $ty {
                fn add_assign(&mut self, other: Self) {
                    self.id += other.id;
                }
            }

            impl AddAssign<u32> for $ty {
                fn add_assign(&mut self, other: u32) {
                    self.id += other;
                }
            }

            // They're all small types and so can be trivially copied
            impl Copy for $ty {}
        )*
    };
}

// Implement basic traits for id type-safe wrappers
impl_id_traits! {
    ScopeId,
    GlobalId,
    ImportId,
    ClassId,
    FuncId,
    StmtId,
    ExprId,
}

impl From<AstUnaryOp> for UnaryOperand {
    fn from(op: AstUnaryOp) -> Self {
        match op {
            AstUnaryOp::Increment => Self::UnaryIncrement,
            AstUnaryOp::Decrement => Self::UnaryDecrement,
            AstUnaryOp::Delete => Self::UnaryDelete,
            AstUnaryOp::Void => Self::UnaryVoid,
            AstUnaryOp::Typeof => Self::UnaryTypeof,
            AstUnaryOp::Plus => Self::UnaryPlus,
            AstUnaryOp::Minus => Self::UnaryMinus,
            AstUnaryOp::BitwiseNot => Self::UnaryBitwiseNot,
            AstUnaryOp::LogicalNot => Self::UnaryLogicalNot,
            AstUnaryOp::Await => Self::UnaryAwait,
        }
    }
}

impl From<AstBinOp> for BinOperand {
    fn from(op: AstBinOp) -> Self {
        match op {
            AstBinOp::LessThan => Self::BinLessThan,
            AstBinOp::GreaterThan => Self::BinGreaterThan,
            AstBinOp::LessThanOrEqual => Self::BinLessThanOrEqual,
            AstBinOp::GreaterThanOrEqual => Self::BinGreaterThanOrEqual,
            AstBinOp::Equality => Self::BinEquality,
            AstBinOp::StrictEquality => Self::BinStrictEquality,
            AstBinOp::Inequality => Self::BinInequality,
            AstBinOp::StrictInequality => Self::BinStrictInequality,
            AstBinOp::Plus => Self::BinPlus,
            AstBinOp::Minus => Self::BinMinus,
            AstBinOp::Times => Self::BinTimes,
            AstBinOp::Divide => Self::BinDivide,
            AstBinOp::Remainder => Self::BinRemainder,
            AstBinOp::Exponent => Self::BinExponent,
            AstBinOp::LeftShift => Self::BinLeftShift,
            AstBinOp::RightShift => Self::BinRightShift,
            AstBinOp::UnsignedRightShift => Self::BinUnsignedRightShift,
            AstBinOp::BitwiseAnd => Self::BinBitwiseAnd,
            AstBinOp::BitwiseOr => Self::BinBitwiseOr,
            AstBinOp::BitwiseXor => Self::BinBitwiseXor,
            AstBinOp::NullishCoalescing => Self::BinNullishCoalescing,
            AstBinOp::LogicalOr => Self::BinLogicalOr,
            AstBinOp::LogicalAnd => Self::BinLogicalAnd,
            AstBinOp::In => Self::BinIn,
            AstBinOp::Instanceof => Self::BinInstanceof,
        }
    }
}

impl From<AstAssignOp> for AssignOperand {
    fn from(op: AstAssignOp) -> Self {
        match op {
            AstAssignOp::Assign => Self::OpAssign,
            AstAssignOp::AddAssign => Self::OpAddAssign,
            AstAssignOp::SubtractAssign => Self::OpSubtractAssign,
            AstAssignOp::TimesAssign => Self::OpTimesAssign,
            AstAssignOp::RemainderAssign => Self::OpRemainderAssign,
            AstAssignOp::ExponentAssign => Self::OpExponentAssign,
            AstAssignOp::LeftShiftAssign => Self::OpLeftShiftAssign,
            AstAssignOp::RightShiftAssign => Self::OpRightShiftAssign,
            AstAssignOp::UnsignedRightShiftAssign => Self::OpUnsignedRightShiftAssign,
            AstAssignOp::BitwiseAndAssign => Self::OpBitwiseAndAssign,
            AstAssignOp::BitwiseOrAssign => Self::OpBitwiseOrAssign,
            AstAssignOp::BitwiseXorAssign => Self::OpBitwiseXorAssign,
            AstAssignOp::LogicalAndAssign => Self::OpLogicalAndAssign,
            AstAssignOp::LogicalOrAssign => Self::OpLogicalOrAssign,
            AstAssignOp::NullishCoalescingAssign => Self::OpNullishCoalescingAssign,
        }
    }
}

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AnyId {
    AnyIdGlobal {
        global: crate::ast::GlobalId
    },
    AnyIdImport {
        import_: crate::ast::ImportId
    },
    AnyIdClass {
        class: crate::ast::ClassId
    },
    AnyIdFunc {
        func: crate::ast::FuncId
    },
    AnyIdStmt {
        stmt: crate::ast::StmtId
    },
    AnyIdExpr {
        expr: crate::ast::ExprId
    }
}
impl abomonation::Abomonation for AnyId{}
::differential_datalog::decl_enum_from_record!(AnyId["ast::AnyId"]<>, AnyIdGlobal["ast::AnyIdGlobal"][1]{[0]global["global"]: crate::ast::GlobalId}, AnyIdImport["ast::AnyIdImport"][1]{[0]import_["import_"]: crate::ast::ImportId}, AnyIdClass["ast::AnyIdClass"][1]{[0]class["class"]: crate::ast::ClassId}, AnyIdFunc["ast::AnyIdFunc"][1]{[0]func["func"]: crate::ast::FuncId}, AnyIdStmt["ast::AnyIdStmt"][1]{[0]stmt["stmt"]: crate::ast::StmtId}, AnyIdExpr["ast::AnyIdExpr"][1]{[0]expr["expr"]: crate::ast::ExprId});
::differential_datalog::decl_enum_into_record!(AnyId<>, AnyIdGlobal["ast::AnyIdGlobal"]{global}, AnyIdImport["ast::AnyIdImport"]{import_}, AnyIdClass["ast::AnyIdClass"]{class}, AnyIdFunc["ast::AnyIdFunc"]{func}, AnyIdStmt["ast::AnyIdStmt"]{stmt}, AnyIdExpr["ast::AnyIdExpr"]{expr});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(AnyId<>, AnyIdGlobal{global: crate::ast::GlobalId}, AnyIdImport{import_: crate::ast::ImportId}, AnyIdClass{class: crate::ast::ClassId}, AnyIdFunc{func: crate::ast::FuncId}, AnyIdStmt{stmt: crate::ast::StmtId}, AnyIdExpr{expr: crate::ast::ExprId});
impl ::std::fmt::Display for AnyId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::AnyId::AnyIdGlobal{global} => {
                __formatter.write_str("ast::AnyIdGlobal{")?;
                ::std::fmt::Debug::fmt(global, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::AnyId::AnyIdImport{import_} => {
                __formatter.write_str("ast::AnyIdImport{")?;
                ::std::fmt::Debug::fmt(import_, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::AnyId::AnyIdClass{class} => {
                __formatter.write_str("ast::AnyIdClass{")?;
                ::std::fmt::Debug::fmt(class, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::AnyId::AnyIdFunc{func} => {
                __formatter.write_str("ast::AnyIdFunc{")?;
                ::std::fmt::Debug::fmt(func, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::AnyId::AnyIdStmt{stmt} => {
                __formatter.write_str("ast::AnyIdStmt{")?;
                ::std::fmt::Debug::fmt(stmt, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::AnyId::AnyIdExpr{expr} => {
                __formatter.write_str("ast::AnyIdExpr{")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for AnyId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for AnyId {
    fn default() -> Self {
        crate::ast::AnyId::AnyIdGlobal{global : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ArrayElement {
    ArrExpr {
        expr: crate::ast::ExprId
    },
    ArrSpread {
        spread: crate::ddlog_std::Option<crate::ast::ExprId>
    }
}
impl abomonation::Abomonation for ArrayElement{}
::differential_datalog::decl_enum_from_record!(ArrayElement["ast::ArrayElement"]<>, ArrExpr["ast::ArrExpr"][1]{[0]expr["expr"]: crate::ast::ExprId}, ArrSpread["ast::ArrSpread"][1]{[0]spread["spread"]: crate::ddlog_std::Option<crate::ast::ExprId>});
::differential_datalog::decl_enum_into_record!(ArrayElement<>, ArrExpr["ast::ArrExpr"]{expr}, ArrSpread["ast::ArrSpread"]{spread});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ArrayElement<>, ArrExpr{expr: crate::ast::ExprId}, ArrSpread{spread: crate::ddlog_std::Option<crate::ast::ExprId>});
impl ::std::fmt::Display for ArrayElement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ArrayElement::ArrExpr{expr} => {
                __formatter.write_str("ast::ArrExpr{")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ArrayElement::ArrSpread{spread} => {
                __formatter.write_str("ast::ArrSpread{")?;
                ::std::fmt::Debug::fmt(spread, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ArrayElement {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for ArrayElement {
    fn default() -> Self {
        crate::ast::ArrayElement::ArrExpr{expr : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum AssignOperand {
    OpAssign,
    OpAddAssign,
    OpSubtractAssign,
    OpTimesAssign,
    OpRemainderAssign,
    OpExponentAssign,
    OpLeftShiftAssign,
    OpRightShiftAssign,
    OpUnsignedRightShiftAssign,
    OpBitwiseAndAssign,
    OpBitwiseOrAssign,
    OpBitwiseXorAssign,
    OpLogicalAndAssign,
    OpLogicalOrAssign,
    OpNullishCoalescingAssign
}
impl abomonation::Abomonation for AssignOperand{}
::differential_datalog::decl_enum_from_record!(AssignOperand["ast::AssignOperand"]<>, OpAssign["ast::OpAssign"][0]{}, OpAddAssign["ast::OpAddAssign"][0]{}, OpSubtractAssign["ast::OpSubtractAssign"][0]{}, OpTimesAssign["ast::OpTimesAssign"][0]{}, OpRemainderAssign["ast::OpRemainderAssign"][0]{}, OpExponentAssign["ast::OpExponentAssign"][0]{}, OpLeftShiftAssign["ast::OpLeftShiftAssign"][0]{}, OpRightShiftAssign["ast::OpRightShiftAssign"][0]{}, OpUnsignedRightShiftAssign["ast::OpUnsignedRightShiftAssign"][0]{}, OpBitwiseAndAssign["ast::OpBitwiseAndAssign"][0]{}, OpBitwiseOrAssign["ast::OpBitwiseOrAssign"][0]{}, OpBitwiseXorAssign["ast::OpBitwiseXorAssign"][0]{}, OpLogicalAndAssign["ast::OpLogicalAndAssign"][0]{}, OpLogicalOrAssign["ast::OpLogicalOrAssign"][0]{}, OpNullishCoalescingAssign["ast::OpNullishCoalescingAssign"][0]{});
::differential_datalog::decl_enum_into_record!(AssignOperand<>, OpAssign["ast::OpAssign"]{}, OpAddAssign["ast::OpAddAssign"]{}, OpSubtractAssign["ast::OpSubtractAssign"]{}, OpTimesAssign["ast::OpTimesAssign"]{}, OpRemainderAssign["ast::OpRemainderAssign"]{}, OpExponentAssign["ast::OpExponentAssign"]{}, OpLeftShiftAssign["ast::OpLeftShiftAssign"]{}, OpRightShiftAssign["ast::OpRightShiftAssign"]{}, OpUnsignedRightShiftAssign["ast::OpUnsignedRightShiftAssign"]{}, OpBitwiseAndAssign["ast::OpBitwiseAndAssign"]{}, OpBitwiseOrAssign["ast::OpBitwiseOrAssign"]{}, OpBitwiseXorAssign["ast::OpBitwiseXorAssign"]{}, OpLogicalAndAssign["ast::OpLogicalAndAssign"]{}, OpLogicalOrAssign["ast::OpLogicalOrAssign"]{}, OpNullishCoalescingAssign["ast::OpNullishCoalescingAssign"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(AssignOperand<>, OpAssign{}, OpAddAssign{}, OpSubtractAssign{}, OpTimesAssign{}, OpRemainderAssign{}, OpExponentAssign{}, OpLeftShiftAssign{}, OpRightShiftAssign{}, OpUnsignedRightShiftAssign{}, OpBitwiseAndAssign{}, OpBitwiseOrAssign{}, OpBitwiseXorAssign{}, OpLogicalAndAssign{}, OpLogicalOrAssign{}, OpNullishCoalescingAssign{});
impl ::std::fmt::Display for AssignOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::AssignOperand::OpAssign{} => {
                __formatter.write_str("ast::OpAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpAddAssign{} => {
                __formatter.write_str("ast::OpAddAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpSubtractAssign{} => {
                __formatter.write_str("ast::OpSubtractAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpTimesAssign{} => {
                __formatter.write_str("ast::OpTimesAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpRemainderAssign{} => {
                __formatter.write_str("ast::OpRemainderAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpExponentAssign{} => {
                __formatter.write_str("ast::OpExponentAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpLeftShiftAssign{} => {
                __formatter.write_str("ast::OpLeftShiftAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpRightShiftAssign{} => {
                __formatter.write_str("ast::OpRightShiftAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpUnsignedRightShiftAssign{} => {
                __formatter.write_str("ast::OpUnsignedRightShiftAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpBitwiseAndAssign{} => {
                __formatter.write_str("ast::OpBitwiseAndAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpBitwiseOrAssign{} => {
                __formatter.write_str("ast::OpBitwiseOrAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpBitwiseXorAssign{} => {
                __formatter.write_str("ast::OpBitwiseXorAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpLogicalAndAssign{} => {
                __formatter.write_str("ast::OpLogicalAndAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpLogicalOrAssign{} => {
                __formatter.write_str("ast::OpLogicalOrAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::AssignOperand::OpNullishCoalescingAssign{} => {
                __formatter.write_str("ast::OpNullishCoalescingAssign{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for AssignOperand {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for AssignOperand {
    fn default() -> Self {
        crate::ast::AssignOperand::OpAssign{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum BinOperand {
    BinLessThan,
    BinGreaterThan,
    BinLessThanOrEqual,
    BinGreaterThanOrEqual,
    BinEquality,
    BinStrictEquality,
    BinInequality,
    BinStrictInequality,
    BinPlus,
    BinMinus,
    BinTimes,
    BinDivide,
    BinRemainder,
    BinExponent,
    BinLeftShift,
    BinRightShift,
    BinUnsignedRightShift,
    BinBitwiseAnd,
    BinBitwiseOr,
    BinBitwiseXor,
    BinNullishCoalescing,
    BinLogicalOr,
    BinLogicalAnd,
    BinIn,
    BinInstanceof
}
impl abomonation::Abomonation for BinOperand{}
::differential_datalog::decl_enum_from_record!(BinOperand["ast::BinOperand"]<>, BinLessThan["ast::BinLessThan"][0]{}, BinGreaterThan["ast::BinGreaterThan"][0]{}, BinLessThanOrEqual["ast::BinLessThanOrEqual"][0]{}, BinGreaterThanOrEqual["ast::BinGreaterThanOrEqual"][0]{}, BinEquality["ast::BinEquality"][0]{}, BinStrictEquality["ast::BinStrictEquality"][0]{}, BinInequality["ast::BinInequality"][0]{}, BinStrictInequality["ast::BinStrictInequality"][0]{}, BinPlus["ast::BinPlus"][0]{}, BinMinus["ast::BinMinus"][0]{}, BinTimes["ast::BinTimes"][0]{}, BinDivide["ast::BinDivide"][0]{}, BinRemainder["ast::BinRemainder"][0]{}, BinExponent["ast::BinExponent"][0]{}, BinLeftShift["ast::BinLeftShift"][0]{}, BinRightShift["ast::BinRightShift"][0]{}, BinUnsignedRightShift["ast::BinUnsignedRightShift"][0]{}, BinBitwiseAnd["ast::BinBitwiseAnd"][0]{}, BinBitwiseOr["ast::BinBitwiseOr"][0]{}, BinBitwiseXor["ast::BinBitwiseXor"][0]{}, BinNullishCoalescing["ast::BinNullishCoalescing"][0]{}, BinLogicalOr["ast::BinLogicalOr"][0]{}, BinLogicalAnd["ast::BinLogicalAnd"][0]{}, BinIn["ast::BinIn"][0]{}, BinInstanceof["ast::BinInstanceof"][0]{});
::differential_datalog::decl_enum_into_record!(BinOperand<>, BinLessThan["ast::BinLessThan"]{}, BinGreaterThan["ast::BinGreaterThan"]{}, BinLessThanOrEqual["ast::BinLessThanOrEqual"]{}, BinGreaterThanOrEqual["ast::BinGreaterThanOrEqual"]{}, BinEquality["ast::BinEquality"]{}, BinStrictEquality["ast::BinStrictEquality"]{}, BinInequality["ast::BinInequality"]{}, BinStrictInequality["ast::BinStrictInequality"]{}, BinPlus["ast::BinPlus"]{}, BinMinus["ast::BinMinus"]{}, BinTimes["ast::BinTimes"]{}, BinDivide["ast::BinDivide"]{}, BinRemainder["ast::BinRemainder"]{}, BinExponent["ast::BinExponent"]{}, BinLeftShift["ast::BinLeftShift"]{}, BinRightShift["ast::BinRightShift"]{}, BinUnsignedRightShift["ast::BinUnsignedRightShift"]{}, BinBitwiseAnd["ast::BinBitwiseAnd"]{}, BinBitwiseOr["ast::BinBitwiseOr"]{}, BinBitwiseXor["ast::BinBitwiseXor"]{}, BinNullishCoalescing["ast::BinNullishCoalescing"]{}, BinLogicalOr["ast::BinLogicalOr"]{}, BinLogicalAnd["ast::BinLogicalAnd"]{}, BinIn["ast::BinIn"]{}, BinInstanceof["ast::BinInstanceof"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(BinOperand<>, BinLessThan{}, BinGreaterThan{}, BinLessThanOrEqual{}, BinGreaterThanOrEqual{}, BinEquality{}, BinStrictEquality{}, BinInequality{}, BinStrictInequality{}, BinPlus{}, BinMinus{}, BinTimes{}, BinDivide{}, BinRemainder{}, BinExponent{}, BinLeftShift{}, BinRightShift{}, BinUnsignedRightShift{}, BinBitwiseAnd{}, BinBitwiseOr{}, BinBitwiseXor{}, BinNullishCoalescing{}, BinLogicalOr{}, BinLogicalAnd{}, BinIn{}, BinInstanceof{});
impl ::std::fmt::Display for BinOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::BinOperand::BinLessThan{} => {
                __formatter.write_str("ast::BinLessThan{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinGreaterThan{} => {
                __formatter.write_str("ast::BinGreaterThan{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinLessThanOrEqual{} => {
                __formatter.write_str("ast::BinLessThanOrEqual{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinGreaterThanOrEqual{} => {
                __formatter.write_str("ast::BinGreaterThanOrEqual{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinEquality{} => {
                __formatter.write_str("ast::BinEquality{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinStrictEquality{} => {
                __formatter.write_str("ast::BinStrictEquality{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinInequality{} => {
                __formatter.write_str("ast::BinInequality{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinStrictInequality{} => {
                __formatter.write_str("ast::BinStrictInequality{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinPlus{} => {
                __formatter.write_str("ast::BinPlus{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinMinus{} => {
                __formatter.write_str("ast::BinMinus{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinTimes{} => {
                __formatter.write_str("ast::BinTimes{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinDivide{} => {
                __formatter.write_str("ast::BinDivide{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinRemainder{} => {
                __formatter.write_str("ast::BinRemainder{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinExponent{} => {
                __formatter.write_str("ast::BinExponent{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinLeftShift{} => {
                __formatter.write_str("ast::BinLeftShift{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinRightShift{} => {
                __formatter.write_str("ast::BinRightShift{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinUnsignedRightShift{} => {
                __formatter.write_str("ast::BinUnsignedRightShift{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinBitwiseAnd{} => {
                __formatter.write_str("ast::BinBitwiseAnd{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinBitwiseOr{} => {
                __formatter.write_str("ast::BinBitwiseOr{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinBitwiseXor{} => {
                __formatter.write_str("ast::BinBitwiseXor{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinNullishCoalescing{} => {
                __formatter.write_str("ast::BinNullishCoalescing{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinLogicalOr{} => {
                __formatter.write_str("ast::BinLogicalOr{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinLogicalAnd{} => {
                __formatter.write_str("ast::BinLogicalAnd{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinIn{} => {
                __formatter.write_str("ast::BinIn{")?;
                __formatter.write_str("}")
            },
            crate::ast::BinOperand::BinInstanceof{} => {
                __formatter.write_str("ast::BinInstanceof{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for BinOperand {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for BinOperand {
    fn default() -> Self {
        crate::ast::BinOperand::BinLessThan{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ClassElement {
    ClassEmptyElem,
    ClassMethod {
        name: crate::ddlog_std::Option<crate::ast::PropertyKey>,
        params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>,
        body: crate::ddlog_std::Option<crate::ast::StmtId>
    },
    ClassStaticMethod {
        name: crate::ddlog_std::Option<crate::ast::PropertyKey>,
        params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>,
        body: crate::ddlog_std::Option<crate::ast::StmtId>
    }
}
impl abomonation::Abomonation for ClassElement{}
::differential_datalog::decl_enum_from_record!(ClassElement["ast::ClassElement"]<>, ClassEmptyElem["ast::ClassEmptyElem"][0]{}, ClassMethod["ast::ClassMethod"][3]{[0]name["name"]: crate::ddlog_std::Option<crate::ast::PropertyKey>, [1]params["params"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>, [2]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>}, ClassStaticMethod["ast::ClassStaticMethod"][3]{[0]name["name"]: crate::ddlog_std::Option<crate::ast::PropertyKey>, [1]params["params"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>, [2]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_enum_into_record!(ClassElement<>, ClassEmptyElem["ast::ClassEmptyElem"]{}, ClassMethod["ast::ClassMethod"]{name, params, body}, ClassStaticMethod["ast::ClassStaticMethod"]{name, params, body});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ClassElement<>, ClassEmptyElem{}, ClassMethod{name: crate::ddlog_std::Option<crate::ast::PropertyKey>, params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>, body: crate::ddlog_std::Option<crate::ast::StmtId>}, ClassStaticMethod{name: crate::ddlog_std::Option<crate::ast::PropertyKey>, params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>, body: crate::ddlog_std::Option<crate::ast::StmtId>});
impl ::std::fmt::Display for ClassElement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ClassElement::ClassEmptyElem{} => {
                __formatter.write_str("ast::ClassEmptyElem{")?;
                __formatter.write_str("}")
            },
            crate::ast::ClassElement::ClassMethod{name,params,body} => {
                __formatter.write_str("ast::ClassMethod{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ClassElement::ClassStaticMethod{name,params,body} => {
                __formatter.write_str("ast::ClassStaticMethod{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ClassElement {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for ClassElement {
    fn default() -> Self {
        crate::ast::ClassElement::ClassEmptyElem{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ClassId {
    pub id: u32,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for ClassId{}
::differential_datalog::decl_struct_from_record!(ClassId["ast::ClassId"]<>, ["ast::ClassId"][2]{[0]id["id"]: u32, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(ClassId, ["ast::ClassId"]<>, id, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ClassId, <>, id: u32, file: crate::ast::FileId);
impl ::std::fmt::Display for ClassId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ClassId{id,file} => {
                __formatter.write_str("ast::ClassId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ClassId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ExportKind {
    WildcardExport,
    NamedExport {
        name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>,
        alias: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
    }
}
impl abomonation::Abomonation for ExportKind{}
::differential_datalog::decl_enum_from_record!(ExportKind["ast::ExportKind"]<>, WildcardExport["ast::WildcardExport"][0]{}, NamedExport["ast::NamedExport"][2]{[0]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, [1]alias["alias"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
::differential_datalog::decl_enum_into_record!(ExportKind<>, WildcardExport["ast::WildcardExport"]{}, NamedExport["ast::NamedExport"]{name, alias});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ExportKind<>, WildcardExport{}, NamedExport{name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, alias: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
impl ::std::fmt::Display for ExportKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ExportKind::WildcardExport{} => {
                __formatter.write_str("ast::WildcardExport{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExportKind::NamedExport{name,alias} => {
                __formatter.write_str("ast::NamedExport{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(alias, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ExportKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for ExportKind {
    fn default() -> Self {
        crate::ast::ExportKind::WildcardExport{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ExprId {
    pub id: u32,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for ExprId{}
::differential_datalog::decl_struct_from_record!(ExprId["ast::ExprId"]<>, ["ast::ExprId"][2]{[0]id["id"]: u32, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(ExprId, ["ast::ExprId"]<>, id, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprId, <>, id: u32, file: crate::ast::FileId);
impl ::std::fmt::Display for ExprId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ExprId{id,file} => {
                __formatter.write_str("ast::ExprId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ExprId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ExprKind {
    ExprLit {
        kind: crate::ast::LitKind
    },
    ExprNameRef,
    ExprYield,
    ExprAwait,
    ExprArrow,
    ExprUnaryOp,
    ExprBinOp,
    ExprTernary,
    ExprThis,
    ExprTemplate,
    ExprArray,
    ExprObject,
    ExprGrouping {
        inner: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    ExprBracket,
    ExprDot,
    ExprNew,
    ExprCall,
    ExprAssign,
    ExprSequence {
        exprs: crate::ddlog_std::Vec<crate::ast::ExprId>
    },
    ExprNewTarget,
    ExprImportMeta,
    ExprInlineFunc,
    ExprSuperCall {
        args: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>
    },
    ExprImportCall {
        arg: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    ExprClass
}
impl abomonation::Abomonation for ExprKind{}
::differential_datalog::decl_enum_from_record!(ExprKind["ast::ExprKind"]<>, ExprLit["ast::ExprLit"][1]{[0]kind["kind"]: crate::ast::LitKind}, ExprNameRef["ast::ExprNameRef"][0]{}, ExprYield["ast::ExprYield"][0]{}, ExprAwait["ast::ExprAwait"][0]{}, ExprArrow["ast::ExprArrow"][0]{}, ExprUnaryOp["ast::ExprUnaryOp"][0]{}, ExprBinOp["ast::ExprBinOp"][0]{}, ExprTernary["ast::ExprTernary"][0]{}, ExprThis["ast::ExprThis"][0]{}, ExprTemplate["ast::ExprTemplate"][0]{}, ExprArray["ast::ExprArray"][0]{}, ExprObject["ast::ExprObject"][0]{}, ExprGrouping["ast::ExprGrouping"][1]{[0]inner["inner"]: crate::ddlog_std::Option<crate::ast::ExprId>}, ExprBracket["ast::ExprBracket"][0]{}, ExprDot["ast::ExprDot"][0]{}, ExprNew["ast::ExprNew"][0]{}, ExprCall["ast::ExprCall"][0]{}, ExprAssign["ast::ExprAssign"][0]{}, ExprSequence["ast::ExprSequence"][1]{[0]exprs["exprs"]: crate::ddlog_std::Vec<crate::ast::ExprId>}, ExprNewTarget["ast::ExprNewTarget"][0]{}, ExprImportMeta["ast::ExprImportMeta"][0]{}, ExprInlineFunc["ast::ExprInlineFunc"][0]{}, ExprSuperCall["ast::ExprSuperCall"][1]{[0]args["args"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>}, ExprImportCall["ast::ExprImportCall"][1]{[0]arg["arg"]: crate::ddlog_std::Option<crate::ast::ExprId>}, ExprClass["ast::ExprClass"][0]{});
::differential_datalog::decl_enum_into_record!(ExprKind<>, ExprLit["ast::ExprLit"]{kind}, ExprNameRef["ast::ExprNameRef"]{}, ExprYield["ast::ExprYield"]{}, ExprAwait["ast::ExprAwait"]{}, ExprArrow["ast::ExprArrow"]{}, ExprUnaryOp["ast::ExprUnaryOp"]{}, ExprBinOp["ast::ExprBinOp"]{}, ExprTernary["ast::ExprTernary"]{}, ExprThis["ast::ExprThis"]{}, ExprTemplate["ast::ExprTemplate"]{}, ExprArray["ast::ExprArray"]{}, ExprObject["ast::ExprObject"]{}, ExprGrouping["ast::ExprGrouping"]{inner}, ExprBracket["ast::ExprBracket"]{}, ExprDot["ast::ExprDot"]{}, ExprNew["ast::ExprNew"]{}, ExprCall["ast::ExprCall"]{}, ExprAssign["ast::ExprAssign"]{}, ExprSequence["ast::ExprSequence"]{exprs}, ExprNewTarget["ast::ExprNewTarget"]{}, ExprImportMeta["ast::ExprImportMeta"]{}, ExprInlineFunc["ast::ExprInlineFunc"]{}, ExprSuperCall["ast::ExprSuperCall"]{args}, ExprImportCall["ast::ExprImportCall"]{arg}, ExprClass["ast::ExprClass"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ExprKind<>, ExprLit{kind: crate::ast::LitKind}, ExprNameRef{}, ExprYield{}, ExprAwait{}, ExprArrow{}, ExprUnaryOp{}, ExprBinOp{}, ExprTernary{}, ExprThis{}, ExprTemplate{}, ExprArray{}, ExprObject{}, ExprGrouping{inner: crate::ddlog_std::Option<crate::ast::ExprId>}, ExprBracket{}, ExprDot{}, ExprNew{}, ExprCall{}, ExprAssign{}, ExprSequence{exprs: crate::ddlog_std::Vec<crate::ast::ExprId>}, ExprNewTarget{}, ExprImportMeta{}, ExprInlineFunc{}, ExprSuperCall{args: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::ExprId>>}, ExprImportCall{arg: crate::ddlog_std::Option<crate::ast::ExprId>}, ExprClass{});
impl ::std::fmt::Display for ExprKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ExprKind::ExprLit{kind} => {
                __formatter.write_str("ast::ExprLit{")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprNameRef{} => {
                __formatter.write_str("ast::ExprNameRef{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprYield{} => {
                __formatter.write_str("ast::ExprYield{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprAwait{} => {
                __formatter.write_str("ast::ExprAwait{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprArrow{} => {
                __formatter.write_str("ast::ExprArrow{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprUnaryOp{} => {
                __formatter.write_str("ast::ExprUnaryOp{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprBinOp{} => {
                __formatter.write_str("ast::ExprBinOp{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprTernary{} => {
                __formatter.write_str("ast::ExprTernary{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprThis{} => {
                __formatter.write_str("ast::ExprThis{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprTemplate{} => {
                __formatter.write_str("ast::ExprTemplate{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprArray{} => {
                __formatter.write_str("ast::ExprArray{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprObject{} => {
                __formatter.write_str("ast::ExprObject{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprGrouping{inner} => {
                __formatter.write_str("ast::ExprGrouping{")?;
                ::std::fmt::Debug::fmt(inner, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprBracket{} => {
                __formatter.write_str("ast::ExprBracket{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprDot{} => {
                __formatter.write_str("ast::ExprDot{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprNew{} => {
                __formatter.write_str("ast::ExprNew{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprCall{} => {
                __formatter.write_str("ast::ExprCall{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprAssign{} => {
                __formatter.write_str("ast::ExprAssign{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprSequence{exprs} => {
                __formatter.write_str("ast::ExprSequence{")?;
                ::std::fmt::Debug::fmt(exprs, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprNewTarget{} => {
                __formatter.write_str("ast::ExprNewTarget{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprImportMeta{} => {
                __formatter.write_str("ast::ExprImportMeta{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprInlineFunc{} => {
                __formatter.write_str("ast::ExprInlineFunc{")?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprSuperCall{args} => {
                __formatter.write_str("ast::ExprSuperCall{")?;
                ::std::fmt::Debug::fmt(args, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprImportCall{arg} => {
                __formatter.write_str("ast::ExprImportCall{")?;
                ::std::fmt::Debug::fmt(arg, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ExprKind::ExprClass{} => {
                __formatter.write_str("ast::ExprClass{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ExprKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for ExprKind {
    fn default() -> Self {
        crate::ast::ExprKind::ExprLit{kind : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct FileId {
    pub id: u32
}
impl abomonation::Abomonation for FileId{}
::differential_datalog::decl_struct_from_record!(FileId["ast::FileId"]<>, ["ast::FileId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(FileId, ["ast::FileId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FileId, <>, id: u32);
impl ::std::fmt::Display for FileId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::FileId{id} => {
                __formatter.write_str("ast::FileId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FileId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum FileKind {
    JavaScript {
        flavor: crate::ast::JSFlavor
    },
    Todo
}
impl abomonation::Abomonation for FileKind{}
::differential_datalog::decl_enum_from_record!(FileKind["ast::FileKind"]<>, JavaScript["ast::JavaScript"][1]{[0]flavor["flavor"]: crate::ast::JSFlavor}, Todo["ast::Todo"][0]{});
::differential_datalog::decl_enum_into_record!(FileKind<>, JavaScript["ast::JavaScript"]{flavor}, Todo["ast::Todo"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(FileKind<>, JavaScript{flavor: crate::ast::JSFlavor}, Todo{});
impl ::std::fmt::Display for FileKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::FileKind::JavaScript{flavor} => {
                __formatter.write_str("ast::JavaScript{")?;
                ::std::fmt::Debug::fmt(flavor, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::FileKind::Todo{} => {
                __formatter.write_str("ast::Todo{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FileKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for FileKind {
    fn default() -> Self {
        crate::ast::FileKind::JavaScript{flavor : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ForInit {
    ForDecl {
        stmt_id: crate::ddlog_std::Option<crate::ast::StmtId>
    },
    ForExpr {
        expr_id: crate::ast::ExprId
    }
}
impl abomonation::Abomonation for ForInit{}
::differential_datalog::decl_enum_from_record!(ForInit["ast::ForInit"]<>, ForDecl["ast::ForDecl"][1]{[0]stmt_id["stmt_id"]: crate::ddlog_std::Option<crate::ast::StmtId>}, ForExpr["ast::ForExpr"][1]{[0]expr_id["expr_id"]: crate::ast::ExprId});
::differential_datalog::decl_enum_into_record!(ForInit<>, ForDecl["ast::ForDecl"]{stmt_id}, ForExpr["ast::ForExpr"]{expr_id});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ForInit<>, ForDecl{stmt_id: crate::ddlog_std::Option<crate::ast::StmtId>}, ForExpr{expr_id: crate::ast::ExprId});
impl ::std::fmt::Display for ForInit {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ForInit::ForDecl{stmt_id} => {
                __formatter.write_str("ast::ForDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ForInit::ForExpr{expr_id} => {
                __formatter.write_str("ast::ForExpr{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ForInit {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for ForInit {
    fn default() -> Self {
        crate::ast::ForInit::ForDecl{stmt_id : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct FuncId {
    pub id: u32,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for FuncId{}
::differential_datalog::decl_struct_from_record!(FuncId["ast::FuncId"]<>, ["ast::FuncId"][2]{[0]id["id"]: u32, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(FuncId, ["ast::FuncId"]<>, id, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FuncId, <>, id: u32, file: crate::ast::FileId);
impl ::std::fmt::Display for FuncId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::FuncId{id,file} => {
                __formatter.write_str("ast::FuncId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FuncId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct GlobalId {
    pub id: u32,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for GlobalId{}
::differential_datalog::decl_struct_from_record!(GlobalId["ast::GlobalId"]<>, ["ast::GlobalId"][2]{[0]id["id"]: u32, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(GlobalId, ["ast::GlobalId"]<>, id, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(GlobalId, <>, id: u32, file: crate::ast::FileId);
impl ::std::fmt::Display for GlobalId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::GlobalId{id,file} => {
                __formatter.write_str("ast::GlobalId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for GlobalId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
pub type IClassElement = crate::internment::Intern<crate::ast::ClassElement>;
pub type IObjectPatternProp = crate::internment::Intern<crate::ast::ObjectPatternProp>;
pub type IPattern = crate::internment::Intern<crate::ast::Pattern>;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ImportClause {
    WildcardImport {
        alias: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
    },
    GroupedImport {
        imports: crate::ddlog_std::Vec<crate::ast::NamedImport>
    },
    SingleImport {
        name: crate::ast::Spanned<crate::ast::Name>
    }
}
impl abomonation::Abomonation for ImportClause{}
::differential_datalog::decl_enum_from_record!(ImportClause["ast::ImportClause"]<>, WildcardImport["ast::WildcardImport"][1]{[0]alias["alias"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>}, GroupedImport["ast::GroupedImport"][1]{[0]imports["imports"]: crate::ddlog_std::Vec<crate::ast::NamedImport>}, SingleImport["ast::SingleImport"][1]{[0]name["name"]: crate::ast::Spanned<crate::ast::Name>});
::differential_datalog::decl_enum_into_record!(ImportClause<>, WildcardImport["ast::WildcardImport"]{alias}, GroupedImport["ast::GroupedImport"]{imports}, SingleImport["ast::SingleImport"]{name});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ImportClause<>, WildcardImport{alias: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>}, GroupedImport{imports: crate::ddlog_std::Vec<crate::ast::NamedImport>}, SingleImport{name: crate::ast::Spanned<crate::ast::Name>});
impl ::std::fmt::Display for ImportClause {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ImportClause::WildcardImport{alias} => {
                __formatter.write_str("ast::WildcardImport{")?;
                ::std::fmt::Debug::fmt(alias, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ImportClause::GroupedImport{imports} => {
                __formatter.write_str("ast::GroupedImport{")?;
                ::std::fmt::Debug::fmt(imports, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ImportClause::SingleImport{name} => {
                __formatter.write_str("ast::SingleImport{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ImportClause {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for ImportClause {
    fn default() -> Self {
        crate::ast::ImportClause::WildcardImport{alias : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ImportId {
    pub id: u32,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for ImportId{}
::differential_datalog::decl_struct_from_record!(ImportId["ast::ImportId"]<>, ["ast::ImportId"][2]{[0]id["id"]: u32, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(ImportId, ["ast::ImportId"]<>, id, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ImportId, <>, id: u32, file: crate::ast::FileId);
impl ::std::fmt::Display for ImportId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ImportId{id,file} => {
                __formatter.write_str("ast::ImportId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ImportId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum JSFlavor {
    Vanilla,
    Module,
    TypeScript
}
impl abomonation::Abomonation for JSFlavor{}
::differential_datalog::decl_enum_from_record!(JSFlavor["ast::JSFlavor"]<>, Vanilla["ast::Vanilla"][0]{}, Module["ast::Module"][0]{}, TypeScript["ast::TypeScript"][0]{});
::differential_datalog::decl_enum_into_record!(JSFlavor<>, Vanilla["ast::Vanilla"]{}, Module["ast::Module"]{}, TypeScript["ast::TypeScript"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(JSFlavor<>, Vanilla{}, Module{}, TypeScript{});
impl ::std::fmt::Display for JSFlavor {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::JSFlavor::Vanilla{} => {
                __formatter.write_str("ast::Vanilla{")?;
                __formatter.write_str("}")
            },
            crate::ast::JSFlavor::Module{} => {
                __formatter.write_str("ast::Module{")?;
                __formatter.write_str("}")
            },
            crate::ast::JSFlavor::TypeScript{} => {
                __formatter.write_str("ast::TypeScript{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for JSFlavor {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for JSFlavor {
    fn default() -> Self {
        crate::ast::JSFlavor::Vanilla{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum LitKind {
    LitNumber,
    LitBigInt,
    LitString,
    LitNull,
    LitBool,
    LitRegex
}
impl abomonation::Abomonation for LitKind{}
::differential_datalog::decl_enum_from_record!(LitKind["ast::LitKind"]<>, LitNumber["ast::LitNumber"][0]{}, LitBigInt["ast::LitBigInt"][0]{}, LitString["ast::LitString"][0]{}, LitNull["ast::LitNull"][0]{}, LitBool["ast::LitBool"][0]{}, LitRegex["ast::LitRegex"][0]{});
::differential_datalog::decl_enum_into_record!(LitKind<>, LitNumber["ast::LitNumber"]{}, LitBigInt["ast::LitBigInt"]{}, LitString["ast::LitString"]{}, LitNull["ast::LitNull"]{}, LitBool["ast::LitBool"]{}, LitRegex["ast::LitRegex"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(LitKind<>, LitNumber{}, LitBigInt{}, LitString{}, LitNull{}, LitBool{}, LitRegex{});
impl ::std::fmt::Display for LitKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::LitKind::LitNumber{} => {
                __formatter.write_str("ast::LitNumber{")?;
                __formatter.write_str("}")
            },
            crate::ast::LitKind::LitBigInt{} => {
                __formatter.write_str("ast::LitBigInt{")?;
                __formatter.write_str("}")
            },
            crate::ast::LitKind::LitString{} => {
                __formatter.write_str("ast::LitString{")?;
                __formatter.write_str("}")
            },
            crate::ast::LitKind::LitNull{} => {
                __formatter.write_str("ast::LitNull{")?;
                __formatter.write_str("}")
            },
            crate::ast::LitKind::LitBool{} => {
                __formatter.write_str("ast::LitBool{")?;
                __formatter.write_str("}")
            },
            crate::ast::LitKind::LitRegex{} => {
                __formatter.write_str("ast::LitRegex{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for LitKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for LitKind {
    fn default() -> Self {
        crate::ast::LitKind::LitNumber{}
    }
}
pub type Name = crate::internment::istring;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NamedImport {
    pub name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>,
    pub alias: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
}
impl abomonation::Abomonation for NamedImport{}
::differential_datalog::decl_struct_from_record!(NamedImport["ast::NamedImport"]<>, ["ast::NamedImport"][2]{[0]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, [1]alias["alias"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
::differential_datalog::decl_struct_into_record!(NamedImport, ["ast::NamedImport"]<>, name, alias);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NamedImport, <>, name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>, alias: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>);
impl ::std::fmt::Display for NamedImport {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::NamedImport{name,alias} => {
                __formatter.write_str("ast::NamedImport{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(alias, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for NamedImport {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ObjectPatternProp {
    ObjAssignPattern {
        assign_key: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>,
        assign_value: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    ObjKeyValuePattern {
        key: crate::ddlog_std::Option<crate::ast::PropertyKey>,
        value: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>
    },
    ObjRestPattern {
        rest: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>
    },
    ObjSinglePattern {
        name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
    }
}
impl abomonation::Abomonation for ObjectPatternProp{}
::differential_datalog::decl_enum_from_record!(ObjectPatternProp["ast::ObjectPatternProp"]<>, ObjAssignPattern["ast::ObjAssignPattern"][2]{[0]assign_key["assign_key"]: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>, [1]assign_value["assign_value"]: crate::ddlog_std::Option<crate::ast::ExprId>}, ObjKeyValuePattern["ast::ObjKeyValuePattern"][2]{[0]key["key"]: crate::ddlog_std::Option<crate::ast::PropertyKey>, [1]value["value"]: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>}, ObjRestPattern["ast::ObjRestPattern"][1]{[0]rest["rest"]: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>}, ObjSinglePattern["ast::ObjSinglePattern"][1]{[0]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
::differential_datalog::decl_enum_into_record!(ObjectPatternProp<>, ObjAssignPattern["ast::ObjAssignPattern"]{assign_key, assign_value}, ObjKeyValuePattern["ast::ObjKeyValuePattern"]{key, value}, ObjRestPattern["ast::ObjRestPattern"]{rest}, ObjSinglePattern["ast::ObjSinglePattern"]{name});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ObjectPatternProp<>, ObjAssignPattern{assign_key: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>, assign_value: crate::ddlog_std::Option<crate::ast::ExprId>}, ObjKeyValuePattern{key: crate::ddlog_std::Option<crate::ast::PropertyKey>, value: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>}, ObjRestPattern{rest: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>}, ObjSinglePattern{name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>});
impl ::std::fmt::Display for ObjectPatternProp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ObjectPatternProp::ObjAssignPattern{assign_key,assign_value} => {
                __formatter.write_str("ast::ObjAssignPattern{")?;
                ::std::fmt::Debug::fmt(assign_key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(assign_value, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ObjectPatternProp::ObjKeyValuePattern{key,value} => {
                __formatter.write_str("ast::ObjKeyValuePattern{")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ObjectPatternProp::ObjRestPattern{rest} => {
                __formatter.write_str("ast::ObjRestPattern{")?;
                ::std::fmt::Debug::fmt(rest, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::ObjectPatternProp::ObjSinglePattern{name} => {
                __formatter.write_str("ast::ObjSinglePattern{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ObjectPatternProp {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for ObjectPatternProp {
    fn default() -> Self {
        crate::ast::ObjectPatternProp::ObjAssignPattern{assign_key : ::std::default::Default::default(), assign_value : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum OneOf<A, B, C> {
    First {
        a: A
    },
    Second {
        b: B
    },
    Third {
        c: C
    }
}
impl <A: crate::Val, B: crate::Val, C: crate::Val> abomonation::Abomonation for OneOf<A, B, C>{}
::differential_datalog::decl_enum_from_record!(OneOf["ast::OneOf"]<A,B,C>, First["ast::First"][1]{[0]a["a"]: A}, Second["ast::Second"][1]{[0]b["b"]: B}, Third["ast::Third"][1]{[0]c["c"]: C});
::differential_datalog::decl_enum_into_record!(OneOf<A,B,C>, First["ast::First"]{a}, Second["ast::Second"]{b}, Third["ast::Third"]{c});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(OneOf<A,B,C>, First{a: A}, Second{b: B}, Third{c: C});
impl <A: ::std::fmt::Debug, B: ::std::fmt::Debug, C: ::std::fmt::Debug> ::std::fmt::Display for OneOf<A, B, C> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::OneOf::First{a} => {
                __formatter.write_str("ast::First{")?;
                ::std::fmt::Debug::fmt(a, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::OneOf::Second{b} => {
                __formatter.write_str("ast::Second{")?;
                ::std::fmt::Debug::fmt(b, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::OneOf::Third{c} => {
                __formatter.write_str("ast::Third{")?;
                ::std::fmt::Debug::fmt(c, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl <A: ::std::fmt::Debug, B: ::std::fmt::Debug, C: ::std::fmt::Debug> ::std::fmt::Debug for OneOf<A, B, C> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl <A: ::std::default::Default, B: ::std::default::Default, C: ::std::default::Default> ::std::default::Default for OneOf<A, B, C> {
    fn default() -> Self {
        crate::ast::OneOf::First{a : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Pattern {
    SinglePattern {
        name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
    },
    RestPattern {
        rest: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>
    },
    AssignPattern {
        key: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>,
        value: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    ObjectPattern {
        props: crate::ddlog_std::Vec<crate::internment::Intern<crate::ast::ObjectPatternProp>>
    },
    ArrayPattern {
        elems: crate::ddlog_std::Vec<crate::internment::Intern<crate::ast::Pattern>>
    }
}
impl abomonation::Abomonation for Pattern{}
::differential_datalog::decl_enum_from_record!(Pattern["ast::Pattern"]<>, SinglePattern["ast::SinglePattern"][1]{[0]name["name"]: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>}, RestPattern["ast::RestPattern"][1]{[0]rest["rest"]: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>}, AssignPattern["ast::AssignPattern"][2]{[0]key["key"]: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>, [1]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>}, ObjectPattern["ast::ObjectPattern"][1]{[0]props["props"]: crate::ddlog_std::Vec<crate::internment::Intern<crate::ast::ObjectPatternProp>>}, ArrayPattern["ast::ArrayPattern"][1]{[0]elems["elems"]: crate::ddlog_std::Vec<crate::internment::Intern<crate::ast::Pattern>>});
::differential_datalog::decl_enum_into_record!(Pattern<>, SinglePattern["ast::SinglePattern"]{name}, RestPattern["ast::RestPattern"]{rest}, AssignPattern["ast::AssignPattern"]{key, value}, ObjectPattern["ast::ObjectPattern"]{props}, ArrayPattern["ast::ArrayPattern"]{elems});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(Pattern<>, SinglePattern{name: crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>}, RestPattern{rest: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>}, AssignPattern{key: crate::ddlog_std::Option<crate::internment::Intern<crate::ast::Pattern>>, value: crate::ddlog_std::Option<crate::ast::ExprId>}, ObjectPattern{props: crate::ddlog_std::Vec<crate::internment::Intern<crate::ast::ObjectPatternProp>>}, ArrayPattern{elems: crate::ddlog_std::Vec<crate::internment::Intern<crate::ast::Pattern>>});
impl ::std::fmt::Display for Pattern {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::Pattern::SinglePattern{name} => {
                __formatter.write_str("ast::SinglePattern{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::Pattern::RestPattern{rest} => {
                __formatter.write_str("ast::RestPattern{")?;
                ::std::fmt::Debug::fmt(rest, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::Pattern::AssignPattern{key,value} => {
                __formatter.write_str("ast::AssignPattern{")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::Pattern::ObjectPattern{props} => {
                __formatter.write_str("ast::ObjectPattern{")?;
                ::std::fmt::Debug::fmt(props, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::Pattern::ArrayPattern{elems} => {
                __formatter.write_str("ast::ArrayPattern{")?;
                ::std::fmt::Debug::fmt(elems, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Pattern {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for Pattern {
    fn default() -> Self {
        crate::ast::Pattern::SinglePattern{name : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PropertyKey {
    ComputedKey {
        prop: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    LiteralKey {
        lit: crate::ast::ExprId
    },
    IdentKey {
        ident: crate::ast::Spanned<crate::ast::Name>
    }
}
impl abomonation::Abomonation for PropertyKey{}
::differential_datalog::decl_enum_from_record!(PropertyKey["ast::PropertyKey"]<>, ComputedKey["ast::ComputedKey"][1]{[0]prop["prop"]: crate::ddlog_std::Option<crate::ast::ExprId>}, LiteralKey["ast::LiteralKey"][1]{[0]lit["lit"]: crate::ast::ExprId}, IdentKey["ast::IdentKey"][1]{[0]ident["ident"]: crate::ast::Spanned<crate::ast::Name>});
::differential_datalog::decl_enum_into_record!(PropertyKey<>, ComputedKey["ast::ComputedKey"]{prop}, LiteralKey["ast::LiteralKey"]{lit}, IdentKey["ast::IdentKey"]{ident});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(PropertyKey<>, ComputedKey{prop: crate::ddlog_std::Option<crate::ast::ExprId>}, LiteralKey{lit: crate::ast::ExprId}, IdentKey{ident: crate::ast::Spanned<crate::ast::Name>});
impl ::std::fmt::Display for PropertyKey {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::PropertyKey::ComputedKey{prop} => {
                __formatter.write_str("ast::ComputedKey{")?;
                ::std::fmt::Debug::fmt(prop, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyKey::LiteralKey{lit} => {
                __formatter.write_str("ast::LiteralKey{")?;
                ::std::fmt::Debug::fmt(lit, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyKey::IdentKey{ident} => {
                __formatter.write_str("ast::IdentKey{")?;
                ::std::fmt::Debug::fmt(ident, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for PropertyKey {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for PropertyKey {
    fn default() -> Self {
        crate::ast::PropertyKey::ComputedKey{prop : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PropertyVal {
    PropLit {
        lit: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    PropGetter {
        body: crate::ddlog_std::Option<crate::ast::StmtId>
    },
    PropSetter {
        params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>
    },
    PropSpread {
        value: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    PropInit {
        value: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    PropIdent,
    PropMethod {
        params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>,
        body: crate::ddlog_std::Option<crate::ast::StmtId>
    }
}
impl abomonation::Abomonation for PropertyVal{}
::differential_datalog::decl_enum_from_record!(PropertyVal["ast::PropertyVal"]<>, PropLit["ast::PropLit"][1]{[0]lit["lit"]: crate::ddlog_std::Option<crate::ast::ExprId>}, PropGetter["ast::PropGetter"][1]{[0]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>}, PropSetter["ast::PropSetter"][1]{[0]params["params"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>}, PropSpread["ast::PropSpread"][1]{[0]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>}, PropInit["ast::PropInit"][1]{[0]value["value"]: crate::ddlog_std::Option<crate::ast::ExprId>}, PropIdent["ast::PropIdent"][0]{}, PropMethod["ast::PropMethod"][2]{[0]params["params"]: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>, [1]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_enum_into_record!(PropertyVal<>, PropLit["ast::PropLit"]{lit}, PropGetter["ast::PropGetter"]{body}, PropSetter["ast::PropSetter"]{params}, PropSpread["ast::PropSpread"]{value}, PropInit["ast::PropInit"]{value}, PropIdent["ast::PropIdent"]{}, PropMethod["ast::PropMethod"]{params, body});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(PropertyVal<>, PropLit{lit: crate::ddlog_std::Option<crate::ast::ExprId>}, PropGetter{body: crate::ddlog_std::Option<crate::ast::StmtId>}, PropSetter{params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>}, PropSpread{value: crate::ddlog_std::Option<crate::ast::ExprId>}, PropInit{value: crate::ddlog_std::Option<crate::ast::ExprId>}, PropIdent{}, PropMethod{params: crate::ddlog_std::Option<crate::ddlog_std::Vec<crate::ast::IPattern>>, body: crate::ddlog_std::Option<crate::ast::StmtId>});
impl ::std::fmt::Display for PropertyVal {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::PropertyVal::PropLit{lit} => {
                __formatter.write_str("ast::PropLit{")?;
                ::std::fmt::Debug::fmt(lit, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyVal::PropGetter{body} => {
                __formatter.write_str("ast::PropGetter{")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyVal::PropSetter{params} => {
                __formatter.write_str("ast::PropSetter{")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyVal::PropSpread{value} => {
                __formatter.write_str("ast::PropSpread{")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyVal::PropInit{value} => {
                __formatter.write_str("ast::PropInit{")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyVal::PropIdent{} => {
                __formatter.write_str("ast::PropIdent{")?;
                __formatter.write_str("}")
            },
            crate::ast::PropertyVal::PropMethod{params,body} => {
                __formatter.write_str("ast::PropMethod{")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for PropertyVal {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for PropertyVal {
    fn default() -> Self {
        crate::ast::PropertyVal::PropLit{lit : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ScopeId {
    pub id: u32,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for ScopeId{}
::differential_datalog::decl_struct_from_record!(ScopeId["ast::ScopeId"]<>, ["ast::ScopeId"][2]{[0]id["id"]: u32, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(ScopeId, ["ast::ScopeId"]<>, id, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ScopeId, <>, id: u32, file: crate::ast::FileId);
impl ::std::fmt::Display for ScopeId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::ScopeId{id,file} => {
                __formatter.write_str("ast::ScopeId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for ScopeId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end: u32
}
impl abomonation::Abomonation for Span{}
::differential_datalog::decl_struct_from_record!(Span["ast::Span"]<>, ["ast::Span"][2]{[0]start["start"]: u32, [1]end["end"]: u32});
::differential_datalog::decl_struct_into_record!(Span, ["ast::Span"]<>, start, end);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Span, <>, start: u32, end: u32);
impl ::std::fmt::Display for Span {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::Span{start,end} => {
                __formatter.write_str("ast::Span{")?;
                ::std::fmt::Debug::fmt(start, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(end, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for Span {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub data: T,
    pub span: crate::ast::Span
}
impl <T: crate::Val> abomonation::Abomonation for Spanned<T>{}
::differential_datalog::decl_struct_from_record!(Spanned["ast::Spanned"]<T>, ["ast::Spanned"][2]{[0]data["data"]: T, [1]span["span"]: crate::ast::Span});
::differential_datalog::decl_struct_into_record!(Spanned, ["ast::Spanned"]<T>, data, span);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Spanned, <T>, data: T, span: crate::ast::Span);
impl <T: ::std::fmt::Debug> ::std::fmt::Display for Spanned<T> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::Spanned{data,span} => {
                __formatter.write_str("ast::Spanned{")?;
                ::std::fmt::Debug::fmt(data, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl <T: ::std::fmt::Debug> ::std::fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct StmtId {
    pub id: u32,
    pub file: crate::ast::FileId
}
impl abomonation::Abomonation for StmtId{}
::differential_datalog::decl_struct_from_record!(StmtId["ast::StmtId"]<>, ["ast::StmtId"][2]{[0]id["id"]: u32, [1]file["file"]: crate::ast::FileId});
::differential_datalog::decl_struct_into_record!(StmtId, ["ast::StmtId"]<>, id, file);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(StmtId, <>, id: u32, file: crate::ast::FileId);
impl ::std::fmt::Display for StmtId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::StmtId{id,file} => {
                __formatter.write_str("ast::StmtId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for StmtId {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum StmtKind {
    StmtVarDecl,
    StmtLetDecl,
    StmtConstDecl,
    StmtExpr {
        expr_id: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    StmtReturn,
    StmtIf,
    StmtBreak,
    StmtDoWhile,
    StmtWhile,
    StmtFor,
    StmtForIn,
    StmtContinue,
    StmtWith,
    StmtLabel,
    StmtSwitch,
    StmtThrow,
    StmtTry,
    StmtDebugger,
    StmtEmpty
}
impl abomonation::Abomonation for StmtKind{}
::differential_datalog::decl_enum_from_record!(StmtKind["ast::StmtKind"]<>, StmtVarDecl["ast::StmtVarDecl"][0]{}, StmtLetDecl["ast::StmtLetDecl"][0]{}, StmtConstDecl["ast::StmtConstDecl"][0]{}, StmtExpr["ast::StmtExpr"][1]{[0]expr_id["expr_id"]: crate::ddlog_std::Option<crate::ast::ExprId>}, StmtReturn["ast::StmtReturn"][0]{}, StmtIf["ast::StmtIf"][0]{}, StmtBreak["ast::StmtBreak"][0]{}, StmtDoWhile["ast::StmtDoWhile"][0]{}, StmtWhile["ast::StmtWhile"][0]{}, StmtFor["ast::StmtFor"][0]{}, StmtForIn["ast::StmtForIn"][0]{}, StmtContinue["ast::StmtContinue"][0]{}, StmtWith["ast::StmtWith"][0]{}, StmtLabel["ast::StmtLabel"][0]{}, StmtSwitch["ast::StmtSwitch"][0]{}, StmtThrow["ast::StmtThrow"][0]{}, StmtTry["ast::StmtTry"][0]{}, StmtDebugger["ast::StmtDebugger"][0]{}, StmtEmpty["ast::StmtEmpty"][0]{});
::differential_datalog::decl_enum_into_record!(StmtKind<>, StmtVarDecl["ast::StmtVarDecl"]{}, StmtLetDecl["ast::StmtLetDecl"]{}, StmtConstDecl["ast::StmtConstDecl"]{}, StmtExpr["ast::StmtExpr"]{expr_id}, StmtReturn["ast::StmtReturn"]{}, StmtIf["ast::StmtIf"]{}, StmtBreak["ast::StmtBreak"]{}, StmtDoWhile["ast::StmtDoWhile"]{}, StmtWhile["ast::StmtWhile"]{}, StmtFor["ast::StmtFor"]{}, StmtForIn["ast::StmtForIn"]{}, StmtContinue["ast::StmtContinue"]{}, StmtWith["ast::StmtWith"]{}, StmtLabel["ast::StmtLabel"]{}, StmtSwitch["ast::StmtSwitch"]{}, StmtThrow["ast::StmtThrow"]{}, StmtTry["ast::StmtTry"]{}, StmtDebugger["ast::StmtDebugger"]{}, StmtEmpty["ast::StmtEmpty"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(StmtKind<>, StmtVarDecl{}, StmtLetDecl{}, StmtConstDecl{}, StmtExpr{expr_id: crate::ddlog_std::Option<crate::ast::ExprId>}, StmtReturn{}, StmtIf{}, StmtBreak{}, StmtDoWhile{}, StmtWhile{}, StmtFor{}, StmtForIn{}, StmtContinue{}, StmtWith{}, StmtLabel{}, StmtSwitch{}, StmtThrow{}, StmtTry{}, StmtDebugger{}, StmtEmpty{});
impl ::std::fmt::Display for StmtKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::StmtKind::StmtVarDecl{} => {
                __formatter.write_str("ast::StmtVarDecl{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtLetDecl{} => {
                __formatter.write_str("ast::StmtLetDecl{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtConstDecl{} => {
                __formatter.write_str("ast::StmtConstDecl{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtExpr{expr_id} => {
                __formatter.write_str("ast::StmtExpr{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtReturn{} => {
                __formatter.write_str("ast::StmtReturn{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtIf{} => {
                __formatter.write_str("ast::StmtIf{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtBreak{} => {
                __formatter.write_str("ast::StmtBreak{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtDoWhile{} => {
                __formatter.write_str("ast::StmtDoWhile{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtWhile{} => {
                __formatter.write_str("ast::StmtWhile{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtFor{} => {
                __formatter.write_str("ast::StmtFor{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtForIn{} => {
                __formatter.write_str("ast::StmtForIn{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtContinue{} => {
                __formatter.write_str("ast::StmtContinue{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtWith{} => {
                __formatter.write_str("ast::StmtWith{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtLabel{} => {
                __formatter.write_str("ast::StmtLabel{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtSwitch{} => {
                __formatter.write_str("ast::StmtSwitch{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtThrow{} => {
                __formatter.write_str("ast::StmtThrow{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtTry{} => {
                __formatter.write_str("ast::StmtTry{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtDebugger{} => {
                __formatter.write_str("ast::StmtDebugger{")?;
                __formatter.write_str("}")
            },
            crate::ast::StmtKind::StmtEmpty{} => {
                __formatter.write_str("ast::StmtEmpty{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for StmtKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for StmtKind {
    fn default() -> Self {
        crate::ast::StmtKind::StmtVarDecl{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SwitchClause {
    CaseClause {
        test: crate::ddlog_std::Option<crate::ast::ExprId>
    },
    DefaultClause
}
impl abomonation::Abomonation for SwitchClause{}
::differential_datalog::decl_enum_from_record!(SwitchClause["ast::SwitchClause"]<>, CaseClause["ast::CaseClause"][1]{[0]test["test"]: crate::ddlog_std::Option<crate::ast::ExprId>}, DefaultClause["ast::DefaultClause"][0]{});
::differential_datalog::decl_enum_into_record!(SwitchClause<>, CaseClause["ast::CaseClause"]{test}, DefaultClause["ast::DefaultClause"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(SwitchClause<>, CaseClause{test: crate::ddlog_std::Option<crate::ast::ExprId>}, DefaultClause{});
impl ::std::fmt::Display for SwitchClause {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::SwitchClause::CaseClause{test} => {
                __formatter.write_str("ast::CaseClause{")?;
                ::std::fmt::Debug::fmt(test, __formatter)?;
                __formatter.write_str("}")
            },
            crate::ast::SwitchClause::DefaultClause{} => {
                __formatter.write_str("ast::DefaultClause{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for SwitchClause {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for SwitchClause {
    fn default() -> Self {
        crate::ast::SwitchClause::CaseClause{test : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct TryHandler {
    pub error: crate::ddlog_std::Option<crate::ast::IPattern>,
    pub body: crate::ddlog_std::Option<crate::ast::StmtId>
}
impl abomonation::Abomonation for TryHandler{}
::differential_datalog::decl_struct_from_record!(TryHandler["ast::TryHandler"]<>, ["ast::TryHandler"][2]{[0]error["error"]: crate::ddlog_std::Option<crate::ast::IPattern>, [1]body["body"]: crate::ddlog_std::Option<crate::ast::StmtId>});
::differential_datalog::decl_struct_into_record!(TryHandler, ["ast::TryHandler"]<>, error, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(TryHandler, <>, error: crate::ddlog_std::Option<crate::ast::IPattern>, body: crate::ddlog_std::Option<crate::ast::StmtId>);
impl ::std::fmt::Display for TryHandler {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::TryHandler{error,body} => {
                __formatter.write_str("ast::TryHandler{")?;
                ::std::fmt::Debug::fmt(error, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for TryHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum UnaryOperand {
    UnaryIncrement,
    UnaryDecrement,
    UnaryDelete,
    UnaryVoid,
    UnaryTypeof,
    UnaryPlus,
    UnaryMinus,
    UnaryBitwiseNot,
    UnaryLogicalNot,
    UnaryAwait
}
impl abomonation::Abomonation for UnaryOperand{}
::differential_datalog::decl_enum_from_record!(UnaryOperand["ast::UnaryOperand"]<>, UnaryIncrement["ast::UnaryIncrement"][0]{}, UnaryDecrement["ast::UnaryDecrement"][0]{}, UnaryDelete["ast::UnaryDelete"][0]{}, UnaryVoid["ast::UnaryVoid"][0]{}, UnaryTypeof["ast::UnaryTypeof"][0]{}, UnaryPlus["ast::UnaryPlus"][0]{}, UnaryMinus["ast::UnaryMinus"][0]{}, UnaryBitwiseNot["ast::UnaryBitwiseNot"][0]{}, UnaryLogicalNot["ast::UnaryLogicalNot"][0]{}, UnaryAwait["ast::UnaryAwait"][0]{});
::differential_datalog::decl_enum_into_record!(UnaryOperand<>, UnaryIncrement["ast::UnaryIncrement"]{}, UnaryDecrement["ast::UnaryDecrement"]{}, UnaryDelete["ast::UnaryDelete"]{}, UnaryVoid["ast::UnaryVoid"]{}, UnaryTypeof["ast::UnaryTypeof"]{}, UnaryPlus["ast::UnaryPlus"]{}, UnaryMinus["ast::UnaryMinus"]{}, UnaryBitwiseNot["ast::UnaryBitwiseNot"]{}, UnaryLogicalNot["ast::UnaryLogicalNot"]{}, UnaryAwait["ast::UnaryAwait"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(UnaryOperand<>, UnaryIncrement{}, UnaryDecrement{}, UnaryDelete{}, UnaryVoid{}, UnaryTypeof{}, UnaryPlus{}, UnaryMinus{}, UnaryBitwiseNot{}, UnaryLogicalNot{}, UnaryAwait{});
impl ::std::fmt::Display for UnaryOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            crate::ast::UnaryOperand::UnaryIncrement{} => {
                __formatter.write_str("ast::UnaryIncrement{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryDecrement{} => {
                __formatter.write_str("ast::UnaryDecrement{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryDelete{} => {
                __formatter.write_str("ast::UnaryDelete{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryVoid{} => {
                __formatter.write_str("ast::UnaryVoid{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryTypeof{} => {
                __formatter.write_str("ast::UnaryTypeof{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryPlus{} => {
                __formatter.write_str("ast::UnaryPlus{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryMinus{} => {
                __formatter.write_str("ast::UnaryMinus{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryBitwiseNot{} => {
                __formatter.write_str("ast::UnaryBitwiseNot{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryLogicalNot{} => {
                __formatter.write_str("ast::UnaryLogicalNot{")?;
                __formatter.write_str("}")
            },
            crate::ast::UnaryOperand::UnaryAwait{} => {
                __formatter.write_str("ast::UnaryAwait{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for UnaryOperand {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for UnaryOperand {
    fn default() -> Self {
        crate::ast::UnaryOperand::UnaryIncrement{}
    }
}
pub fn bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat: & crate::ast::IPattern) -> crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>>
{   match (*crate::internment::ival(pat)) {
        crate::ast::Pattern::SinglePattern{name: crate::ddlog_std::Option::Some{x: ref name}} => {
                                                                                                     let ref mut __vec: crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>> = (*(&*crate::__STATIC_0)).clone();
                                                                                                     crate::ddlog_std::push::<crate::ast::Spanned<crate::ast::Name>>(__vec, name);
                                                                                                     (*__vec).clone()
                                                                                                 },
        crate::ast::Pattern::RestPattern{rest: crate::ddlog_std::Option::Some{x: ref rest}} => crate::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(rest),
        crate::ast::Pattern::AssignPattern{key: crate::ddlog_std::Option::Some{x: ref key}, value: _} => crate::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(key),
        crate::ast::Pattern::ObjectPattern{props: ref props} => crate::vec::flatmap::<crate::internment::Intern<crate::ast::ObjectPatternProp>, crate::ast::Spanned<crate::ast::Name>>(props, (&{
                                                                                                                                                                                                    (Box::new(closure::ClosureImpl{
                                                                                                                                                                                                        description: "(function(prop: internment::Intern<ast::ObjectPatternProp>):ddlog_std::Vec<ast::Spanned<ast::Name>>{(ast::bound_vars(prop))})",
                                                                                                                                                                                                        captured: (),
                                                                                                                                                                                                        f: {
                                                                                                                                                                                                               fn __f(__args:*const crate::internment::Intern<crate::ast::ObjectPatternProp>, __captured: &()) -> crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>>
                                                                                                                                                                                                               {
                                                                                                                                                                                                                   let prop = unsafe{&*__args};
                                                                                                                                                                                                                   crate::ast::bound_vars_internment_Intern__ast_ObjectPatternProp_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(prop)
                                                                                                                                                                                                               }
                                                                                                                                                                                                               __f
                                                                                                                                                                                                           }
                                                                                                                                                                                                    }) as Box<dyn closure::Closure<(*const crate::internment::Intern<crate::ast::ObjectPatternProp>), crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>>>>)
                                                                                                                                                                                                })),
        crate::ast::Pattern::ArrayPattern{elems: ref elems} => crate::vec::flatmap::<crate::internment::Intern<crate::ast::Pattern>, crate::ast::Spanned<crate::ast::Name>>(elems, (&{
                                                                                                                                                                                         (Box::new(closure::ClosureImpl{
                                                                                                                                                                                             description: "(function(elem: internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>{(ast::bound_vars(elem))})",
                                                                                                                                                                                             captured: (),
                                                                                                                                                                                             f: {
                                                                                                                                                                                                    fn __f(__args:*const crate::internment::Intern<crate::ast::Pattern>, __captured: &()) -> crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>>
                                                                                                                                                                                                    {
                                                                                                                                                                                                        let elem = unsafe{&*__args};
                                                                                                                                                                                                        crate::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(elem)
                                                                                                                                                                                                    }
                                                                                                                                                                                                    __f
                                                                                                                                                                                                }
                                                                                                                                                                                         }) as Box<dyn closure::Closure<(*const crate::internment::Intern<crate::ast::Pattern>), crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>>>>)
                                                                                                                                                                                     })),
        _ => (*(&*crate::__STATIC_1)).clone()
    }
}
pub fn bound_vars_internment_Intern__ast_ObjectPatternProp_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat: & crate::ast::IObjectPatternProp) -> crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>>
{   match (*crate::internment::ival(pat)) {
        crate::ast::ObjectPatternProp::ObjAssignPattern{assign_key: crate::ddlog_std::Option::Some{x: ref key}, assign_value: _} => crate::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(key),
        crate::ast::ObjectPatternProp::ObjKeyValuePattern{key: _, value: crate::ddlog_std::Option::Some{x: ref value}} => crate::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(value),
        crate::ast::ObjectPatternProp::ObjRestPattern{rest: crate::ddlog_std::Option::Some{x: ref rest}} => crate::ast::bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(rest),
        crate::ast::ObjectPatternProp::ObjSinglePattern{name: crate::ddlog_std::Option::Some{x: ref name}} => {
                                                                                                                  let ref mut __vec: crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>> = (*(&*crate::__STATIC_0)).clone();
                                                                                                                  crate::ddlog_std::push::<crate::ast::Spanned<crate::ast::Name>>(__vec, name);
                                                                                                                  (*__vec).clone()
                                                                                                              },
        _ => (*(&*crate::__STATIC_1)).clone()
    }
}
pub fn file_id(id: & crate::ast::AnyId) -> crate::ast::FileId
{   match (*id) {
        crate::ast::AnyId::AnyIdGlobal{global: crate::ast::GlobalId{id: _, file: ref file}} => (*file).clone(),
        crate::ast::AnyId::AnyIdImport{import_: crate::ast::ImportId{id: _, file: ref file}} => (*file).clone(),
        crate::ast::AnyId::AnyIdClass{class: crate::ast::ClassId{id: _, file: ref file}} => (*file).clone(),
        crate::ast::AnyId::AnyIdFunc{func: crate::ast::FuncId{id: _, file: ref file}} => (*file).clone(),
        crate::ast::AnyId::AnyIdStmt{stmt: crate::ast::StmtId{id: _, file: ref file}} => (*file).clone(),
        crate::ast::AnyId::AnyIdExpr{expr: crate::ast::ExprId{id: _, file: ref file}} => (*file).clone()
    }
}
pub fn free_variable(clause: & crate::ast::NamedImport) -> crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
{   crate::utils::or_else::<crate::ast::Spanned<crate::ast::Name>>((&clause.alias), (&clause.name))
}
pub fn free_variables(clause: & crate::ast::ImportClause) -> crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>>
{   match (*clause) {
        crate::ast::ImportClause::WildcardImport{alias: crate::ddlog_std::Option::Some{x: ref alias}} => {
                                                                                                             let ref mut __vec: crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>> = (*(&*crate::__STATIC_0)).clone();
                                                                                                             crate::ddlog_std::push::<crate::ast::Spanned<crate::ast::Name>>(__vec, alias);
                                                                                                             (*__vec).clone()
                                                                                                         },
        crate::ast::ImportClause::GroupedImport{imports: ref imports} => crate::vec::filter_map::<crate::ast::NamedImport, crate::ast::Spanned<crate::ast::Name>>(imports, (&{
                                                                                                                                                                                 (Box::new(closure::ClosureImpl{
                                                                                                                                                                                     description: "(function(named: ast::NamedImport):ddlog_std::Option<ast::Spanned<ast::Name>>{(ast::free_variable(named))})",
                                                                                                                                                                                     captured: (),
                                                                                                                                                                                     f: {
                                                                                                                                                                                            fn __f(__args:*const crate::ast::NamedImport, __captured: &()) -> crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>
                                                                                                                                                                                            {
                                                                                                                                                                                                let named = unsafe{&*__args};
                                                                                                                                                                                                crate::ast::free_variable(named)
                                                                                                                                                                                            }
                                                                                                                                                                                            __f
                                                                                                                                                                                        }
                                                                                                                                                                                 }) as Box<dyn closure::Closure<(*const crate::ast::NamedImport), crate::ddlog_std::Option<crate::ast::Spanned<crate::ast::Name>>>>)
                                                                                                                                                                             })),
        crate::ast::ImportClause::SingleImport{name: ref name} => {
                                                                      let ref mut __vec: crate::ddlog_std::Vec<crate::ast::Spanned<crate::ast::Name>> = (*(&*crate::__STATIC_0)).clone();
                                                                      crate::ddlog_std::push::<crate::ast::Spanned<crate::ast::Name>>(__vec, name);
                                                                      (*__vec).clone()
                                                                  },
        _ => (*(&*crate::__STATIC_1)).clone()
    }
}
pub fn is_expr(id: & crate::ast::AnyId) -> bool
{   match (*id) {
        crate::ast::AnyId::AnyIdExpr{expr: _} => true,
        _ => false
    }
}
pub fn is_global(id: & crate::ast::AnyId) -> bool
{   match (*id) {
        crate::ast::AnyId::AnyIdGlobal{global: _} => true,
        _ => false
    }
}
pub fn is_variable_decl(kind: & crate::ast::StmtKind) -> bool
{   ((((&*kind) == (&*(&(crate::ast::StmtKind::StmtVarDecl{})))) || ((&*kind) == (&*(&(crate::ast::StmtKind::StmtLetDecl{}))))) || ((&*kind) == (&*(&(crate::ast::StmtKind::StmtConstDecl{})))))
}
pub fn to_string_ast_ScopeId___Stringval(scope: & crate::ast::ScopeId) -> String
{   string_append(String::from(r###"Scope_"###), (&crate::ddlog_std::__builtin_2string((&scope.id))))
}
pub fn to_string_ast_AnyId___Stringval(id: & crate::ast::AnyId) -> String
{   match (*id) {
        crate::ast::AnyId::AnyIdGlobal{global: crate::ast::GlobalId{id: ref id, file: _}} => string_append(String::from(r###"Global_"###), (&crate::ddlog_std::__builtin_2string(id))),
        crate::ast::AnyId::AnyIdImport{import_: crate::ast::ImportId{id: ref id, file: _}} => string_append(String::from(r###"Import_"###), (&crate::ddlog_std::__builtin_2string(id))),
        crate::ast::AnyId::AnyIdClass{class: crate::ast::ClassId{id: ref id, file: _}} => string_append(String::from(r###"Class_"###), (&crate::ddlog_std::__builtin_2string(id))),
        crate::ast::AnyId::AnyIdFunc{func: crate::ast::FuncId{id: ref id, file: _}} => string_append(String::from(r###"Func_"###), (&crate::ddlog_std::__builtin_2string(id))),
        crate::ast::AnyId::AnyIdStmt{stmt: crate::ast::StmtId{id: ref id, file: _}} => string_append(String::from(r###"Stmt_"###), (&crate::ddlog_std::__builtin_2string(id))),
        crate::ast::AnyId::AnyIdExpr{expr: crate::ast::ExprId{id: ref id, file: _}} => string_append(String::from(r###"Expr_"###), (&crate::ddlog_std::__builtin_2string(id)))
    }
}
pub fn to_string_ast_Span___Stringval(span: & crate::ast::Span) -> String
{   string_append_str(string_append(string_append_str(string_append(String::from(r###"("###), (&crate::ddlog_std::__builtin_2string((&span.start)))), r###", "###), (&crate::ddlog_std::__builtin_2string((&span.end)))), r###")"###)
}