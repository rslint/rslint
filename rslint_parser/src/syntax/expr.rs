//! Expressions, these include `this`, identifiers, arrays, objects,
//! binary expressions, unary expressions, and more.
//!
//! See the [ECMAScript spec](https://www.ecma-international.org/ecma-262/5.1/#sec-11).

use super::util::*;
use crate::{SyntaxKind::*, *};

pub const LITERAL: TokenSet = token_set![TRUE_KW, FALSE_KW, NUMBER, STRING, NULL_KW,];

// TODO: We might want to add semicolon to this
pub const EXPR_RECOVERY_SET: TokenSet = token_set![VAR_KW];

pub const ASSIGN_TOKENS: TokenSet = token_set![
    T![=],
    T![+=],
    T![-=],
    T![*=],
    T![%=],
    T![<<=],
    T![>>=],
    T![>>>=],
    T![&=],
    T![|=],
    T![^=]
];

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

/// An assignment expression such as `foo += bar` or `foo = 5`.
pub fn assign_expr(p: &mut Parser) -> Option<CompletedMarker> {
    let target = conditional_expr(p)?;
    match target.kind() {
        COND_EXPR | BIN_EXPR | UNARY_EXPR | POSTFIX_EXPR => return Some(target),
        _ => {}
    }
    assign_expr_recursive(p, target)
}

fn assign_expr_recursive(p: &mut Parser, target: CompletedMarker) -> Option<CompletedMarker> {
    if p.at_ts(ASSIGN_TOKENS) {
        check_assign_target(p, &p.parse_marker(&target));
        let m = target.precede(p);
        p.bump_any();
        assign_expr(p);
        Some(m.complete(p, ASSIGN_EXPR))
    } else {
        Some(target)
    }
}

/// A conditional expression such as `foo ? bar : baz`
pub fn conditional_expr(p: &mut Parser) -> Option<CompletedMarker> {
    let lhs = binary_expr(p);

    if p.at(T![?]) {
        let m = lhs?.precede(p);
        p.bump_any();
        binary_expr(p);
        p.expect(T![:]);
        binary_expr(p);
        return Some(m.complete(p, COND_EXPR));
    }
    lhs
}

/// A binary expression such as `2 + 2` or `foo * bar + 2`
pub fn binary_expr(p: &mut Parser) -> Option<CompletedMarker> {
    let left = unary_expr(p)?;
    binary_expr_recursive(p, left, 0)
}

fn binary_expr_recursive(
    p: &mut Parser,
    left: CompletedMarker,
    min_prec: u8,
) -> Option<CompletedMarker> {
    let precedence = match p.cur() {
        T![in] if p.state.include_in => 7,
        T![instanceof] => 7,
        _ => {
            if let Some(prec) = current_precedence(p) {
                prec
            } else {
                return Some(left);
            }
        }
    };

    if precedence <= min_prec {
        return Some(left);
    }

    let m = left.precede(p);
    p.bump_any();
    let left_of_right = unary_expr(p)?;
    binary_expr_recursive(p, left_of_right, precedence)?;
    let complete = m.complete(p, BIN_EXPR);

    binary_expr_recursive(p, complete, min_prec)
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

/// A member or new expression with subscripts. e.g. `new foo`, `new Foo()`, `foo`, or `foo().bar[5]`
pub fn member_or_new_expr(p: &mut Parser, new_expr: bool) -> Option<CompletedMarker> {
    if p.at(T![new]) {
        // We must start the marker here and not outside or else we make
        // a needless node if the node ends up just being a primary expr
        let m = p.start();
        p.bump_any();

        member_or_new_expr(p, new_expr)?;

        if !new_expr || p.at(T!['(']) {
            args(p);
            let complete = m.complete(p, NEW_EXPR);
            return Some(subscripts(p, complete, true));
        }
        return Some(m.complete(p, NEW_EXPR));
    }

    let lhs = primary_expr(p)?;
    Some(subscripts(p, lhs, true))
}

/// Dot, Array, or Call expr subscripts.
pub fn subscripts(p: &mut Parser, lhs: CompletedMarker, no_call: bool) -> CompletedMarker {
    let mut lhs = lhs;
    while !p.at(EOF) {
        match p.cur() {
            T!['('] if !no_call => {
                lhs = {
                    let m = lhs.precede(p);
                    args(p);
                    m.complete(p, CALL_EXPR)
                }
            }
            T!['['] => lhs = bracket_expr(p, lhs),
            T![.] => lhs = dot_expr(p, lhs),
            _ => return lhs,
        }
    }
    lhs
}

/// A dot expression for accessing a property
pub fn dot_expr(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    let m = lhs.precede(p);
    p.expect(T![.]);
    identifier_name(p);
    let comp = m.complete(p, DOT_EXPR);
    comp
}

/// An array expression for property access or indexing, such as `foo[0]` or `foo["bar"]`
pub fn bracket_expr(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    let m = lhs.precede(p);
    p.expect(T!['[']);
    expr(p);
    p.expect(T![']']);
    m.complete(p, BRACKET_EXPR)
}

/// An identifier name, either an ident or a keyword
pub fn identifier_name(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    match p.cur() {
        t if t.is_keyword() || t == T![ident] => p.bump_any(),
        _ => {
            let err = p
                .err_builder("Expected an identifier or keyword")
                .primary(p.cur_tok().range, "Expected an identifier or keyword here");
            p.error(err);
        }
    }
    m.complete(p, NAME)
}

/// Arguments to a function.
///
/// `"(" (AssignExpr ",")* ")"`
pub fn args(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T!['(']);
    let mut first = false;

    while !p.at(EOF) && !p.eat(T![')']) {
        if first {
            first = false;
        } else {
            p.expect(T![,]);
        }
        primary_expr(p);
    }

    m.complete(p, ARG_LIST)
}

/// An general expression.
pub fn expr(p: &mut Parser) -> Option<CompletedMarker> {
    let first = assign_expr(p)?;

    if p.at(T![,]) {
        let m = first.precede(p);
        p.bump_any();
        assign_expr(p)?;

        while p.at(T![,]) {
            p.bump_any();
            binary_expr(p)?;
        }

        return Some(m.complete(p, SEQUENCE_EXPR));
    }

    Some(first)
}

/// A primary expression such as a literal, an object, an array, or `this`.
///
/// `ThisExpr | Name | Literal | ParenExpr | ObjectExpr | ArrayExpr`
pub fn primary_expr(p: &mut Parser) -> Option<CompletedMarker> {
    if let Some(m) = literal(p) {
        return Some(m);
    }

    let complete = match p.cur() {
        T![this] => {
            let m = p.start();
            p.bump_any();
            m.complete(p, THIS_EXPR)
        }
        T![ident] => {
            let m = p.start();
            p.bump_any();
            m.complete(p, NAME)
        }
        T!['('] => paren_expr(p),
        T!['['] => array_expr(p),
        T!['{'] => object_expr(p),
        _ => {
            let err = p
                .err_builder("Expected an expression, but found none")
                .primary(p.cur_tok().range, "Expected an expression here");
            p.err_recover(err, EXPR_RECOVERY_SET);
            return None;
        }
    };

    Some(complete)
}

/// An array literal such as `[foo, bar, baz]`.
pub fn array_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T!['[']);

    while !p.at(EOF) && !p.at(T![']']) {
        if p.eat(T![,]) {
            continue;
        }
        assign_expr(p);
        if !p.at(T![']']) {
            p.expect(T![,]);
        }
    }

    p.expect(T![']']);
    m.complete(p, ARRAY_EXPR)
}

/// An object literal such as `{ a: b, "b": 5 + 5 }`.
pub fn object_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T!['{']);
    let mut first = true;

    while !p.at(EOF) && !p.at(T!['}']) {
        if first {
            first = false;
        } else {
            p.expect(T![,]);
            if p.at(T!['}']) {
                break;
            }
        }
        object_property(p);
    }

    p.expect(T!['}']);
    m.complete(p, OBJECT_EXPR)
}

/// An individual object property such as `"a": b` or `5: 6 + 6`.
// TODO: getters and setters
pub fn object_property(p: &mut Parser) -> CompletedMarker {
    const OBJECT_KEY_TOKENS: TokenSet = token_set![NUMBER, STRING, IDENT];
    let m = p.start();

    if !p.at_ts(OBJECT_KEY_TOKENS) {
        let err = p.err_builder("Expected a string, number, or identifier for an object key, but found none")
            .primary(p.cur_tok().range, "This is not a valid object key");

        p.error(err);
    }
    p.bump_any();
    p.expect(T![:]);
    assign_expr(p);
    m.complete(p, LITERAL_PROP)
}

/// A left hand side expression, either a member expression or a call expression such as `foo()`.
pub fn lhs_expr(p: &mut Parser) -> Option<CompletedMarker> {
    let lhs = member_or_new_expr(p, true)?;
    Some(if p.at(T!['(']) {
        subscripts(p, lhs, false)
    } else {
        lhs
    })
}

/// A postifx expression, either `LHSExpr [no linebreak] ++` or `LHSExpr [no linebreak] --`.
pub fn postfix_expr(p: &mut Parser) -> Option<CompletedMarker> {
    let lhs = lhs_expr(p);
    if !p.has_linebreak_before_n(0) {
        match p.cur() {
            T![++] => {
                check_assign_target(p, &p.parse_marker(&lhs?));
                let m = lhs?.precede(p);
                p.bump(T![++]);
                let complete = m.complete(p, POSTFIX_EXPR);
                Some(complete)
            }
            T![--] => {
                check_assign_target(p, &p.parse_marker(&lhs?));
                let m = lhs?.precede(p);
                p.bump(T![--]);
                let complete = m.complete(p, POSTFIX_EXPR);
                Some(complete)
            }
            _ => lhs,
        }
    } else {
        lhs
    }
}

/// A unary expression such as `!foo` or `++bar`
pub fn unary_expr(p: &mut Parser) -> Option<CompletedMarker> {
    const UNARY_SINGLE: TokenSet =
        token_set![T![delete], T![void], T![typeof], T![+], T![-], T![~], T![!]];

    if p.at(T![++]) {
        let m = p.start();
        p.bump(T![++]);
        let right = unary_expr(p)?;
        let complete = m.complete(p, UNARY_EXPR);
        check_assign_target(p, &p.parse_marker(&right));
        return Some(complete);
    }
    if p.at(T![--]) {
        let m = p.start();
        p.bump(T![--]);
        let right = unary_expr(p)?;
        let complete = m.complete(p, UNARY_EXPR);
        check_assign_target(p, &p.parse_marker(&right));
        return Some(complete);
    }

    if p.at_ts(UNARY_SINGLE) {
        let m = p.start();
        p.bump_any();
        unary_expr(p)?;
        return Some(m.complete(p, UNARY_EXPR));
    }

    postfix_expr(p)
}
