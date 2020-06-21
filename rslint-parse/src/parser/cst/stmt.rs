use super::expr::*;
use crate::span::Span;

#[derive(Clone, PartialEq, Debug)]
pub enum Stmt {
    Variable(VariableDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Semicolon {
    /// An automatically inserted semicolon
    Implicit,
    /// A semicolon explicitly typed out
    Explicit(LiteralWhitespace),
}

impl Semicolon {
    /// Get the span of an explicit semicolon or None if the semicolon is implicit
    pub fn span(&self) -> Option<Span> {
        if let Semicolon::Explicit(ref data) = *self {
            Some(Span::new(data.before.end, data.after.start))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declarator {
    pub name: LiteralExpr,
    pub value: Option<Expr>,
    /// The optional whitespace of the `=`
    pub initializer_whitespace: Option<LiteralWhitespace>,
}

impl Declarator {
    /// Get the end to a declarator, the span of the value if it has one, or the span of the name otherwise
    pub fn span(&self) -> Span {
        self.value
            .as_ref()
            .map(|expr| expr.span())
            .unwrap_or(&self.name.span)
            .to_owned()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDeclaration {
    pub span: Span,
    /// It can only be an identifier so we can just use literal expr for this
    /// We might want to reconsider this choice later on
    pub declared: Vec<Declarator>,
    pub comma_whitespaces: Vec<LiteralWhitespace>,
    /// The whitespace of the `var` keyword
    pub var_whitespace: LiteralWhitespace,
    pub semi: Semicolon,
}
