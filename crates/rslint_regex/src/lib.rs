//! This crate provides a RegEx parser which targets the [RegEx syntax] specified
//! by [EcmaScript]
//!
//! [EcmaScript]: https://tc39.es/ecma262
//! [RegEx syntax]: https://tc39.es/ecma262/#sec-patterns

#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

mod ir;
mod parser;

pub use parser::*;

use rslint_errors::Diagnostic;

pub type Result<T, E = Diagnostic> = std::result::Result<T, E>;

#[derive(Debug, Clone, PartialEq, Eq)]
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
}

impl rslint_errors::Span for Span {
    fn as_range(&self) -> std::ops::Range<usize> {
        self.abs_start()..self.abs_end()
    }
}
