//! This crate provides a RegEx parser which targets the [RegEx syntax] specified
//! by [EcmaScript]
//!
//! [EcmaScript]: https://tc39.es/ecma262
//! [RegEx syntax]: https://tc39.es/ecma262/#sec-patterns

#![deny(rust_2018_idioms)]

mod ir;
#[allow(clippy::range_plus_one)]
mod parser;
#[cfg(test)]
mod tests;
mod unicode;

pub use parser::*;

use std::ops::Range;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Span {
    /// The offset in the whole file to calculate the absolute position.
    pub offset: usize,
    /// The relative start of this `Span` inside a pattern.
    pub start: usize,
    /// The relative end of this `Span` inside a pattern.
    pub end: usize,
}

impl Span {
    /// Create a new `Span`
    pub fn new(offset: usize, start: usize, end: usize) -> Self {
        Self { offset, start, end }
    }

    /// Calculates the absolute start using `self.offset + self.start`.
    pub fn abs_start(&self) -> usize {
        self.offset + self.start
    }

    /// Calculates the absolute end using `self.offset + self.end`.
    pub fn abs_end(&self) -> usize {
        self.offset + self.end
    }

    pub fn as_range(&self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Span::new(0, range.start, range.end)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    pub span: Span,
    pub message: String,
}

impl Error {
    pub fn new(message: impl ToString, span: Span) -> Self {
        Self {
            span,
            message: message.to_string(),
        }
    }

    pub(crate) fn primary(self, span: impl Into<Span>, _msg: &str) -> Self {
        Self {
            span: span.into(),
            message: self.message,
        }
    }
}
