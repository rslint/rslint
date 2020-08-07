//! Extended AST node definitions for statements which are unique and special enough to generate code for manually

use crate::{
    ast::{AstNode, Declaration, Expr, Stmt, VarStmt},
    syntax_node::SyntaxNode,
    SyntaxKind,
};

/// Either a statement or a declaration such as a function
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StmtListItem {
    Stmt(Stmt),
    Declaration(Declaration),
}

impl AstNode for StmtListItem {
    fn can_cast(kind: SyntaxKind) -> bool {
        Stmt::can_cast(kind) || Declaration::can_cast(kind)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Stmt::can_cast(syntax.kind()) {
            Some(StmtListItem::Stmt(Stmt::cast(syntax)?))
        } else {
            Some(StmtListItem::Declaration(Declaration::cast(syntax)?))
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            StmtListItem::Stmt(stmt) => stmt.syntax(),
            StmtListItem::Declaration(decl) => decl.syntax(),
        }
    }
}

/// The beginning to a For or For..in statement which can either be a var stmt or an expression
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ForHead {
    VarStmt(VarStmt),
    Expr(Expr),
}

impl AstNode for ForHead {
    fn can_cast(kind: SyntaxKind) -> bool {
        VarStmt::can_cast(kind) || Expr::can_cast(kind)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if VarStmt::can_cast(syntax.kind()) {
            Some(ForHead::VarStmt(VarStmt::cast(syntax)?))
        } else {
            Some(ForHead::Expr(Expr::cast(syntax)?))
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            ForHead::VarStmt(stmt) => stmt.syntax(),
            ForHead::Expr(expr) => expr.syntax(),
        }
    }
}
