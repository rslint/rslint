//! Automatic rule fixing utilities

mod apply;

use crate::{Span, SyntaxKind};
use rslint_lexer::{Lexer, Token};
use rslint_parser::{ast, AstNode, SyntaxNode, SyntaxNodeExt};
use rslint_text_edit::apply_indels;
use rslint_text_edit::Indel;
use std::borrow::Borrow;
use std::sync::Arc;

pub use apply::{recursively_apply_fixes, MAX_FIX_ITERATIONS};

/// A simple interface for applying changes to source code
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fixer {
    pub indels: Vec<Indel>,
    pub src: Arc<str>,
}

impl Fixer {
    pub fn new(src: Arc<str>) -> Fixer {
        Self {
            indels: vec![],
            src,
        }
    }

    /// Apply this fixer to its source code
    pub fn apply(&self) -> String {
        let mut new = (&*self.src).to_string();
        apply_indels(&self.indels, &mut new);
        new
    }

    /// Replace some area in the source code with a string
    pub fn replace(&mut self, from: impl Span, to: impl ToString) -> &mut Self {
        self.indels
            .push(Indel::replace(from.as_text_range(), to.to_string()));
        self
    }

    /// Replace some area in the source code with another area in the source code
    pub fn replace_with(&mut self, from: impl Span, to: impl Span) -> &mut Self {
        self.indels.push(Indel::replace(
            from.as_text_range(),
            self.src[to.as_range()].into(),
        ));
        self
    }

    pub fn insert(&mut self, offset: usize, text: impl ToString) -> &mut Self {
        self.indels
            .push(Indel::insert((offset as u32).into(), text.to_string()));
        self
    }

    pub fn wrap(&mut self, span: impl Span, wrapping: Wrapping) -> &mut Self {
        let range = span.as_range();
        self.indels.push(Indel::insert(
            (range.start as u32).into(),
            wrapping.start_char().to_string(),
        ));
        self.indels.push(Indel::insert(
            (range.end as u32).into(),
            wrapping.end_char().to_string(),
        ));
        self
    }

    pub fn wrap_with(
        &mut self,
        span: impl Span,
        left: impl ToString,
        right: impl ToString,
    ) -> &mut Self {
        let range = span.as_range();
        self.indels.push(Indel::insert(
            (range.start.saturating_sub(1) as u32).into(),
            left.to_string(),
        ));
        self.indels.push(Indel::insert(
            (range.end.saturating_add(1) as u32).into(),
            right.to_string(),
        ));
        self
    }

    pub fn delete(&mut self, span: impl Span) -> &mut Self {
        self.indels.push(Indel::delete(span.as_text_range()));
        self
    }

    pub fn cancel_if_has_comments(&mut self, node: impl Borrow<SyntaxNode>) -> &mut Self {
        if node.borrow().contains_comments() {
            self.indels.clear();
        }
        self
    }

    pub fn unwrap(&mut self, node: impl Unwrappable) -> &mut Self {
        self.indels.push(node.unwrap());
        self
    }

    pub fn insert_before(&mut self, span: impl Span, text: impl ToString) -> &mut Self {
        self.indels.push(Indel::insert(
            span.as_text_range().start(),
            text.to_string(),
        ));
        self
    }

    pub fn insert_after(&mut self, span: impl Span, text: impl ToString) -> &mut Self {
        self.indels
            .push(Indel::insert(span.as_text_range().end(), text.to_string()));
        self
    }

    pub fn eat_trailing_whitespace(&mut self, span: impl Span) -> &mut Self {
        let mut lexer = Lexer::from_str(&self.src[span.as_range().end..], 0);
        if let Some((
            Token {
                kind: SyntaxKind::WHITESPACE,
                len,
            },
            _,
        )) = lexer.next()
        {
            let range = span.as_range();
            self.delete(range.end..range.end + len)
        } else {
            self
        }
    }

    pub fn eat_leading_whitespace(&mut self, span: impl Span) -> &mut Self {
        let reversed = self.src[..span.as_range().start]
            .chars()
            .rev()
            .collect::<String>();

        let mut lexer = Lexer::from_str(&reversed, 0);
        if let Some((
            Token {
                kind: SyntaxKind::WHITESPACE,
                len,
            },
            _,
        )) = lexer.next()
        {
            let range = span.as_range();
            self.delete(range.start - len..range.start)
        } else {
            self
        }
    }

    /// Delete multiple spans of code
    pub fn delete_multiple(&mut self, spans: impl IntoIterator<Item = impl Span>) -> &mut Self {
        for span in spans {
            self.delete(span);
        }
        self
    }
}

/// The different kinds of chars something could be wrapped inside of
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Wrapping {
    Parens,
    Curlies,
    Brackets,
    SingleQuotes,
    DoubleQuotes,
    Other(char, char),
}

impl Wrapping {
    pub fn start_char(&self) -> char {
        use Wrapping::*;

        match self {
            Parens => '(',
            Curlies => '{',
            Brackets => '[',
            SingleQuotes => '\'',
            DoubleQuotes => '"',
            Other(l, _) => *l,
        }
    }

    pub fn end_char(&self) -> char {
        use Wrapping::*;

        match self {
            Parens => ')',
            Curlies => '}',
            Brackets => ']',
            SingleQuotes => '\'',
            DoubleQuotes => '"',
            Other(_, r) => *r,
        }
    }
}

/// A trait describing AST nodes which can be "unwrapped" such as grouping expressions
pub trait Unwrappable: AstNode {
    fn unwrap(&self) -> Indel;
}

impl Unwrappable for ast::GroupingExpr {
    fn unwrap(&self) -> Indel {
        Indel::replace(
            self.range(),
            self.inner().map(|e| e.text()).unwrap_or_default(),
        )
    }
}

impl Unwrappable for ast::Condition {
    fn unwrap(&self) -> Indel {
        Indel::replace(
            self.range(),
            self.condition().map(|e| e.text()).unwrap_or_default(),
        )
    }
}
