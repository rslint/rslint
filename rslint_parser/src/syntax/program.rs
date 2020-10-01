//! Top level functions for parsing a script or module, also includes module specific items.

use super::decl::{class_decl, function_decl};
use super::expr::assign_expr;
use super::pat::binding_identifier;
use super::stmt::{block_items, semi, var_decl, STMT_RECOVERY_SET};
use crate::{SyntaxKind::*, *};

/// Parse an ECMAScript script.
///
/// # Panics
/// Panics if the parser is configured to parse a module.
pub fn script(p: &mut Parser) -> CompletedMarker {
    assert!(
        !p.state.is_module,
        "Using the script parsing function for modules is erroneous"
    );
    let m = p.start();
    block_items(p, true, true, None);
    m.complete(p, SyntaxKind::SCRIPT)
}

/// Parse an ECMAScript module.
///
/// # Panics
/// Panics if the parser is configured to parse a script.
pub fn module(p: &mut Parser) -> CompletedMarker {
    assert!(
        p.state.is_module,
        "Using the module parsing function for scripts is erroneous"
    );
    let m = p.start();
    block_items(p, true, true, None);
    m.complete(p, SyntaxKind::MODULE)
}

/// A module import declaration such as `import * from "a"`
/// This will not automatically issue an error if the parser isnt configured to parse a module
pub fn import_decl(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T![import]);

    match p.cur() {
        STRING => p.bump_any(),
        T![*] => {
            let inner = p.start();
            p.bump_any();
            if p.cur_src() == "as" {
                p.bump_any();
                binding_identifier(p);
            }
            inner.complete(p, WILDCARD_IMPORT);
            from_clause(p);
        }
        T!['{'] => {
            named_imports(p);
            from_clause(p);
        }
        T![ident] | T![await] | T![yield] => {
            binding_identifier(p);
            if p.eat(T![,]) && p.at_ts(token_set![T![*], T!['{']]) {
                match p.cur() {
                    T![*] => wildcard(p, None).complete(p, WILDCARD_IMPORT),
                    _ => named_list(p, None).complete(p, NAMED_IMPORTS),
                };
            }
            from_clause(p);
        }
        _ => {
            let err = p
                .err_builder("Expected an import clause, but found none")
                .primary(p.cur_tok(), "");

            p.err_recover(err, STMT_RECOVERY_SET);
        }
    }

    p.expect(T![;]);
    m.complete(p, IMPORT_DECL)
}

pub(crate) fn named_imports(p: &mut Parser) -> CompletedMarker {
    named_list(p, None).complete(p, NAMED_IMPORTS)
}

fn wildcard(p: &mut Parser, m: impl Into<Option<Marker>>) -> Marker {
    let m = m.into().unwrap_or_else(|| p.start());
    p.bump_any();
    if p.cur_src() == "as" {
        p.bump_any();
        binding_identifier(p);
    }
    m
}

fn named_list(p: &mut Parser, m: impl Into<Option<Marker>>) -> Marker {
    let m = m.into().unwrap_or_else(|| p.start());
    p.expect(T!['{']);
    let mut first = true;
    while !p.at(EOF) && !p.at(T!['}']) {
        if first {
            first = false;
        } else if p.at(T![,]) && p.nth_at(1, T!['}']) {
            p.bump_any();
            break;
        } else {
            p.expect(T![,]);
        }

        specifier(p);
    }
    p.expect(T!['}']);
    m
}

fn specifier(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    binding_identifier(p);
    if p.cur_src() == "as" {
        p.bump_any();
        binding_identifier(p);
    }
    m.complete(p, SPECIFIER)
}

fn from_clause(p: &mut Parser) {
    if p.cur_src() != "from" {
        let err = p
            .err_builder("Expected a `from` clause, but found none")
            .primary(p.cur_tok(), "");

        p.err_recover(err, STMT_RECOVERY_SET);
    } else {
        p.bump_any();
    }

    p.expect(STRING);
}

pub fn export_decl(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    let start = p.cur_tok().range.start;
    p.expect(T![export]);

    if p.eat(T![default]) {
        let complete = match p.cur() {
            T![function] => {
                let inner = p.start();
                function_decl(p, inner);
                m.complete(p, EXPORT_DEFAULT_DECL)
            }
            T![class] => {
                class_decl(p, false);
                m.complete(p, EXPORT_DEFAULT_DECL)
            }
            _ => {
                if p.cur_src() == "async"
                    && p.nth_at(1, T![function])
                    && !p.has_linebreak_before_n(1)
                {
                    let inner = p.start();
                    p.bump_any();
                    function_decl(
                        &mut *p.with_state(ParserState {
                            in_async: true,
                            ..p.state.clone()
                        }),
                        inner,
                    );
                    m.complete(p, EXPORT_DEFAULT_DECL)
                } else {
                    let range = assign_expr(p).map(|it| it.range(p));
                    semi(
                        p,
                        range
                            .map(|it| it.into())
                            .unwrap_or_else(|| p.cur_tok().range),
                    );
                    m.complete(p, EXPORT_DEFAULT_EXPR)
                }
            }
        };
        let mut state = p.state.clone();
        let complete = state.check_default(p, complete);
        p.state = state;
        complete
    } else {
        match p.cur() {
            T![const] | T![var] => {
                var_decl(p, false);
                m.complete(p, EXPORT_DECL)
            }
            T![class] => {
                class_decl(p, false);
                m.complete(p, EXPORT_DECL)
            }
            T![function] => {
                let inner = p.start();
                function_decl(p, inner);
                m.complete(p, EXPORT_DECL)
            }
            T!['{'] => {
                let start_marker = p.start();
                let inner = named_list(p, start_marker);
                if p.cur_src() == "from" {
                    from_clause(p);
                }
                inner.complete(p, EXPORT_NAMED)
            }
            T![*] => {
                let start_marker = p.start();
                let inner = wildcard(p, start_marker);
                from_clause(p);
                semi(p, start..p.cur_tok().range.start);
                inner.complete(p, EXPORT_WILDCARD)
            }
            _ => {
                if p.cur_src() == "let" {
                    var_decl(p, false);
                    m.complete(p, EXPORT_DECL)
                } else if p.cur_src() == "async"
                    && p.nth_at(1, T![function])
                    && !p.has_linebreak_before_n(1)
                {
                    let inner = p.start();
                    p.bump_any();
                    function_decl(
                        &mut *p.with_state(ParserState {
                            in_async: true,
                            ..p.state.clone()
                        }),
                        inner,
                    );
                    m.complete(p, EXPORT_DECL)
                } else {
                    let err = p
                        .err_builder("Expected an item to export, but found none")
                        .primary(p.cur_tok(), "");

                    p.error(err);
                    m.complete(p, ERROR)
                }
            }
        }
    }
}
