//! Class and function declarations.

use super::expr::{assign_expr, lhs_expr, object_prop_name};
use super::pat::{binding_identifier, opt_binding_identifier, pattern};
use super::stmt::block_stmt;
use super::typescript::*;
use crate::{SyntaxKind::*, *};
use std::collections::HashMap;

pub const BASE_METHOD_RECOVERY_SET: TokenSet = token_set![
    T!['['],
    T![ident],
    T![yield],
    T![await],
    T![;],
    T!['}'],
    NUMBER,
    STRING
];

/// A function declaration, this could be async and or a generator. This takes a marker
/// because you need to first advance over async or start a marker and feed it in.
// test function_decl
// function foo() {}
// function *foo() {}
// function foo(await) {}
// async function *foo() {}
// async function foo() {}
// function *foo() {
//   yield foo;
// }
pub fn function_decl(p: &mut Parser, m: Marker, fn_expr: bool) -> CompletedMarker {
    // test_err function_decl_err
    // function() {}
    // function {}
    // function *() {}
    // async function() {}
    // async function *() {}
    // function *foo() {}
    // yield foo;
    p.expect(T![function]);
    let in_generator = p.eat(T![*]);

    let complete = opt_binding_identifier(p);
    if complete.is_none() && !fn_expr {
        let err = p
            .err_builder(
                "expected a name for the function in a function declaration, but found none",
            )
            .primary(p.cur_tok().range, "");

        p.error(err);
    }
    formal_parameters(p);

    block_stmt(
        &mut *p.with_state(ParserState {
            labels: HashMap::new(),
            in_function: true,
            in_generator,
            ..p.state.clone()
        }),
        true,
        None,
    );
    m.complete(p, FN_DECL)
}

#[allow(clippy::unnecessary_unwrap)]
fn formal_param_pat(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    if p.typescript() {
        if let Some(modifier) = maybe_eat_incorrect_modifier(p) {
            let err = p
                .err_builder("modifiers on parameters are only allowed in constructors")
                .primary(modifier.range(p), "");

            p.error(err);
        }
    }

    let pat = pattern(p, true)?;
    let pat_range = pat.range(p);
    let mut kind = pat.kind();
    pat.undo_completion(p).abandon(p);

    let mut opt = None;

    if p.at(T![?]) {
        opt = Some(p.cur_tok().range);
        let range = p.cur_tok().range;
        match kind {
            SINGLE_PATTERN | ARRAY_PATTERN | OBJECT_PATTERN => {
                p.bump_any();
            }
            _ if p.state.in_declare => {
                let m = p.start();
                p.bump_any();
                m.complete(p, ERROR);
            }
            _ => {
                let m = p.start();
                p.bump_any();
                m.complete(p, ERROR);
                let err = p
                    .err_builder("Binding patterns cannot be optional")
                    .primary(pat_range, "");

                p.error(err);
            }
        }
        if !p.typescript() {
            let err = p
                .err_builder(
                    "optional parameter syntax with `?` can only be used in TypeScript files",
                )
                .primary(range, "");

            p.error(err);
        }
    }
    if p.at(T![:]) {
        let maybe_err = p.start();
        let start = p.cur_tok().range.start;
        p.bump_any();
        let compl = ts_type(p);
        let end = compl
            .map(|x| usize::from(x.range(p).end()))
            .unwrap_or_else(|| p.cur_tok().range.start);

        if !p.typescript() {
            let err = p
                .err_builder("patterns can only have type parameters in TypeScript files")
                .primary(start..end, "");

            p.error(err);
            maybe_err.complete(p, ERROR);
        } else {
            m.abandon(p);
        }
    }

    if p.at(T![=]) {
        let start = p.cur_tok().range.start;
        p.bump_any();

        let expr = assign_expr(p);
        let end = expr
            .map(|x| usize::from(x.range(p).end()))
            .unwrap_or_else(|| p.cur_tok().range.start);
        if let Some(range) = opt {
            let err = p
                .err_builder("optional parameters cannot have initializers")
                .primary(start..end, "")
                .secondary(range, "");

            p.error(err);
        }

        kind = ASSIGN_PATTERN;
    }

    Some(m.complete(p, kind))
}

fn access_modifier<'a>(p: &'a mut Parser) -> Option<&'a str> {
    if matches!(p.cur_src(), "public" | "private" | "protected") {
        Some(p.cur_src())
    } else {
        None
    }
}

fn constructor_param_pat(p: &mut Parser) -> Option<CompletedMarker> {
    todo!();
    // let m = p.start();
    // let readonly = if  {
    //     let readonly = p.cur_src() == "readonly";
    //     p.bump_any();
    //     readonly
    // } else {
    //     false
    // };
}

pub fn formal_parameters(p: &mut Parser) -> CompletedMarker {
    parameters_common(p, false)
}

pub fn constructor_params(p: &mut Parser) -> CompletedMarker {
    parameters_common(p, true)
}

fn parameters_common(p: &mut Parser, constructor_params: bool) -> CompletedMarker {
    let m = p.start();
    let mut first = true;

    p.state.allow_object_expr = p.expect(T!['(']);

    while !p.at(EOF) && !p.at(T![')']) {
        if first {
            first = false;
        } else if p.nth_at(1, T![')']) {
            p.eat(T![,]);
            break;
        } else {
            p.expect(T![,]);
        }

        if p.at(T![...]) {
            let m = p.start();
            p.bump_any();
            if let Some(pat) = pattern(p, false) {
                pat.undo_completion(p).abandon(p);
            }

            // rest patterns cannot be optional: `...foo?: number[]`
            if p.at(T![?]) {
                let err = p
                    .err_builder("rest patterns cannot be optional")
                    .primary(p.cur_tok().range, "");

                p.error(err);
                let m = p.start();
                p.bump_any();
                m.complete(p, ERROR);
            }

            // type annotation `...foo: number[]`
            if p.eat(T![:]) {
                let complete = ts_type(p);
                if let Some(mut res) = complete {
                    res.err_if_not_ts(p, "type annotations can only be used in TypeScript files");
                }
            }

            if p.at(T![=]) {
                let start = p.cur_tok().range.start;
                let m = p.start();
                p.bump_any();
                let expr = assign_expr(&mut *p);
                let end = expr
                    .map(|x| usize::from(x.range(p).end()))
                    .unwrap_or_else(|| p.cur_tok().range.start);
                let err = p
                    .err_builder("rest elements may not have default initializers")
                    .primary(start..end, "");

                p.error(err);
                m.complete(p, ERROR);
            }
            m.complete(p, REST_PATTERN);

            // FIXME: this should be handled better, we should keep trying to parse params but issue an error for each one
            // which would allow for better recovery from `foo, ...bar, foo`
            if p.at(T![,]) {
                let m = p.start();
                let range = p.cur_tok().range;
                p.bump_any();
                m.complete(p, ERROR);
                let err = p
                    .err_builder("rest elements may not have trailing commas")
                    .primary(range, "");

                p.error(err);
            }
            break;
        } else {
            let func = if constructor_params {
                constructor_param_pat
            } else {
                formal_param_pat
            };
            // test_err formal_params_no_binding_element
            // function foo(true) {}
            if func(p).is_none() {
                p.err_recover_no_err(
                    token_set![
                        T![ident],
                        T![await],
                        T![yield],
                        T![,],
                        T![=],
                        T!['['],
                        T![...],
                        T![')'],
                    ],
                    true,
                );
            }
        }
    }

    p.state.allow_object_expr = true;
    p.expect(T![')']);
    m.complete(p, PARAMETER_LIST)
}

pub fn arrow_body(p: &mut Parser) -> Option<CompletedMarker> {
    let mut guard = p.with_state(ParserState {
        in_function: true,
        ..p.state.clone()
    });
    if guard.at(T!['{']) {
        block_stmt(&mut *guard, true, None)
    } else {
        assign_expr(&mut *guard)
    }
}

// test class_decl
// class foo {}
// class foo extends bar {}
// class foo extends foo.bar {}
pub fn class_decl(p: &mut Parser, expr: bool) -> CompletedMarker {
    // test_err class_decl_err
    // class {}
    // class extends bar {}
    // class extends {}
    // class
    // class foo { set {} }
    let m = p.start();
    p.expect(T![class]);
    // class bodies are implicitly strict
    let mut guard = p.with_state(ParserState {
        strict: Some(StrictMode::Class(p.cur_tok().range)),
        ..p.state.clone()
    });

    if !guard.at(T!['{']) && !guard.at(T![extends]) {
        binding_identifier(&mut *guard);
    } else if !expr {
        let err = guard
            .err_builder("class declarations must have a name")
            .primary(guard.cur_tok().range, "");

        guard.error(err);
    }

    if guard.eat(T![extends]) {
        lhs_expr(&mut *guard);
    }

    class_body(&mut *guard);

    m.complete(&mut *guard, CLASS_DECL)
}

pub(crate) fn class_body(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T!['{']);

    while !p.at(EOF) && !p.at(T!['}']) {
        match p.cur() {
            // test class_empty_element
            // class foo { ;;;;;;;;;; get foo() {};;;;}
            T![;] => {
                let inner = p.start();
                p.bump_any();
                inner.complete(p, EMPTY_STMT);
            }
            // test static_method
            // class foo {
            //  static foo(bar) {}
            //  static *foo() {}
            //  static async foo() {}
            //  static async *foo() {}
            // }
            _ if p.cur_src() == "static" => {
                let inner = p.start();
                p.bump_any();
                method(p, None, BASE_METHOD_RECOVERY_SET.union(token_set![T![;]]));
                inner.complete(p, STATIC_METHOD);
            }
            _ => {
                method(p, None, BASE_METHOD_RECOVERY_SET.union(token_set![T![;]]));
            }
        }
    }
    p.expect(T!['}']);
    m.complete(p, CLASS_BODY)
}

/// A method definition, this takes an optional markers for object props
pub fn method(
    p: &mut Parser,
    marker: impl Into<Option<Marker>>,
    recovery_set: impl Into<Option<TokenSet>>,
) -> Option<CompletedMarker> {
    let m = marker.into().unwrap_or_else(|| p.start());
    let old = p.state.to_owned();
    p.state.in_function = true;
    // FIXME: handle get* which is a property + a generator
    let complete = match p.cur() {
        // FIXME: this is wrong and it wrongfully allows things like `class foo { (bar) {} }`
        T!['('] => {
            formal_parameters(p);
            block_stmt(p, true, None);
            m.complete(p, METHOD)
        }

        // test method_getter
        // class foo {
        //  get bar() {}
        // }

        // test_err method_getter_err
        // class foo {
        //  get {}
        // }
        T![ident] if p.cur_src() == "get" && p.nth(1) != T!['('] => {
            p.bump_any();
            object_prop_name(p, false);
            p.expect(T!['(']);
            p.expect(T![')']);
            block_stmt(p, true, None);
            m.complete(p, GETTER)
        }
        // test method_setter
        // class foo {
        //  set bar() {}
        // }
        T![ident] if p.cur_src() == "set" && p.nth(1) != T!['('] => {
            p.bump_any();
            object_prop_name(p, false);
            formal_parameters(p);
            block_stmt(p, true, None);
            m.complete(p, SETTER)
        }
        // test async_method
        // class foo {
        //  async foo() {}
        //  async *foo() {}
        // }
        T![ident] if p.cur_src() == "async" && !p.has_linebreak_before_n(1) => {
            p.bump_any();
            let in_generator = p.eat(T![*]);
            let mut guard = p.with_state(ParserState {
                in_async: true,
                in_generator,
                ..p.state.clone()
            });
            object_prop_name(&mut *guard, false);
            formal_parameters(&mut *guard);
            block_stmt(&mut *guard, true, None);
            drop(guard);
            m.complete(p, METHOD)
        }
        T![*] | STRING | NUMBER | T![await] | T![ident] | T![yield] | T!['['] => {
            let in_generator = p.eat(T![*]);
            let mut guard = p.with_state(ParserState {
                in_generator,
                ..p.state.clone()
            });
            object_prop_name(&mut *guard, false);
            formal_parameters(&mut *guard);
            block_stmt(&mut *guard, true, None);
            drop(guard);
            m.complete(p, METHOD)
        }
        t if t.is_keyword() => {
            let in_generator = p.eat(T![*]);
            let mut guard = p.with_state(ParserState {
                in_generator,
                ..p.state.clone()
            });
            object_prop_name(&mut *guard, false);
            formal_parameters(&mut *guard);
            block_stmt(&mut *guard, true, None);
            drop(guard);
            m.complete(p, METHOD)
        }
        _ => {
            let err = p
                .err_builder("expected a method definition, but found none")
                .primary(p.cur_tok().range, "");

            p.err_recover(
                err,
                recovery_set.into().unwrap_or(BASE_METHOD_RECOVERY_SET),
                false,
            );
            return None;
        }
    };
    p.state = old;
    Some(complete)
}
