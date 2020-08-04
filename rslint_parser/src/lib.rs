//! The JavaScript parser used by RSLint.
//! 
//! The parser uses an abstraction over non-whitespace tokens. 
//! This allows us to losslessly parse code without requiring explicit handling of whitespace. 
//! The parser yields events, not an AST, the events are resolved into untyped syntax nodes, which can then
//! be casted into a typed AST.
//! 
//! The parser is able to produce a valid AST from **any** source code. 
//! Erroneous productions are wrapped into `ERROR` syntax nodes, the original source code
//! is completely represented in the final syntax nodes.
//! 
//! You probably do not want to use this crate, unless you want to parse fragments of Js source code or make your own productions. 
//! It is a lot easier to use [rslint_syntax](../rslint_syntax/index.html) and its abstractions which are more user friendly.
//! 
//! This is derived from the rust analyzer parser but adapted for JavaScript.

#[macro_export]
mod syntax_kind;
mod parser;
mod event;
#[macro_export]
mod token_set;
mod diagnostics;

pub mod syntax;

pub use crate::{
    parser::{Marker, CompletedMarker, Parser},
    event::{process, Event},
    token_set::TokenSet,
    diagnostics::ErrorBuilder
};

pub use syntax_kind::SyntaxKind;

/// The type of error emitted by the parser, this includes warnings, notes, and errors.  
/// It also includes labels and possibly notes
pub type ParserError = codespan_reporting::diagnostic::Diagnostic<usize>;

use std::ops::Range;

/// An abstraction over the source of tokens for the parser
pub trait TokenSource {
    /// Get the current token
    fn current(&self) -> Token;

    /// Get the source code of the token source. 
    /// If the source and token ranges do not match this will result in panics while parsing.
    fn source(&self) -> &str;

    /// Lookahead n token
    fn lookahead_nth(&self, n: usize) -> Token;

    /// bump cursor to next token
    fn bump(&mut self);

    /// Whether there was a linebreak before the current token.  
    /// This is required for ASI, return, postfix, etc.
    fn had_linebreak_before_cur(&self) -> bool;

    /// Is the current token a specified keyword?
    fn is_keyword(&self, kw: &str) -> bool;
}

/// Abstracted token for `TokenSource`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Token {
    /// What kind of token it is
    pub kind: SyntaxKind,
    /// Is the current token joined to the next one? (`> >` vs `>>`).
    pub is_jointed_to_next: bool,
    /// The range (in byte indices) of the token
    pub range: Range<usize>,
}

/// An abstraction for syntax tree implementations
pub trait TreeSink {
    /// Adds new token to the current branch.
    fn token(&mut self, kind: SyntaxKind, n_tokens: u8);

    /// Start new branch and make it current.
    fn start_node(&mut self, kind: SyntaxKind);

    /// Finish current branch and restore previous
    /// branch as current.
    fn finish_node(&mut self);

    /// Emit an error
    fn error(&mut self, error: ParserError);
}