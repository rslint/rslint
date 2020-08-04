//! Expressions, these include `this`, identifiers, arrays, objects,
//! binary expressions, unary expressions, and more. 
//! 
//! See the [] 

use crate::{*, SyntaxKind::*};

pub const LITERAL: TokenSet = token_set![
    TRUE_KW,
    FALSE_KW,
    NUMBER,
    STRING,
    NULL_KW,
];

// TODO: We might want to add semicolon to this
pub const EXPR_RECOVERY_SET: TokenSet = token_set![VAR_KW];

/// A literal expression. 
/// 
/// `TRUE | FALSE | NUMBER | STRING | NULL`
pub fn literal(p: &mut Parser) -> Option<CompletedMarker> {
    if !p.at_ts(LITERAL) {
        return None;
    }
    let m = p.start();
    p.bump_any();
    Some(m.complete(p, SyntaxKind::LITERAL))
}

/// A parenthesis expression, also called a grouping expression. 
/// 
/// `"(" Expr ")"`
pub fn paren_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T!['(']);
    expr(p);
    p.expect(T![')']);
    m.complete(p, GROUPING_EXPR)
}

/// An expression. 
pub fn expr(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    let first = primary_expr(p)?;

    if p.at(T![,]) {
        p.bump_any();
        primary_expr(p)?;

        while p.at(T![,]) {
            p.bump_any();
            primary_expr(p)?; 
        }

        return Some(m.complete(p, SEQUENCE_EXPR));
    }

    Some(first)
}

/// A primary expression.
/// 
/// `ThisExpr | Name | Literal | ParenExpr | ObjectExpr | ArrayExpr`
pub fn primary_expr(p: &mut Parser) -> Option<CompletedMarker> {
    if let Some(m) = literal(p) {
        return Some(m);
    }

    let m = p.start();
    let complete = match p.cur() {
        T![this] => {
            p.bump_any();
            m.complete(p, T![this])
        },
        T![ident] => {
            p.bump_any();
            m.complete(p, NAME)
        },
        T!['('] => paren_expr(p),
        _ => {
            let err = p.err_builder("Expected an expression, but found none").primary(p.cur_tok().range, "Expected an expression here");
            p.err_recover(err, EXPR_RECOVERY_SET);
            return None;
        }
    };

    Some(complete)
}