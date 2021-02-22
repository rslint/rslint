//! This module defines the Concrete Syntax Tree used by RSLint.
//!
//! The tree is entirely lossless, whitespace, comments, and errors are preserved.
//! It also provides traversal methods including parent, children, and siblings of nodes.
//!
//! This is a simple wrapper around the `rowan` crate which does most of the heavy lifting and is language agnostic.

use crate::SyntaxKind;
use rslint_rowan::{GreenNodeBuilder, Interner, Language};

pub use rslint_rowan::{GreenNode, JsLanguage};

/// Simple wrapper around a rslint_rowan [`GreenNodeBuilder`]
#[derive(Default, Debug)]
pub struct SyntaxTreeBuilder {
    inner: GreenNodeBuilder<'static, 'static>,
}

impl SyntaxTreeBuilder {
    pub fn finish(self) -> (GreenNode, Interner) {
        let (node, interner) = self.inner.finish();
        (node, interner.unwrap())
    }

    pub fn token(&mut self, kind: SyntaxKind, text: &str) {
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
}
