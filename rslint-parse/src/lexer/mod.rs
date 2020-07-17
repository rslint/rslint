//! A fast, lossless, lookup table and trie based lexer for ECMAScript used by the RSLint parser.

pub mod lexer;
pub mod token;
pub mod util;
pub mod identifier;
pub mod error;
pub mod state;
pub mod tests;
pub mod numbers;
pub mod lookup;