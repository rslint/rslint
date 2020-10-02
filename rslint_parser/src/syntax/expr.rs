//! Expressions, these include `this`, identifiers, arrays, objects,
//! binary expressions, unary expressions, and more.
//!
//! See the [ECMAScript spec](https://www.ecma-international.org/ecma-262/5.1/#sec-11).

use super::decl::{arrow_body, class_decl, formal_parameters, function_decl, method};
use super::pat::pattern;
use super::util::*;
use crate::{
    ast::{BinExpr, BinOp, Expr, UnaryExpr},
    SyntaxKind::*,
    *,
};

pub const LITERAL: TokenSet = token_set![TRUE_KW, FALSE_KW, NUMBER, STRING, NULL_KW, REGEX];

// TODO: We might want to add semicolon to this
pub const EXPR_RECOVERY_SET: TokenSet =
    token_set![VAR_KW, SEMICOLON, R_PAREN, L_PAREN, L_BRACK, R_BRACK, SEMICOLON];

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
    T![^=],
    T![&&=],
    T![||=],
    T![??=]
];

pub const STARTS_EXPR: TokenSet = token_set![
    T![!],
    T!['('],
    T!['['],
    T!['{'],
    T![++],
    T![--],
    T![~],
    T![+],
    T![-],
    T![throw],
    T![new],
    T![typeof],
    T![void],
    T![delete],
    T![ident],
    T![...],
    T![this],
    T![yield],
    T![await],
    T![function],
    T![class],
    T![import],
    T![super],
    BACKTICK,
]
.union(LITERAL);

/// A literal expression.
///
/// `TRUE | FALSE | NUMBER | STRING | NULL`
// test literals
// 5
// true
// false
// 5n
// "foo"
// 'bar'
// null
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
    if p.state.in_generator && p.at(T![yield]) {
        return Some(yield_expr(p));
    }
    if p.state.in_async && p.at(T![await]) {
        let m = p.start();
        p.bump_any();
        unary_expr(p);
        return Some(m.complete(p, AWAIT_EXPR));
    }

    p.state.potential_arrow_start = matches!(p.cur(), T![ident] | T!['('] | T![yield] | T![await]);

    let token_cur = p.token_pos();
    let event_cur = p.cur_event_pos();
    let target = conditional_expr(p)?;
    assign_expr_recursive(p, target, token_cur, event_cur)
}

// test assign_expr
// foo += bar = b ??= 3;
// foo -= bar;
// [foo, bar] = baz;
// ({ bar, baz } = {});
// ({ bar: [baz], foo } = {});
fn assign_expr_recursive(
    p: &mut Parser,
    mut target: CompletedMarker,
    token_cur: usize,
    event_cur: usize,
) -> Option<CompletedMarker> {
    if p.at_ts(ASSIGN_TOKENS) {
        if p.at(T![=]) {
            if ![DOT_EXPR, BRACKET_EXPR, NAME_REF].contains(&target.kind()) {
                p.rewind(token_cur);
                p.drain_events(p.cur_event_pos() - event_cur);
                target = pattern(p)?;
            }
        } else {
            check_simple_assign_target(p, &p.parse_marker(&target), target.range(p));
        }
        let m = target.precede(p);
        p.bump_any();
        assign_expr(p);
        Some(m.complete(p, ASSIGN_EXPR))
    } else {
        Some(target)
    }
}

// test yield_expr
// function *foo() {
//  yield foo;
//  yield* foo;
// }
pub fn yield_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T![yield]);

    if !p.has_linebreak_before_n(1) {
        p.eat(T![*]);
        assign_expr(p);
    }
    m.complete(p, YIELD_EXPR)
}

/// A conditional expression such as `foo ? bar : baz`
// test conditional_expr
// foo ? bar : baz
// foo ? bar : baz ? bar : baz
pub fn conditional_expr(p: &mut Parser) -> Option<CompletedMarker> {
    // test_err conditional_expr_err
    // foo ? bar baz
    // foo ? bar baz ? foo : bar
    let lhs = binary_expr(p);

    if p.at(T![?]) {
        let m = lhs?.precede(p);
        p.bump_any();
        assign_expr(p);
        p.expect(T![:]);
        assign_expr(p);
        return Some(m.complete(p, COND_EXPR));
    }
    lhs
}

/// A binary expression such as `2 + 2` or `foo * bar + 2`
pub fn binary_expr(p: &mut Parser) -> Option<CompletedMarker> {
    let left = unary_expr(p);
    binary_expr_recursive(p, left, 0)
}

// test binary_expressions
// 5 * 5
// 6 ** 6 ** 7
// 1 + 2 * 3
// (1 + 2) * 3
// 1 / 2
// 74 in foo
// foo instanceof Array
// foo ?? bar
// 1 + 1 + 1 + 1
// 5 + 6 - 1 * 2 / 1 ** 6

// test_err binary_expressions_err
// foo(foo +);
// foo + * 2;
// !foo * bar;
fn binary_expr_recursive(
    p: &mut Parser,
    left: Option<CompletedMarker>,
    min_prec: u8,
) -> Option<CompletedMarker> {
    let precedence = match p.cur() {
        T![in] if p.state.include_in => 7,
        T![instanceof] => 7,
        _ => {
            if let Some(prec) = get_precedence(p.cur()) {
                prec
            } else {
                return left;
            }
        }
    };

    if precedence <= min_prec {
        return left;
    }

    let op = p.cur();
    let op_tok = p.cur_tok();

    let err: Option<(ErrorBuilder, TextRange, TextRange)> = if let Some(UNARY_EXPR) =
        left.map(|x| x.kind())
    {
        let left_ref = left.as_ref().unwrap();

        if op == T![**] && !is_update_expr(p, left.as_ref().unwrap()) {
            let err = p.err_builder("Exponentiation cannot be applied to a unary expression because it is ambiguous")
                .secondary(left_ref.range(p), "Because this expression would first be evaluated...")
                .primary(p.cur_tok(), "...Then it would be used for the value to be exponentiated, which is most likely unwanted behavior");

            let parsed = p.parse_marker::<UnaryExpr>(left_ref);
            let start = left_ref.offset_range(p, parsed.expr().unwrap().syntax().text_range());
            let op = left_ref.offset_range(p, parsed.op_token().unwrap().text_range());
            Some((err, start, op))
        } else {
            None
        }
    } else {
        None
    };

    if op == T![??] && left.is_some() {
        let left_ref = left.as_ref().unwrap();
        if left_ref.kind() == BIN_EXPR {
            let parsed = p.parse_marker::<BinExpr>(left_ref);
            if let Some(BinOp::LogicalAnd) | Some(BinOp::LogicalOr) = parsed.op() {
                let err = p.err_builder("The nullish coalescing operator (??) cannot be mixed with logical operators (|| and &&)")
                    .secondary(left_ref.range(p), "Because this expression would first be evaluated...")
                    .primary(op_tok.range.to_owned(), "...Then it would be used for the left hand side value of this operator, which is most likely unwanted behavior")
                    .help(&format!("Note: if this is expected, indicate precedence by wrapping `{}` in parentheses", color(left_ref.text(p))));

                p.error(err);
            }
        }
    }

    let m = left.map(|m| m.precede(p)).unwrap_or_else(|| p.start());
    p.bump_any();

    // This is a hack to allow us to effectively recover from `foo + / bar`
    let right = if get_precedence(p.cur()).is_some() && !p.at_ts(token_set![T![-], T![+]]) {
        let err = p.err_builder(&format!("Expected an expression for the right hand side of a `{}`, but found an operator instead", p.token_src(&op_tok)))
            .secondary(op_tok.to_owned(), "This operator requires a right hand side value")
            .primary(p.cur_tok(), "But this operator was encountered instead");

        p.error(err);
        None
    } else {
        unary_expr(p)
    };

    binary_expr_recursive(
        p,
        right,
        // ** is right recursive
        if op == T![**] {
            precedence - 1
        } else {
            precedence
        },
    );

    let complete = m.complete(p, BIN_EXPR);
    let recursive_right = binary_expr_recursive(p, Some(complete), min_prec);

    if let Some(marker) = recursive_right {
        if let Some((mut err, start, op)) = err {
            let range = TextRange::new(start.start(), marker.range(p).end());
            let text = &format!("{}({})", p.source(op), p.source(range));

            err = err.help(&format!("Help: did you mean `{}`?", color(text)));
            p.error(err);
        }
    }

    // Still parsing this is *technically* wrong because no production matches this, but for the purposes of error recovery
    // we still parse it. The parser takes some liberties with things such as this to still provide meaningful errors and recover.
    // Even if parsing this is technically not ECMA compatible
    if let Some(BIN_EXPR) = recursive_right.map(|x| x.kind()) {
        let right_ref = recursive_right.as_ref().unwrap();
        let parsed = p.parse_marker::<BinExpr>(right_ref);
        if parsed.op() == Some(BinOp::NullishCoalescing)
            && matches!(parsed.rhs(), Some(Expr::BinExpr(bin)) if bin.op() == Some(BinOp::LogicalAnd) || bin.op() == Some(BinOp::LogicalOr))
        {
            let rhs_range = right_ref.offset_range(p, parsed.rhs().unwrap().syntax().text_range());

            let err = p.err_builder("The nullish coalescing operator (??) cannot be mixed with logical operators (|| and &&)")
                    .secondary(rhs_range, "Because this expression would first be evaluated...")
                    .primary(op_tok.range, "...Then it would be used for the right hand side value of this operator, which is most likely unwanted behavior")
                    .help(&format!("Note: if this is expected, indicate precedence by wrapping `{}` in parentheses", color(parsed.rhs().unwrap().syntax().text().to_string().trim())));

            p.error(err);
        }

        // || has the same precedence as ?? so catching `foo ?? bar || baz` is not the same as `foo ?? bar && baz`
        if parsed.op() == Some(BinOp::LogicalOr) && op == T![??] && parsed.lhs().is_some() {
            let err = p.err_builder("The nullish coalescing operator (??) cannot be mixed with logical operators (|| and &&)")
                .secondary(right_ref.offset_range(p, parsed.lhs().unwrap().syntax().text_range()), "Because this expression would first be evaluated...")
                .primary(right_ref.offset_range(p, parsed.op_token().unwrap().text_range()), "...Then it would be used for the left hand side value of this operator, which is most likely unwanted behavior")
                .help(&format!("Note: if this is expected, indicate precedence by wrapping `{}` in parentheses", color(parsed.lhs().unwrap().syntax().text().to_string().trim())));

            p.error(err);
        }
    }

    recursive_right
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
// test new_exprs
// new Foo()
// new foo;
// new.target
// new new new new Foo();
// new Foo(bar, baz, 6 + 6, foo[bar] + (foo) => {} * foo?.bar)
pub fn member_or_new_expr(p: &mut Parser, new_expr: bool) -> Option<CompletedMarker> {
    if p.at(T![new]) {
        // We must start the marker here and not outside or else we make
        // a needless node if the node ends up just being a primary expr
        let m = p.start();
        p.bump_any();

        // new.target
        if p.at(T![.]) && p.token_src(&p.nth_tok(1)) == "target" {
            p.bump_any();
            p.bump_any();
            let complete = m.complete(p, NEW_TARGET);
            return Some(subscripts(p, complete, true));
        }

        member_or_new_expr(p, new_expr)?;

        if !new_expr || p.at(T!['(']) {
            args(p);
            let complete = m.complete(p, NEW_EXPR);
            return Some(subscripts(p, complete, true));
        }
        return Some(m.complete(p, NEW_EXPR));
    }

    // super.foo and super[bar]
    // test super_property_access
    // super.foo
    // super[bar]
    // super[foo][bar]
    if p.at(T![super]) && token_set!(T![.], T!['[']).contains(p.nth(1)) {
        let m = p.start();
        p.bump_any();
        let lhs = match p.cur() {
            T![.] => {
                p.bump_any();
                identifier_name(p);
                m.complete(p, DOT_EXPR)
            }
            T!['['] => {
                p.bump_any();
                expr(p);
                p.expect(T![']']);
                m.complete(p, BRACKET_EXPR)
            }
            _ => unreachable!(),
        };
        return Some(subscripts(p, lhs, true));
    }

    let lhs = primary_expr(p)?;
    Some(subscripts(p, lhs, true))
}

/// Dot, Array, or Call expr subscripts. Including optional chaining.
// test subscripts
// foo`bar`
// foo(bar)(baz)(baz)[bar]
pub fn subscripts(p: &mut Parser, lhs: CompletedMarker, no_call: bool) -> CompletedMarker {
    // test_err subscripts_err
    // foo()?.baz[].
    // BAR`b
    let mut lhs = optional_chain(p, lhs);
    while !p.at(EOF) {
        match p.cur() {
            T!['('] if !no_call => {
                lhs = {
                    let m = lhs.precede(p);
                    args(p);
                    m.complete(p, CALL_EXPR)
                }
            }
            T!['['] => lhs = bracket_expr(p, lhs, false),
            T![.] => lhs = dot_expr(p, lhs, false),
            BACKTICK => lhs = template(p, Some(lhs)),
            _ => return lhs,
        }
    }
    lhs
}

// An optional chain such as `foo?.bar?.(baz)?.[foo]`
// test optional_chain
// foo?.bar?.(baz)?.[foo]
// foo.bar?.(f).baz
// foo[bar]?.baz
pub fn optional_chain(p: &mut Parser, lhs: CompletedMarker) -> CompletedMarker {
    let mut lhs = lhs;
    while !p.at(EOF) {
        match p.cur() {
            T![?.] => match p.nth(1) {
                T!['('] => {
                    lhs = {
                        let m = lhs.precede(p);
                        p.bump_any();
                        args(p);
                        m.complete(p, CALL_EXPR)
                    }
                }
                T!['['] => lhs = bracket_expr(p, lhs, true),
                BACKTICK => {
                    let m = p.start();
                    let range = p.cur_tok().range;
                    p.bump_any();
                    template(p, None);
                    m.complete(p, ERROR);

                    let err = p.err_builder("optional chains may not be followed by template literals")
                            .primary(range, "a bracket, identifier, or arguments was expected for this optional chain");

                    p.error(err);
                    return lhs;
                }
                _ => lhs = dot_expr(p, lhs, true),
            },
            _ => return lhs,
        }
    }

    lhs
}

/// A dot expression for accessing a property
// test dot_expr
// foo.bar
// foo.await
// foo.yield
// foo.for
// foo?.for
// foo?.bar
pub fn dot_expr(p: &mut Parser, lhs: CompletedMarker, optional_chain: bool) -> CompletedMarker {
    let m = lhs.precede(p);
    if optional_chain {
        p.expect(T![?.]);
    } else {
        p.expect(T![.]);
    }
    identifier_name(p);
    m.complete(p, DOT_EXPR)
}

/// An array expression for property access or indexing, such as `foo[0]` or `foo?.["bar"]`
// test bracket_expr
// foo[bar]
// foo[5 + 5]
// foo["bar"]
// foo[bar][baz]
// foo?.[bar]
pub fn bracket_expr(p: &mut Parser, lhs: CompletedMarker, optional_chain: bool) -> CompletedMarker {
    // test_err bracket_expr_err
    // foo[]
    // foo?.[]
    // foo[
    let m = lhs.precede(p);
    if optional_chain {
        p.expect(T![?.]);
    }
    p.expect(T!['[']);
    expr(p);
    p.expect(T![']']);
    m.complete(p, BRACKET_EXPR)
}

/// An identifier name, either an ident or a keyword
pub fn identifier_name(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    match p.cur() {
        t if t.is_keyword() || t == T![ident] => p.bump_remap(T![ident]),
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

    while !p.at(EOF) && !p.at(T![')']) {
        if p.at(T![...]) {
            spread_element(p);
        } else {
            assign_expr(p);
        }

        if p.at(T![,]) {
            p.bump_any();
        } else {
            break;
        }
    }

    p.expect(T![')']);
    m.complete(p, ARG_LIST)
}

// test paren_or_arrow_expr
// (foo);
// (foo) => {};
// (5 + 5);
// ({foo, bar, b: [f, ...baz]}) => {};

// test_err paren_or_arrow_expr_invalid_params
// (5 + 5) => {}
pub fn paren_or_arrow_expr(p: &mut Parser, can_be_arrow: bool) -> CompletedMarker {
    let m = p.start();
    let token_cur = p.token_pos();
    let event_cur = p.cur_event_pos();
    p.expect(T!['(']);
    let mut spread_range = None;

    let mut temp = p.with_state(ParserState {
        potential_arrow_start: true,
        ..p.state.clone()
    });
    if temp.at_ts(STARTS_EXPR) {
        expr(&mut *temp);
    }
    if temp.at(T![...]) {
        let spread_marker = temp.start();
        temp.bump_any();
        pattern(&mut *temp);
        spread_range = Some(spread_marker.complete(&mut *temp, SPREAD_ELEMENT));
    }
    drop(temp);

    p.expect(T![')']);

    // This is an arrow expr, so we rewind the parser and reparse as parameters
    // This is kind of inefficient but in the grand scheme of things it does not matter
    // since the parser is already crazy fast
    if p.at(T![=>]) && !p.has_linebreak_before_n(0) {
        if !can_be_arrow {
            let err = p
                .err_builder("Unexpected token `=>`")
                .primary(p.cur_tok(), "an arrow expression is not allowed here");

            p.error(err);
        } else {
            // Rewind the parser so we can reparse as formal parameters
            p.rewind(token_cur);
            p.drain_events(p.cur_event_pos() - event_cur);
            formal_parameters(p);

            p.bump_any();
            arrow_body(p);
            return m.complete(p, ARROW_EXPR);
        }
    }

    if let Some(m) = spread_range {
        let err = p
            .err_builder("Illegal spread element inside grouping expression")
            .primary(m.range(p), "");

        p.error(err);
    }

    m.complete(p, GROUPING_EXPR)
}

pub fn expr_or_spread(p: &mut Parser) -> Option<CompletedMarker> {
    if p.at(T![...]) {
        let m = p.start();
        p.bump_any();
        assign_expr(p);
        Some(m.complete(p, SPREAD_ELEMENT))
    } else {
        assign_expr(p)
    }
}

/// A general expression.
// test sequence_expr
// 1, 2, 3, 4, 5
pub fn expr(p: &mut Parser) -> Option<CompletedMarker> {
    let first = assign_expr(p)?;

    if p.at(T![,]) {
        let m = first.precede(p);
        p.bump_any();
        assign_expr(p)?;

        while p.at(T![,]) {
            p.bump_any();
            assign_expr(p)?;
        }

        return Some(m.complete(p, SEQUENCE_EXPR));
    }

    Some(first)
}

/// A primary expression such as a literal, an object, an array, or `this`.
pub fn primary_expr(p: &mut Parser) -> Option<CompletedMarker> {
    if let Some(m) = literal(p) {
        return Some(m);
    }

    let complete = match p.cur() {
        T![this] => {
            // test this_expr
            // this
            // this.foo
            let m = p.start();
            p.bump_any();
            m.complete(p, THIS_EXPR)
        }
        T![class] => {
            // test class_expr
            // let a = class {};
            // let a = class foo {
            //  constructor() {}
            // }
            // foo[class {}]
            let mut m = class_decl(p, true);
            m.change_kind(p, CLASS_EXPR);
            m
        }
        // test async_ident
        // let a = async;
        T![ident] if p.cur_src() == "async" => {
            // test async_function_expr
            // let a = async function() {};
            // let b = async function foo() {};
            if p.nth_at(1, T![function]) {
                let m = p.start();
                p.bump_any();
                let mut complete = function_decl(
                    &mut *p.with_state(ParserState {
                        in_async: true,
                        ..p.state.clone()
                    }),
                    m,
                );
                complete.change_kind(p, FN_EXPR);
                complete
            } else {
                // `async a => {}` and `async (a) => {}`
                if p.state.potential_arrow_start
                    && token_set![T![ident], T![yield], T!['(']].contains(p.nth(1))
                {
                    // test async_arrow_expr
                    // let a = async foo => {}
                    // let b = async (bar) => {}
                    // async (foo, bar, ...baz) => foo
                    // async (yield) => {}
                    let m = p.start();
                    p.bump_any();
                    if p.at(T!['(']) {
                        formal_parameters(p);
                    } else {
                        // test_err async_arrow_expr_await_parameter
                        // let a = async await => {}
                        p.bump_remap(T![ident]);
                    }
                    p.expect(T![=>]);
                    arrow_body(&mut *p.with_state(ParserState {
                        in_async: true,
                        ..p.state.clone()
                    }));
                    m.complete(p, ARROW_EXPR)
                } else {
                    identifier_reference(p)?
                }
            }
        }
        T![function] => {
            // test function_expr
            // let a = function() {}
            // let b = function foo() {}
            let m = p.start();
            let mut complete = function_decl(p, m);
            complete.change_kind(p, FN_EXPR);
            complete
        }
        T![ident] | T![yield] | T![await] => {
            // test identifier_reference
            // foo;
            // yield;
            // await;
            let ident = identifier_reference(p)?;
            if p.state.potential_arrow_start && p.at(T![=>]) && !p.has_linebreak_before_n(1) {
                // test arrow_expr_single_param
                // foo => {}
                // yield => {}
                // await => {}
                let m = ident.precede(p);
                p.bump_any();
                arrow_body(p);
                m.complete(p, ARROW_EXPR)
            } else {
                ident
            }
        }
        // test grouping_expr
        // ((foo))
        // (foo)
        T!['('] => paren_or_arrow_expr(p, p.state.potential_arrow_start),
        T!['['] => array_expr(p),
        T!['{'] => object_expr(p),
        T![import] => {
            let m = p.start();
            p.bump_any();

            // test import_meta
            // import.meta
            if p.eat(T![.]) {
                // test_err import_no_meta
                // import.foo
                // import.metaa
                if p.at(T![ident]) && p.token_src(&p.cur_tok()) == "meta" {
                    p.bump_any();
                    m.complete(p, IMPORT_META)
                } else if p.at(T![ident]) {
                    let err = p
                        .err_builder(&format!(
                            "Expected `meta` following an import keyword, but found `{}`",
                            p.token_src(&p.cur_tok())
                        ))
                        .primary(p.cur_tok(), "");

                    p.err_and_bump(err);
                    m.complete(p, ERROR)
                } else {
                    let err = p
                        .err_builder("Expected `meta` following an import keyword, but found none")
                        .primary(p.cur_tok(), "");

                    p.error(err);
                    m.complete(p, ERROR)
                }
            } else {
                // test_err import_call_no_arg
                // let a = import();
                // foo();

                // test import_call
                // import("foo")
                p.expect(T!['(']);
                assign_expr(p);
                p.expect(T![')']);
                m.complete(p, IMPORT_CALL)
            }
        }
        BACKTICK => template(p, None),
        ERROR_TOKEN => {
            let m = p.start();
            p.bump_any();
            m.complete(p, ERROR)
        }
        // test_err primary_expr_invalid_recovery
        // let a = \; foo();
        _ => {
            let err = p
                .err_builder("Expected an expression, but found none")
                .primary(p.cur_tok().range, "Expected an expression here");
            p.err_recover(err, p.state.expr_recovery_set);
            return None;
        }
    };

    Some(complete)
}

pub fn identifier_reference(p: &mut Parser) -> Option<CompletedMarker> {
    match p.cur() {
        T![ident] | T![yield] | T![await] => {
            let m = p.start();
            p.bump_remap(T![ident]);
            Some(m.complete(p, NAME_REF))
        }
        _ => {
            let err = p
                .err_builder("Expected an identifier, but found none")
                .primary(p.cur_tok(), "");

            p.err_recover(err, p.state.expr_recovery_set);
            None
        }
    }
}

/// A template literal such as "`abcd ${efg}`"
// test template_literal
// let a = `foo ${bar}`;
// let a = ``;
// let a = `${foo}`;
// let a = `foo`;
pub fn template(p: &mut Parser, tag: Option<CompletedMarker>) -> CompletedMarker {
    let m = tag.map(|m| m.precede(p)).unwrap_or_else(|| p.start());
    p.expect(BACKTICK);

    while !p.at(EOF) && !p.at(BACKTICK) {
        match p.cur() {
            TEMPLATE_CHUNK => p.bump_any(),
            DOLLARCURLY => {
                let e = p.start();
                p.bump_any();
                expr(p);
                p.expect(T!['}']);
                e.complete(p, TEMPLATE_ELEMENT);
            },
            t => unreachable!("Anything not template chunk or dollarcurly should have been eaten by the lexer, but {:?} was found", t),
        }
    }

    // test_err template_literal_unterminated
    // let a = `${foo} bar

    // The lexer already should throw an error for unterminated template literal
    p.eat(BACKTICK);
    m.complete(p, TEMPLATE)
}

/// An array literal such as `[foo, bar, ...baz]`.
// test array_expr
// [foo, bar];
// [foo];
// [,foo];
// [foo,];
// [,,,,,foo,,,,];
// [...a, ...b];
pub fn array_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T!['[']);

    while !p.at(EOF) {
        while p.eat(T![,]) {}

        if p.at(T![']']) {
            break;
        }

        if p.at(T![...]) {
            spread_element(p);
        } else {
            assign_expr(p);
        }

        if p.at(T![']']) {
            break;
        }

        p.expect(T![,]);
    }

    p.expect(T![']']);
    m.complete(p, ARRAY_EXPR)
}

/// A spread element consisting of three dots and an assignment expression such as `...foo`
pub fn spread_element(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T![...]);
    assign_expr(p);
    m.complete(p, SPREAD_ELEMENT)
}

/// An object literal such as `{ a: b, "b": 5 + 5 }`.
// test object_expr
// let a = {};
// let b = {foo,}
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

const STARTS_OBJ_PROP: TokenSet =
    token_set![STRING, NUMBER, T![ident], T![await], T![yield], T!['[']];

/// An individual object property such as `"a": b` or `5: 6 + 6`.
pub fn object_property(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();

    match p.cur() {
        // test object_expr_getter_setter
        // let a = {
        //  get foo() {
        //    return foo;
        //  }
        // }
        //
        // let b = {
        //  set foo(bar) {
        //     return 5;
        //  }
        // }
        T![ident] if (p.cur_src() == "get" || p.cur_src() == "set") && !p.nth_at(1, T![:]) => {
            method(p, None)
        }
        // test object_expr_async_method
        // let a = {
        //   async foo() {},
        //   async *foo() {}
        // }
        T![ident]
            if p.cur_src() == "async"
                && !p.has_linebreak_before_n(1)
                && (STARTS_OBJ_PROP.contains(p.nth(1)) || p.nth_at(1, T![*])) =>
        {
            method(p, None)
        }
        // test object_expr_spread_prop
        // let a = {...foo}
        T![...] => {
            p.bump_any();
            assign_expr(p);
            Some(m.complete(p, SPREAD_PROP))
        }
        T![*] => {
            // test object_expr_generator_method
            // let b = { *foo() {} }
            let m = p.start();
            method(p, m)
        }
        _ => {
            let prop = object_prop_name(p, false);
            // test object_expr_assign_prop
            // let b = { foo = 4, foo = bar }
            if let Some(NAME) = prop.map(|m| m.kind()) {
                if p.eat(T![=]) {
                    assign_expr(p);
                    return Some(m.complete(p, INITIALIZED_PROP));
                }
            }

            // test object_expr_method
            // let b = {
            //  foo() {},
            // }
            if p.at(T!['(']) {
                method(p, m)
            } else {
                // test_err object_expr_non_ident_literal_prop
                // let b = {5}
                if prop?.kind() != NAME || p.at(T![:]) {
                    p.expect(T![:]);
                    assign_expr(p);
                    Some(m.complete(p, LITERAL_PROP))
                } else {
                    // test object_expr_ident_prop
                    // let b = {foo}
                    Some(m.complete(p, IDENT_PROP))
                }
            }
        }
    }
}

// test object_prop_name
// let a = {"foo": foo, [6 + 6]: foo, bar: foo, 7: foo}
pub fn object_prop_name(p: &mut Parser, binding: bool) -> Option<CompletedMarker> {
    match p.cur() {
        STRING | NUMBER => literal(p),
        T!['['] => {
            let m = p.start();
            p.bump_any();
            assign_expr(p);
            p.expect(T![']']);
            Some(m.complete(p, COMPUTED_PROPERTY_NAME))
        }
        _ if binding => super::pat::binding_identifier(p),
        _ => Some(identifier_name(p)),
    }
}

/// A left hand side expression, either a member expression or a call expression such as `foo()`.
pub fn lhs_expr(p: &mut Parser) -> Option<CompletedMarker> {
    if p.at(T![super]) && p.nth_at(1, T!['(']) {
        let m = p.start();
        p.bump_any();
        args(p);
        let lhs = m.complete(p, SUPER_CALL);
        return Some(subscripts(p, lhs, false));
    }

    let lhs = member_or_new_expr(p, true)?;
    Some(if p.at(T!['(']) {
        subscripts(p, lhs, false)
    } else {
        lhs
    })
}

/// A postifx expression, either `LHSExpr [no linebreak] ++` or `LHSExpr [no linebreak] --`.
// test postfix_expr
// foo++
// foo--
pub fn postfix_expr(p: &mut Parser) -> Option<CompletedMarker> {
    let lhs = lhs_expr(p);
    if !p.has_linebreak_before_n(0) {
        match p.cur() {
            T![++] => {
                check_simple_assign_target(p, &p.parse_marker(&lhs?), lhs?.range(p));
                let m = lhs?.precede(p);
                p.bump(T![++]);
                let complete = m.complete(p, UNARY_EXPR);
                Some(complete)
            }
            T![--] => {
                check_simple_assign_target(p, &p.parse_marker(&lhs?), lhs?.range(p));
                let m = lhs?.precede(p);
                p.bump(T![--]);
                let complete = m.complete(p, UNARY_EXPR);
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
        check_simple_assign_target(p, &p.parse_marker(&right), right.range(p));
        return Some(complete);
    }
    if p.at(T![--]) {
        let m = p.start();
        p.bump(T![--]);
        let right = unary_expr(p)?;
        let complete = m.complete(p, UNARY_EXPR);
        check_simple_assign_target(p, &p.parse_marker(&right), right.range(p));
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
