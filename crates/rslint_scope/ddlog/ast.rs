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
