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


pub static __STATIC_1: ::once_cell::sync::Lazy<ddlog_std::Vec<Spanned<Name>>> = ::once_cell::sync::Lazy::new(|| ddlog_std::vec_empty());
pub static __STATIC_0: ::once_cell::sync::Lazy<ddlog_std::Vec<Spanned<Name>>> = ::once_cell::sync::Lazy::new(|| ddlog_std::vec_with_capacity((&(1 as u64))));
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

#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::AnyId")]
pub enum AnyId {
    #[ddlog(rename = "ast::AnyIdGlobal")]
    AnyIdGlobal {
        global: GlobalId
    },
    #[ddlog(rename = "ast::AnyIdImport")]
    AnyIdImport {
        import_: ImportId
    },
    #[ddlog(rename = "ast::AnyIdClass")]
    AnyIdClass {
        class: ClassId
    },
    #[ddlog(rename = "ast::AnyIdFunc")]
    AnyIdFunc {
        func: FuncId
    },
    #[ddlog(rename = "ast::AnyIdStmt")]
    AnyIdStmt {
        stmt: StmtId
    },
    #[ddlog(rename = "ast::AnyIdExpr")]
    AnyIdExpr {
        expr: ExprId
    },
    #[ddlog(rename = "ast::AnyIdFile")]
    AnyIdFile {
        file: FileId
    }
}
impl abomonation::Abomonation for AnyId{}
impl ::std::fmt::Display for AnyId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            AnyId::AnyIdGlobal{global} => {
                __formatter.write_str("ast::AnyIdGlobal{")?;
                ::std::fmt::Debug::fmt(global, __formatter)?;
                __formatter.write_str("}")
            },
            AnyId::AnyIdImport{import_} => {
                __formatter.write_str("ast::AnyIdImport{")?;
                ::std::fmt::Debug::fmt(import_, __formatter)?;
                __formatter.write_str("}")
            },
            AnyId::AnyIdClass{class} => {
                __formatter.write_str("ast::AnyIdClass{")?;
                ::std::fmt::Debug::fmt(class, __formatter)?;
                __formatter.write_str("}")
            },
            AnyId::AnyIdFunc{func} => {
                __formatter.write_str("ast::AnyIdFunc{")?;
                ::std::fmt::Debug::fmt(func, __formatter)?;
                __formatter.write_str("}")
            },
            AnyId::AnyIdStmt{stmt} => {
                __formatter.write_str("ast::AnyIdStmt{")?;
                ::std::fmt::Debug::fmt(stmt, __formatter)?;
                __formatter.write_str("}")
            },
            AnyId::AnyIdExpr{expr} => {
                __formatter.write_str("ast::AnyIdExpr{")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            },
            AnyId::AnyIdFile{file} => {
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
        AnyId::AnyIdGlobal{global : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ArrayElement")]
pub enum ArrayElement {
    #[ddlog(rename = "ast::ArrExpr")]
    ArrExpr {
        expr: ExprId
    },
    #[ddlog(rename = "ast::ArrSpread")]
    ArrSpread {
        spread: ddlog_std::Option<ExprId>
    }
}
impl abomonation::Abomonation for ArrayElement{}
impl ::std::fmt::Display for ArrayElement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ArrayElement::ArrExpr{expr} => {
                __formatter.write_str("ast::ArrExpr{")?;
                ::std::fmt::Debug::fmt(expr, __formatter)?;
                __formatter.write_str("}")
            },
            ArrayElement::ArrSpread{spread} => {
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
        ArrayElement::ArrExpr{expr : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::AssignOperand")]
pub enum AssignOperand {
    #[ddlog(rename = "ast::OpAssign")]
    OpAssign,
    #[ddlog(rename = "ast::OpAddAssign")]
    OpAddAssign,
    #[ddlog(rename = "ast::OpSubtractAssign")]
    OpSubtractAssign,
    #[ddlog(rename = "ast::OpTimesAssign")]
    OpTimesAssign,
    #[ddlog(rename = "ast::OpRemainderAssign")]
    OpRemainderAssign,
    #[ddlog(rename = "ast::OpExponentAssign")]
    OpExponentAssign,
    #[ddlog(rename = "ast::OpLeftShiftAssign")]
    OpLeftShiftAssign,
    #[ddlog(rename = "ast::OpRightShiftAssign")]
    OpRightShiftAssign,
    #[ddlog(rename = "ast::OpUnsignedRightShiftAssign")]
    OpUnsignedRightShiftAssign,
    #[ddlog(rename = "ast::OpBitwiseAndAssign")]
    OpBitwiseAndAssign,
    #[ddlog(rename = "ast::OpBitwiseOrAssign")]
    OpBitwiseOrAssign,
    #[ddlog(rename = "ast::OpBitwiseXorAssign")]
    OpBitwiseXorAssign,
    #[ddlog(rename = "ast::OpLogicalAndAssign")]
    OpLogicalAndAssign,
    #[ddlog(rename = "ast::OpLogicalOrAssign")]
    OpLogicalOrAssign,
    #[ddlog(rename = "ast::OpNullishCoalescingAssign")]
    OpNullishCoalescingAssign
}
impl abomonation::Abomonation for AssignOperand{}
impl ::std::fmt::Display for AssignOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            AssignOperand::OpAssign{} => {
                __formatter.write_str("ast::OpAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpAddAssign{} => {
                __formatter.write_str("ast::OpAddAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpSubtractAssign{} => {
                __formatter.write_str("ast::OpSubtractAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpTimesAssign{} => {
                __formatter.write_str("ast::OpTimesAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpRemainderAssign{} => {
                __formatter.write_str("ast::OpRemainderAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpExponentAssign{} => {
                __formatter.write_str("ast::OpExponentAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpLeftShiftAssign{} => {
                __formatter.write_str("ast::OpLeftShiftAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpRightShiftAssign{} => {
                __formatter.write_str("ast::OpRightShiftAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpUnsignedRightShiftAssign{} => {
                __formatter.write_str("ast::OpUnsignedRightShiftAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpBitwiseAndAssign{} => {
                __formatter.write_str("ast::OpBitwiseAndAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpBitwiseOrAssign{} => {
                __formatter.write_str("ast::OpBitwiseOrAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpBitwiseXorAssign{} => {
                __formatter.write_str("ast::OpBitwiseXorAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpLogicalAndAssign{} => {
                __formatter.write_str("ast::OpLogicalAndAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpLogicalOrAssign{} => {
                __formatter.write_str("ast::OpLogicalOrAssign{")?;
                __formatter.write_str("}")
            },
            AssignOperand::OpNullishCoalescingAssign{} => {
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
        AssignOperand::OpAssign{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::BinOperand")]
pub enum BinOperand {
    #[ddlog(rename = "ast::BinLessThan")]
    BinLessThan,
    #[ddlog(rename = "ast::BinGreaterThan")]
    BinGreaterThan,
    #[ddlog(rename = "ast::BinLessThanOrEqual")]
    BinLessThanOrEqual,
    #[ddlog(rename = "ast::BinGreaterThanOrEqual")]
    BinGreaterThanOrEqual,
    #[ddlog(rename = "ast::BinEquality")]
    BinEquality,
    #[ddlog(rename = "ast::BinStrictEquality")]
    BinStrictEquality,
    #[ddlog(rename = "ast::BinInequality")]
    BinInequality,
    #[ddlog(rename = "ast::BinStrictInequality")]
    BinStrictInequality,
    #[ddlog(rename = "ast::BinPlus")]
    BinPlus,
    #[ddlog(rename = "ast::BinMinus")]
    BinMinus,
    #[ddlog(rename = "ast::BinTimes")]
    BinTimes,
    #[ddlog(rename = "ast::BinDivide")]
    BinDivide,
    #[ddlog(rename = "ast::BinRemainder")]
    BinRemainder,
    #[ddlog(rename = "ast::BinExponent")]
    BinExponent,
    #[ddlog(rename = "ast::BinLeftShift")]
    BinLeftShift,
    #[ddlog(rename = "ast::BinRightShift")]
    BinRightShift,
    #[ddlog(rename = "ast::BinUnsignedRightShift")]
    BinUnsignedRightShift,
    #[ddlog(rename = "ast::BinBitwiseAnd")]
    BinBitwiseAnd,
    #[ddlog(rename = "ast::BinBitwiseOr")]
    BinBitwiseOr,
    #[ddlog(rename = "ast::BinBitwiseXor")]
    BinBitwiseXor,
    #[ddlog(rename = "ast::BinNullishCoalescing")]
    BinNullishCoalescing,
    #[ddlog(rename = "ast::BinLogicalOr")]
    BinLogicalOr,
    #[ddlog(rename = "ast::BinLogicalAnd")]
    BinLogicalAnd,
    #[ddlog(rename = "ast::BinIn")]
    BinIn,
    #[ddlog(rename = "ast::BinInstanceof")]
    BinInstanceof
}
impl abomonation::Abomonation for BinOperand{}
impl ::std::fmt::Display for BinOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            BinOperand::BinLessThan{} => {
                __formatter.write_str("ast::BinLessThan{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinGreaterThan{} => {
                __formatter.write_str("ast::BinGreaterThan{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinLessThanOrEqual{} => {
                __formatter.write_str("ast::BinLessThanOrEqual{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinGreaterThanOrEqual{} => {
                __formatter.write_str("ast::BinGreaterThanOrEqual{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinEquality{} => {
                __formatter.write_str("ast::BinEquality{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinStrictEquality{} => {
                __formatter.write_str("ast::BinStrictEquality{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinInequality{} => {
                __formatter.write_str("ast::BinInequality{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinStrictInequality{} => {
                __formatter.write_str("ast::BinStrictInequality{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinPlus{} => {
                __formatter.write_str("ast::BinPlus{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinMinus{} => {
                __formatter.write_str("ast::BinMinus{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinTimes{} => {
                __formatter.write_str("ast::BinTimes{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinDivide{} => {
                __formatter.write_str("ast::BinDivide{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinRemainder{} => {
                __formatter.write_str("ast::BinRemainder{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinExponent{} => {
                __formatter.write_str("ast::BinExponent{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinLeftShift{} => {
                __formatter.write_str("ast::BinLeftShift{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinRightShift{} => {
                __formatter.write_str("ast::BinRightShift{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinUnsignedRightShift{} => {
                __formatter.write_str("ast::BinUnsignedRightShift{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinBitwiseAnd{} => {
                __formatter.write_str("ast::BinBitwiseAnd{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinBitwiseOr{} => {
                __formatter.write_str("ast::BinBitwiseOr{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinBitwiseXor{} => {
                __formatter.write_str("ast::BinBitwiseXor{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinNullishCoalescing{} => {
                __formatter.write_str("ast::BinNullishCoalescing{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinLogicalOr{} => {
                __formatter.write_str("ast::BinLogicalOr{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinLogicalAnd{} => {
                __formatter.write_str("ast::BinLogicalAnd{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinIn{} => {
                __formatter.write_str("ast::BinIn{")?;
                __formatter.write_str("}")
            },
            BinOperand::BinInstanceof{} => {
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
        BinOperand::BinLessThan{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ClassElement")]
pub enum ClassElement {
    #[ddlog(rename = "ast::ClassEmptyElem")]
    ClassEmptyElem,
    #[ddlog(rename = "ast::ClassMethod")]
    ClassMethod {
        name: ddlog_std::Option<PropertyKey>,
        params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>,
        body: ddlog_std::Option<StmtId>,
        is_static: bool
    }
}
impl abomonation::Abomonation for ClassElement{}
impl ::std::fmt::Display for ClassElement {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ClassElement::ClassEmptyElem{} => {
                __formatter.write_str("ast::ClassEmptyElem{")?;
                __formatter.write_str("}")
            },
            ClassElement::ClassMethod{name,params,body,is_static} => {
                __formatter.write_str("ast::ClassMethod{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(is_static, __formatter)?;
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
        ClassElement::ClassEmptyElem{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ClassId")]
pub struct ClassId {
    pub id: u32
}
impl abomonation::Abomonation for ClassId{}
impl ::std::fmt::Display for ClassId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ClassId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ExportKind")]
pub enum ExportKind {
    #[ddlog(rename = "ast::WildcardExport")]
    WildcardExport,
    #[ddlog(rename = "ast::NamedExport")]
    NamedExport {
        name: ddlog_std::Option<Spanned<Name>>,
        alias: ddlog_std::Option<Spanned<Name>>
    }
}
impl abomonation::Abomonation for ExportKind{}
impl ::std::fmt::Display for ExportKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExportKind::WildcardExport{} => {
                __formatter.write_str("ast::WildcardExport{")?;
                __formatter.write_str("}")
            },
            ExportKind::NamedExport{name,alias} => {
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
        ExportKind::WildcardExport{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ExprId")]
pub struct ExprId {
    pub id: u32
}
impl abomonation::Abomonation for ExprId{}
impl ::std::fmt::Display for ExprId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ExprKind")]
pub enum ExprKind {
    #[ddlog(rename = "ast::ExprLit")]
    ExprLit {
        kind: LitKind
    },
    #[ddlog(rename = "ast::ExprNameRef")]
    ExprNameRef,
    #[ddlog(rename = "ast::ExprYield")]
    ExprYield,
    #[ddlog(rename = "ast::ExprAwait")]
    ExprAwait,
    #[ddlog(rename = "ast::ExprArrow")]
    ExprArrow,
    #[ddlog(rename = "ast::ExprUnaryOp")]
    ExprUnaryOp,
    #[ddlog(rename = "ast::ExprBinOp")]
    ExprBinOp,
    #[ddlog(rename = "ast::ExprTernary")]
    ExprTernary,
    #[ddlog(rename = "ast::ExprThis")]
    ExprThis,
    #[ddlog(rename = "ast::ExprTemplate")]
    ExprTemplate,
    #[ddlog(rename = "ast::ExprArray")]
    ExprArray,
    #[ddlog(rename = "ast::ExprObject")]
    ExprObject,
    #[ddlog(rename = "ast::ExprGrouping")]
    ExprGrouping {
        inner: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::ExprBracket")]
    ExprBracket,
    #[ddlog(rename = "ast::ExprDot")]
    ExprDot,
    #[ddlog(rename = "ast::ExprNew")]
    ExprNew,
    #[ddlog(rename = "ast::ExprCall")]
    ExprCall,
    #[ddlog(rename = "ast::ExprAssign")]
    ExprAssign,
    #[ddlog(rename = "ast::ExprSequence")]
    ExprSequence {
        exprs: ddlog_std::Vec<ExprId>
    },
    #[ddlog(rename = "ast::ExprNewTarget")]
    ExprNewTarget,
    #[ddlog(rename = "ast::ExprImportMeta")]
    ExprImportMeta,
    #[ddlog(rename = "ast::ExprInlineFunc")]
    ExprInlineFunc,
    #[ddlog(rename = "ast::ExprSuperCall")]
    ExprSuperCall {
        args: ddlog_std::Option<ddlog_std::Vec<ExprId>>
    },
    #[ddlog(rename = "ast::ExprImportCall")]
    ExprImportCall {
        arg: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::ExprClass")]
    ExprClass,
    #[ddlog(rename = "ast::ExprUnimplemented")]
    ExprUnimplemented
}
impl abomonation::Abomonation for ExprKind{}
impl ::std::fmt::Display for ExprKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ExprKind::ExprLit{kind} => {
                __formatter.write_str("ast::ExprLit{")?;
                ::std::fmt::Debug::fmt(kind, __formatter)?;
                __formatter.write_str("}")
            },
            ExprKind::ExprNameRef{} => {
                __formatter.write_str("ast::ExprNameRef{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprYield{} => {
                __formatter.write_str("ast::ExprYield{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprAwait{} => {
                __formatter.write_str("ast::ExprAwait{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprArrow{} => {
                __formatter.write_str("ast::ExprArrow{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprUnaryOp{} => {
                __formatter.write_str("ast::ExprUnaryOp{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprBinOp{} => {
                __formatter.write_str("ast::ExprBinOp{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprTernary{} => {
                __formatter.write_str("ast::ExprTernary{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprThis{} => {
                __formatter.write_str("ast::ExprThis{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprTemplate{} => {
                __formatter.write_str("ast::ExprTemplate{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprArray{} => {
                __formatter.write_str("ast::ExprArray{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprObject{} => {
                __formatter.write_str("ast::ExprObject{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprGrouping{inner} => {
                __formatter.write_str("ast::ExprGrouping{")?;
                ::std::fmt::Debug::fmt(inner, __formatter)?;
                __formatter.write_str("}")
            },
            ExprKind::ExprBracket{} => {
                __formatter.write_str("ast::ExprBracket{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprDot{} => {
                __formatter.write_str("ast::ExprDot{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprNew{} => {
                __formatter.write_str("ast::ExprNew{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprCall{} => {
                __formatter.write_str("ast::ExprCall{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprAssign{} => {
                __formatter.write_str("ast::ExprAssign{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprSequence{exprs} => {
                __formatter.write_str("ast::ExprSequence{")?;
                ::std::fmt::Debug::fmt(exprs, __formatter)?;
                __formatter.write_str("}")
            },
            ExprKind::ExprNewTarget{} => {
                __formatter.write_str("ast::ExprNewTarget{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprImportMeta{} => {
                __formatter.write_str("ast::ExprImportMeta{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprInlineFunc{} => {
                __formatter.write_str("ast::ExprInlineFunc{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprSuperCall{args} => {
                __formatter.write_str("ast::ExprSuperCall{")?;
                ::std::fmt::Debug::fmt(args, __formatter)?;
                __formatter.write_str("}")
            },
            ExprKind::ExprImportCall{arg} => {
                __formatter.write_str("ast::ExprImportCall{")?;
                ::std::fmt::Debug::fmt(arg, __formatter)?;
                __formatter.write_str("}")
            },
            ExprKind::ExprClass{} => {
                __formatter.write_str("ast::ExprClass{")?;
                __formatter.write_str("}")
            },
            ExprKind::ExprUnimplemented{} => {
                __formatter.write_str("ast::ExprUnimplemented{")?;
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
        ExprKind::ExprLit{kind : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::FileId")]
pub struct FileId {
    pub id: u32
}
impl abomonation::Abomonation for FileId{}
impl ::std::fmt::Display for FileId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FileId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::FileKind")]
pub enum FileKind {
    #[ddlog(rename = "ast::JavaScript")]
    JavaScript {
        flavor: JSFlavor
    },
    #[ddlog(rename = "ast::Todo")]
    Todo
}
impl abomonation::Abomonation for FileKind{}
impl ::std::fmt::Display for FileKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FileKind::JavaScript{flavor} => {
                __formatter.write_str("ast::JavaScript{")?;
                ::std::fmt::Debug::fmt(flavor, __formatter)?;
                __formatter.write_str("}")
            },
            FileKind::Todo{} => {
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
        FileKind::JavaScript{flavor : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ForInit")]
pub enum ForInit {
    #[ddlog(rename = "ast::ForDecl")]
    ForDecl {
        stmt_id: ddlog_std::Option<StmtId>
    },
    #[ddlog(rename = "ast::ForExpr")]
    ForExpr {
        expr_id: ExprId
    }
}
impl abomonation::Abomonation for ForInit{}
impl ::std::fmt::Display for ForInit {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ForInit::ForDecl{stmt_id} => {
                __formatter.write_str("ast::ForDecl{")?;
                ::std::fmt::Debug::fmt(stmt_id, __formatter)?;
                __formatter.write_str("}")
            },
            ForInit::ForExpr{expr_id} => {
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
        ForInit::ForDecl{stmt_id : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::FuncId")]
pub struct FuncId {
    pub id: u32
}
impl abomonation::Abomonation for FuncId{}
impl ::std::fmt::Display for FuncId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FuncId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::FuncParam")]
pub struct FuncParam {
    pub pattern: IPattern,
    pub implicit: bool
}
impl abomonation::Abomonation for FuncParam{}
impl ::std::fmt::Display for FuncParam {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            FuncParam{pattern,implicit} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::GlobalId")]
pub struct GlobalId {
    pub id: u32
}
impl abomonation::Abomonation for GlobalId{}
impl ::std::fmt::Display for GlobalId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            GlobalId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::GlobalPriv")]
pub enum GlobalPriv {
    #[ddlog(rename = "ast::ReadonlyGlobal")]
    ReadonlyGlobal,
    #[ddlog(rename = "ast::ReadWriteGlobal")]
    ReadWriteGlobal
}
impl abomonation::Abomonation for GlobalPriv{}
impl ::std::fmt::Display for GlobalPriv {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            GlobalPriv::ReadonlyGlobal{} => {
                __formatter.write_str("ast::ReadonlyGlobal{")?;
                __formatter.write_str("}")
            },
            GlobalPriv::ReadWriteGlobal{} => {
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
        GlobalPriv::ReadonlyGlobal{}
    }
}
pub type IClassElement = internment::Intern<ClassElement>;
pub type IObjectPatternProp = internment::Intern<ObjectPatternProp>;
pub type IPattern = internment::Intern<Pattern>;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ImportClause")]
pub enum ImportClause {
    #[ddlog(rename = "ast::WildcardImport")]
    WildcardImport {
        alias: ddlog_std::Option<Spanned<Name>>
    },
    #[ddlog(rename = "ast::GroupedImport")]
    GroupedImport {
        imports: ddlog_std::Vec<NamedImport>
    },
    #[ddlog(rename = "ast::SingleImport")]
    SingleImport {
        name: Spanned<Name>
    }
}
impl abomonation::Abomonation for ImportClause{}
impl ::std::fmt::Display for ImportClause {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImportClause::WildcardImport{alias} => {
                __formatter.write_str("ast::WildcardImport{")?;
                ::std::fmt::Debug::fmt(alias, __formatter)?;
                __formatter.write_str("}")
            },
            ImportClause::GroupedImport{imports} => {
                __formatter.write_str("ast::GroupedImport{")?;
                ::std::fmt::Debug::fmt(imports, __formatter)?;
                __formatter.write_str("}")
            },
            ImportClause::SingleImport{name} => {
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
        ImportClause::WildcardImport{alias : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ImportId")]
pub struct ImportId {
    pub id: u32
}
impl abomonation::Abomonation for ImportId{}
impl ::std::fmt::Display for ImportId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ImportId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::JSFlavor")]
pub enum JSFlavor {
    #[ddlog(rename = "ast::Vanilla")]
    Vanilla,
    #[ddlog(rename = "ast::Module")]
    Module,
    #[ddlog(rename = "ast::TypeScript")]
    TypeScript
}
impl abomonation::Abomonation for JSFlavor{}
impl ::std::fmt::Display for JSFlavor {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            JSFlavor::Vanilla{} => {
                __formatter.write_str("ast::Vanilla{")?;
                __formatter.write_str("}")
            },
            JSFlavor::Module{} => {
                __formatter.write_str("ast::Module{")?;
                __formatter.write_str("}")
            },
            JSFlavor::TypeScript{} => {
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
        JSFlavor::Vanilla{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::LitKind")]
pub enum LitKind {
    #[ddlog(rename = "ast::LitNumber")]
    LitNumber,
    #[ddlog(rename = "ast::LitBigInt")]
    LitBigInt,
    #[ddlog(rename = "ast::LitString")]
    LitString,
    #[ddlog(rename = "ast::LitNull")]
    LitNull,
    #[ddlog(rename = "ast::LitBool")]
    LitBool,
    #[ddlog(rename = "ast::LitRegex")]
    LitRegex
}
impl abomonation::Abomonation for LitKind{}
impl ::std::fmt::Display for LitKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            LitKind::LitNumber{} => {
                __formatter.write_str("ast::LitNumber{")?;
                __formatter.write_str("}")
            },
            LitKind::LitBigInt{} => {
                __formatter.write_str("ast::LitBigInt{")?;
                __formatter.write_str("}")
            },
            LitKind::LitString{} => {
                __formatter.write_str("ast::LitString{")?;
                __formatter.write_str("}")
            },
            LitKind::LitNull{} => {
                __formatter.write_str("ast::LitNull{")?;
                __formatter.write_str("}")
            },
            LitKind::LitBool{} => {
                __formatter.write_str("ast::LitBool{")?;
                __formatter.write_str("}")
            },
            LitKind::LitRegex{} => {
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
        LitKind::LitNumber{}
    }
}
pub type Name = internment::istring;
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::NamedImport")]
pub struct NamedImport {
    pub name: ddlog_std::Option<Spanned<Name>>,
    pub alias: ddlog_std::Option<Spanned<Name>>
}
impl abomonation::Abomonation for NamedImport{}
impl ::std::fmt::Display for NamedImport {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            NamedImport{name,alias} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ObjectPatternProp")]
pub enum ObjectPatternProp {
    #[ddlog(rename = "ast::ObjAssignPattern")]
    ObjAssignPattern {
        assign_key: ddlog_std::Option<internment::Intern<Pattern>>,
        assign_value: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::ObjKeyValuePattern")]
    ObjKeyValuePattern {
        key: ddlog_std::Option<PropertyKey>,
        value: ddlog_std::Option<internment::Intern<Pattern>>
    },
    #[ddlog(rename = "ast::ObjRestPattern")]
    ObjRestPattern {
        rest: ddlog_std::Option<internment::Intern<Pattern>>
    },
    #[ddlog(rename = "ast::ObjSinglePattern")]
    ObjSinglePattern {
        name: ddlog_std::Option<Spanned<Name>>
    }
}
impl abomonation::Abomonation for ObjectPatternProp{}
impl ::std::fmt::Display for ObjectPatternProp {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ObjectPatternProp::ObjAssignPattern{assign_key,assign_value} => {
                __formatter.write_str("ast::ObjAssignPattern{")?;
                ::std::fmt::Debug::fmt(assign_key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(assign_value, __formatter)?;
                __formatter.write_str("}")
            },
            ObjectPatternProp::ObjKeyValuePattern{key,value} => {
                __formatter.write_str("ast::ObjKeyValuePattern{")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            ObjectPatternProp::ObjRestPattern{rest} => {
                __formatter.write_str("ast::ObjRestPattern{")?;
                ::std::fmt::Debug::fmt(rest, __formatter)?;
                __formatter.write_str("}")
            },
            ObjectPatternProp::ObjSinglePattern{name} => {
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
        ObjectPatternProp::ObjAssignPattern{assign_key : ::std::default::Default::default(), assign_value : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::OneOf")]
pub enum OneOf<A, B, C> {
    #[ddlog(rename = "ast::First")]
    First {
        a: A
    },
    #[ddlog(rename = "ast::Second")]
    Second {
        b: B
    },
    #[ddlog(rename = "ast::Third")]
    Third {
        c: C
    }
}
impl <A: ::ddlog_rt::Val, B: ::ddlog_rt::Val, C: ::ddlog_rt::Val> abomonation::Abomonation for OneOf<A, B, C>{}
impl <A: ::std::fmt::Debug, B: ::std::fmt::Debug, C: ::std::fmt::Debug> ::std::fmt::Display for OneOf<A, B, C> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            OneOf::First{a} => {
                __formatter.write_str("ast::First{")?;
                ::std::fmt::Debug::fmt(a, __formatter)?;
                __formatter.write_str("}")
            },
            OneOf::Second{b} => {
                __formatter.write_str("ast::Second{")?;
                ::std::fmt::Debug::fmt(b, __formatter)?;
                __formatter.write_str("}")
            },
            OneOf::Third{c} => {
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
        OneOf::First{a : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::Pattern")]
pub enum Pattern {
    #[ddlog(rename = "ast::SinglePattern")]
    SinglePattern {
        name: ddlog_std::Option<Spanned<Name>>
    },
    #[ddlog(rename = "ast::RestPattern")]
    RestPattern {
        rest: ddlog_std::Option<internment::Intern<Pattern>>
    },
    #[ddlog(rename = "ast::AssignPattern")]
    AssignPattern {
        key: ddlog_std::Option<internment::Intern<Pattern>>,
        value: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::ObjectPattern")]
    ObjectPattern {
        props: ddlog_std::Vec<internment::Intern<ObjectPatternProp>>
    },
    #[ddlog(rename = "ast::ArrayPattern")]
    ArrayPattern {
        elems: ddlog_std::Vec<internment::Intern<Pattern>>
    }
}
impl abomonation::Abomonation for Pattern{}
impl ::std::fmt::Display for Pattern {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Pattern::SinglePattern{name} => {
                __formatter.write_str("ast::SinglePattern{")?;
                ::std::fmt::Debug::fmt(name, __formatter)?;
                __formatter.write_str("}")
            },
            Pattern::RestPattern{rest} => {
                __formatter.write_str("ast::RestPattern{")?;
                ::std::fmt::Debug::fmt(rest, __formatter)?;
                __formatter.write_str("}")
            },
            Pattern::AssignPattern{key,value} => {
                __formatter.write_str("ast::AssignPattern{")?;
                ::std::fmt::Debug::fmt(key, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            Pattern::ObjectPattern{props} => {
                __formatter.write_str("ast::ObjectPattern{")?;
                ::std::fmt::Debug::fmt(props, __formatter)?;
                __formatter.write_str("}")
            },
            Pattern::ArrayPattern{elems} => {
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
        Pattern::SinglePattern{name : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::PropertyKey")]
pub enum PropertyKey {
    #[ddlog(rename = "ast::ComputedKey")]
    ComputedKey {
        prop: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::LiteralKey")]
    LiteralKey {
        lit: ExprId
    },
    #[ddlog(rename = "ast::IdentKey")]
    IdentKey {
        ident: Spanned<Name>
    }
}
impl abomonation::Abomonation for PropertyKey{}
impl ::std::fmt::Display for PropertyKey {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            PropertyKey::ComputedKey{prop} => {
                __formatter.write_str("ast::ComputedKey{")?;
                ::std::fmt::Debug::fmt(prop, __formatter)?;
                __formatter.write_str("}")
            },
            PropertyKey::LiteralKey{lit} => {
                __formatter.write_str("ast::LiteralKey{")?;
                ::std::fmt::Debug::fmt(lit, __formatter)?;
                __formatter.write_str("}")
            },
            PropertyKey::IdentKey{ident} => {
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
        PropertyKey::ComputedKey{prop : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::PropertyVal")]
pub enum PropertyVal {
    #[ddlog(rename = "ast::PropLit")]
    PropLit {
        lit: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::PropGetter")]
    PropGetter {
        body: ddlog_std::Option<StmtId>
    },
    #[ddlog(rename = "ast::PropSetter")]
    PropSetter {
        params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>,
        body: ddlog_std::Option<StmtId>
    },
    #[ddlog(rename = "ast::PropSpread")]
    PropSpread {
        value: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::PropInit")]
    PropInit {
        value: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::PropIdent")]
    PropIdent,
    #[ddlog(rename = "ast::PropMethod")]
    PropMethod {
        params: ddlog_std::Option<ddlog_std::Vec<FuncParam>>,
        body: ddlog_std::Option<StmtId>
    }
}
impl abomonation::Abomonation for PropertyVal{}
impl ::std::fmt::Display for PropertyVal {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            PropertyVal::PropLit{lit} => {
                __formatter.write_str("ast::PropLit{")?;
                ::std::fmt::Debug::fmt(lit, __formatter)?;
                __formatter.write_str("}")
            },
            PropertyVal::PropGetter{body} => {
                __formatter.write_str("ast::PropGetter{")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            },
            PropertyVal::PropSetter{params,body} => {
                __formatter.write_str("ast::PropSetter{")?;
                ::std::fmt::Debug::fmt(params, __formatter)?;
                __formatter.write_str(",")?;
                ::std::fmt::Debug::fmt(body, __formatter)?;
                __formatter.write_str("}")
            },
            PropertyVal::PropSpread{value} => {
                __formatter.write_str("ast::PropSpread{")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            PropertyVal::PropInit{value} => {
                __formatter.write_str("ast::PropInit{")?;
                ::std::fmt::Debug::fmt(value, __formatter)?;
                __formatter.write_str("}")
            },
            PropertyVal::PropIdent{} => {
                __formatter.write_str("ast::PropIdent{")?;
                __formatter.write_str("}")
            },
            PropertyVal::PropMethod{params,body} => {
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
        PropertyVal::PropLit{lit : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::ScopeId")]
pub struct ScopeId {
    pub id: u32
}
impl abomonation::Abomonation for ScopeId{}
impl ::std::fmt::Display for ScopeId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            ScopeId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::Span")]
pub struct Span {
    pub start: u32,
    pub end: u32
}
impl abomonation::Abomonation for Span{}
impl ::std::fmt::Display for Span {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Span{start,end} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::Spanned")]
pub struct Spanned<T> {
    pub data: T,
    pub span: Span
}
impl <T: ::ddlog_rt::Val> abomonation::Abomonation for Spanned<T>{}
impl <T: ::std::fmt::Debug> ::std::fmt::Display for Spanned<T> {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            Spanned{data,span} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::StmtId")]
pub struct StmtId {
    pub id: u32
}
impl abomonation::Abomonation for StmtId{}
impl ::std::fmt::Display for StmtId {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            StmtId{id} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::StmtKind")]
pub enum StmtKind {
    #[ddlog(rename = "ast::StmtVarDecl")]
    StmtVarDecl,
    #[ddlog(rename = "ast::StmtLetDecl")]
    StmtLetDecl,
    #[ddlog(rename = "ast::StmtConstDecl")]
    StmtConstDecl,
    #[ddlog(rename = "ast::StmtExpr")]
    StmtExpr {
        expr_id: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::StmtReturn")]
    StmtReturn,
    #[ddlog(rename = "ast::StmtIf")]
    StmtIf,
    #[ddlog(rename = "ast::StmtBreak")]
    StmtBreak,
    #[ddlog(rename = "ast::StmtDoWhile")]
    StmtDoWhile,
    #[ddlog(rename = "ast::StmtWhile")]
    StmtWhile,
    #[ddlog(rename = "ast::StmtFor")]
    StmtFor,
    #[ddlog(rename = "ast::StmtForIn")]
    StmtForIn,
    #[ddlog(rename = "ast::StmtForOf")]
    StmtForOf,
    #[ddlog(rename = "ast::StmtContinue")]
    StmtContinue,
    #[ddlog(rename = "ast::StmtWith")]
    StmtWith,
    #[ddlog(rename = "ast::StmtLabel")]
    StmtLabel,
    #[ddlog(rename = "ast::StmtSwitch")]
    StmtSwitch,
    #[ddlog(rename = "ast::StmtThrow")]
    StmtThrow,
    #[ddlog(rename = "ast::StmtTry")]
    StmtTry,
    #[ddlog(rename = "ast::StmtDebugger")]
    StmtDebugger,
    #[ddlog(rename = "ast::StmtEmpty")]
    StmtEmpty
}
impl abomonation::Abomonation for StmtKind{}
impl ::std::fmt::Display for StmtKind {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            StmtKind::StmtVarDecl{} => {
                __formatter.write_str("ast::StmtVarDecl{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtLetDecl{} => {
                __formatter.write_str("ast::StmtLetDecl{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtConstDecl{} => {
                __formatter.write_str("ast::StmtConstDecl{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtExpr{expr_id} => {
                __formatter.write_str("ast::StmtExpr{")?;
                ::std::fmt::Debug::fmt(expr_id, __formatter)?;
                __formatter.write_str("}")
            },
            StmtKind::StmtReturn{} => {
                __formatter.write_str("ast::StmtReturn{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtIf{} => {
                __formatter.write_str("ast::StmtIf{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtBreak{} => {
                __formatter.write_str("ast::StmtBreak{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtDoWhile{} => {
                __formatter.write_str("ast::StmtDoWhile{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtWhile{} => {
                __formatter.write_str("ast::StmtWhile{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtFor{} => {
                __formatter.write_str("ast::StmtFor{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtForIn{} => {
                __formatter.write_str("ast::StmtForIn{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtForOf{} => {
                __formatter.write_str("ast::StmtForOf{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtContinue{} => {
                __formatter.write_str("ast::StmtContinue{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtWith{} => {
                __formatter.write_str("ast::StmtWith{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtLabel{} => {
                __formatter.write_str("ast::StmtLabel{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtSwitch{} => {
                __formatter.write_str("ast::StmtSwitch{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtThrow{} => {
                __formatter.write_str("ast::StmtThrow{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtTry{} => {
                __formatter.write_str("ast::StmtTry{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtDebugger{} => {
                __formatter.write_str("ast::StmtDebugger{")?;
                __formatter.write_str("}")
            },
            StmtKind::StmtEmpty{} => {
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
        StmtKind::StmtVarDecl{}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::SwitchClause")]
pub enum SwitchClause {
    #[ddlog(rename = "ast::CaseClause")]
    CaseClause {
        test: ddlog_std::Option<ExprId>
    },
    #[ddlog(rename = "ast::DefaultClause")]
    DefaultClause
}
impl abomonation::Abomonation for SwitchClause{}
impl ::std::fmt::Display for SwitchClause {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            SwitchClause::CaseClause{test} => {
                __formatter.write_str("ast::CaseClause{")?;
                ::std::fmt::Debug::fmt(test, __formatter)?;
                __formatter.write_str("}")
            },
            SwitchClause::DefaultClause{} => {
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
        SwitchClause::CaseClause{test : ::std::default::Default::default()}
    }
}
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Default, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::TryHandler")]
pub struct TryHandler {
    pub error: ddlog_std::Option<IPattern>,
    pub body: ddlog_std::Option<StmtId>
}
impl abomonation::Abomonation for TryHandler{}
impl ::std::fmt::Display for TryHandler {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            TryHandler{error,body} => {
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
#[derive(Eq, Ord, Clone, Hash, PartialEq, PartialOrd, IntoRecord, Mutator, Serialize, Deserialize, FromRecord)]
#[ddlog(rename = "ast::UnaryOperand")]
pub enum UnaryOperand {
    #[ddlog(rename = "ast::UnaryIncrement")]
    UnaryIncrement,
    #[ddlog(rename = "ast::UnaryDecrement")]
    UnaryDecrement,
    #[ddlog(rename = "ast::UnaryDelete")]
    UnaryDelete,
    #[ddlog(rename = "ast::UnaryVoid")]
    UnaryVoid,
    #[ddlog(rename = "ast::UnaryTypeof")]
    UnaryTypeof,
    #[ddlog(rename = "ast::UnaryPlus")]
    UnaryPlus,
    #[ddlog(rename = "ast::UnaryMinus")]
    UnaryMinus,
    #[ddlog(rename = "ast::UnaryBitwiseNot")]
    UnaryBitwiseNot,
    #[ddlog(rename = "ast::UnaryLogicalNot")]
    UnaryLogicalNot,
    #[ddlog(rename = "ast::UnaryAwait")]
    UnaryAwait
}
impl abomonation::Abomonation for UnaryOperand{}
impl ::std::fmt::Display for UnaryOperand {
    fn fmt(&self, __formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match self {
            UnaryOperand::UnaryIncrement{} => {
                __formatter.write_str("ast::UnaryIncrement{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryDecrement{} => {
                __formatter.write_str("ast::UnaryDecrement{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryDelete{} => {
                __formatter.write_str("ast::UnaryDelete{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryVoid{} => {
                __formatter.write_str("ast::UnaryVoid{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryTypeof{} => {
                __formatter.write_str("ast::UnaryTypeof{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryPlus{} => {
                __formatter.write_str("ast::UnaryPlus{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryMinus{} => {
                __formatter.write_str("ast::UnaryMinus{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryBitwiseNot{} => {
                __formatter.write_str("ast::UnaryBitwiseNot{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryLogicalNot{} => {
                __formatter.write_str("ast::UnaryLogicalNot{")?;
                __formatter.write_str("}")
            },
            UnaryOperand::UnaryAwait{} => {
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
        UnaryOperand::UnaryIncrement{}
    }
}
pub fn any_id(global: & GlobalId) -> AnyId
{   (AnyId::AnyIdGlobal{global: (*global).clone()})
}
pub fn body_ast_PropertyVal_ddlog_std_Option__ast_StmtId(prop: & PropertyVal) -> ddlog_std::Option<StmtId>
{   match (*prop) {
        PropertyVal::PropGetter{body: ddlog_std::Option::Some{x: ref body}} => (ddlog_std::Option::Some{x: (*body).clone()}),
        PropertyVal::PropSetter{params: _, body: ddlog_std::Option::Some{x: ref body}} => (ddlog_std::Option::Some{x: (*body).clone()}),
        PropertyVal::PropMethod{params: _, body: ddlog_std::Option::Some{x: ref body}} => (ddlog_std::Option::Some{x: (*body).clone()}),
        _ => (ddlog_std::Option::None{})
    }
}
pub fn body_ast_ClassElement_ddlog_std_Option__ast_StmtId(elem: & ClassElement) -> ddlog_std::Option<StmtId>
{   match (*elem) {
        ClassElement::ClassMethod{name: _, params: _, body: ddlog_std::Option::Some{x: ref body}, is_static: _} => (ddlog_std::Option::Some{x: (*body).clone()}),
        _ => (ddlog_std::Option::None{})
    }
}
pub fn bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat: & IPattern) -> ddlog_std::Vec<Spanned<Name>>
{   match (*internment::ival(pat)) {
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
pub fn bound_vars_internment_Intern__ast_ObjectPatternProp_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval(pat: & IObjectPatternProp) -> ddlog_std::Vec<Spanned<Name>>
{   match (*internment::ival(pat)) {
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
pub fn bound_vars_ast_FuncParam_ddlog_std_Vec____Tuple2__ast_Spanned__internment_Intern____Stringval___Boolval(param: & FuncParam) -> ddlog_std::Vec<ddlog_std::tuple2<Spanned<Name>, bool>>
{   types__vec::map::<Spanned<Name>, ddlog_std::tuple2<Spanned<Name>, bool>>((&bound_vars_internment_Intern__ast_Pattern_ddlog_std_Vec__ast_Spanned__internment_Intern____Stringval((&param.pattern))), (&{
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
pub fn free_variable(clause: & NamedImport) -> ddlog_std::Option<Spanned<Name>>
{   types__utils::or_else::<Spanned<Name>>((&clause.alias), (&clause.name))
}
pub fn free_variables(clause: & ImportClause) -> ddlog_std::Vec<Spanned<Name>>
{   match (*clause) {
        ImportClause::WildcardImport{alias: ddlog_std::Option::Some{x: ref alias}} => {
                                                                                          let ref mut __vec: ddlog_std::Vec<Spanned<Name>> = (*(&*crate::__STATIC_0)).clone();
                                                                                          ddlog_std::push::<Spanned<Name>>(__vec, alias);
                                                                                          (*__vec).clone()
                                                                                      },
        ImportClause::GroupedImport{imports: ref imports} => types__vec::filter_map::<NamedImport, Spanned<Name>>(imports, (&{
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
                                                                                                                             })),
        ImportClause::SingleImport{name: ref name} => {
                                                          let ref mut __vec: ddlog_std::Vec<Spanned<Name>> = (*(&*crate::__STATIC_0)).clone();
                                                          ddlog_std::push::<Spanned<Name>>(__vec, name);
                                                          (*__vec).clone()
                                                      },
        _ => (*(&*crate::__STATIC_1)).clone()
    }
}
pub fn is_expr(id: & AnyId) -> bool
{   match (*id) {
        AnyId::AnyIdExpr{expr: _} => true,
        _ => false
    }
}
pub fn is_function(id: & AnyId) -> bool
{   match (*id) {
        AnyId::AnyIdFunc{func: _} => true,
        _ => false
    }
}
pub fn is_global(id: & AnyId) -> bool
{   match (*id) {
        AnyId::AnyIdGlobal{global: _} => true,
        _ => false
    }
}
pub fn is_variable_decl(kind: & StmtKind) -> bool
{   ((((&*kind) == (&*(&(StmtKind::StmtVarDecl{})))) || ((&*kind) == (&*(&(StmtKind::StmtLetDecl{}))))) || ((&*kind) == (&*(&(StmtKind::StmtConstDecl{})))))
}
pub fn method_comps_ast_PropertyVal_ddlog_std_Option____Tuple2__ddlog_std_Vec__ast_FuncParam_ast_StmtId(prop: & PropertyVal) -> ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Vec<FuncParam>, StmtId>>
{   match (*prop) {
        PropertyVal::PropSetter{params: ddlog_std::Option::Some{x: ref params}, body: ddlog_std::Option::Some{x: ref body}} => (ddlog_std::Option::Some{x: ddlog_std::tuple2((*params).clone(), (*body).clone())}),
        PropertyVal::PropMethod{params: ddlog_std::Option::Some{x: ref params}, body: ddlog_std::Option::Some{x: ref body}} => (ddlog_std::Option::Some{x: ddlog_std::tuple2((*params).clone(), (*body).clone())}),
        _ => (ddlog_std::Option::None{})
    }
}
pub fn method_comps_ast_ClassElement_ddlog_std_Option____Tuple2__ddlog_std_Vec__ast_FuncParam_ast_StmtId(elem: & ClassElement) -> ddlog_std::Option<ddlog_std::tuple2<ddlog_std::Vec<FuncParam>, StmtId>>
{   match (*elem) {
        ClassElement::ClassMethod{name: _, params: ddlog_std::Option::Some{x: ref params}, body: ddlog_std::Option::Some{x: ref body}, is_static: _} => (ddlog_std::Option::Some{x: ddlog_std::tuple2((*params).clone(), (*body).clone())}),
        _ => (ddlog_std::Option::None{})
    }
}
pub fn to_string_ast_ScopeId___Stringval(scope: & ScopeId) -> String
{   ::ddlog_rt::string_append(String::from(r###"Scope_"###), (&ddlog_std::__builtin_2string((&scope.id))))
}
pub fn to_string_ast_AnyId___Stringval(id: & AnyId) -> String
{   match (*id) {
        AnyId::AnyIdGlobal{global: GlobalId{id: ref id}} => ::ddlog_rt::string_append(String::from(r###"Global_"###), (&ddlog_std::__builtin_2string(id))),
        AnyId::AnyIdImport{import_: ImportId{id: ref id}} => ::ddlog_rt::string_append(String::from(r###"Import_"###), (&ddlog_std::__builtin_2string(id))),
        AnyId::AnyIdClass{class: ClassId{id: ref id}} => ::ddlog_rt::string_append(String::from(r###"Class_"###), (&ddlog_std::__builtin_2string(id))),
        AnyId::AnyIdFunc{func: FuncId{id: ref id}} => ::ddlog_rt::string_append(String::from(r###"Func_"###), (&ddlog_std::__builtin_2string(id))),
        AnyId::AnyIdStmt{stmt: StmtId{id: ref id}} => ::ddlog_rt::string_append(String::from(r###"Stmt_"###), (&ddlog_std::__builtin_2string(id))),
        AnyId::AnyIdExpr{expr: ExprId{id: ref id}} => ::ddlog_rt::string_append(String::from(r###"Expr_"###), (&ddlog_std::__builtin_2string(id))),
        AnyId::AnyIdFile{file: FileId{id: ref id}} => ::ddlog_rt::string_append(String::from(r###"File_"###), (&ddlog_std::__builtin_2string(id)))
    }
}
pub fn to_string_ast_Span___Stringval(span: & Span) -> String
{   ::ddlog_rt::string_append_str(::ddlog_rt::string_append(::ddlog_rt::string_append_str(::ddlog_rt::string_append(String::from(r###"("###), (&ddlog_std::__builtin_2string((&span.start)))), r###", "###), (&ddlog_std::__builtin_2string((&span.end)))), r###")"###)
}