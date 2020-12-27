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

//use ::serde::de::DeserializeOwned;
use ::differential_datalog::ddval::DDValConvert;
use ::differential_datalog::ddval::DDValue;
use ::differential_datalog::program;
use ::differential_datalog::program::TupleTS;
use ::differential_datalog::program::Weight;
use ::differential_datalog::program::XFormArrangement;
use ::differential_datalog::program::XFormCollection;
use ::differential_datalog::record::FromRecord;
use ::differential_datalog::record::IntoRecord;
use ::differential_datalog::record::Mutator;
use ::serde::Deserialize;
use ::serde::Serialize;

// `usize` and `isize` are builtin Rust types; we therefore declare an alias to DDlog's `usize` and
// `isize`.
pub type std_usize = u64;
pub type std_isize = i64;

pub static __STATIC_1: ::once_cell::sync::Lazy<ddlog_std::Vec<Spanned<Name>>> =
    ::once_cell::sync::Lazy::new(|| ddlog_std::vec_empty());
pub static __STATIC_0: ::once_cell::sync::Lazy<ddlog_std::Vec<Spanned<Name>>> =
    ::once_cell::sync::Lazy::new(|| ddlog_std::vec_with_capacity((&(1 as u64))));
use rslint_parser::{
    ast::{AssignOp as AstAssignOp, BinOp as AstBinOp, UnaryOp as AstUnaryOp},
    TextRange,
};
use std::{
    cell::Cell,
    ops::{Add, AddAssign, Range},
};

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
                pub const fn new(id: u32) -> Self {
                    Self { id }
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
                    Self {
                        id: self.id + other.id,
                    }
                }
            }

            impl Add<u32> for $ty {
                type Output = Self;

                fn add(self, other: u32) -> Self {
                    Self {
                        id: self.id + other,
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
    FileId,
    ScopeId,
    GlobalId,
    ImportId,
    ClassId,
    FuncId,
    StmtId,
    ExprId,
}

impl FuncParam {
    pub const fn new(pattern: IPattern, implicit: bool) -> Self {
        Self { pattern, implicit }
    }

    pub const fn explicit(pattern: IPattern) -> Self {
        Self::new(pattern, false)
    }

    pub const fn implicit(pattern: IPattern) -> Self {
        Self::new(pattern, true)
    }
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
    AnyIdGlobal { global: GlobalId },
    AnyIdImport { import_: ImportId },
    AnyIdClass { class: ClassId },
    AnyIdFunc { func: FuncId },
    AnyIdStmt { stmt: StmtId },
    AnyIdExpr { expr: ExprId },
    AnyIdFile { file: FileId },
}
impl abomonation::Abomonation for AnyId {}
::differential_datalog::decl_enum_from_record!(AnyId["ast::AnyId"]<>, AnyIdGlobal["ast::AnyIdGlobal"][1]{[0]global["global"]: GlobalId}, AnyIdImport["ast::AnyIdImport"][1]{[0]import_["import_"]: ImportId}, AnyIdClass["ast::AnyIdClass"][1]{[0]class["class"]: ClassId}, AnyIdFunc["ast::AnyIdFunc"][1]{[0]func["func"]: FuncId}, AnyIdStmt["ast::AnyIdStmt"][1]{[0]stmt["stmt"]: StmtId}, AnyIdExpr["ast::AnyIdExpr"][1]{[0]expr["expr"]: ExprId}, AnyIdFile["ast::AnyIdFile"][1]{[0]file["file"]: FileId});
::differential_datalog::decl_enum_into_record!(AnyId<>, AnyIdGlobal["ast::AnyIdGlobal"]{global}, AnyIdImport["ast::AnyIdImport"]{import_}, AnyIdClass["ast::AnyIdClass"]{class}, AnyIdFunc["ast::AnyIdFunc"]{func}, AnyIdStmt["ast::AnyIdStmt"]{stmt}, AnyIdExpr["ast::AnyIdExpr"]{expr}, AnyIdFile["ast::AnyIdFile"]{file});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(AnyId<>, AnyIdGlobal{global: GlobalId}, AnyIdImport{import_: ImportId}, AnyIdClass{class: ClassId}, AnyIdFunc{func: FuncId}, AnyIdStmt{stmt: StmtId}, AnyIdExpr{expr: ExprId}, AnyIdFile{file: FileId});
impl ::std::fmt::Display for AnyId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            AnyId::AnyIdGlobal { global } => {
                __formatter.write_str("ast::AnyIdGlobal{")?;
                ::std::fmt::Debug::fmt(global, __formatter)?;
                __formatter.write_str("}")
            }
            AnyId::AnyIdImport { import_ } => {
                __formatter.write_str("ast::AnyIdImport{")?;
                ::std::fmt::Debug::fmt(import_, __formatter)?;
                __formatter.write_str("}")
            }
            AnyId::AnyIdClass { class } => {
                __formatter.write_str("ast::AnyIdClass{")?;
                ::std::fmt::Debug::fmt(class, __formatter)?;
                __formatter.write_str("}")
            }
            AnyId::AnyIdFunc { func } => {
                __formatter.write_str("ast::AnyIdFunc{")?;
                ::std::fmt::Debug::fmt(func, __formatter)?;
                __formatter.write_str("}")
            }
            AnyId::AnyIdStmt { stmt } => {
                __formatter.write_str("ast::AnyIdStmt{")?;
                ::std::fmt::Debug::fmt(stmt, __formatter)?;
                __formatter.write_str("}")
            }
            AnyId::AnyIdExpr { expr } => {
                __formatter.write_str("ast::AnyIdExpr{")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            }
            AnyId::AnyIdFile { file } => {
                __formatter.write_str("ast::AnyIdFile{")?;
                ::std::fmt::Debug::fmt(file, __formatter)?;
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
        AnyId::AnyIdGlobal {
            global: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ArrayElement {
    ArrExpr { expr: ExprId },
    ArrSpread { spread: ddlog_std::Option<ExprId> },
}
impl abomonation::Abomonation for ArrayElement {}
::differential_datalog::decl_enum_from_record!(ArrayElement["ast::ArrayElement"]<>, ArrExpr["ast::ArrExpr"][1]{[0]expr["expr"]: ExprId}, ArrSpread["ast::ArrSpread"][1]{[0]spread["spread"]: ddlog_std::Option<ExprId>});
::differential_datalog::decl_enum_into_record!(ArrayElement<>, ArrExpr["ast::ArrExpr"]{expr}, ArrSpread["ast::ArrSpread"]{spread});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ArrayElement<>, ArrExpr{expr: ExprId}, ArrSpread{spread: ddlog_std::Option<ExprId>});
impl ::std::fmt::Display for ArrayElement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ArrayElement::ArrExpr { expr } => {
                __formatter.write_str("ast::ArrExpr{")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            }
            ArrayElement::ArrSpread { spread } => {
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
        ArrayElement::ArrExpr {
            expr: ::std::default::Default::default(),
        }
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
    OpNullishCoalescingAssign,
}
impl abomonation::Abomonation for AssignOperand {}
::differential_datalog::decl_enum_from_record!(AssignOperand["ast::AssignOperand"]<>, OpAssign["ast::OpAssign"][0]{}, OpAddAssign["ast::OpAddAssign"][0]{}, OpSubtractAssign["ast::OpSubtractAssign"][0]{}, OpTimesAssign["ast::OpTimesAssign"][0]{}, OpRemainderAssign["ast::OpRemainderAssign"][0]{}, OpExponentAssign["ast::OpExponentAssign"][0]{}, OpLeftShiftAssign["ast::OpLeftShiftAssign"][0]{}, OpRightShiftAssign["ast::OpRightShiftAssign"][0]{}, OpUnsignedRightShiftAssign["ast::OpUnsignedRightShiftAssign"][0]{}, OpBitwiseAndAssign["ast::OpBitwiseAndAssign"][0]{}, OpBitwiseOrAssign["ast::OpBitwiseOrAssign"][0]{}, OpBitwiseXorAssign["ast::OpBitwiseXorAssign"][0]{}, OpLogicalAndAssign["ast::OpLogicalAndAssign"][0]{}, OpLogicalOrAssign["ast::OpLogicalOrAssign"][0]{}, OpNullishCoalescingAssign["ast::OpNullishCoalescingAssign"][0]{});
::differential_datalog::decl_enum_into_record!(AssignOperand<>, OpAssign["ast::OpAssign"]{}, OpAddAssign["ast::OpAddAssign"]{}, OpSubtractAssign["ast::OpSubtractAssign"]{}, OpTimesAssign["ast::OpTimesAssign"]{}, OpRemainderAssign["ast::OpRemainderAssign"]{}, OpExponentAssign["ast::OpExponentAssign"]{}, OpLeftShiftAssign["ast::OpLeftShiftAssign"]{}, OpRightShiftAssign["ast::OpRightShiftAssign"]{}, OpUnsignedRightShiftAssign["ast::OpUnsignedRightShiftAssign"]{}, OpBitwiseAndAssign["ast::OpBitwiseAndAssign"]{}, OpBitwiseOrAssign["ast::OpBitwiseOrAssign"]{}, OpBitwiseXorAssign["ast::OpBitwiseXorAssign"]{}, OpLogicalAndAssign["ast::OpLogicalAndAssign"]{}, OpLogicalOrAssign["ast::OpLogicalOrAssign"]{}, OpNullishCoalescingAssign["ast::OpNullishCoalescingAssign"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(AssignOperand<>, OpAssign{}, OpAddAssign{}, OpSubtractAssign{}, OpTimesAssign{}, OpRemainderAssign{}, OpExponentAssign{}, OpLeftShiftAssign{}, OpRightShiftAssign{}, OpUnsignedRightShiftAssign{}, OpBitwiseAndAssign{}, OpBitwiseOrAssign{}, OpBitwiseXorAssign{}, OpLogicalAndAssign{}, OpLogicalOrAssign{}, OpNullishCoalescingAssign{});
impl ::std::fmt::Display for AssignOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            AssignOperand::OpAssign {} => {
                __formatter.write_str("ast::OpAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpAddAssign {} => {
                __formatter.write_str("ast::OpAddAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpSubtractAssign {} => {
                __formatter.write_str("ast::OpSubtractAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpTimesAssign {} => {
                __formatter.write_str("ast::OpTimesAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpRemainderAssign {} => {
                __formatter.write_str("ast::OpRemainderAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpExponentAssign {} => {
                __formatter.write_str("ast::OpExponentAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpLeftShiftAssign {} => {
                __formatter.write_str("ast::OpLeftShiftAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpRightShiftAssign {} => {
                __formatter.write_str("ast::OpRightShiftAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpUnsignedRightShiftAssign {} => {
                __formatter.write_str("ast::OpUnsignedRightShiftAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpBitwiseAndAssign {} => {
                __formatter.write_str("ast::OpBitwiseAndAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpBitwiseOrAssign {} => {
                __formatter.write_str("ast::OpBitwiseOrAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpBitwiseXorAssign {} => {
                __formatter.write_str("ast::OpBitwiseXorAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpLogicalAndAssign {} => {
                __formatter.write_str("ast::OpLogicalAndAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpLogicalOrAssign {} => {
                __formatter.write_str("ast::OpLogicalOrAssign{")?;
                __formatter.write_str("}")
            }
            AssignOperand::OpNullishCoalescingAssign {} => {
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
        AssignOperand::OpAssign {}
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
    BinInstanceof,
}
impl abomonation::Abomonation for BinOperand {}
::differential_datalog::decl_enum_from_record!(BinOperand["ast::BinOperand"]<>, BinLessThan["ast::BinLessThan"][0]{}, BinGreaterThan["ast::BinGreaterThan"][0]{}, BinLessThanOrEqual["ast::BinLessThanOrEqual"][0]{}, BinGreaterThanOrEqual["ast::BinGreaterThanOrEqual"][0]{}, BinEquality["ast::BinEquality"][0]{}, BinStrictEquality["ast::BinStrictEquality"][0]{}, BinInequality["ast::BinInequality"][0]{}, BinStrictInequality["ast::BinStrictInequality"][0]{}, BinPlus["ast::BinPlus"][0]{}, BinMinus["ast::BinMinus"][0]{}, BinTimes["ast::BinTimes"][0]{}, BinDivide["ast::BinDivide"][0]{}, BinRemainder["ast::BinRemainder"][0]{}, BinExponent["ast::BinExponent"][0]{}, BinLeftShift["ast::BinLeftShift"][0]{}, BinRightShift["ast::BinRightShift"][0]{}, BinUnsignedRightShift["ast::BinUnsignedRightShift"][0]{}, BinBitwiseAnd["ast::BinBitwiseAnd"][0]{}, BinBitwiseOr["ast::BinBitwiseOr"][0]{}, BinBitwiseXor["ast::BinBitwiseXor"][0]{}, BinNullishCoalescing["ast::BinNullishCoalescing"][0]{}, BinLogicalOr["ast::BinLogicalOr"][0]{}, BinLogicalAnd["ast::BinLogicalAnd"][0]{}, BinIn["ast::BinIn"][0]{}, BinInstanceof["ast::BinInstanceof"][0]{});
::differential_datalog::decl_enum_into_record!(BinOperand<>, BinLessThan["ast::BinLessThan"]{}, BinGreaterThan["ast::BinGreaterThan"]{}, BinLessThanOrEqual["ast::BinLessThanOrEqual"]{}, BinGreaterThanOrEqual["ast::BinGreaterThanOrEqual"]{}, BinEquality["ast::BinEquality"]{}, BinStrictEquality["ast::BinStrictEquality"]{}, BinInequality["ast::BinInequality"]{}, BinStrictInequality["ast::BinStrictInequality"]{}, BinPlus["ast::BinPlus"]{}, BinMinus["ast::BinMinus"]{}, BinTimes["ast::BinTimes"]{}, BinDivide["ast::BinDivide"]{}, BinRemainder["ast::BinRemainder"]{}, BinExponent["ast::BinExponent"]{}, BinLeftShift["ast::BinLeftShift"]{}, BinRightShift["ast::BinRightShift"]{}, BinUnsignedRightShift["ast::BinUnsignedRightShift"]{}, BinBitwiseAnd["ast::BinBitwiseAnd"]{}, BinBitwiseOr["ast::BinBitwiseOr"]{}, BinBitwiseXor["ast::BinBitwiseXor"]{}, BinNullishCoalescing["ast::BinNullishCoalescing"]{}, BinLogicalOr["ast::BinLogicalOr"]{}, BinLogicalAnd["ast::BinLogicalAnd"]{}, BinIn["ast::BinIn"]{}, BinInstanceof["ast::BinInstanceof"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(BinOperand<>, BinLessThan{}, BinGreaterThan{}, BinLessThanOrEqual{}, BinGreaterThanOrEqual{}, BinEquality{}, BinStrictEquality{}, BinInequality{}, BinStrictInequality{}, BinPlus{}, BinMinus{}, BinTimes{}, BinDivide{}, BinRemainder{}, BinExponent{}, BinLeftShift{}, BinRightShift{}, BinUnsignedRightShift{}, BinBitwiseAnd{}, BinBitwiseOr{}, BinBitwiseXor{}, BinNullishCoalescing{}, BinLogicalOr{}, BinLogicalAnd{}, BinIn{}, BinInstanceof{});
impl ::std::fmt::Display for BinOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            BinOperand::BinLessThan {} => {
                __formatter.write_str("ast::BinLessThan{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinGreaterThan {} => {
                __formatter.write_str("ast::BinGreaterThan{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinLessThanOrEqual {} => {
                __formatter.write_str("ast::BinLessThanOrEqual{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinGreaterThanOrEqual {} => {
                __formatter.write_str("ast::BinGreaterThanOrEqual{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinEquality {} => {
                __formatter.write_str("ast::BinEquality{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinStrictEquality {} => {
                __formatter.write_str("ast::BinStrictEquality{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinInequality {} => {
                __formatter.write_str("ast::BinInequality{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinStrictInequality {} => {
                __formatter.write_str("ast::BinStrictInequality{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinPlus {} => {
                __formatter.write_str("ast::BinPlus{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinMinus {} => {
                __formatter.write_str("ast::BinMinus{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinTimes {} => {
                __formatter.write_str("ast::BinTimes{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinDivide {} => {
                __formatter.write_str("ast::BinDivide{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinRemainder {} => {
                __formatter.write_str("ast::BinRemainder{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinExponent {} => {
                __formatter.write_str("ast::BinExponent{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinLeftShift {} => {
                __formatter.write_str("ast::BinLeftShift{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinRightShift {} => {
                __formatter.write_str("ast::BinRightShift{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinUnsignedRightShift {} => {
                __formatter.write_str("ast::BinUnsignedRightShift{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinBitwiseAnd {} => {
                __formatter.write_str("ast::BinBitwiseAnd{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinBitwiseOr {} => {
                __formatter.write_str("ast::BinBitwiseOr{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinBitwiseXor {} => {
                __formatter.write_str("ast::BinBitwiseXor{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinNullishCoalescing {} => {
                __formatter.write_str("ast::BinNullishCoalescing{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinLogicalOr {} => {
                __formatter.write_str("ast::BinLogicalOr{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinLogicalAnd {} => {
                __formatter.write_str("ast::BinLogicalAnd{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinIn {} => {
                __formatter.write_str("ast::BinIn{")?;
                __formatter.write_str("}")
            }
            BinOperand::BinInstanceof {} => {
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
        BinOperand::BinLessThan {}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ClassElement {
    ClassEmptyElem,
    ClassMethod {
        name: ddlog_std::Option<PropertyKey>,
        params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>,
        body: ddlog_std::Option<StmtId>,
    },
    ClassStaticMethod {
        name: ddlog_std::Option<PropertyKey>,
        params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>,
        body: ddlog_std::Option<StmtId>,
    },
}
impl abomonation::Abomonation for ClassElement {}
::differential_datalog::decl_enum_from_record!(ClassElement["ast::ClassElement"]<>, ClassEmptyElem["ast::ClassEmptyElem"][0]{}, ClassMethod["ast::ClassMethod"][3]{[0]name["name"]: ddlog_std::Option<PropertyKey>, [1]params["params"]: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, [2]body["body"]: ddlog_std::Option<StmtId>}, ClassStaticMethod["ast::ClassStaticMethod"][3]{[0]name["name"]: ddlog_std::Option<PropertyKey>, [1]params["params"]: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, [2]body["body"]: ddlog_std::Option<StmtId>});
::differential_datalog::decl_enum_into_record!(ClassElement<>, ClassEmptyElem["ast::ClassEmptyElem"]{}, ClassMethod["ast::ClassMethod"]{name, params, body}, ClassStaticMethod["ast::ClassStaticMethod"]{name, params, body});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ClassElement<>, ClassEmptyElem{}, ClassMethod{name: ddlog_std::Option<PropertyKey>, params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, body: ddlog_std::Option<StmtId>}, ClassStaticMethod{name: ddlog_std::Option<PropertyKey>, params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, body: ddlog_std::Option<StmtId>});
impl ::std::fmt::Display for ClassElement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ClassElement::ClassEmptyElem {} => {
                __formatter.write_str("ast::ClassEmptyElem{")?;
                __formatter.write_str("}")
            }
            ClassElement::ClassMethod { name, params, body } => {
                __formatter.write_str("ast::ClassMethod{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
            ClassElement::ClassStaticMethod { name, params, body } => {
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
        ClassElement::ClassEmptyElem {}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ClassId {
    pub id: u32,
}
impl abomonation::Abomonation for ClassId {}
::differential_datalog::decl_struct_from_record!(ClassId["ast::ClassId"]<>, ["ast::ClassId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(ClassId, ["ast::ClassId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ClassId, <>, id: u32);
impl ::std::fmt::Display for ClassId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ClassId { id } => {
                __formatter.write_str("ast::ClassId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
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
        name: ddlog_std::Option<Spanned<Name>>,
        alias: ddlog_std::Option<Spanned<Name>>,
    },
}
impl abomonation::Abomonation for ExportKind {}
::differential_datalog::decl_enum_from_record!(ExportKind["ast::ExportKind"]<>, WildcardExport["ast::WildcardExport"][0]{}, NamedExport["ast::NamedExport"][2]{[0]name["name"]: ddlog_std::Option<Spanned<Name>>, [1]alias["alias"]: ddlog_std::Option<Spanned<Name>>});
::differential_datalog::decl_enum_into_record!(ExportKind<>, WildcardExport["ast::WildcardExport"]{}, NamedExport["ast::NamedExport"]{name, alias});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ExportKind<>, WildcardExport{}, NamedExport{name: ddlog_std::Option<Spanned<Name>>, alias: ddlog_std::Option<Spanned<Name>>});
impl ::std::fmt::Display for ExportKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExportKind::WildcardExport {} => {
                __formatter.write_str("ast::WildcardExport{")?;
                __formatter.write_str("}")
            }
            ExportKind::NamedExport { name, alias } => {
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
        ExportKind::WildcardExport {}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ExprId {
    pub id: u32,
}
impl abomonation::Abomonation for ExprId {}
::differential_datalog::decl_struct_from_record!(ExprId["ast::ExprId"]<>, ["ast::ExprId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(ExprId, ["ast::ExprId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ExprId, <>, id: u32);
impl ::std::fmt::Display for ExprId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprId { id } => {
                __formatter.write_str("ast::ExprId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
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
        kind: LitKind,
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
        inner: ddlog_std::Option<ExprId>,
    },
    ExprBracket,
    ExprDot,
    ExprNew,
    ExprCall,
    ExprAssign,
    ExprSequence {
        exprs: ddlog_std::Vec<ExprId>,
    },
    ExprNewTarget,
    ExprImportMeta,
    ExprInlineFunc,
    ExprSuperCall {
        args: ddlog_std::Option<ddlog_std::Vec<ExprId>>,
    },
    ExprImportCall {
        arg: ddlog_std::Option<ExprId>,
    },
    ExprClass,
}
impl abomonation::Abomonation for ExprKind {}
::differential_datalog::decl_enum_from_record!(ExprKind["ast::ExprKind"]<>, ExprLit["ast::ExprLit"][1]{[0]kind["kind"]: LitKind}, ExprNameRef["ast::ExprNameRef"][0]{}, ExprYield["ast::ExprYield"][0]{}, ExprAwait["ast::ExprAwait"][0]{}, ExprArrow["ast::ExprArrow"][0]{}, ExprUnaryOp["ast::ExprUnaryOp"][0]{}, ExprBinOp["ast::ExprBinOp"][0]{}, ExprTernary["ast::ExprTernary"][0]{}, ExprThis["ast::ExprThis"][0]{}, ExprTemplate["ast::ExprTemplate"][0]{}, ExprArray["ast::ExprArray"][0]{}, ExprObject["ast::ExprObject"][0]{}, ExprGrouping["ast::ExprGrouping"][1]{[0]inner["inner"]: ddlog_std::Option<ExprId>}, ExprBracket["ast::ExprBracket"][0]{}, ExprDot["ast::ExprDot"][0]{}, ExprNew["ast::ExprNew"][0]{}, ExprCall["ast::ExprCall"][0]{}, ExprAssign["ast::ExprAssign"][0]{}, ExprSequence["ast::ExprSequence"][1]{[0]exprs["exprs"]: ddlog_std::Vec<ExprId>}, ExprNewTarget["ast::ExprNewTarget"][0]{}, ExprImportMeta["ast::ExprImportMeta"][0]{}, ExprInlineFunc["ast::ExprInlineFunc"][0]{}, ExprSuperCall["ast::ExprSuperCall"][1]{[0]args["args"]: ddlog_std::Option<ddlog_std::Vec<ExprId>>}, ExprImportCall["ast::ExprImportCall"][1]{[0]arg["arg"]: ddlog_std::Option<ExprId>}, ExprClass["ast::ExprClass"][0]{});
::differential_datalog::decl_enum_into_record!(ExprKind<>, ExprLit["ast::ExprLit"]{kind}, ExprNameRef["ast::ExprNameRef"]{}, ExprYield["ast::ExprYield"]{}, ExprAwait["ast::ExprAwait"]{}, ExprArrow["ast::ExprArrow"]{}, ExprUnaryOp["ast::ExprUnaryOp"]{}, ExprBinOp["ast::ExprBinOp"]{}, ExprTernary["ast::ExprTernary"]{}, ExprThis["ast::ExprThis"]{}, ExprTemplate["ast::ExprTemplate"]{}, ExprArray["ast::ExprArray"]{}, ExprObject["ast::ExprObject"]{}, ExprGrouping["ast::ExprGrouping"]{inner}, ExprBracket["ast::ExprBracket"]{}, ExprDot["ast::ExprDot"]{}, ExprNew["ast::ExprNew"]{}, ExprCall["ast::ExprCall"]{}, ExprAssign["ast::ExprAssign"]{}, ExprSequence["ast::ExprSequence"]{exprs}, ExprNewTarget["ast::ExprNewTarget"]{}, ExprImportMeta["ast::ExprImportMeta"]{}, ExprInlineFunc["ast::ExprInlineFunc"]{}, ExprSuperCall["ast::ExprSuperCall"]{args}, ExprImportCall["ast::ExprImportCall"]{arg}, ExprClass["ast::ExprClass"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ExprKind<>, ExprLit{kind: LitKind}, ExprNameRef{}, ExprYield{}, ExprAwait{}, ExprArrow{}, ExprUnaryOp{}, ExprBinOp{}, ExprTernary{}, ExprThis{}, ExprTemplate{}, ExprArray{}, ExprObject{}, ExprGrouping{inner: ddlog_std::Option<ExprId>}, ExprBracket{}, ExprDot{}, ExprNew{}, ExprCall{}, ExprAssign{}, ExprSequence{exprs: ddlog_std::Vec<ExprId>}, ExprNewTarget{}, ExprImportMeta{}, ExprInlineFunc{}, ExprSuperCall{args: ddlog_std::Option<ddlog_std::Vec<ExprId>>}, ExprImportCall{arg: ddlog_std::Option<ExprId>}, ExprClass{});
impl ::std::fmt::Display for ExprKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprKind::ExprLit { kind } => {
                __formatter.write_str("ast::ExprLit{")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str("}")
            }
            ExprKind::ExprNameRef {} => {
                __formatter.write_str("ast::ExprNameRef{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprYield {} => {
                __formatter.write_str("ast::ExprYield{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprAwait {} => {
                __formatter.write_str("ast::ExprAwait{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprArrow {} => {
                __formatter.write_str("ast::ExprArrow{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprUnaryOp {} => {
                __formatter.write_str("ast::ExprUnaryOp{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprBinOp {} => {
                __formatter.write_str("ast::ExprBinOp{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprTernary {} => {
                __formatter.write_str("ast::ExprTernary{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprThis {} => {
                __formatter.write_str("ast::ExprThis{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprTemplate {} => {
                __formatter.write_str("ast::ExprTemplate{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprArray {} => {
                __formatter.write_str("ast::ExprArray{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprObject {} => {
                __formatter.write_str("ast::ExprObject{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprGrouping { inner } => {
                __formatter.write_str("ast::ExprGrouping{")?;
                ::std::fmt::Debug::fmt(inner, __formatter)?;
                __formatter.write_str("}")
            }
            ExprKind::ExprBracket {} => {
                __formatter.write_str("ast::ExprBracket{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprDot {} => {
                __formatter.write_str("ast::ExprDot{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprNew {} => {
                __formatter.write_str("ast::ExprNew{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprCall {} => {
                __formatter.write_str("ast::ExprCall{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprAssign {} => {
                __formatter.write_str("ast::ExprAssign{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprSequence { exprs } => {
                __formatter.write_str("ast::ExprSequence{")?;
                ::std::fmt::Debug::fmt(exprs, __formatter)?;
                __formatter.write_str("}")
            }
            ExprKind::ExprNewTarget {} => {
                __formatter.write_str("ast::ExprNewTarget{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprImportMeta {} => {
                __formatter.write_str("ast::ExprImportMeta{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprInlineFunc {} => {
                __formatter.write_str("ast::ExprInlineFunc{")?;
                __formatter.write_str("}")
            }
            ExprKind::ExprSuperCall { args } => {
                __formatter.write_str("ast::ExprSuperCall{")?;
                ::std::fmt::Debug::fmt(args, __formatter)?;
                __formatter.write_str("}")
            }
            ExprKind::ExprImportCall { arg } => {
                __formatter.write_str("ast::ExprImportCall{")?;
                ::std::fmt::Debug::fmt(arg, __formatter)?;
                __formatter.write_str("}")
            }
            ExprKind::ExprClass {} => {
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
        ExprKind::ExprLit {
            kind: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct FileId {
    pub id: u32,
}
impl abomonation::Abomonation for FileId {}
::differential_datalog::decl_struct_from_record!(FileId["ast::FileId"]<>, ["ast::FileId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(FileId, ["ast::FileId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FileId, <>, id: u32);
impl ::std::fmt::Display for FileId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FileId { id } => {
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
    JavaScript { flavor: JSFlavor },
    Todo,
}
impl abomonation::Abomonation for FileKind {}
::differential_datalog::decl_enum_from_record!(FileKind["ast::FileKind"]<>, JavaScript["ast::JavaScript"][1]{[0]flavor["flavor"]: JSFlavor}, Todo["ast::Todo"][0]{});
::differential_datalog::decl_enum_into_record!(FileKind<>, JavaScript["ast::JavaScript"]{flavor}, Todo["ast::Todo"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(FileKind<>, JavaScript{flavor: JSFlavor}, Todo{});
impl ::std::fmt::Display for FileKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FileKind::JavaScript { flavor } => {
                __formatter.write_str("ast::JavaScript{")?;
                ::std::fmt::Debug::fmt(flavor, __formatter)?;
                __formatter.write_str("}")
            }
            FileKind::Todo {} => {
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
        FileKind::JavaScript {
            flavor: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ForInit {
    ForDecl { stmt_id: ddlog_std::Option<StmtId> },
    ForExpr { expr_id: ExprId },
}
impl abomonation::Abomonation for ForInit {}
::differential_datalog::decl_enum_from_record!(ForInit["ast::ForInit"]<>, ForDecl["ast::ForDecl"][1]{[0]stmt_id["stmt_id"]: ddlog_std::Option<StmtId>}, ForExpr["ast::ForExpr"][1]{[0]expr_id["expr_id"]: ExprId});
::differential_datalog::decl_enum_into_record!(ForInit<>, ForDecl["ast::ForDecl"]{stmt_id}, ForExpr["ast::ForExpr"]{expr_id});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ForInit<>, ForDecl{stmt_id: ddlog_std::Option<StmtId>}, ForExpr{expr_id: ExprId});
impl ::std::fmt::Display for ForInit {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ForInit::ForDecl { stmt_id } => {
                __formatter.write_str("ast::ForDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str("}")
            }
            ForInit::ForExpr { expr_id } => {
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
        ForInit::ForDecl {
            stmt_id: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct FuncId {
    pub id: u32,
}
impl abomonation::Abomonation for FuncId {}
::differential_datalog::decl_struct_from_record!(FuncId["ast::FuncId"]<>, ["ast::FuncId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(FuncId, ["ast::FuncId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FuncId, <>, id: u32);
impl ::std::fmt::Display for FuncId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FuncId { id } => {
                __formatter.write_str("ast::FuncId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
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
pub struct FuncParam {
    pub pattern: IPattern,
    pub implicit: bool,
}
impl abomonation::Abomonation for FuncParam {}
::differential_datalog::decl_struct_from_record!(FuncParam["ast::FuncParam"]<>, ["ast::FuncParam"][2]{[0]pattern["pattern"]: IPattern, [1]implicit["implicit"]: bool});
::differential_datalog::decl_struct_into_record!(FuncParam, ["ast::FuncParam"]<>, pattern, implicit);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(FuncParam, <>, pattern: IPattern, implicit: bool);
impl ::std::fmt::Display for FuncParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FuncParam { pattern, implicit } => {
                __formatter.write_str("ast::FuncParam{")?;
                ::std::fmt::Debug::fmt(pattern, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(implicit, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for FuncParam {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct GlobalId {
    pub id: u32,
}
impl abomonation::Abomonation for GlobalId {}
::differential_datalog::decl_struct_from_record!(GlobalId["ast::GlobalId"]<>, ["ast::GlobalId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(GlobalId, ["ast::GlobalId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(GlobalId, <>, id: u32);
impl ::std::fmt::Display for GlobalId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            GlobalId { id } => {
                __formatter.write_str("ast::GlobalId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum GlobalPriv {
    ReadonlyGlobal,
    ReadWriteGlobal,
}
impl abomonation::Abomonation for GlobalPriv {}
::differential_datalog::decl_enum_from_record!(GlobalPriv["ast::GlobalPriv"]<>, ReadonlyGlobal["ast::ReadonlyGlobal"][0]{}, ReadWriteGlobal["ast::ReadWriteGlobal"][0]{});
::differential_datalog::decl_enum_into_record!(GlobalPriv<>, ReadonlyGlobal["ast::ReadonlyGlobal"]{}, ReadWriteGlobal["ast::ReadWriteGlobal"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(GlobalPriv<>, ReadonlyGlobal{}, ReadWriteGlobal{});
impl ::std::fmt::Display for GlobalPriv {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            GlobalPriv::ReadonlyGlobal {} => {
                __formatter.write_str("ast::ReadonlyGlobal{")?;
                __formatter.write_str("}")
            }
            GlobalPriv::ReadWriteGlobal {} => {
                __formatter.write_str("ast::ReadWriteGlobal{")?;
                __formatter.write_str("}")
            }
        }
    }
}
impl ::std::fmt::Debug for GlobalPriv {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl ::std::default::Default for GlobalPriv {
    fn default() -> Self {
        GlobalPriv::ReadonlyGlobal {}
    }
}
pub type IClassElement = internment::Intern<ClassElement>;
pub type IObjectPatternProp = internment::Intern<ObjectPatternProp>;
pub type IPattern = internment::Intern<Pattern>;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum ImportClause {
    WildcardImport {
        alias: ddlog_std::Option<Spanned<Name>>,
    },
    GroupedImport {
        imports: ddlog_std::Vec<NamedImport>,
    },
    SingleImport {
        name: Spanned<Name>,
    },
}
impl abomonation::Abomonation for ImportClause {}
::differential_datalog::decl_enum_from_record!(ImportClause["ast::ImportClause"]<>, WildcardImport["ast::WildcardImport"][1]{[0]alias["alias"]: ddlog_std::Option<Spanned<Name>>}, GroupedImport["ast::GroupedImport"][1]{[0]imports["imports"]: ddlog_std::Vec<NamedImport>}, SingleImport["ast::SingleImport"][1]{[0]name["name"]: Spanned<Name>});
::differential_datalog::decl_enum_into_record!(ImportClause<>, WildcardImport["ast::WildcardImport"]{alias}, GroupedImport["ast::GroupedImport"]{imports}, SingleImport["ast::SingleImport"]{name});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ImportClause<>, WildcardImport{alias: ddlog_std::Option<Spanned<Name>>}, GroupedImport{imports: ddlog_std::Vec<NamedImport>}, SingleImport{name: Spanned<Name>});
impl ::std::fmt::Display for ImportClause {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImportClause::WildcardImport { alias } => {
                __formatter.write_str("ast::WildcardImport{")?;
                ::std::fmt::Debug::fmt(alias, __formatter)?;
                __formatter.write_str("}")
            }
            ImportClause::GroupedImport { imports } => {
                __formatter.write_str("ast::GroupedImport{")?;
                ::std::fmt::Debug::fmt(imports, __formatter)?;
                __formatter.write_str("}")
            }
            ImportClause::SingleImport { name } => {
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
        ImportClause::WildcardImport {
            alias: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ImportId {
    pub id: u32,
}
impl abomonation::Abomonation for ImportId {}
::differential_datalog::decl_struct_from_record!(ImportId["ast::ImportId"]<>, ["ast::ImportId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(ImportId, ["ast::ImportId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ImportId, <>, id: u32);
impl ::std::fmt::Display for ImportId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImportId { id } => {
                __formatter.write_str("ast::ImportId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
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
    TypeScript,
}
impl abomonation::Abomonation for JSFlavor {}
::differential_datalog::decl_enum_from_record!(JSFlavor["ast::JSFlavor"]<>, Vanilla["ast::Vanilla"][0]{}, Module["ast::Module"][0]{}, TypeScript["ast::TypeScript"][0]{});
::differential_datalog::decl_enum_into_record!(JSFlavor<>, Vanilla["ast::Vanilla"]{}, Module["ast::Module"]{}, TypeScript["ast::TypeScript"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(JSFlavor<>, Vanilla{}, Module{}, TypeScript{});
impl ::std::fmt::Display for JSFlavor {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            JSFlavor::Vanilla {} => {
                __formatter.write_str("ast::Vanilla{")?;
                __formatter.write_str("}")
            }
            JSFlavor::Module {} => {
                __formatter.write_str("ast::Module{")?;
                __formatter.write_str("}")
            }
            JSFlavor::TypeScript {} => {
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
        JSFlavor::Vanilla {}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum LitKind {
    LitNumber,
    LitBigInt,
    LitString,
    LitNull,
    LitBool,
    LitRegex,
}
impl abomonation::Abomonation for LitKind {}
::differential_datalog::decl_enum_from_record!(LitKind["ast::LitKind"]<>, LitNumber["ast::LitNumber"][0]{}, LitBigInt["ast::LitBigInt"][0]{}, LitString["ast::LitString"][0]{}, LitNull["ast::LitNull"][0]{}, LitBool["ast::LitBool"][0]{}, LitRegex["ast::LitRegex"][0]{});
::differential_datalog::decl_enum_into_record!(LitKind<>, LitNumber["ast::LitNumber"]{}, LitBigInt["ast::LitBigInt"]{}, LitString["ast::LitString"]{}, LitNull["ast::LitNull"]{}, LitBool["ast::LitBool"]{}, LitRegex["ast::LitRegex"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(LitKind<>, LitNumber{}, LitBigInt{}, LitString{}, LitNull{}, LitBool{}, LitRegex{});
impl ::std::fmt::Display for LitKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            LitKind::LitNumber {} => {
                __formatter.write_str("ast::LitNumber{")?;
                __formatter.write_str("}")
            }
            LitKind::LitBigInt {} => {
                __formatter.write_str("ast::LitBigInt{")?;
                __formatter.write_str("}")
            }
            LitKind::LitString {} => {
                __formatter.write_str("ast::LitString{")?;
                __formatter.write_str("}")
            }
            LitKind::LitNull {} => {
                __formatter.write_str("ast::LitNull{")?;
                __formatter.write_str("}")
            }
            LitKind::LitBool {} => {
                __formatter.write_str("ast::LitBool{")?;
                __formatter.write_str("}")
            }
            LitKind::LitRegex {} => {
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
        LitKind::LitNumber {}
    }
}
pub type Name = internment::istring;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct NamedImport {
    pub name: ddlog_std::Option<Spanned<Name>>,
    pub alias: ddlog_std::Option<Spanned<Name>>,
}
impl abomonation::Abomonation for NamedImport {}
::differential_datalog::decl_struct_from_record!(NamedImport["ast::NamedImport"]<>, ["ast::NamedImport"][2]{[0]name["name"]: ddlog_std::Option<Spanned<Name>>, [1]alias["alias"]: ddlog_std::Option<Spanned<Name>>});
::differential_datalog::decl_struct_into_record!(NamedImport, ["ast::NamedImport"]<>, name, alias);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(NamedImport, <>, name: ddlog_std::Option<Spanned<Name>>, alias: ddlog_std::Option<Spanned<Name>>);
impl ::std::fmt::Display for NamedImport {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NamedImport { name, alias } => {
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
        assign_key: ddlog_std::Option<internment::Intern<Pattern>>,
        assign_value: ddlog_std::Option<ExprId>,
    },
    ObjKeyValuePattern {
        key: ddlog_std::Option<PropertyKey>,
        value: ddlog_std::Option<internment::Intern<Pattern>>,
    },
    ObjRestPattern {
        rest: ddlog_std::Option<internment::Intern<Pattern>>,
    },
    ObjSinglePattern {
        name: ddlog_std::Option<Spanned<Name>>,
    },
}
impl abomonation::Abomonation for ObjectPatternProp {}
::differential_datalog::decl_enum_from_record!(ObjectPatternProp["ast::ObjectPatternProp"]<>, ObjAssignPattern["ast::ObjAssignPattern"][2]{[0]assign_key["assign_key"]: ddlog_std::Option<internment::Intern<Pattern>>, [1]assign_value["assign_value"]: ddlog_std::Option<ExprId>}, ObjKeyValuePattern["ast::ObjKeyValuePattern"][2]{[0]key["key"]: ddlog_std::Option<PropertyKey>, [1]value["value"]: ddlog_std::Option<internment::Intern<Pattern>>}, ObjRestPattern["ast::ObjRestPattern"][1]{[0]rest["rest"]: ddlog_std::Option<internment::Intern<Pattern>>}, ObjSinglePattern["ast::ObjSinglePattern"][1]{[0]name["name"]: ddlog_std::Option<Spanned<Name>>});
::differential_datalog::decl_enum_into_record!(ObjectPatternProp<>, ObjAssignPattern["ast::ObjAssignPattern"]{assign_key, assign_value}, ObjKeyValuePattern["ast::ObjKeyValuePattern"]{key, value}, ObjRestPattern["ast::ObjRestPattern"]{rest}, ObjSinglePattern["ast::ObjSinglePattern"]{name});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(ObjectPatternProp<>, ObjAssignPattern{assign_key: ddlog_std::Option<internment::Intern<Pattern>>, assign_value: ddlog_std::Option<ExprId>}, ObjKeyValuePattern{key: ddlog_std::Option<PropertyKey>, value: ddlog_std::Option<internment::Intern<Pattern>>}, ObjRestPattern{rest: ddlog_std::Option<internment::Intern<Pattern>>}, ObjSinglePattern{name: ddlog_std::Option<Spanned<Name>>});
impl ::std::fmt::Display for ObjectPatternProp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ObjectPatternProp::ObjAssignPattern {
                assign_key,
                assign_value,
            } => {
                __formatter.write_str("ast::ObjAssignPattern{")?;
                ::std::fmt::Debug::fmt(assign_key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(assign_value, __formatter)?;
                __formatter.write_str("}")
            }
            ObjectPatternProp::ObjKeyValuePattern { key, value } => {
                __formatter.write_str("ast::ObjKeyValuePattern{")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
            ObjectPatternProp::ObjRestPattern { rest } => {
                __formatter.write_str("ast::ObjRestPattern{")?;
                ::std::fmt::Debug::fmt(rest, __formatter)?;
                __formatter.write_str("}")
            }
            ObjectPatternProp::ObjSinglePattern { name } => {
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
        ObjectPatternProp::ObjAssignPattern {
            assign_key: ::std::default::Default::default(),
            assign_value: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum OneOf<A, B, C> {
    First { a: A },
    Second { b: B },
    Third { c: C },
}
impl<A: ::ddlog_rt::Val, B: ::ddlog_rt::Val, C: ::ddlog_rt::Val> abomonation::Abomonation
    for OneOf<A, B, C>
{
}
::differential_datalog::decl_enum_from_record!(OneOf["ast::OneOf"]<A,B,C>, First["ast::First"][1]{[0]a["a"]: A}, Second["ast::Second"][1]{[0]b["b"]: B}, Third["ast::Third"][1]{[0]c["c"]: C});
::differential_datalog::decl_enum_into_record!(OneOf<A,B,C>, First["ast::First"]{a}, Second["ast::Second"]{b}, Third["ast::Third"]{c});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(OneOf<A,B,C>, First{a: A}, Second{b: B}, Third{c: C});
impl<A: ::std::fmt::Debug, B: ::std::fmt::Debug, C: ::std::fmt::Debug> ::std::fmt::Display
    for OneOf<A, B, C>
{
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            OneOf::First { a } => {
                __formatter.write_str("ast::First{")?;
                ::std::fmt::Debug::fmt(a, __formatter)?;
                __formatter.write_str("}")
            }
            OneOf::Second { b } => {
                __formatter.write_str("ast::Second{")?;
                ::std::fmt::Debug::fmt(b, __formatter)?;
                __formatter.write_str("}")
            }
            OneOf::Third { c } => {
                __formatter.write_str("ast::Third{")?;
                ::std::fmt::Debug::fmt(c, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl<A: ::std::fmt::Debug, B: ::std::fmt::Debug, C: ::std::fmt::Debug> ::std::fmt::Debug
    for OneOf<A, B, C>
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
impl<A: ::std::default::Default, B: ::std::default::Default, C: ::std::default::Default>
    ::std::default::Default for OneOf<A, B, C>
{
    fn default() -> Self {
        OneOf::First {
            a: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Pattern {
    SinglePattern {
        name: ddlog_std::Option<Spanned<Name>>,
    },
    RestPattern {
        rest: ddlog_std::Option<internment::Intern<Pattern>>,
    },
    AssignPattern {
        key: ddlog_std::Option<internment::Intern<Pattern>>,
        value: ddlog_std::Option<ExprId>,
    },
    ObjectPattern {
        props: ddlog_std::Vec<internment::Intern<ObjectPatternProp>>,
    },
    ArrayPattern {
        elems: ddlog_std::Vec<internment::Intern<Pattern>>,
    },
}
impl abomonation::Abomonation for Pattern {}
::differential_datalog::decl_enum_from_record!(Pattern["ast::Pattern"]<>, SinglePattern["ast::SinglePattern"][1]{[0]name["name"]: ddlog_std::Option<Spanned<Name>>}, RestPattern["ast::RestPattern"][1]{[0]rest["rest"]: ddlog_std::Option<internment::Intern<Pattern>>}, AssignPattern["ast::AssignPattern"][2]{[0]key["key"]: ddlog_std::Option<internment::Intern<Pattern>>, [1]value["value"]: ddlog_std::Option<ExprId>}, ObjectPattern["ast::ObjectPattern"][1]{[0]props["props"]: ddlog_std::Vec<internment::Intern<ObjectPatternProp>>}, ArrayPattern["ast::ArrayPattern"][1]{[0]elems["elems"]: ddlog_std::Vec<internment::Intern<Pattern>>});
::differential_datalog::decl_enum_into_record!(Pattern<>, SinglePattern["ast::SinglePattern"]{name}, RestPattern["ast::RestPattern"]{rest}, AssignPattern["ast::AssignPattern"]{key, value}, ObjectPattern["ast::ObjectPattern"]{props}, ArrayPattern["ast::ArrayPattern"]{elems});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(Pattern<>, SinglePattern{name: ddlog_std::Option<Spanned<Name>>}, RestPattern{rest: ddlog_std::Option<internment::Intern<Pattern>>}, AssignPattern{key: ddlog_std::Option<internment::Intern<Pattern>>, value: ddlog_std::Option<ExprId>}, ObjectPattern{props: ddlog_std::Vec<internment::Intern<ObjectPatternProp>>}, ArrayPattern{elems: ddlog_std::Vec<internment::Intern<Pattern>>});
impl ::std::fmt::Display for Pattern {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Pattern::SinglePattern { name } => {
                __formatter.write_str("ast::SinglePattern{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str("}")
            }
            Pattern::RestPattern { rest } => {
                __formatter.write_str("ast::RestPattern{")?;
                ::std::fmt::Debug::fmt(rest, __formatter)?;
                __formatter.write_str("}")
            }
            Pattern::AssignPattern { key, value } => {
                __formatter.write_str("ast::AssignPattern{")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
            Pattern::ObjectPattern { props } => {
                __formatter.write_str("ast::ObjectPattern{")?;
                ::std::fmt::Debug::fmt(props, __formatter)?;
                __formatter.write_str("}")
            }
            Pattern::ArrayPattern { elems } => {
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
        Pattern::SinglePattern {
            name: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PropertyKey {
    ComputedKey { prop: ddlog_std::Option<ExprId> },
    LiteralKey { lit: ExprId },
    IdentKey { ident: Spanned<Name> },
}
impl abomonation::Abomonation for PropertyKey {}
::differential_datalog::decl_enum_from_record!(PropertyKey["ast::PropertyKey"]<>, ComputedKey["ast::ComputedKey"][1]{[0]prop["prop"]: ddlog_std::Option<ExprId>}, LiteralKey["ast::LiteralKey"][1]{[0]lit["lit"]: ExprId}, IdentKey["ast::IdentKey"][1]{[0]ident["ident"]: Spanned<Name>});
::differential_datalog::decl_enum_into_record!(PropertyKey<>, ComputedKey["ast::ComputedKey"]{prop}, LiteralKey["ast::LiteralKey"]{lit}, IdentKey["ast::IdentKey"]{ident});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(PropertyKey<>, ComputedKey{prop: ddlog_std::Option<ExprId>}, LiteralKey{lit: ExprId}, IdentKey{ident: Spanned<Name>});
impl ::std::fmt::Display for PropertyKey {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            PropertyKey::ComputedKey { prop } => {
                __formatter.write_str("ast::ComputedKey{")?;
                ::std::fmt::Debug::fmt(prop, __formatter)?;
                __formatter.write_str("}")
            }
            PropertyKey::LiteralKey { lit } => {
                __formatter.write_str("ast::LiteralKey{")?;
                ::std::fmt::Debug::fmt(lit, __formatter)?;
                __formatter.write_str("}")
            }
            PropertyKey::IdentKey { ident } => {
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
        PropertyKey::ComputedKey {
            prop: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum PropertyVal {
    PropLit {
        lit: ddlog_std::Option<ExprId>,
    },
    PropGetter {
        body: ddlog_std::Option<StmtId>,
    },
    PropSetter {
        params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>,
        body: ddlog_std::Option<StmtId>,
    },
    PropSpread {
        value: ddlog_std::Option<ExprId>,
    },
    PropInit {
        value: ddlog_std::Option<ExprId>,
    },
    PropIdent,
    PropMethod {
        params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>,
        body: ddlog_std::Option<StmtId>,
    },
}
impl abomonation::Abomonation for PropertyVal {}
::differential_datalog::decl_enum_from_record!(PropertyVal["ast::PropertyVal"]<>, PropLit["ast::PropLit"][1]{[0]lit["lit"]: ddlog_std::Option<ExprId>}, PropGetter["ast::PropGetter"][1]{[0]body["body"]: ddlog_std::Option<StmtId>}, PropSetter["ast::PropSetter"][2]{[0]params["params"]: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, [1]body["body"]: ddlog_std::Option<StmtId>}, PropSpread["ast::PropSpread"][1]{[0]value["value"]: ddlog_std::Option<ExprId>}, PropInit["ast::PropInit"][1]{[0]value["value"]: ddlog_std::Option<ExprId>}, PropIdent["ast::PropIdent"][0]{}, PropMethod["ast::PropMethod"][2]{[0]params["params"]: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, [1]body["body"]: ddlog_std::Option<StmtId>});
::differential_datalog::decl_enum_into_record!(PropertyVal<>, PropLit["ast::PropLit"]{lit}, PropGetter["ast::PropGetter"]{body}, PropSetter["ast::PropSetter"]{params, body}, PropSpread["ast::PropSpread"]{value}, PropInit["ast::PropInit"]{value}, PropIdent["ast::PropIdent"]{}, PropMethod["ast::PropMethod"]{params, body});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(PropertyVal<>, PropLit{lit: ddlog_std::Option<ExprId>}, PropGetter{body: ddlog_std::Option<StmtId>}, PropSetter{params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, body: ddlog_std::Option<StmtId>}, PropSpread{value: ddlog_std::Option<ExprId>}, PropInit{value: ddlog_std::Option<ExprId>}, PropIdent{}, PropMethod{params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>, body: ddlog_std::Option<StmtId>});
impl ::std::fmt::Display for PropertyVal {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            PropertyVal::PropLit { lit } => {
                __formatter.write_str("ast::PropLit{")?;
                ::std::fmt::Debug::fmt(lit, __formatter)?;
                __formatter.write_str("}")
            }
            PropertyVal::PropGetter { body } => {
                __formatter.write_str("ast::PropGetter{")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
            PropertyVal::PropSetter { params, body } => {
                __formatter.write_str("ast::PropSetter{")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            }
            PropertyVal::PropSpread { value } => {
                __formatter.write_str("ast::PropSpread{")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
            PropertyVal::PropInit { value } => {
                __formatter.write_str("ast::PropInit{")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            }
            PropertyVal::PropIdent {} => {
                __formatter.write_str("ast::PropIdent{")?;
                __formatter.write_str("}")
            }
            PropertyVal::PropMethod { params, body } => {
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
        PropertyVal::PropLit {
            lit: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct ScopeId {
    pub id: u32,
}
impl abomonation::Abomonation for ScopeId {}
::differential_datalog::decl_struct_from_record!(ScopeId["ast::ScopeId"]<>, ["ast::ScopeId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(ScopeId, ["ast::ScopeId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(ScopeId, <>, id: u32);
impl ::std::fmt::Display for ScopeId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ScopeId { id } => {
                __formatter.write_str("ast::ScopeId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
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
    pub end: u32,
}
impl abomonation::Abomonation for Span {}
::differential_datalog::decl_struct_from_record!(Span["ast::Span"]<>, ["ast::Span"][2]{[0]start["start"]: u32, [1]end["end"]: u32});
::differential_datalog::decl_struct_into_record!(Span, ["ast::Span"]<>, start, end);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Span, <>, start: u32, end: u32);
impl ::std::fmt::Display for Span {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Span { start, end } => {
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
    pub span: Span,
}
impl<T: ::ddlog_rt::Val> abomonation::Abomonation for Spanned<T> {}
::differential_datalog::decl_struct_from_record!(Spanned["ast::Spanned"]<T>, ["ast::Spanned"][2]{[0]data["data"]: T, [1]span["span"]: Span});
::differential_datalog::decl_struct_into_record!(Spanned, ["ast::Spanned"]<T>, data, span);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(Spanned, <T>, data: T, span: Span);
impl<T: ::std::fmt::Debug> ::std::fmt::Display for Spanned<T> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Spanned { data, span } => {
                __formatter.write_str("ast::Spanned{")?;
                ::std::fmt::Debug::fmt(data, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(span, __formatter)?;
                __formatter.write_str("}")
            }
        }
    }
}
impl<T: ::std::fmt::Debug> ::std::fmt::Debug for Spanned<T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self, f)
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct StmtId {
    pub id: u32,
}
impl abomonation::Abomonation for StmtId {}
::differential_datalog::decl_struct_from_record!(StmtId["ast::StmtId"]<>, ["ast::StmtId"][1]{[0]id["id"]: u32});
::differential_datalog::decl_struct_into_record!(StmtId, ["ast::StmtId"]<>, id);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(StmtId, <>, id: u32);
impl ::std::fmt::Display for StmtId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            StmtId { id } => {
                __formatter.write_str("ast::StmtId{")?;
                ::std::fmt::Debug::fmt(id, __formatter)?;
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
    StmtExpr { expr_id: ddlog_std::Option<ExprId> },
    StmtReturn,
    StmtIf,
    StmtBreak,
    StmtDoWhile,
    StmtWhile,
    StmtFor,
    StmtForIn,
    StmtForOf,
    StmtContinue,
    StmtWith,
    StmtLabel,
    StmtSwitch,
    StmtThrow,
    StmtTry,
    StmtDebugger,
    StmtEmpty,
}
impl abomonation::Abomonation for StmtKind {}
::differential_datalog::decl_enum_from_record!(StmtKind["ast::StmtKind"]<>, StmtVarDecl["ast::StmtVarDecl"][0]{}, StmtLetDecl["ast::StmtLetDecl"][0]{}, StmtConstDecl["ast::StmtConstDecl"][0]{}, StmtExpr["ast::StmtExpr"][1]{[0]expr_id["expr_id"]: ddlog_std::Option<ExprId>}, StmtReturn["ast::StmtReturn"][0]{}, StmtIf["ast::StmtIf"][0]{}, StmtBreak["ast::StmtBreak"][0]{}, StmtDoWhile["ast::StmtDoWhile"][0]{}, StmtWhile["ast::StmtWhile"][0]{}, StmtFor["ast::StmtFor"][0]{}, StmtForIn["ast::StmtForIn"][0]{}, StmtForOf["ast::StmtForOf"][0]{}, StmtContinue["ast::StmtContinue"][0]{}, StmtWith["ast::StmtWith"][0]{}, StmtLabel["ast::StmtLabel"][0]{}, StmtSwitch["ast::StmtSwitch"][0]{}, StmtThrow["ast::StmtThrow"][0]{}, StmtTry["ast::StmtTry"][0]{}, StmtDebugger["ast::StmtDebugger"][0]{}, StmtEmpty["ast::StmtEmpty"][0]{});
::differential_datalog::decl_enum_into_record!(StmtKind<>, StmtVarDecl["ast::StmtVarDecl"]{}, StmtLetDecl["ast::StmtLetDecl"]{}, StmtConstDecl["ast::StmtConstDecl"]{}, StmtExpr["ast::StmtExpr"]{expr_id}, StmtReturn["ast::StmtReturn"]{}, StmtIf["ast::StmtIf"]{}, StmtBreak["ast::StmtBreak"]{}, StmtDoWhile["ast::StmtDoWhile"]{}, StmtWhile["ast::StmtWhile"]{}, StmtFor["ast::StmtFor"]{}, StmtForIn["ast::StmtForIn"]{}, StmtForOf["ast::StmtForOf"]{}, StmtContinue["ast::StmtContinue"]{}, StmtWith["ast::StmtWith"]{}, StmtLabel["ast::StmtLabel"]{}, StmtSwitch["ast::StmtSwitch"]{}, StmtThrow["ast::StmtThrow"]{}, StmtTry["ast::StmtTry"]{}, StmtDebugger["ast::StmtDebugger"]{}, StmtEmpty["ast::StmtEmpty"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(StmtKind<>, StmtVarDecl{}, StmtLetDecl{}, StmtConstDecl{}, StmtExpr{expr_id: ddlog_std::Option<ExprId>}, StmtReturn{}, StmtIf{}, StmtBreak{}, StmtDoWhile{}, StmtWhile{}, StmtFor{}, StmtForIn{}, StmtForOf{}, StmtContinue{}, StmtWith{}, StmtLabel{}, StmtSwitch{}, StmtThrow{}, StmtTry{}, StmtDebugger{}, StmtEmpty{});
impl ::std::fmt::Display for StmtKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            StmtKind::StmtVarDecl {} => {
                __formatter.write_str("ast::StmtVarDecl{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtLetDecl {} => {
                __formatter.write_str("ast::StmtLetDecl{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtConstDecl {} => {
                __formatter.write_str("ast::StmtConstDecl{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtExpr { expr_id } => {
                __formatter.write_str("ast::StmtExpr{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str("}")
            }
            StmtKind::StmtReturn {} => {
                __formatter.write_str("ast::StmtReturn{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtIf {} => {
                __formatter.write_str("ast::StmtIf{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtBreak {} => {
                __formatter.write_str("ast::StmtBreak{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtDoWhile {} => {
                __formatter.write_str("ast::StmtDoWhile{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtWhile {} => {
                __formatter.write_str("ast::StmtWhile{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtFor {} => {
                __formatter.write_str("ast::StmtFor{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtForIn {} => {
                __formatter.write_str("ast::StmtForIn{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtForOf {} => {
                __formatter.write_str("ast::StmtForOf{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtContinue {} => {
                __formatter.write_str("ast::StmtContinue{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtWith {} => {
                __formatter.write_str("ast::StmtWith{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtLabel {} => {
                __formatter.write_str("ast::StmtLabel{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtSwitch {} => {
                __formatter.write_str("ast::StmtSwitch{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtThrow {} => {
                __formatter.write_str("ast::StmtThrow{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtTry {} => {
                __formatter.write_str("ast::StmtTry{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtDebugger {} => {
                __formatter.write_str("ast::StmtDebugger{")?;
                __formatter.write_str("}")
            }
            StmtKind::StmtEmpty {} => {
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
        StmtKind::StmtVarDecl {}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SwitchClause {
    CaseClause { test: ddlog_std::Option<ExprId> },
    DefaultClause,
}
impl abomonation::Abomonation for SwitchClause {}
::differential_datalog::decl_enum_from_record!(SwitchClause["ast::SwitchClause"]<>, CaseClause["ast::CaseClause"][1]{[0]test["test"]: ddlog_std::Option<ExprId>}, DefaultClause["ast::DefaultClause"][0]{});
::differential_datalog::decl_enum_into_record!(SwitchClause<>, CaseClause["ast::CaseClause"]{test}, DefaultClause["ast::DefaultClause"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(SwitchClause<>, CaseClause{test: ddlog_std::Option<ExprId>}, DefaultClause{});
impl ::std::fmt::Display for SwitchClause {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            SwitchClause::CaseClause { test } => {
                __formatter.write_str("ast::CaseClause{")?;
                ::std::fmt::Debug::fmt(test, __formatter)?;
                __formatter.write_str("}")
            }
            SwitchClause::DefaultClause {} => {
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
        SwitchClause::CaseClause {
            test: ::std::default::Default::default(),
        }
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, Default, Serialize, Deserialize)]
pub struct TryHandler {
    pub error: ddlog_std::Option<IPattern>,
    pub body: ddlog_std::Option<StmtId>,
}
impl abomonation::Abomonation for TryHandler {}
::differential_datalog::decl_struct_from_record!(TryHandler["ast::TryHandler"]<>, ["ast::TryHandler"][2]{[0]error["error"]: ddlog_std::Option<IPattern>, [1]body["body"]: ddlog_std::Option<StmtId>});
::differential_datalog::decl_struct_into_record!(TryHandler, ["ast::TryHandler"]<>, error, body);
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_struct!(TryHandler, <>, error: ddlog_std::Option<IPattern>, body: ddlog_std::Option<StmtId>);
impl ::std::fmt::Display for TryHandler {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            TryHandler { error, body } => {
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
    UnaryAwait,
}
impl abomonation::Abomonation for UnaryOperand {}
::differential_datalog::decl_enum_from_record!(UnaryOperand["ast::UnaryOperand"]<>, UnaryIncrement["ast::UnaryIncrement"][0]{}, UnaryDecrement["ast::UnaryDecrement"][0]{}, UnaryDelete["ast::UnaryDelete"][0]{}, UnaryVoid["ast::UnaryVoid"][0]{}, UnaryTypeof["ast::UnaryTypeof"][0]{}, UnaryPlus["ast::UnaryPlus"][0]{}, UnaryMinus["ast::UnaryMinus"][0]{}, UnaryBitwiseNot["ast::UnaryBitwiseNot"][0]{}, UnaryLogicalNot["ast::UnaryLogicalNot"][0]{}, UnaryAwait["ast::UnaryAwait"][0]{});
::differential_datalog::decl_enum_into_record!(UnaryOperand<>, UnaryIncrement["ast::UnaryIncrement"]{}, UnaryDecrement["ast::UnaryDecrement"]{}, UnaryDelete["ast::UnaryDelete"]{}, UnaryVoid["ast::UnaryVoid"]{}, UnaryTypeof["ast::UnaryTypeof"]{}, UnaryPlus["ast::UnaryPlus"]{}, UnaryMinus["ast::UnaryMinus"]{}, UnaryBitwiseNot["ast::UnaryBitwiseNot"]{}, UnaryLogicalNot["ast::UnaryLogicalNot"]{}, UnaryAwait["ast::UnaryAwait"]{});
#[rustfmt::skip] ::differential_datalog::decl_record_mutator_enum!(UnaryOperand<>, UnaryIncrement{}, UnaryDecrement{}, UnaryDelete{}, UnaryVoid{}, UnaryTypeof{}, UnaryPlus{}, UnaryMinus{}, UnaryBitwiseNot{}, UnaryLogicalNot{}, UnaryAwait{});
impl ::std::fmt::Display for UnaryOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            UnaryOperand::UnaryIncrement {} => {
                __formatter.write_str("ast::UnaryIncrement{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryDecrement {} => {
                __formatter.write_str("ast::UnaryDecrement{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryDelete {} => {
                __formatter.write_str("ast::UnaryDelete{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryVoid {} => {
                __formatter.write_str("ast::UnaryVoid{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryTypeof {} => {
                __formatter.write_str("ast::UnaryTypeof{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryPlus {} => {
                __formatter.write_str("ast::UnaryPlus{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryMinus {} => {
                __formatter.write_str("ast::UnaryMinus{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryBitwiseNot {} => {
                __formatter.write_str("ast::UnaryBitwiseNot{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryLogicalNot {} => {
                __formatter.write_str("ast::UnaryLogicalNot{")?;
                __formatter.write_str("}")
            }
            UnaryOperand::UnaryAwait {} => {
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
        UnaryOperand::UnaryIncrement {}
    }
}
pub fn any_id(global: &GlobalId) -> AnyId {
    (AnyId::AnyIdGlobal {
        global: (*global).clone(),
    })
}
pub fn body_ast_PropertyVal_ddlog_std_Option__ast_StmtId(
    prop: &PropertyVal,
) -> ddlog_std::Option<StmtId> {
    match (*prop) {
        PropertyVal::PropGetter {
            body: ddlog_std::Option::Some { x: ref body },
        } => (ddlog_std::Option::Some { x: (*body).clone() }),
        PropertyVal::PropSetter {
            params: _,
            body: ddlog_std::Option::Some { x: ref body },
        } => (ddlog_std::Option::Some { x: (*body).clone() }),
        PropertyVal::PropMethod {
            params: _,
            body: ddlog_std::Option::Some { x: ref body },
        } => (ddlog_std::Option::Some { x: (*body).clone() }),
        _ => (ddlog_std::Option::None {}),
    }
}
pub fn body_ast_ClassElement_ddlog_std_Option__ast_StmtId(
    elem: &ClassElement,
) -> ddlog_std::Option<StmtId> {
    match (*elem) {
        ClassElement::ClassMethod {
            name: _,
            params: _,
            body: ddlog_std::Option::Some { x: ref body },
        } => (ddlog_std::Option::Some { x: (*body).clone() }),
        ClassElement::ClassStaticMethod {
            name: _,
            params: _,
            body: ddlog_std::Option::Some { x: ref body },
        } => (ddlog_std::Option::Some { x: (*body).clone() }),
        _ => (ddlog_std::Option::None {}),
    }
}
pub fn bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(
    pat: &IPattern,
) -> ddlog_std::Vec<Spanned<Name>> {
    match (*internment::ival(pat)) {
        Pattern::SinglePattern{name: ddlog_std::Option::Some{x: ref name}} => {
                                                                                  let ref mut __vec: ddlog_std::Vec<Spanned<Name>> = (*(&*crate::__STATIC_0)).clone();
                                                                                  ddlog_std::push::<Spanned<Name>>(__vec, name);
                                                                                  (*__vec).clone()
                                                                              },
        Pattern::RestPattern{rest: ddlog_std::Option::Some{x: ref rest}} => bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(rest),
        Pattern::AssignPattern{key: ddlog_std::Option::Some{x: ref key}, value: _} => bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(key),
        Pattern::ObjectPattern{props: ref props} => types__vec::flatmap::<internment::Intern<ObjectPatternProp>, Spanned<Name>>(props, (&{
                                                                                                                                             (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                                 description: "(function(prop: internment::Intern<ast::ObjectPatternProp>):ddlog_std::Vec<ast::Spanned<ast::Name>>{(ast::bound_vars(prop))})",
                                                                                                                                                 captured: (),
                                                                                                                                                 f: {
                                                                                                                                                        fn __f(__args:*const internment::Intern<ObjectPatternProp>, __captured: &()) -> ddlog_std::Vec<Spanned<Name>>
                                                                                                                                                        {
                                                                                                                                                            let prop = unsafe{&*__args};
                                                                                                                                                            bound_vars_internment_Intern__ast_ObjectPatternProp_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(prop)
                                                                                                                                                        }
                                                                                                                                                        __f
                                                                                                                                                    }
                                                                                                                                             }) as Box<dyn ::ddlog_rt::Closure<(*const internment::Intern<ObjectPatternProp>), ddlog_std::Vec<Spanned<Name>>>>)
                                                                                                                                         })),
        Pattern::ArrayPattern{elems: ref elems} => types__vec::flatmap::<internment::Intern<Pattern>, Spanned<Name>>(elems, (&{
                                                                                                                                  (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                      description: "(function(elem: internment::Intern<ast::Pattern>):ddlog_std::Vec<ast::Spanned<ast::Name>>{(ast::bound_vars(elem))})",
                                                                                                                                      captured: (),
                                                                                                                                      f: {
                                                                                                                                             fn __f(__args:*const internment::Intern<Pattern>, __captured: &()) -> ddlog_std::Vec<Spanned<Name>>
                                                                                                                                             {
                                                                                                                                                 let elem = unsafe{&*__args};
                                                                                                                                                 bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(elem)
                                                                                                                                             }
                                                                                                                                             __f
                                                                                                                                         }
                                                                                                                                  }) as Box<dyn ::ddlog_rt::Closure<(*const internment::Intern<Pattern>), ddlog_std::Vec<Spanned<Name>>>>)
                                                                                                                              })),
        _ => (*(&*crate::__STATIC_1)).clone()
    }
}
pub fn bound_vars_internment_Intern__ast_ObjectPatternProp_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(
    pat: &IObjectPatternProp,
) -> ddlog_std::Vec<Spanned<Name>> {
    match (*internment::ival(pat)) {
        ObjectPatternProp::ObjAssignPattern{assign_key: ddlog_std::Option::Some{x: ref key}, assign_value: _} => bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(key),
        ObjectPatternProp::ObjKeyValuePattern{key: _, value: ddlog_std::Option::Some{x: ref value}} => bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(value),
        ObjectPatternProp::ObjRestPattern{rest: ddlog_std::Option::Some{x: ref rest}} => bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(rest),
        ObjectPatternProp::ObjSinglePattern{name: ddlog_std::Option::Some{x: ref name}} => {
                                                                                               let ref mut __vec: ddlog_std::Vec<Spanned<Name>> = (*(&*crate::__STATIC_0)).clone();
                                                                                               ddlog_std::push::<Spanned<Name>>(__vec, name);
                                                                                               (*__vec).clone()
                                                                                           },
        _ => (*(&*crate::__STATIC_1)).clone()
    }
}
pub fn bound_vars_ast_FuncParam_ddlog_std_Vec____Tuple2__ast_Spanned__internment_Intern____Stringval___Boolval(
    param: &FuncParam,
) -> ddlog_std::Vec<ddlog_std::tuple2<Spanned<Name>, bool>> {
    types__vec::map::<Spanned<Name>, ddlog_std::tuple2<Spanned<Name>, bool>>((&bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval((&param.pattern))), (&{
                                                                                                                                                                                                              (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                                                                                                  description: "(function(v: ast::Spanned<ast::Name>):(ast::Spanned<ast::Name>, bool){(v, (param.implicit))})",
                                                                                                                                                                                                                  captured: param.clone(),
                                                                                                                                                                                                                  f: {
                                                                                                                                                                                                                         fn __f(__args:*const Spanned<Name>, __captured: &FuncParam) -> ddlog_std::tuple2<Spanned<Name>, bool>
                                                                                                                                                                                                                         {
                                                                                                                                                                                                                             let param = __captured;
                                                                                                                                                                                                                             let v = unsafe{&*__args};
                                                                                                                                                                                                                             ddlog_std::tuple2((*v).clone(), param.implicit.clone())
                                                                                                                                                                                                                         }
                                                                                                                                                                                                                         __f
                                                                                                                                                                                                                     }
                                                                                                                                                                                                              }) as Box<dyn ::ddlog_rt::Closure<(*const Spanned<Name>), ddlog_std::tuple2<Spanned<Name>, bool>>>)
                                                                                                                                                                                                          }))
}
pub fn free_variable(clause: &NamedImport) -> ddlog_std::Option<Spanned<Name>> {
    types__utils::or_else::<Spanned<Name>>((&clause.alias), (&clause.name))
}
pub fn free_variables(clause: &ImportClause) -> ddlog_std::Vec<Spanned<Name>> {
    match (*clause) {
        ImportClause::WildcardImport {
            alias: ddlog_std::Option::Some { x: ref alias },
        } => {
            let ref mut __vec: ddlog_std::Vec<Spanned<Name>> = (*(&*crate::__STATIC_0)).clone();
            ddlog_std::push::<Spanned<Name>>(__vec, alias);
            (*__vec).clone()
        }
        ImportClause::GroupedImport {
            imports: ref imports,
        } => types__vec::filter_map::<NamedImport, Spanned<Name>>(
            imports,
            (&{
                (Box::new(::ddlog_rt::ClosureImpl{
                                                                                                                                     description: "(function(named: ast::NamedImport):ddlog_std::Option<ast::Spanned<ast::Name>>{(ast::free_variable(named))})",
                                                                                                                                     captured: (),
                                                                                                                                     f: {
                                                                                                                                            fn __f(__args:*const NamedImport, __captured: &()) -> ddlog_std::Option<Spanned<Name>>
                                                                                                                                            {
                                                                                                                                                let named = unsafe{&*__args};
                                                                                                                                                free_variable(named)
                                                                                                                                            }
                                                                                                                                            __f
                                                                                                                                        }
                                                                                                                                 }) as Box<dyn ::ddlog_rt::Closure<(*const NamedImport), ddlog_std::Option<Spanned<Name>>>>)
            }),
        ),
        ImportClause::SingleImport { name: ref name } => {
            let ref mut __vec: ddlog_std::Vec<Spanned<Name>> = (*(&*crate::__STATIC_0)).clone();
            ddlog_std::push::<Spanned<Name>>(__vec, name);
            (*__vec).clone()
        }
        _ => (*(&*crate::__STATIC_1)).clone(),
    }
}
pub fn is_expr(id: &AnyId) -> bool {
    match (*id) {
        AnyId::AnyIdExpr { expr: _ } => true,
        _ => false,
    }
}
pub fn is_function(id: &AnyId) -> bool {
    match (*id) {
        AnyId::AnyIdFunc { func: _ } => true,
        _ => false,
    }
}
pub fn is_global(id: &AnyId) -> bool {
    match (*id) {
        AnyId::AnyIdGlobal { global: _ } => true,
        _ => false,
    }
}
pub fn is_variable_decl(kind: &StmtKind) -> bool {
    ((((&*kind) == (&*(&(StmtKind::StmtVarDecl {}))))
        || ((&*kind) == (&*(&(StmtKind::StmtLetDecl {})))))
        || ((&*kind) == (&*(&(StmtKind::StmtConstDecl {})))))
}
pub fn method_comps_ast_PropertyVal_ddlog_std_Option____Tuple2__ddlog_std_Vec__ast_FuncParam_ast_StmtId(
    prop: &PropertyVal,
) -> ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Vec<FuncParam>, StmtId>> {
    match (*prop) {
        PropertyVal::PropSetter {
            params: ddlog_std::Option::Some { x: ref params },
            body: ddlog_std::Option::Some { x: ref body },
        } => {
            (ddlog_std::Option::Some {
                x: ddlog_std::tuple2((*params).clone(), (*body).clone()),
            })
        }
        PropertyVal::PropMethod {
            params: ddlog_std::Option::Some { x: ref params },
            body: ddlog_std::Option::Some { x: ref body },
        } => {
            (ddlog_std::Option::Some {
                x: ddlog_std::tuple2((*params).clone(), (*body).clone()),
            })
        }
        _ => (ddlog_std::Option::None {}),
    }
}
pub fn method_comps_ast_ClassElement_ddlog_std_Option____Tuple2__ddlog_std_Vec__ast_FuncParam_ast_StmtId(
    elem: &ClassElement,
) -> ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Vec<FuncParam>, StmtId>> {
    match (*elem) {
        ClassElement::ClassMethod {
            name: _,
            params: ddlog_std::Option::Some { x: ref params },
            body: ddlog_std::Option::Some { x: ref body },
        } => {
            (ddlog_std::Option::Some {
                x: ddlog_std::tuple2((*params).clone(), (*body).clone()),
            })
        }
        ClassElement::ClassStaticMethod {
            name: _,
            params: ddlog_std::Option::Some { x: ref params },
            body: ddlog_std::Option::Some { x: ref body },
        } => {
            (ddlog_std::Option::Some {
                x: ddlog_std::tuple2((*params).clone(), (*body).clone()),
            })
        }
        _ => (ddlog_std::Option::None {}),
    }
}
pub fn to_string_ast_ScopeId___Stringval(scope: &ScopeId) -> String {
    ::ddlog_rt::string_append(
        String::from(r###"Scope_"###),
        (&ddlog_std::__builtin_2string((&scope.id))),
    )
}
pub fn to_string_ast_AnyId___Stringval(id: &AnyId) -> String {
    match (*id) {
        AnyId::AnyIdGlobal {
            global: GlobalId { id: ref id },
        } => ::ddlog_rt::string_append(
            String::from(r###"Global_"###),
            (&ddlog_std::__builtin_2string(id)),
        ),
        AnyId::AnyIdImport {
            import_: ImportId { id: ref id },
        } => ::ddlog_rt::string_append(
            String::from(r###"Import_"###),
            (&ddlog_std::__builtin_2string(id)),
        ),
        AnyId::AnyIdClass {
            class: ClassId { id: ref id },
        } => ::ddlog_rt::string_append(
            String::from(r###"Class_"###),
            (&ddlog_std::__builtin_2string(id)),
        ),
        AnyId::AnyIdFunc {
            func: FuncId { id: ref id },
        } => ::ddlog_rt::string_append(
            String::from(r###"Func_"###),
            (&ddlog_std::__builtin_2string(id)),
        ),
        AnyId::AnyIdStmt {
            stmt: StmtId { id: ref id },
        } => ::ddlog_rt::string_append(
            String::from(r###"Stmt_"###),
            (&ddlog_std::__builtin_2string(id)),
        ),
        AnyId::AnyIdExpr {
            expr: ExprId { id: ref id },
        } => ::ddlog_rt::string_append(
            String::from(r###"Expr_"###),
            (&ddlog_std::__builtin_2string(id)),
        ),
        AnyId::AnyIdFile {
            file: FileId { id: ref id },
        } => ::ddlog_rt::string_append(
            String::from(r###"File_"###),
            (&ddlog_std::__builtin_2string(id)),
        ),
    }
}
pub fn to_string_ast_Span___Stringval(span: &Span) -> String {
    ::ddlog_rt::string_append_str(
        ::ddlog_rt::string_append(
            ::ddlog_rt::string_append_str(
                ::ddlog_rt::string_append(
                    String::from(r###"("###),
                    (&ddlog_std::__builtin_2string((&span.start))),
                ),
                r###", "###,
            ),
            (&ddlog_std::__builtin_2string((&span.end))),
        ),
        r###")"###,
    )
}
