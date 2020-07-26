//! A crate encompassing the JavaScript syntax definitions. 
//! The crate "glues" together rowan and syntax definitions with the rslint-parse Parser. 
//! No other crate knows about rowan, it all goes through this crate. 
//! It contains multiple functions for parsing a syntax tree from tokens or raw source code.
//!  
//! It is heavily derived from the Rust analyzer ra_syntax crate, but adapted for JavaScript parsing.

mod syntax_kind;
mod syntax_node;

pub mod ast;

pub use crate::{
    ast::{AstNode, AstToken},
    syntax_node::{
        Direction, GreenNode, NodeOrToken, SyntaxElement, SyntaxElementChildren, SyntaxNode,
        SyntaxNodeChildren, SyntaxToken, SyntaxTreeBuilder,
    },
    syntax_kind::SyntaxKind,
};

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