//! Rules which relate to regular expressions.

use crate::group;
use once_cell::sync::Lazy;
use rslint_errors::Span;
use rslint_lexer::SyntaxKind;
use rslint_parser::{
    ast::{ArgList, Literal, LiteralKind},
    AstNode, SyntaxNode, SyntaxNodeExt,
};
use rslint_regex::{validate_flags, EcmaVersion, Flags, Parser, Regex};
use std::sync::Mutex;
use std::{collections::HashMap, ops::Range};

type RegexResult = Result<(Regex, Range<usize>), (Range<usize>, String)>;

pub(crate) static REGEX_MAP: Lazy<Mutex<HashMap<Range<usize>, RegexResult>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

group! {
    /// Rules which relate to regular expressions.
    regex,
    no_invalid_regexp::NoInvalidRegexp,
    simplify_regex::SimplifyRegex
}

pub(crate) fn maybe_parse_and_store_regex(
    node: &SyntaxNode,
    file_id: usize,
) -> Option<RegexResult> {
    let mut map_handle = REGEX_MAP.lock().unwrap();
    if let Some(r) = map_handle.get(&node.as_range()) {
        return Some(r.to_owned());
    }
    let r = collect_regex_from_node(node, file_id)?;
    map_handle.insert(node.as_range(), r.clone());
    Some(r)
}

fn collect_regex_from_node(node: &SyntaxNode, file_id: usize) -> Option<RegexResult> {
    match node.kind() {
        SyntaxKind::NEW_EXPR | SyntaxKind::CALL_EXPR => {
            let name = node.child_with_kind(SyntaxKind::NAME_REF);
            if name.map_or(false, |x| x.text() == "RegExp") {
                let mut args = node
                    .child_with_ast::<ArgList>()
                    .map(|x| x.args())
                    .into_iter()
                    .flatten();
                let pat = args.next().and_then(|x| {
                    Some((
                        x.syntax().try_to::<Literal>()?.inner_string_text()?,
                        x.range(),
                    ))
                });

                let flags = args.next().and_then(|x| {
                    Some((
                        x.syntax().try_to::<Literal>()?.inner_string_text()?,
                        x.range(),
                    ))
                });

                if let Some((pat, range)) = pat {
                    let range = range.as_range();
                    let new_range = range.start + 1..range.end - 1;
                    let flags = if let Some((flags, flag_range)) = flags {
                        match validate_flags(&flags.to_string(), EcmaVersion::ES2021) {
                            Ok(f) => f,
                            Err(err) => {
                                return Some(Err((flag_range.as_range(), err)));
                            }
                        }
                    } else {
                        Flags::empty()
                    };

                    let pattern = &pat.to_string();
                    let parser = Parser::new_from_pattern_and_flags(
                        pattern,
                        file_id,
                        range.as_range().start + 1,
                        EcmaVersion::ES2021,
                        false,
                        flags,
                    );
                    Some(match parser.parse() {
                        Ok(r) => Ok((r, new_range)),
                        Err(err) => Err((err.span.as_range(), err.message)),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
        SyntaxKind::LITERAL if node.to::<Literal>().kind() == LiteralKind::Regex => {
            let pattern = &node.text().to_string();
            let parser = Parser::new(
                pattern,
                file_id,
                node.as_range().start,
                EcmaVersion::ES2021,
                false,
            );
            let range = node.as_range();
            let new_range = range.start + 1..range.end - 1;
            let res = match parser {
                Ok(p) => p.parse(),
                Err(err) => {
                    return Some(Err((err.span.as_range(), err.message)));
                }
            };
            Some(match res {
                Ok(r) => Ok((r, new_range)),
                Err(err) => Err((err.span.as_range(), err.message)),
            })
        }
        _ => None,
    }
}
