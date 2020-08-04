//! This module defines the Concrete Syntax Tree used by RSLint. 
//! 
//! The tree is entirely lossless, whitespace, comments, and errors are preserved. 
//! It also provides traversal methods including parent, children, and siblings of nodes. 
//! 
//! This is a simple wrapper around the `rowan` crate which does most of the heavy lifting and is language agnostic. 

use codespan_reporting::diagnostic::Diagnostic;
use rowan::{GreenNodeBuilder, Language};
use crate::{Parse, SmolStr, SyntaxKind};

pub use rowan::GreenNode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsLanguage;

impl Language for JsLanguage {
    type Kind = SyntaxKind;

    fn kind_from_raw(raw: rowan::SyntaxKind) -> SyntaxKind {
        SyntaxKind::from(raw.0)
    }

    fn kind_to_raw(kind: SyntaxKind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind.into())
    }
}

pub type SyntaxNode = rowan::SyntaxNode<JsLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<JsLanguage>;
pub type SyntaxElement = rowan::SyntaxElement<JsLanguage>;
pub type SyntaxNodeChildren = rowan::SyntaxNodeChildren<JsLanguage>;
pub type SyntaxElementChildren = rowan::SyntaxElementChildren<JsLanguage>;

pub use rowan::{Direction, NodeOrToken};

#[derive(Default)]
pub struct SyntaxTreeBuilder {
    errors: Vec<Diagnostic<usize>>,
    inner: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    pub(crate) fn finish_raw(self) -> (GreenNode, Vec<Diagnostic<usize>>) {
        let green = self.inner.finish();
        (green, self.errors)
    }

    pub fn finish(self) -> Parse<SyntaxNode> {
        let (green, errors) = self.finish_raw();
        Parse::new(green, errors)
    }

    pub fn token(&mut self, kind: SyntaxKind, text: SmolStr) {
        let kind = JsLanguage::kind_to_raw(kind);
        self.inner.token(kind, text)
    }

    pub fn start_node(&mut self, kind: SyntaxKind) {
        let kind = JsLanguage::kind_to_raw(kind);
        self.inner.start_node(kind)
    }

    pub fn finish_node(&mut self) {
        self.inner.finish_node()
    }

    pub fn error(&mut self, error: Diagnostic<usize>) {
        self.errors.push(error)
    }
}