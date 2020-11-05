//! TypeScript specific functions.
//!
//! Most of the functions do not check if the parser is configured for TypeScript.
//! Functions that do check will say so in the docs.

use super::decl::*;
use super::expr::{assign_expr, identifier_name, literal, template};
use super::stmt::{semi, var_decl};
use crate::ast::Template;
use crate::{SyntaxKind::*, *};

pub const BASE_TS_RECOVERY_SET: TokenSet = token_set![
    T![void],
    T![ident],
    T![ident],
    T![await],
    T![null],
    T![break],
    T!['['],
];

pub(crate) fn ts_expr_stmt(p: &mut Parser) -> Option<CompletedMarker> {
    match p.cur_src() {
        "declare" => ts_declare(p),
        "global" => todo!(),
        _ => ts_decl(p),
    }
}

pub(crate) fn ts_declare(p: &mut Parser) -> Option<CompletedMarker> {
    debug_assert_eq!(p.cur_src(), "declare");
    Some(match p.nth(1) {
        T![function] => {
            let m = p.start();
            p.bump_any();
            function_decl(p, m, false)
        }
        T![class] => {
            let m = p.start();
            p.bump_any();
            class_decl(p, false).undo_completion(p).abandon(p);
            m.complete(p, CLASS_DECL)
        }
        t if (t == T![const] && p.nth_at(2, T![enum])) || t == T![enum] => {
            let m = p.start();
            p.bump_any();
            ts_enum(p).undo_completion(p).abandon(p);
            m.complete(p, TS_ENUM)
        }
        T![const] | T![var] => {
            let m = p.start();
            p.bump_any();
            // unwrap the marker so its children go to `m`
            var_decl(p, false).undo_completion(p).abandon(p);
            m.complete(p, VAR_DECL)
        }
        _ if p.nth_src(1) == "let" => {
            let m = p.start();
            p.bump_any();
            var_decl(p, false).undo_completion(p).abandon(p);
            m.complete(p, VAR_DECL)
        }
        _ => {
            let checkpoint = p.checkpoint();
            let m = p.start();
            p.bump_any();
            let res = ts_decl(p);
            if let Some(res) = res {
                let kind = res.kind();
                res.undo_completion(p).abandon(p);
                return Some(m.complete(p, kind));
            } else {
                m.abandon(p);
                p.rewind(checkpoint);
                return None;
            }
        }
    })
}

pub(crate) fn ts_decl(p: &mut Parser) -> Option<CompletedMarker> {
    if p.cur_src() == "abstract" {
        let m = p.start();
        p.bump_any();
        class_decl(p, false).undo_completion(p).abandon(p);
        return Some(m.complete(p, CLASS_DECL));
    }

    if p.at(T![enum]) {
        let m = p.start();
        p.bump_any();
        ts_enum(p).undo_completion(p).abandon(p);
        return Some(m.complete(p, TS_ENUM));
    }

    if p.cur_src() == "interface" {
        todo!("interfaces");
    }

    if p.cur_src() == "module" {
        todo!("modules");
    }

    if p.cur_src() == "namespace" {
        todo!("namespaces");
    }

    if p.cur_src() == "type" {
        let m = p.start();
        let start = p.cur_tok().range.start;
        p.bump_any();
        identifier_name(p);
        if p.at(T![<]) {
            ts_type_params(p);
        }
        let end = p.cur_tok().range.end;
        p.expect(T![=]);
        semi(p, start..end);
        return Some(m.complete(p, TS_TYPE_ALIAS_DECL));
    }

    None
}

// ambiguity is fun!
macro_rules! no_recover {
    ($p:expr, $res:expr) => {
        if $res.is_none() && $p.state.no_recovery {
            return None;
        }
    };
}

pub fn ts_enum(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.eat(T![const]);
    p.expect(T![enum]);
    identifier_name(p);
    p.expect(T!['{']);
    let mut first = true;

    while !p.at(EOF) && !p.at(T!['}']) {
        if first {
            first = false;
        } else {
            p.expect(T![,]);
        }

        let member = p.start();
        let err_occured =
            if !p.at_ts(token_set![T![ident], T![yield], T![await]]) && !p.cur().is_keyword() {
                let err = p
                    .err_builder("expected an identifier for an enum variant, but found none")
                    .primary(p.cur_tok().range, "");

                p.err_recover(
                    err,
                    token_set![T!['}'], T![ident], T![yield], T![await], T![=], T![,]],
                    false,
                );
                true
            } else {
                identifier_name(p).unwrap().undo_completion(p).abandon(p);
                false
            };

        if p.eat(T![=]) {
            assign_expr(p);
            member.complete(p, TS_ENUM_MEMBER);
        } else if err_occured {
            member.abandon(p);
        } else {
            member.complete(p, TS_ENUM_MEMBER);
        }
    }

    p.expect(T!['}']);
    m.complete(p, TS_ENUM)
}

pub fn try_parse_ts(
    p: &mut Parser,
    func: impl FnOnce(&mut Parser) -> Option<CompletedMarker>,
) -> Option<CompletedMarker> {
    let checkpoint = p.checkpoint();
    let res = func(&mut *p.with_state(ParserState {
        no_recovery: true,
        ..p.state.clone()
    }));
    if res.is_none() {
        p.rewind(checkpoint);
    }
    res
}

pub fn ts_type(p: &mut Parser) -> Option<CompletedMarker> {
    let ty = ts_non_conditional_type(p);
    if p.has_linebreak_before_n(0) || !p.at(T![extends]) {
        return ty;
    }

    let m = ty.map(|x| x.precede(p)).unwrap_or_else(|| p.start());
    ts_non_conditional_type(p);
    let compl = m.complete(p, TS_EXTENDS);
    let m = compl.precede(p);
    p.expect_no_recover(T![?])?;
    no_recover!(p, ts_type(p));
    p.expect_no_recover(T![:])?;
    no_recover!(p, ts_type(p));
    Some(m.complete(p, TS_CONDITIONAL_TYPE))
}

pub fn ts_fn_or_constructor_type(p: &mut Parser, fn_type: bool) -> Option<CompletedMarker> {
    let m = p.start();
    if !fn_type {
        p.expect_no_recover(T![new])?;
    }

    if p.at(T![<]) {
        ts_type_params(p);
    }
    formal_parameters(p);
    no_recover!(p, ts_type_or_type_predicate_ann(p, T![=>]));
    Some(m.complete(
        p,
        if fn_type {
            TS_FN_TYPE
        } else {
            TS_CONSTRUCTOR_TYPE
        },
    ))
}

fn ts_type_or_type_predicate_ann(
    p: &mut Parser,
    return_token: SyntaxKind,
) -> Option<CompletedMarker> {
    let ident_ref_set = token_set![T![await], T![yield], T![ident]];
    p.expect_no_recover(return_token)?;

    let type_pred = (p.cur_src() == "asserts" && ident_ref_set.contains(p.nth(1)))
        || (p.at_ts(ident_ref_set) && p.nth_src(1) == "is" && !p.has_linebreak_before_n(1));

    if type_pred {
        ts_predicate(p)
    } else {
        ts_type(p)
    }
}

pub fn ts_non_conditional_type(p: &mut Parser) -> Option<CompletedMarker> {
    if is_start_of_fn_type(p) {
        return ts_fn_or_constructor_type(p, true);
    }

    if p.at(T![new]) {
        return ts_fn_or_constructor_type(p, true);
    }

    intersection_or_union(p, false, |p| ts_intersection_type_or_higher(p), T![|])
}

fn ts_intersection_type_or_higher(p: &mut Parser) -> Option<CompletedMarker> {
    intersection_or_union(p, true, |p| ts_type_operator_or_higher(p), T![&])
}

fn look_ahead(p: &mut Parser, func: impl FnOnce(&mut Parser) -> bool) -> bool {
    let checkpoint = p.checkpoint();
    let res = func(p);
    p.rewind(checkpoint);
    res
}

fn is_start_of_fn_type(p: &mut Parser) -> bool {
    p.at(T![<]) || (p.at(T!['(']) && look_ahead(p, is_unambiguously_start_of_fn_type))
}

fn is_unambiguously_start_of_fn_type(p: &mut Parser) -> bool {
    p.eat(T!['(']);
    if p.at(T![')']) || p.at(T![...]) {
        return true;
    }

    if skip_parameter_start(p) {
        if p.at_ts(token_set![T![:], T![,], T![?], T![=]]) {
            return true;
        }
        if p.at(T![')']) && p.nth_at(1, T![=>]) {
            return true;
        }
    }
    false
}

fn skip_parameter_start(p: &mut Parser) -> bool {
    maybe_eat_incorrect_modifier(p);
    if p.at_ts(token_set![T![this], T![yield], T![ident], T![await]]) {
        p.bump_any();
        return true;
    }

    if p.eat(T!['{']) {
        let mut counter = 1;

        while counter > 0 {
            if p.eat(T!['{']) {
                counter += 1;
            } else if p.eat(T!['}']) {
                counter -= 1;
            }
        }
        return true;
    }

    if p.eat(T!['[']) {
        let mut counter = 1;

        while counter > 0 {
            if p.eat(T!['[']) {
                counter += 1;
            } else if p.eat(T![']']) {
                counter -= 1;
            }
        }
        return true;
    }
    false
}

fn intersection_or_union(
    p: &mut Parser,
    intersection: bool,
    mut constituent: impl FnMut(&mut Parser) -> Option<CompletedMarker>,
    op: SyntaxKind,
) -> Option<CompletedMarker> {
    let kind = if intersection {
        TS_INTERSECTION
    } else {
        TS_UNION
    };
    let m = p.start();
    let saw_op = p.eat(op);
    let ty = constituent(p);
    if p.at(op) {
        while p.eat(op) {
            constituent(p);
        }

        Some(m.complete(p, kind))
    } else if !saw_op && ty.is_none() {
        m.abandon(p);
        None
    } else {
        Some(m.complete(p, kind))
    }
}

pub fn ts_type_operator_or_higher(p: &mut Parser) -> Option<CompletedMarker> {
    if matches!(p.cur_src(), "keyof" | "unique" | "readonly") {
        let m = p.start();
        p.bump_any();
        no_recover!(p, ts_type_operator_or_higher(p));
        Some(m.complete(p, TS_TYPE_OPERATOR))
    } else if p.cur_src() == "infer" {
        todo!("infer")
    } else {
        // FIXME: readonly should apparently be handled here?
        // but the previous matches should have accounted for it 🤔
        ts_array_type_or_higher(p)
    }
}

pub fn ts_array_type_or_higher(p: &mut Parser) -> Option<CompletedMarker> {
    let ty = ts_non_array_type(p);

    if !p.has_linebreak_before_n(0) && p.at(T!['[']) {
        let m = ty.map(|x| x.precede(p)).unwrap_or_else(|| p.start());
        if p.eat(T![']']) {
            Some(m.complete(p, TS_ARRAY))
        } else {
            no_recover!(p, ts_type(p));
            p.expect_no_recover(T![']'])?;
            Some(m.complete(p, TS_INDEXED_ARRAY))
        }
    } else {
        ty
    }
}

pub fn ts_non_array_type(p: &mut Parser) -> Option<CompletedMarker> {
    match p.cur() {
        T![ident] | T![void] | T![yield] | T![null] | T![await] | T![break] => {
            if p.cur_src() == "asserts" && p.nth_at(1, T![this]) {
                p.bump_any();
                return ts_predicate(p);
            }

            let kind = match p.cur_src() {
                "void" => TS_VOID,
                "null" => TS_NULL,
                "any" => TS_ANY,
                "boolean" => TS_BOOLEAN,
                "bigint" => TS_BIGINT,
                "never" => TS_NEVER,
                "number" => TS_NUMBER,
                "object" => TS_OBJECT,
                "string" => TS_STRING,
                "symbol" => TS_SYMBOL,
                "unknown" => TS_UNKNOWN,
                "undefined" => TS_UNDEFINED,
                _ =>
                /* dummy value */
                {
                    ERROR
                }
            };

            if kind != ERROR && !p.nth_at(1, T![.]) {
                let m = p.start();
                p.bump_any();
                Some(m.complete(p, kind))
            } else {
                ts_type_ref(p, None)
            }
        }
        NUMBER | STRING | TRUE_KW | FALSE_KW | REGEX => {
            Some(literal(p).unwrap().precede(p).complete(p, TS_LITERAL))
        }
        BACKTICK => {
            let complete = template(p, None);
            // TODO: we can do this more efficiently by just looking at each event
            let parsed = p.parse_marker::<Template>(&complete);
            for elem in parsed.elements() {
                let err = p
                    .err_builder(
                        "template literals used as TypeScript types may not contain expressions",
                    )
                    .primary(elem.range(), "");

                p.error(err);
            }
            Some(complete.precede(p).complete(p, TS_TEMPLATE))
        }
        T![-] => {
            let m = p.start();
            p.bump_any();
            if p.at(NUMBER) {
                let _m = p.start();
                p.bump_any();
                _m.complete(p, LITERAL);
            } else {
                p.expect_no_recover(NUMBER)?;
            }
            Some(m.complete(p, TS_LITERAL))
        }
        T![import] => ts_import(p),
        T![this] => {
            if p.nth_src(1) == "is" {
                ts_predicate(p)
            } else {
                let m = p.start();
                p.bump_any();
                Some(m.complete(p, TS_THIS))
            }
        }
        T![typeof] => ts_type_query(p),
        T!['{'] => {
            if is_mapped_type_start(p) {
                ts_mapped_type(p)
            } else {
                todo!("object types")
            }
        }
        T!['['] => todo!("tuples"),
        T!['('] => {
            let m = p.start();
            p.bump_any();
            no_recover!(p, ts_type(p));
            p.expect_no_recover(T![')'])?;
            Some(m.complete(p, TS_PAREN))
        }
        _ => {
            let err = p
                .err_builder("expected a type")
                .primary(p.cur_tok().range, "");

            p.err_recover(
                err,
                BASE_TS_RECOVERY_SET.union(token_set![
                    T![typeof],
                    T!['{'],
                    T!['['],
                    T!['('],
                    T![this],
                    T![import],
                    T![-],
                    NUMBER,
                    STRING,
                    TRUE_KW,
                    FALSE_KW,
                    REGEX,
                    BACKTICK,
                    T![&],
                    T![|]
                ]),
                false,
            );
            None
        }
    }
}

pub fn ts_type_args(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    p.expect_no_recover(T![<])?;
    let mut first = true;

    while !p.at(EOF) && !p.at(T![>]) {
        if first {
            first = false;
        } else if p.at(T![,]) && p.nth_at(1, T![>]) {
            let m = p.start();
            let range = p.cur_tok().range;
            p.bump_any();
            m.complete(p, ERROR);
            let err = p
                .err_builder("type arguments may not contain trailing commas")
                .primary(range, "help: remove this comma");

            p.error(err);
        } else {
            p.expect_no_recover(T![,])?;
        }
        no_recover!(p, ts_type(p));
    }
    p.expect_no_recover(T![>])?;
    Some(m.complete(p, TS_TYPE_ARGS))
}

// FIXME: `<T() => {}` causes infinite recursion if the parser isnt being run with `no_recovery`
pub fn ts_type_params(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    p.expect_no_recover(T![<])?;
    let mut first = true;

    while !p.at(EOF) && !p.at(T![>]) {
        if first {
            first = false;
        } else {
            p.expect_no_recover(T![,])?;
        }
        no_recover!(p, type_param(p));
    }
    p.expect_no_recover(T![>])?;
    Some(m.complete(p, TS_TYPE_PARAMS))
}

fn type_param(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    if let Some(x) = identifier_name(p) {
        x.undo_completion(p).abandon(p)
    }
    if p.at(T![extends]) {
        let _m = p.start();
        p.bump_any();
        no_recover!(p, ts_type(p));
        _m.complete(p, TS_CONSTRAINT);
    }
    if p.at(T![=]) {
        let _m = p.start();
        p.bump_any();
        no_recover!(p, ts_type(p));
        _m.complete(p, TS_DEFAULT);
    }
    Some(m.complete(p, TS_TYPE_PARAM))
}

pub fn ts_import(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    p.expect_no_recover(T![import])?;
    p.expect_no_recover(T!['('])?;
    p.expect_no_recover(STRING)?;
    p.expect_no_recover(T![')'])?;
    if p.eat(T![.]) {
        ts_entity_name(p, None, false);
    }
    if p.at(T![<]) && !p.has_linebreak_before_n(0) {
        ts_type_args(p);
    }

    Some(m.complete(p, TS_IMPORT))
}

pub fn ts_type_query(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    p.expect_no_recover(T![typeof])?;

    if p.at(T![import]) {
        no_recover!(p, ts_import(p));
    } else {
        no_recover!(p, ts_entity_name(p, None, true));
    }
    Some(m.complete(p, TS_TYPE_QUERY))
}

pub fn ts_mapped_type(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    p.expect_no_recover(T!['{'])?;
    let tok = p.cur_tok().range;
    let _m = p.start();
    if p.eat(T![+]) || p.eat(T![-]) {
        if p.cur_src() != "readonly" {
            let err = p
                .err_builder("`+` and `-` modifiers in mapped types must be followed by `readonly`")
                .primary(tok, "");

            p.error(err);
        } else {
            p.bump_any();
        }
        _m.complete(p, TS_MAPPED_TYPE_READONLY);
    } else if p.cur_src() == "readonly" {
        p.bump_any();
        _m.complete(p, TS_MAPPED_TYPE_READONLY);
    } else {
        _m.abandon(p);
    }

    let param = p.start();
    p.expect_no_recover(T!['['])?;
    // This is basically to unwrap the marker from a node to a single token
    if let Some(x) = identifier_name(p) {
        x.undo_completion(p).abandon(p)
    }
    if p.cur_src() != "in" {
        let err = p
            .err_builder("expected `in` after a mapped type parameter name")
            .primary(p.cur_tok().range, "");

        p.error(err);
    } else {
        p.bump_any();
    }
    if p.cur_src() == "as" {
        p.bump_any();
        ts_type(p);
    }
    p.expect_no_recover(T![']'])?;
    param.complete(p, TS_MAPPED_TYPE_PARAM);
    let tok = p.cur_tok().range;
    if p.eat(T![+]) || p.eat(T![-]) {
        if !p.at(T![?]) {
            // TODO: Im not sure of the proper terminology for this, someone should clarify this error
            let err = p
                .err_builder("`+` and `-` modifiers in mapped types must be followed by `?`")
                .primary(tok, "");

            p.error(err);
        } else {
            p.bump_any();
        }
    } else if p.at(T![?]) {
        p.bump_any();
    }

    p.expect_no_recover(T![:])?;
    no_recover!(p, ts_type(p));
    // FIXME: This should issue an error for no semi and no ASI, but the fact that a `}` is expected
    // after should make this case kind of rare
    p.eat(T![;]);
    p.expect_no_recover(T!['}'])?;
    Some(m.complete(p, TS_MAPPED_TYPE))
}

fn is_mapped_type_start(p: &Parser) -> bool {
    if (p.nth_at(1, T![+]) || p.nth_at(1, T![+])) && p.nth_src(1) == "readonly" {
        return true;
    }
    let mut cur = 1;
    if p.cur_src() == "readonly" {
        cur += 1;
    }
    if !p.nth_at(cur, T!['[']) {
        return false;
    }
    cur += 1;
    if !matches!(p.nth(cur), T![yield] | T![await] | T![ident]) {
        return false;
    }
    cur += 1;
    p.nth_at(cur, T![in])
}

pub fn ts_predicate(p: &mut Parser) -> Option<CompletedMarker> {
    let m = p.start();
    let mut advanced = false;

    if p.cur_src() == "asserts" {
        p.bump_any();
        advanced = true;
    }

    if p.at(T![this]) {
        let _m = p.start();
        p.bump_any();
        _m.complete(p, TS_THIS);
        advanced = true;
    } else if p.at_ts(token_set![T![await], T![yield], T![ident]]) {
        let _m = p.start();
        p.bump_any();
        _m.complete(p, TS_TYPE_NAME);
        advanced = true;
    }

    if p.cur_src() == "is" {
        p.bump_any();
        no_recover!(p, ts_type(p));
        advanced = true;
    }

    if !advanced {
        m.abandon(p);
        None
    } else {
        Some(m.complete(p, TS_PREDICATE))
    }
}

fn maybe_eat_incorrect_modifier(p: &mut Parser) -> Option<CompletedMarker> {
    if matches!(p.cur_src(), "public" | "private" | "protected" | "readonly") {
        let m = p.start();
        p.bump_any();
        Some(m.complete(p, ERROR))
    } else {
        None
    }
}

pub fn ts_type_ref(
    p: &mut Parser,
    recovery_set: impl Into<Option<TokenSet>> + Clone,
) -> Option<CompletedMarker> {
    let m = p.start();
    if let Some(err_m) = maybe_eat_incorrect_modifier(p) {
        let err = p
            .err_builder("a parameter property is only allowed in a constructor implementation")
            .primary(err_m.range(p), "");

        p.error(err);
    }

    ts_entity_name(p, recovery_set, true)?;
    if !p.has_linebreak_before_n(0) && p.at(T![<]) {
        no_recover!(p, ts_type_args(p));
    }

    Some(m.complete(p, TS_TYPE_REF))
}

pub fn ts_entity_name(
    p: &mut Parser,
    recovery_set: impl Into<Option<TokenSet>> + Clone,
    allow_reserved: bool,
) -> Option<CompletedMarker> {
    let init = ts_type_name(p, recovery_set.clone(), false)?;
    // TODO: maybe we should recover if no init at this point?

    let mut lhs = init;
    let set = recovery_set
        .into()
        .unwrap_or(BASE_TS_RECOVERY_SET)
        .union(token_set![T![.]]);

    while p.at(T![.]) {
        let m = lhs.precede(p);
        // TODO: we should maybe move recovery out of ts_type_name since we dont need recovery here
        no_recover!(p, ts_type_name(p, set, allow_reserved));
        lhs = m.complete(p, TS_QUALIFIED_PATH);
    }
    Some(lhs)
}

pub fn ts_type_name(
    p: &mut Parser,
    recovery_set: impl Into<Option<TokenSet>>,
    allow_reserved: bool,
) -> Option<CompletedMarker> {
    if p.at(T![ident]) || (p.cur().is_keyword() && allow_reserved) {
        let m = p.start();
        p.bump_remap(T![ident]);
        return Some(m.complete(p, TS_TYPE_NAME));
    }

    // FIXME: move the recovery job out of this method
    let set = recovery_set.into().unwrap_or(BASE_TS_RECOVERY_SET);
    let err = p
        .err_builder(&format!(
            "expected a TypeScript type name, but instead found `{}`",
            p.cur_src()
        ))
        .primary(p.cur_tok().range, "");

    p.err_recover(err, set, false)?;
    None
}
