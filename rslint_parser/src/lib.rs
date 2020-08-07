//! Extremely fast, lossless, and error tolerant JavaScript Parser.
//!
//! The parser uses an abstraction over non-whitespace tokens.
//! This allows us to losslessly or lossly parse code without requiring explicit handling of whitespace.
//! The parser yields events, not an AST, the events are resolved into untyped syntax nodes, which can then
//! be casted into a typed AST.
//!
//! The parser is able to produce a valid AST from **any** source code.
//! Erroneous productions are wrapped into `ERROR` syntax nodes, the original source code
//! is completely represented in the final syntax nodes.
//!
//! You probably do not want to use the parser struct, unless you want to parse fragments of Js source code or make your own productions.
//! Instead use functions such as [`parse_text`] and [`parse_text_lossy`] which offer abstracted versions for parsing.
//!
//! Notable features of the parser are:
//! - Extremely fast parsing and lexing through the extremely fast [`rslint_lexer`].
//! - Ability to do Lossy or Lossless parsing on demand without explicit whitespace handling.
//! - Customizable, able to parse any fragments of JS code at your discretion.
//! - Completely error tolerant, able to produce an AST from any source code.
//! - Zero cost for converting untyped nodes to a typed AST.
//! - Ability to go from AST to SyntaxNodes to SyntaxTokens to source code and back very easily with nearly zero cost.
//! - Very easy tree traversal through [`SyntaxNode`](rowan::SyntaxNode).
//! - Descriptive errors with multiple labels and notes.
//! - Very cheap cloning, cloning an ast node or syntax node is the cost of adding a reference to an Rc
//!
//! This is inspired by the rust analyzer parser but adapted for JavaScript.

mod event;
mod parser;
#[macro_use]
mod token_set;
mod diagnostics;
mod parse;
mod syntax_node;
mod lossless_tree_sink;
mod lossy_tree_sink;
mod token_source;
mod state;

pub mod ast;
pub mod syntax;
pub mod util;

pub use crate::{
    ast::{AstNode, AstToken},
    diagnostics::ErrorBuilder,
    event::{process, Event},
    parse::*,
    parser::{CompletedMarker, Marker, Parser},
    syntax_node::*,
    lossless_tree_sink::LosslessTreeSink,
    lossy_tree_sink::LossyTreeSink,
    token_set::TokenSet,
    token_source::TokenSource,
    util::SyntaxNodeExt,
    state::ParserState
};

pub use rowan::{SmolStr, SyntaxText, TextRange, TextSize, WalkEvent};

pub use rslint_syntax::*;

/// The type of error emitted by the parser, this includes warnings, notes, and errors.  
/// It also includes labels and possibly notes
pub type ParserError = codespan_reporting::diagnostic::Diagnostic<usize>;

use std::ops::Range;

/// Abstracted token for `TokenSource`
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Token {
    /// What kind of token it is
    pub kind: SyntaxKind,
    /// The range (in byte indices) of the token
    pub range: Range<usize>,
    /// How long the token is
    pub len: TextSize,
}

/// An abstraction for syntax tree implementations
pub trait TreeSink {
    /// Adds new token to the current branch.
    fn token(&mut self, kind: SyntaxKind);

    /// Start new branch and make it current.
    fn start_node(&mut self, kind: SyntaxKind);

    /// Finish current branch and restore previous
    /// branch as current.
    fn finish_node(&mut self);

    /// Emit an error
    fn error(&mut self, error: ParserError);
}
