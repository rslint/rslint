//! Extensions to TypeScript AST elements

use crate::{ast::*, syntax_node::SyntaxNode, SyntaxKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TsTypeQueryExpr {
    TsEntityName(TsEntityName),
    /* TsImport */
}

impl AstNode for TsTypeQueryExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        TsEntityName::can_cast(kind) || todo!("TsImport")
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        match syntax.kind() {
            n if TsEntityName::can_cast(n) => Some(TsTypeQueryExpr::TsEntityName(
                TsEntityName::cast(syntax).unwrap(),
            )),
            _ => todo!("TsImport"),
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            TsTypeQueryExpr::TsEntityName(it) => it.syntax(),
            _ => todo!("TsImport"),
        }
    }
}
