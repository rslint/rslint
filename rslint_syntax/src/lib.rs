//! A crate encompassing the JavaScript syntax definitions. 
//! The crate "glues" together rowan and syntax definitions with the rslint-parse Parser. 
//! No other crate knows about rowan, it all goes through this crate. 
//! It contains multiple functions for parsing a syntax tree from tokens or raw source code.
//!  
//! It is heavily derived from the Rust analyzer ra_syntax crate, but adapted for JavaScript parsing.

mod syntax_node;
mod token_source;
mod tree_sink;

pub mod ast;

pub use crate::{
    ast::{AstNode, AstToken},
    syntax_node::{
        Direction, GreenNode, NodeOrToken, SyntaxElement, SyntaxElementChildren, SyntaxNode,
        SyntaxNodeChildren, SyntaxToken, SyntaxTreeBuilder,
    },
    token_source::TextTokenSource,
    tree_sink::TextTreeSink,
};
pub use rslint_lexer::Token;
pub use rslint_parser::{T, SyntaxKind, ParserError};

use std::{marker::PhantomData, sync::Arc};
use codespan_reporting::diagnostic::Diagnostic;

pub use rowan::{SmolStr, SyntaxText, TextRange, TextSize, TokenAtOffset, WalkEvent};

#[derive(Debug)]
pub struct Parse<T> {
    green: GreenNode,
    errors: Arc<Vec<Diagnostic<usize>>>,
    _ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Parse<T> {
        Parse { green: self.green.clone(), errors: self.errors.clone(), _ty: PhantomData }
    }
}

impl<T> Parse<T> {
    fn new(green: GreenNode, errors: Vec<Diagnostic<usize>>) -> Parse<T> {
        Parse { green, errors: Arc::new(errors), _ty: PhantomData }
    }

    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

impl<T: AstNode> Parse<T> {
    pub fn to_syntax(self) -> Parse<SyntaxNode> {
        Parse { green: self.green, errors: self.errors, _ty: PhantomData }
    }

    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).unwrap()
    }

    pub fn errors(&self) -> &[Diagnostic<usize>] {
        &*self.errors
    }

    pub fn ok(self) -> Result<T, Arc<Vec<Diagnostic<usize>>>> {
        if self.errors.is_empty() {
            Ok(self.tree())
        } else {
            Err(self.errors)
        }
    }
}

/// Run the rslint_lexer lexer to turn source code into tokens and errors produced by the lexer
pub fn tokenize(text: &str, file_id: usize) -> (Vec<Token>, Vec<Diagnostic<usize>>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for (tok, error) in rslint_lexer::Lexer::from_str(text, file_id) {
        tokens.push(tok);
        if let Some(err) = error {
            errors.push(err)
        }
    }
    (tokens, errors)
}

/// Parse text into a GreenNode which can then be turned into a SyntaxNode through `SyntaxNode::new_root()`
pub fn parse_text(text: &str, file_id: usize) -> (GreenNode, Vec<ParserError>) {
    let (tokens, lexer_errors) = tokenize(&text, file_id);

    let mut tok_source = TextTokenSource::new(text, &tokens);
    let mut tree_sink = TextTreeSink::new(text, &tokens);

    let mut parser = rslint_parser::Parser::new(&mut tok_source, file_id);
    /* PLACEHOLDER */
    let m = parser.start();
    rslint_parser::syntax::expr::expr(&mut parser);
    rslint_parser::syntax::expr::expr(&mut parser);
    m.complete(&mut parser, SyntaxKind::PROGRAM);
    rslint_parser::process(&mut tree_sink, parser.finish());
    tree_sink.finish()
}
