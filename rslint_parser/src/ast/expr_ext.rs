//! Extensions for things which are not easily generated in ast expr nodes

use crate::ast::*;
use SyntaxKind::*;

impl BracketExpr {
    pub fn object(&self) -> Option<Expr> {
        support::children(self.syntax()).next()
    }

    pub fn prop(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }
}

impl CondExpr {
    pub fn test(&self) -> Option<Expr> {
        support::children(self.syntax()).next()
    }

    pub fn cons(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }

    pub fn alt(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(2)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralPropKey {
    /// Either a number or a string
    Literal(SyntaxNode),
    Ident(SyntaxNode),
}

impl AstNode for LiteralPropKey {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NAME | LITERAL => true,
            _ => false,
        }
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(syntax.kind()) {
            None
        } else {
            Some(match syntax.kind() {
                LITERAL => LiteralPropKey::Literal(syntax),
                NAME => LiteralPropKey::Ident(syntax),
                _ => unreachable!(),
            })
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            LiteralPropKey::Ident(s) | LiteralPropKey::Literal(s) => s,
        }
    }
}

impl LiteralPropKey {
    pub fn key(&self) -> Option<Expr> {
        support::child(self.syntax())
    }

    pub fn as_string(&self) -> Option<std::string::String> {
        Some(self.key()?.syntax().text().to_string())
    }
}

impl LiteralProp {
    pub fn key(&self) -> Option<LiteralPropKey> {
        if LiteralPropKey::can_cast(
            support::children::<Expr>(self.syntax())
                .next()?
                .syntax()
                .kind(),
        ) {
            LiteralPropKey::cast(
                support::children::<Expr>(self.syntax())
                    .next()
                    .unwrap()
                    .syntax()
                    .to_owned(),
            )
        } else {
            None
        }
    }

    pub fn value(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }
}
