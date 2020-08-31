use super::expr::{assign_expr, identifier_reference, object_prop_name, EXPR_RECOVERY_SET};
use crate::{SyntaxKind::*, *};

pub fn pattern(p: &mut Parser) -> Option<CompletedMarker> {
    Some(match p.cur() {
        T![ident] | T![yield] | T![await] => {
            let m = p.start();
            binding_identifier(p);
            m.complete(p, SINGLE_PATTERN)
        }
        T!['['] => array_binding_pattern(p),
        T!['{'] => object_binding_pattern(p),
        _ => {
            let err = p
                .err_builder("Expected an identifier or pattern, but found none")
                .primary(p.cur_tok(), "");

            p.err_recover(err, EXPR_RECOVERY_SET);
            return None;
        }
    })
}

pub fn opt_binding_identifier(p: &mut Parser) -> Option<CompletedMarker> {
    const BINDING_IDENTS: TokenSet = token_set![T![ident], T![yield], T![await]];

    if p.at_ts(BINDING_IDENTS) {
        binding_identifier(p)
    } else {
        None
    }
}

pub fn binding_identifier(p: &mut Parser) -> Option<CompletedMarker> {
    if p.at(T![yield]) && p.state.in_generator {
        let err = p.err_builder("Illegal use of `yield` as an identifier in generator function")
            .primary(p.cur_tok(), "");

        p.error(err);
    }

    identifier_reference(p)
}

pub fn binding_element(p: &mut Parser) -> Option<CompletedMarker> {
    let left = pattern(p);

    if p.at(T![=]) {
        let m = left.map(|m| m.precede(p)).unwrap_or_else(|| p.start());
        p.bump_any();

        assign_expr(p);
        return Some(m.complete(p, ASSIGN_PATTERN));
    }

    left
}

pub fn array_binding_pattern(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.expect(T!['[']);

    while !p.at(EOF) && !p.at(T![']']) {
        if p.eat(T![,]) {
            continue;
        }
        if p.at(T![...]) {
            let m = p.start();
            p.bump_any();

            pattern(p);

            m.complete(p, REST_PATTERN);
            break;
        } else {
            binding_element(p);
        }
        if !p.at(T![']']) {
            p.expect(T![,]);
        }
    }

    p.expect(T![']']);
    m.complete(p, ARRAY_PATTERN)
}

pub fn object_binding_pattern(p: &mut Parser) -> CompletedMarker {
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

        if p.at(T![...]) {
            let m = p.start();
            p.bump_any();

            pattern(p);
            m.complete(p, REST_PATTERN);
            break;
        }

        object_binding_prop(p);
    }
    p.expect(T!['}']);
    m.complete(p, OBJECT_PATTERN)
}

fn object_binding_prop(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    let name = object_prop_name(p, true);
    if p.eat(T![:]) {
        binding_element(p);
        return Some(m.complete(p, KEY_VALUE_PATTERN));
    }

    if name?.kind() != NAME {
        let err = p
            .err_builder("Expected an identifier for a pattern, but found none")
            .primary(name.unwrap().range(p), "");

        p.error(err);
        return None;
    }

    if p.eat(T![=]) {
        assign_expr(p);
        name.unwrap().change_kind(p, SINGLE_PATTERN);
        Some(m.complete(p, ASSIGN_PATTERN))
    } else {
        Some(name?.precede(p).complete(p, SINGLE_PATTERN))
    }
}
