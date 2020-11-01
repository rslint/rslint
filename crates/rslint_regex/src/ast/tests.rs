use std::ops::Range;

use ast::parse::{Parser, ParserBuilder, ParserI, Primitive};
use ast::{self, Ast, Position, Span};

// Our own assert_eq, which has slightly better formatting (but honestly
// still kind of crappy).
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {{
        match (&$left, &$right) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    panic!(
                        "assertion failed: `(left == right)`\n\n\
                             left:  `{:?}`\nright: `{:?}`\n\n",
                        left_val, right_val
                    )
                }
            }
        }
    }};
}

// We create these errors to compare with real ast::Errors in the tests.
// We define equality between TestError and ast::Error to disregard the
// pattern string in ast::Error, which is annoying to provide in tests.
#[derive(Clone, Debug)]
struct TestError {
    span: Span,
    kind: ast::ErrorKind,
}

impl PartialEq<ast::Error> for TestError {
    fn eq(&self, other: &ast::Error) -> bool {
        self.span == other.span && self.kind == other.kind
    }
}

impl PartialEq<TestError> for ast::Error {
    fn eq(&self, other: &TestError) -> bool {
        self.span == other.span && self.kind == other.kind
    }
}

fn s(str: &str) -> String {
    str.to_string()
}

fn parser(pattern: &str) -> ParserI<Parser> {
    ParserI::new(Parser::new(), pattern)
}

fn parser_octal(pattern: &str) -> ParserI<Parser> {
    let parser = ParserBuilder::new().octal(true).build();
    ParserI::new(parser, pattern)
}

fn parser_nest_limit(pattern: &str, nest_limit: u32) -> ParserI<Parser> {
    let p = ParserBuilder::new().nest_limit(nest_limit).build();
    ParserI::new(p, pattern)
}

fn parser_ignore_whitespace(pattern: &str) -> ParserI<Parser> {
    let p = ParserBuilder::new().ignore_whitespace(true).build();
    ParserI::new(p, pattern)
}

/// Short alias for creating a new span.
fn nspan(start: Position, end: Position) -> Span {
    Span::new(start, end)
}

/// Short alias for creating a new position.
fn npos(offset: usize, line: usize, column: usize) -> Position {
    Position::new(offset, line, column)
}

/// Create a new span from the given offset range. This assumes a single
/// line and sets the columns based on the offsets. i.e., This only works
/// out of the box for ASCII, which is fine for most tests.
fn span(range: Range<usize>) -> Span {
    let start = Position::new(range.start, 1, range.start + 1);
    let end = Position::new(range.end, 1, range.end + 1);
    Span::new(start, end)
}

/// Create a new span for the corresponding byte range in the given string.
fn span_range(subject: &str, range: Range<usize>) -> Span {
    let start = Position {
        offset: range.start,
        line: 1 + subject[..range.start].matches('\n').count(),
        column: 1 + subject[..range.start]
            .chars()
            .rev()
            .position(|c| c == '\n')
            .unwrap_or_else(|| subject[..range.start].chars().count()),
    };
    let end = Position {
        offset: range.end,
        line: 1 + subject[..range.end].matches('\n').count(),
        column: 1 + subject[..range.end]
            .chars()
            .rev()
            .position(|c| c == '\n')
            .unwrap_or_else(|| subject[..range.end].chars().count()),
    };
    Span::new(start, end)
}

/// Create a verbatim literal starting at the given position.
fn lit(c: char, start: usize) -> Ast {
    lit_with(c, span(start..start + c.len_utf8()))
}

/// Create a punctuation literal starting at the given position.
fn punct_lit(c: char, span: Span) -> Ast {
    Ast::Literal(ast::Literal {
        span,
        kind: ast::LiteralKind::Punctuation,
        c,
    })
}

/// Create a verbatim literal with the given span.
fn lit_with(c: char, span: Span) -> Ast {
    Ast::Literal(ast::Literal {
        span,
        kind: ast::LiteralKind::Verbatim,
        c,
    })
}

/// Create a concatenation with the given range.
fn concat(range: Range<usize>, asts: Vec<Ast>) -> Ast {
    concat_with(span(range), asts)
}

/// Create a concatenation with the given span.
fn concat_with(span: Span, asts: Vec<Ast>) -> Ast {
    Ast::Concat(ast::Concat { span, asts })
}

/// Create an alternation with the given span.
fn alt(range: Range<usize>, asts: Vec<Ast>) -> Ast {
    Ast::Alternation(ast::Alternation {
        span: span(range),
        asts,
    })
}

/// Create a capturing group with the given span.
fn group(range: Range<usize>, index: u32, ast: Ast) -> Ast {
    Ast::Group(ast::Group {
        span: span(range),
        kind: ast::GroupKind::CaptureIndex(index),
        ast: Box::new(ast),
    })
}

/// Create an ast::SetFlags.
///
/// The given pattern should be the full pattern string. The range given
/// should correspond to the byte offsets where the flag set occurs.
///
/// If negated is true, then the set is interpreted as beginning with a
/// negation.
fn flag_set(pat: &str, range: Range<usize>, flag: ast::Flag, negated: bool) -> Ast {
    let mut items = vec![ast::FlagsItem {
        span: span_range(pat, (range.end - 2)..(range.end - 1)),
        kind: ast::FlagsItemKind::Flag(flag),
    }];
    if negated {
        items.insert(
            0,
            ast::FlagsItem {
                span: span_range(pat, (range.start + 2)..(range.end - 2)),
                kind: ast::FlagsItemKind::Negation,
            },
        );
    }
    Ast::Flags(ast::SetFlags {
        span: span_range(pat, range.clone()),
        flags: ast::Flags {
            span: span_range(pat, (range.start + 2)..(range.end - 1)),
            items,
        },
    })
}

#[test]
fn parse_nest_limit() {
    // A nest limit of 0 still allows some types of regexes.
    assert_eq!(parser_nest_limit("", 0).parse(), Ok(Ast::Empty(span(0..0))));
    assert_eq!(parser_nest_limit("a", 0).parse(), Ok(lit('a', 0)));

    // Test repetition operations, which require one level of nesting.
    assert_eq!(
        parser_nest_limit("a+", 0).parse().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::NestLimitExceeded(0),
        }
    );
    assert_eq!(
        parser_nest_limit("a+", 1).parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..2),
            op: ast::RepetitionOp {
                span: span(1..2),
                kind: ast::RepetitionKind::OneOrMore,
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser_nest_limit("(a)+", 1).parse().unwrap_err(),
        TestError {
            span: span(0..3),
            kind: ast::ErrorKind::NestLimitExceeded(1),
        }
    );
    assert_eq!(
        parser_nest_limit("a+*", 1).parse().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::NestLimitExceeded(1),
        }
    );
    assert_eq!(
        parser_nest_limit("a+*", 2).parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..3),
            op: ast::RepetitionOp {
                span: span(2..3),
                kind: ast::RepetitionKind::ZeroOrMore,
            },
            greedy: true,
            ast: Box::new(Ast::Repetition(ast::Repetition {
                span: span(0..2),
                op: ast::RepetitionOp {
                    span: span(1..2),
                    kind: ast::RepetitionKind::OneOrMore,
                },
                greedy: true,
                ast: Box::new(lit('a', 0)),
            })),
        }))
    );

    // Test concatenations. A concatenation requires one level of nesting.
    assert_eq!(
        parser_nest_limit("ab", 0).parse().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::NestLimitExceeded(0),
        }
    );
    assert_eq!(
        parser_nest_limit("ab", 1).parse(),
        Ok(concat(0..2, vec![lit('a', 0), lit('b', 1)]))
    );
    assert_eq!(
        parser_nest_limit("abc", 1).parse(),
        Ok(concat(0..3, vec![lit('a', 0), lit('b', 1), lit('c', 2)]))
    );

    // Test alternations. An alternation requires one level of nesting.
    assert_eq!(
        parser_nest_limit("a|b", 0).parse().unwrap_err(),
        TestError {
            span: span(0..3),
            kind: ast::ErrorKind::NestLimitExceeded(0),
        }
    );
    assert_eq!(
        parser_nest_limit("a|b", 1).parse(),
        Ok(alt(0..3, vec![lit('a', 0), lit('b', 2)]))
    );
    assert_eq!(
        parser_nest_limit("a|b|c", 1).parse(),
        Ok(alt(0..5, vec![lit('a', 0), lit('b', 2), lit('c', 4)]))
    );

    // Test character classes. Classes form their own mini-recursive
    // syntax!
    assert_eq!(
        parser_nest_limit("[a]", 0).parse().unwrap_err(),
        TestError {
            span: span(0..3),
            kind: ast::ErrorKind::NestLimitExceeded(0),
        }
    );
    assert_eq!(
        parser_nest_limit("[a]", 1).parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..3),
            negated: false,
            kind: ast::ClassSet::Item(ast::ClassSetItem::Literal(ast::Literal {
                span: span(1..2),
                kind: ast::LiteralKind::Verbatim,
                c: 'a',
            })),
        })))
    );
    assert_eq!(
        parser_nest_limit("[ab]", 1).parse().unwrap_err(),
        TestError {
            span: span(1..3),
            kind: ast::ErrorKind::NestLimitExceeded(1),
        }
    );
    assert_eq!(
        parser_nest_limit("[ab[cd]]", 2).parse().unwrap_err(),
        TestError {
            span: span(3..7),
            kind: ast::ErrorKind::NestLimitExceeded(2),
        }
    );
    assert_eq!(
        parser_nest_limit("[ab[cd]]", 3).parse().unwrap_err(),
        TestError {
            span: span(4..6),
            kind: ast::ErrorKind::NestLimitExceeded(3),
        }
    );
    assert_eq!(
        parser_nest_limit("[a--b]", 1).parse().unwrap_err(),
        TestError {
            span: span(1..5),
            kind: ast::ErrorKind::NestLimitExceeded(1),
        }
    );
    assert_eq!(
        parser_nest_limit("[a--bc]", 2).parse().unwrap_err(),
        TestError {
            span: span(4..6),
            kind: ast::ErrorKind::NestLimitExceeded(2),
        }
    );
}

#[test]
fn parse_comments() {
    let pat = "(?x)
# This is comment 1.
foo # This is comment 2.
  # This is comment 3.
bar
# This is comment 4.";
    let astc = parser(pat).parse_with_comments().unwrap();
    assert_eq!(
        astc.ast,
        concat_with(
            span_range(pat, 0..pat.len()),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                lit_with('f', span_range(pat, 26..27)),
                lit_with('o', span_range(pat, 27..28)),
                lit_with('o', span_range(pat, 28..29)),
                lit_with('b', span_range(pat, 74..75)),
                lit_with('a', span_range(pat, 75..76)),
                lit_with('r', span_range(pat, 76..77)),
            ]
        )
    );
    assert_eq!(
        astc.comments,
        vec![
            ast::Comment {
                span: span_range(pat, 5..26),
                comment: s(" This is comment 1."),
            },
            ast::Comment {
                span: span_range(pat, 30..51),
                comment: s(" This is comment 2."),
            },
            ast::Comment {
                span: span_range(pat, 53..74),
                comment: s(" This is comment 3."),
            },
            ast::Comment {
                span: span_range(pat, 78..98),
                comment: s(" This is comment 4."),
            },
        ]
    );
}

#[test]
fn parse_holistic() {
    assert_eq!(parser("]").parse(), Ok(lit(']', 0)));
    assert_eq!(
        parser(r"\\\.\+\*\?\(\)\|\[\]\{\}\^\$\#\&\-\~").parse(),
        Ok(concat(
            0..36,
            vec![
                punct_lit('\\', span(0..2)),
                punct_lit('.', span(2..4)),
                punct_lit('+', span(4..6)),
                punct_lit('*', span(6..8)),
                punct_lit('?', span(8..10)),
                punct_lit('(', span(10..12)),
                punct_lit(')', span(12..14)),
                punct_lit('|', span(14..16)),
                punct_lit('[', span(16..18)),
                punct_lit(']', span(18..20)),
                punct_lit('{', span(20..22)),
                punct_lit('}', span(22..24)),
                punct_lit('^', span(24..26)),
                punct_lit('$', span(26..28)),
                punct_lit('#', span(28..30)),
                punct_lit('&', span(30..32)),
                punct_lit('-', span(32..34)),
                punct_lit('~', span(34..36)),
            ]
        ))
    );
}

#[test]
fn parse_ignore_whitespace() {
    // Test that basic whitespace insensitivity works.
    let pat = "(?x)a b";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            nspan(npos(0, 1, 1), npos(7, 1, 8)),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                lit_with('a', nspan(npos(4, 1, 5), npos(5, 1, 6))),
                lit_with('b', nspan(npos(6, 1, 7), npos(7, 1, 8))),
            ]
        ))
    );

    // Test that we can toggle whitespace insensitivity.
    let pat = "(?x)a b(?-x)a b";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            nspan(npos(0, 1, 1), npos(15, 1, 16)),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                lit_with('a', nspan(npos(4, 1, 5), npos(5, 1, 6))),
                lit_with('b', nspan(npos(6, 1, 7), npos(7, 1, 8))),
                flag_set(pat, 7..12, ast::Flag::IgnoreWhitespace, true),
                lit_with('a', nspan(npos(12, 1, 13), npos(13, 1, 14))),
                lit_with(' ', nspan(npos(13, 1, 14), npos(14, 1, 15))),
                lit_with('b', nspan(npos(14, 1, 15), npos(15, 1, 16))),
            ]
        ))
    );

    // Test that nesting whitespace insensitive flags works.
    let pat = "a (?x:a )a ";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..11),
            vec![
                lit_with('a', span_range(pat, 0..1)),
                lit_with(' ', span_range(pat, 1..2)),
                Ast::Group(ast::Group {
                    span: span_range(pat, 2..9),
                    kind: ast::GroupKind::NonCapturing(ast::Flags {
                        span: span_range(pat, 4..5),
                        items: vec![ast::FlagsItem {
                            span: span_range(pat, 4..5),
                            kind: ast::FlagsItemKind::Flag(ast::Flag::IgnoreWhitespace),
                        },],
                    }),
                    ast: Box::new(lit_with('a', span_range(pat, 6..7))),
                }),
                lit_with('a', span_range(pat, 9..10)),
                lit_with(' ', span_range(pat, 10..11)),
            ]
        ))
    );

    // Test that whitespace after an opening paren is insignificant.
    let pat = "(?x)( ?P<foo> a )";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..pat.len()),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                Ast::Group(ast::Group {
                    span: span_range(pat, 4..pat.len()),
                    kind: ast::GroupKind::CaptureName(ast::CaptureName {
                        span: span_range(pat, 9..12),
                        name: s("foo"),
                        index: 1,
                    }),
                    ast: Box::new(lit_with('a', span_range(pat, 14..15))),
                }),
            ]
        ))
    );
    let pat = "(?x)(  a )";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..pat.len()),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                Ast::Group(ast::Group {
                    span: span_range(pat, 4..pat.len()),
                    kind: ast::GroupKind::CaptureIndex(1),
                    ast: Box::new(lit_with('a', span_range(pat, 7..8))),
                }),
            ]
        ))
    );
    let pat = "(?x)(  ?:  a )";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..pat.len()),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                Ast::Group(ast::Group {
                    span: span_range(pat, 4..pat.len()),
                    kind: ast::GroupKind::NonCapturing(ast::Flags {
                        span: span_range(pat, 8..8),
                        items: vec![],
                    }),
                    ast: Box::new(lit_with('a', span_range(pat, 11..12))),
                }),
            ]
        ))
    );
    let pat = r"(?x)\x { 53 }";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..pat.len()),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                Ast::Literal(ast::Literal {
                    span: span(4..13),
                    kind: ast::LiteralKind::HexBrace(ast::HexLiteralKind::X),
                    c: 'S',
                }),
            ]
        ))
    );

    // Test that whitespace after an escape is OK.
    let pat = r"(?x)\ ";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..pat.len()),
            vec![
                flag_set(pat, 0..4, ast::Flag::IgnoreWhitespace, false),
                Ast::Literal(ast::Literal {
                    span: span_range(pat, 4..6),
                    kind: ast::LiteralKind::Special(ast::SpecialLiteralKind::Space),
                    c: ' ',
                }),
            ]
        ))
    );
    // ... but only when `x` mode is enabled.
    let pat = r"\ ";
    assert_eq!(
        parser(pat).parse().unwrap_err(),
        TestError {
            span: span_range(pat, 0..2),
            kind: ast::ErrorKind::EscapeUnrecognized,
        }
    );
}

#[test]
fn parse_newlines() {
    let pat = ".\n.";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..3),
            vec![
                Ast::Dot(span_range(pat, 0..1)),
                lit_with('\n', span_range(pat, 1..2)),
                Ast::Dot(span_range(pat, 2..3)),
            ]
        ))
    );

    let pat = "foobar\nbaz\nquux\n";
    assert_eq!(
        parser(pat).parse(),
        Ok(concat_with(
            span_range(pat, 0..pat.len()),
            vec![
                lit_with('f', nspan(npos(0, 1, 1), npos(1, 1, 2))),
                lit_with('o', nspan(npos(1, 1, 2), npos(2, 1, 3))),
                lit_with('o', nspan(npos(2, 1, 3), npos(3, 1, 4))),
                lit_with('b', nspan(npos(3, 1, 4), npos(4, 1, 5))),
                lit_with('a', nspan(npos(4, 1, 5), npos(5, 1, 6))),
                lit_with('r', nspan(npos(5, 1, 6), npos(6, 1, 7))),
                lit_with('\n', nspan(npos(6, 1, 7), npos(7, 2, 1))),
                lit_with('b', nspan(npos(7, 2, 1), npos(8, 2, 2))),
                lit_with('a', nspan(npos(8, 2, 2), npos(9, 2, 3))),
                lit_with('z', nspan(npos(9, 2, 3), npos(10, 2, 4))),
                lit_with('\n', nspan(npos(10, 2, 4), npos(11, 3, 1))),
                lit_with('q', nspan(npos(11, 3, 1), npos(12, 3, 2))),
                lit_with('u', nspan(npos(12, 3, 2), npos(13, 3, 3))),
                lit_with('u', nspan(npos(13, 3, 3), npos(14, 3, 4))),
                lit_with('x', nspan(npos(14, 3, 4), npos(15, 3, 5))),
                lit_with('\n', nspan(npos(15, 3, 5), npos(16, 4, 1))),
            ]
        ))
    );
}

#[test]
fn parse_uncounted_repetition() {
    assert_eq!(
        parser(r"a*").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..2),
            op: ast::RepetitionOp {
                span: span(1..2),
                kind: ast::RepetitionKind::ZeroOrMore,
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a+").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..2),
            op: ast::RepetitionOp {
                span: span(1..2),
                kind: ast::RepetitionKind::OneOrMore,
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );

    assert_eq!(
        parser(r"a?").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..2),
            op: ast::RepetitionOp {
                span: span(1..2),
                kind: ast::RepetitionKind::ZeroOrOne,
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a??").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..3),
            op: ast::RepetitionOp {
                span: span(1..3),
                kind: ast::RepetitionKind::ZeroOrOne,
            },
            greedy: false,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a?").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..2),
            op: ast::RepetitionOp {
                span: span(1..2),
                kind: ast::RepetitionKind::ZeroOrOne,
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a?b").parse(),
        Ok(concat(
            0..3,
            vec![
                Ast::Repetition(ast::Repetition {
                    span: span(0..2),
                    op: ast::RepetitionOp {
                        span: span(1..2),
                        kind: ast::RepetitionKind::ZeroOrOne,
                    },
                    greedy: true,
                    ast: Box::new(lit('a', 0)),
                }),
                lit('b', 2),
            ]
        ))
    );
    assert_eq!(
        parser(r"a??b").parse(),
        Ok(concat(
            0..4,
            vec![
                Ast::Repetition(ast::Repetition {
                    span: span(0..3),
                    op: ast::RepetitionOp {
                        span: span(1..3),
                        kind: ast::RepetitionKind::ZeroOrOne,
                    },
                    greedy: false,
                    ast: Box::new(lit('a', 0)),
                }),
                lit('b', 3),
            ]
        ))
    );
    assert_eq!(
        parser(r"ab?").parse(),
        Ok(concat(
            0..3,
            vec![
                lit('a', 0),
                Ast::Repetition(ast::Repetition {
                    span: span(1..3),
                    op: ast::RepetitionOp {
                        span: span(2..3),
                        kind: ast::RepetitionKind::ZeroOrOne,
                    },
                    greedy: true,
                    ast: Box::new(lit('b', 1)),
                }),
            ]
        ))
    );
    assert_eq!(
        parser(r"(ab)?").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..5),
            op: ast::RepetitionOp {
                span: span(4..5),
                kind: ast::RepetitionKind::ZeroOrOne,
            },
            greedy: true,
            ast: Box::new(group(
                0..4,
                1,
                concat(1..3, vec![lit('a', 1), lit('b', 2),])
            )),
        }))
    );
    assert_eq!(
        parser(r"|a?").parse(),
        Ok(alt(
            0..3,
            vec![
                Ast::Empty(span(0..0)),
                Ast::Repetition(ast::Repetition {
                    span: span(1..3),
                    op: ast::RepetitionOp {
                        span: span(2..3),
                        kind: ast::RepetitionKind::ZeroOrOne,
                    },
                    greedy: true,
                    ast: Box::new(lit('a', 1)),
                }),
            ]
        ))
    );

    assert_eq!(
        parser(r"*").parse().unwrap_err(),
        TestError {
            span: span(0..0),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"(?i)*").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"(*)").parse().unwrap_err(),
        TestError {
            span: span(1..1),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"(?:?)").parse().unwrap_err(),
        TestError {
            span: span(3..3),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"+").parse().unwrap_err(),
        TestError {
            span: span(0..0),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"?").parse().unwrap_err(),
        TestError {
            span: span(0..0),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"(?)").parse().unwrap_err(),
        TestError {
            span: span(1..1),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"|*").parse().unwrap_err(),
        TestError {
            span: span(1..1),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"|+").parse().unwrap_err(),
        TestError {
            span: span(1..1),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"|?").parse().unwrap_err(),
        TestError {
            span: span(1..1),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
}

#[test]
fn parse_counted_repetition() {
    assert_eq!(
        parser(r"a{5}").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..4),
            op: ast::RepetitionOp {
                span: span(1..4),
                kind: ast::RepetitionKind::Range(ast::RepetitionRange::Exactly(5)),
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a{5,}").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..5),
            op: ast::RepetitionOp {
                span: span(1..5),
                kind: ast::RepetitionKind::Range(ast::RepetitionRange::AtLeast(5)),
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a{5,9}").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..6),
            op: ast::RepetitionOp {
                span: span(1..6),
                kind: ast::RepetitionKind::Range(ast::RepetitionRange::Bounded(5, 9)),
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a{5}?").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..5),
            op: ast::RepetitionOp {
                span: span(1..5),
                kind: ast::RepetitionKind::Range(ast::RepetitionRange::Exactly(5)),
            },
            greedy: false,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"ab{5}").parse(),
        Ok(concat(
            0..5,
            vec![
                lit('a', 0),
                Ast::Repetition(ast::Repetition {
                    span: span(1..5),
                    op: ast::RepetitionOp {
                        span: span(2..5),
                        kind: ast::RepetitionKind::Range(ast::RepetitionRange::Exactly(5)),
                    },
                    greedy: true,
                    ast: Box::new(lit('b', 1)),
                }),
            ]
        ))
    );
    assert_eq!(
        parser(r"ab{5}c").parse(),
        Ok(concat(
            0..6,
            vec![
                lit('a', 0),
                Ast::Repetition(ast::Repetition {
                    span: span(1..5),
                    op: ast::RepetitionOp {
                        span: span(2..5),
                        kind: ast::RepetitionKind::Range(ast::RepetitionRange::Exactly(5)),
                    },
                    greedy: true,
                    ast: Box::new(lit('b', 1)),
                }),
                lit('c', 5),
            ]
        ))
    );

    assert_eq!(
        parser(r"a{ 5 }").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..6),
            op: ast::RepetitionOp {
                span: span(1..6),
                kind: ast::RepetitionKind::Range(ast::RepetitionRange::Exactly(5)),
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser(r"a{ 5 , 9 }").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..10),
            op: ast::RepetitionOp {
                span: span(1..10),
                kind: ast::RepetitionKind::Range(ast::RepetitionRange::Bounded(5, 9)),
            },
            greedy: true,
            ast: Box::new(lit('a', 0)),
        }))
    );
    assert_eq!(
        parser_ignore_whitespace(r"a{5,9} ?").parse(),
        Ok(Ast::Repetition(ast::Repetition {
            span: span(0..8),
            op: ast::RepetitionOp {
                span: span(1..8),
                kind: ast::RepetitionKind::Range(ast::RepetitionRange::Bounded(5, 9)),
            },
            greedy: false,
            ast: Box::new(lit('a', 0)),
        }))
    );

    assert_eq!(
        parser(r"(?i){0}").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"(?m){1,1}").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"a{]}").parse().unwrap_err(),
        TestError {
            span: span(2..2),
            kind: ast::ErrorKind::RepetitionCountDecimalEmpty,
        }
    );
    assert_eq!(
        parser(r"a{1,]}").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::RepetitionCountDecimalEmpty,
        }
    );
    assert_eq!(
        parser(r"a{").parse().unwrap_err(),
        TestError {
            span: span(1..2),
            kind: ast::ErrorKind::RepetitionCountUnclosed,
        }
    );
    assert_eq!(
        parser(r"a{}").parse().unwrap_err(),
        TestError {
            span: span(2..2),
            kind: ast::ErrorKind::RepetitionCountDecimalEmpty,
        }
    );
    assert_eq!(
        parser(r"a{a").parse().unwrap_err(),
        TestError {
            span: span(2..2),
            kind: ast::ErrorKind::RepetitionCountDecimalEmpty,
        }
    );
    assert_eq!(
        parser(r"a{9999999999}").parse().unwrap_err(),
        TestError {
            span: span(2..12),
            kind: ast::ErrorKind::DecimalInvalid,
        }
    );
    assert_eq!(
        parser(r"a{9").parse().unwrap_err(),
        TestError {
            span: span(1..3),
            kind: ast::ErrorKind::RepetitionCountUnclosed,
        }
    );
    assert_eq!(
        parser(r"a{9,a").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::RepetitionCountDecimalEmpty,
        }
    );
    assert_eq!(
        parser(r"a{9,9999999999}").parse().unwrap_err(),
        TestError {
            span: span(4..14),
            kind: ast::ErrorKind::DecimalInvalid,
        }
    );
    assert_eq!(
        parser(r"a{9,").parse().unwrap_err(),
        TestError {
            span: span(1..4),
            kind: ast::ErrorKind::RepetitionCountUnclosed,
        }
    );
    assert_eq!(
        parser(r"a{9,11").parse().unwrap_err(),
        TestError {
            span: span(1..6),
            kind: ast::ErrorKind::RepetitionCountUnclosed,
        }
    );
    assert_eq!(
        parser(r"a{2,1}").parse().unwrap_err(),
        TestError {
            span: span(1..6),
            kind: ast::ErrorKind::RepetitionCountInvalid,
        }
    );
    assert_eq!(
        parser(r"{5}").parse().unwrap_err(),
        TestError {
            span: span(0..0),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
    assert_eq!(
        parser(r"|{5}").parse().unwrap_err(),
        TestError {
            span: span(1..1),
            kind: ast::ErrorKind::RepetitionMissing,
        }
    );
}

#[test]
fn parse_alternate() {
    assert_eq!(
        parser(r"a|b").parse(),
        Ok(Ast::Alternation(ast::Alternation {
            span: span(0..3),
            asts: vec![lit('a', 0), lit('b', 2)],
        }))
    );
    assert_eq!(
        parser(r"(a|b)").parse(),
        Ok(group(
            0..5,
            1,
            Ast::Alternation(ast::Alternation {
                span: span(1..4),
                asts: vec![lit('a', 1), lit('b', 3)],
            })
        ))
    );

    assert_eq!(
        parser(r"a|b|c").parse(),
        Ok(Ast::Alternation(ast::Alternation {
            span: span(0..5),
            asts: vec![lit('a', 0), lit('b', 2), lit('c', 4)],
        }))
    );
    assert_eq!(
        parser(r"ax|by|cz").parse(),
        Ok(Ast::Alternation(ast::Alternation {
            span: span(0..8),
            asts: vec![
                concat(0..2, vec![lit('a', 0), lit('x', 1)]),
                concat(3..5, vec![lit('b', 3), lit('y', 4)]),
                concat(6..8, vec![lit('c', 6), lit('z', 7)]),
            ],
        }))
    );
    assert_eq!(
        parser(r"(ax|by|cz)").parse(),
        Ok(group(
            0..10,
            1,
            Ast::Alternation(ast::Alternation {
                span: span(1..9),
                asts: vec![
                    concat(1..3, vec![lit('a', 1), lit('x', 2)]),
                    concat(4..6, vec![lit('b', 4), lit('y', 5)]),
                    concat(7..9, vec![lit('c', 7), lit('z', 8)]),
                ],
            })
        ))
    );
    assert_eq!(
        parser(r"(ax|(by|(cz)))").parse(),
        Ok(group(
            0..14,
            1,
            alt(
                1..13,
                vec![
                    concat(1..3, vec![lit('a', 1), lit('x', 2)]),
                    group(
                        4..13,
                        2,
                        alt(
                            5..12,
                            vec![
                                concat(5..7, vec![lit('b', 5), lit('y', 6)]),
                                group(8..12, 3, concat(9..11, vec![lit('c', 9), lit('z', 10),])),
                            ]
                        )
                    ),
                ]
            )
        ))
    );

    assert_eq!(
        parser(r"|").parse(),
        Ok(alt(
            0..1,
            vec![Ast::Empty(span(0..0)), Ast::Empty(span(1..1)),]
        ))
    );
    assert_eq!(
        parser(r"||").parse(),
        Ok(alt(
            0..2,
            vec![
                Ast::Empty(span(0..0)),
                Ast::Empty(span(1..1)),
                Ast::Empty(span(2..2)),
            ]
        ))
    );
    assert_eq!(
        parser(r"a|").parse(),
        Ok(alt(0..2, vec![lit('a', 0), Ast::Empty(span(2..2)),]))
    );
    assert_eq!(
        parser(r"|a").parse(),
        Ok(alt(0..2, vec![Ast::Empty(span(0..0)), lit('a', 1),]))
    );

    assert_eq!(
        parser(r"(|)").parse(),
        Ok(group(
            0..3,
            1,
            alt(1..2, vec![Ast::Empty(span(1..1)), Ast::Empty(span(2..2)),])
        ))
    );
    assert_eq!(
        parser(r"(a|)").parse(),
        Ok(group(
            0..4,
            1,
            alt(1..3, vec![lit('a', 1), Ast::Empty(span(3..3)),])
        ))
    );
    assert_eq!(
        parser(r"(|a)").parse(),
        Ok(group(
            0..4,
            1,
            alt(1..3, vec![Ast::Empty(span(1..1)), lit('a', 2),])
        ))
    );

    assert_eq!(
        parser(r"a|b)").parse().unwrap_err(),
        TestError {
            span: span(3..4),
            kind: ast::ErrorKind::GroupUnopened,
        }
    );
    assert_eq!(
        parser(r"(a|b").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::GroupUnclosed,
        }
    );
}

#[test]
fn parse_unsupported_lookaround() {
    assert_eq!(
        parser(r"(?=a)").parse().unwrap_err(),
        TestError {
            span: span(0..3),
            kind: ast::ErrorKind::UnsupportedLookAround,
        }
    );
    assert_eq!(
        parser(r"(?!a)").parse().unwrap_err(),
        TestError {
            span: span(0..3),
            kind: ast::ErrorKind::UnsupportedLookAround,
        }
    );
    assert_eq!(
        parser(r"(?<=a)").parse().unwrap_err(),
        TestError {
            span: span(0..4),
            kind: ast::ErrorKind::UnsupportedLookAround,
        }
    );
    assert_eq!(
        parser(r"(?<!a)").parse().unwrap_err(),
        TestError {
            span: span(0..4),
            kind: ast::ErrorKind::UnsupportedLookAround,
        }
    );
}

#[test]
fn parse_group() {
    assert_eq!(
        parser("(?i)").parse(),
        Ok(Ast::Flags(ast::SetFlags {
            span: span(0..4),
            flags: ast::Flags {
                span: span(2..3),
                items: vec![ast::FlagsItem {
                    span: span(2..3),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                }],
            },
        }))
    );
    assert_eq!(
        parser("(?iU)").parse(),
        Ok(Ast::Flags(ast::SetFlags {
            span: span(0..5),
            flags: ast::Flags {
                span: span(2..4),
                items: vec![
                    ast::FlagsItem {
                        span: span(2..3),
                        kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                    },
                    ast::FlagsItem {
                        span: span(3..4),
                        kind: ast::FlagsItemKind::Flag(ast::Flag::SwapGreed),
                    },
                ],
            },
        }))
    );
    assert_eq!(
        parser("(?i-U)").parse(),
        Ok(Ast::Flags(ast::SetFlags {
            span: span(0..6),
            flags: ast::Flags {
                span: span(2..5),
                items: vec![
                    ast::FlagsItem {
                        span: span(2..3),
                        kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                    },
                    ast::FlagsItem {
                        span: span(3..4),
                        kind: ast::FlagsItemKind::Negation,
                    },
                    ast::FlagsItem {
                        span: span(4..5),
                        kind: ast::FlagsItemKind::Flag(ast::Flag::SwapGreed),
                    },
                ],
            },
        }))
    );

    assert_eq!(
        parser("()").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..2),
            kind: ast::GroupKind::CaptureIndex(1),
            ast: Box::new(Ast::Empty(span(1..1))),
        }))
    );
    assert_eq!(
        parser("(a)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..3),
            kind: ast::GroupKind::CaptureIndex(1),
            ast: Box::new(lit('a', 1)),
        }))
    );
    assert_eq!(
        parser("(())").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..4),
            kind: ast::GroupKind::CaptureIndex(1),
            ast: Box::new(Ast::Group(ast::Group {
                span: span(1..3),
                kind: ast::GroupKind::CaptureIndex(2),
                ast: Box::new(Ast::Empty(span(2..2))),
            })),
        }))
    );

    assert_eq!(
        parser("(?:a)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..5),
            kind: ast::GroupKind::NonCapturing(ast::Flags {
                span: span(2..2),
                items: vec![],
            }),
            ast: Box::new(lit('a', 3)),
        }))
    );

    assert_eq!(
        parser("(?i:a)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..6),
            kind: ast::GroupKind::NonCapturing(ast::Flags {
                span: span(2..3),
                items: vec![ast::FlagsItem {
                    span: span(2..3),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                },],
            }),
            ast: Box::new(lit('a', 4)),
        }))
    );
    assert_eq!(
        parser("(?i-U:a)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..8),
            kind: ast::GroupKind::NonCapturing(ast::Flags {
                span: span(2..5),
                items: vec![
                    ast::FlagsItem {
                        span: span(2..3),
                        kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                    },
                    ast::FlagsItem {
                        span: span(3..4),
                        kind: ast::FlagsItemKind::Negation,
                    },
                    ast::FlagsItem {
                        span: span(4..5),
                        kind: ast::FlagsItemKind::Flag(ast::Flag::SwapGreed),
                    },
                ],
            }),
            ast: Box::new(lit('a', 6)),
        }))
    );

    assert_eq!(
        parser("(").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::GroupUnclosed,
        }
    );
    assert_eq!(
        parser("(?").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::GroupUnclosed,
        }
    );
    assert_eq!(
        parser("(?P").parse().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::FlagUnrecognized,
        }
    );
    assert_eq!(
        parser("(?P<").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::GroupNameUnexpectedEof,
        }
    );
    assert_eq!(
        parser("(a").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::GroupUnclosed,
        }
    );
    assert_eq!(
        parser("(()").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::GroupUnclosed,
        }
    );
    assert_eq!(
        parser(")").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::GroupUnopened,
        }
    );
    assert_eq!(
        parser("a)").parse().unwrap_err(),
        TestError {
            span: span(1..2),
            kind: ast::ErrorKind::GroupUnopened,
        }
    );
}

#[test]
fn parse_capture_name() {
    assert_eq!(
        parser("(?P<a>z)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..8),
            kind: ast::GroupKind::CaptureName(ast::CaptureName {
                span: span(4..5),
                name: s("a"),
                index: 1,
            }),
            ast: Box::new(lit('z', 6)),
        }))
    );
    assert_eq!(
        parser("(?P<abc>z)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..10),
            kind: ast::GroupKind::CaptureName(ast::CaptureName {
                span: span(4..7),
                name: s("abc"),
                index: 1,
            }),
            ast: Box::new(lit('z', 8)),
        }))
    );

    assert_eq!(
        parser("(?P<a_1>z)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..10),
            kind: ast::GroupKind::CaptureName(ast::CaptureName {
                span: span(4..7),
                name: s("a_1"),
                index: 1,
            }),
            ast: Box::new(lit('z', 8)),
        }))
    );

    assert_eq!(
        parser("(?P<a.1>z)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..10),
            kind: ast::GroupKind::CaptureName(ast::CaptureName {
                span: span(4..7),
                name: s("a.1"),
                index: 1,
            }),
            ast: Box::new(lit('z', 8)),
        }))
    );

    assert_eq!(
        parser("(?P<a[1]>z)").parse(),
        Ok(Ast::Group(ast::Group {
            span: span(0..11),
            kind: ast::GroupKind::CaptureName(ast::CaptureName {
                span: span(4..8),
                name: s("a[1]"),
                index: 1,
            }),
            ast: Box::new(lit('z', 9)),
        }))
    );

    assert_eq!(
        parser("(?P<").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::GroupNameUnexpectedEof,
        }
    );
    assert_eq!(
        parser("(?P<>z)").parse().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::GroupNameEmpty,
        }
    );
    assert_eq!(
        parser("(?P<a").parse().unwrap_err(),
        TestError {
            span: span(5..5),
            kind: ast::ErrorKind::GroupNameUnexpectedEof,
        }
    );
    assert_eq!(
        parser("(?P<ab").parse().unwrap_err(),
        TestError {
            span: span(6..6),
            kind: ast::ErrorKind::GroupNameUnexpectedEof,
        }
    );
    assert_eq!(
        parser("(?P<0a").parse().unwrap_err(),
        TestError {
            span: span(4..5),
            kind: ast::ErrorKind::GroupNameInvalid,
        }
    );
    assert_eq!(
        parser("(?P<~").parse().unwrap_err(),
        TestError {
            span: span(4..5),
            kind: ast::ErrorKind::GroupNameInvalid,
        }
    );
    assert_eq!(
        parser("(?P<abc~").parse().unwrap_err(),
        TestError {
            span: span(7..8),
            kind: ast::ErrorKind::GroupNameInvalid,
        }
    );
    assert_eq!(
        parser("(?P<a>y)(?P<a>z)").parse().unwrap_err(),
        TestError {
            span: span(12..13),
            kind: ast::ErrorKind::GroupNameDuplicate {
                original: span(4..5),
            },
        }
    );
}

#[test]
fn parse_flags() {
    assert_eq!(
        parser("i:").parse_flags(),
        Ok(ast::Flags {
            span: span(0..1),
            items: vec![ast::FlagsItem {
                span: span(0..1),
                kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
            }],
        })
    );
    assert_eq!(
        parser("i)").parse_flags(),
        Ok(ast::Flags {
            span: span(0..1),
            items: vec![ast::FlagsItem {
                span: span(0..1),
                kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
            }],
        })
    );

    assert_eq!(
        parser("isU:").parse_flags(),
        Ok(ast::Flags {
            span: span(0..3),
            items: vec![
                ast::FlagsItem {
                    span: span(0..1),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                },
                ast::FlagsItem {
                    span: span(1..2),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::DotMatchesNewLine),
                },
                ast::FlagsItem {
                    span: span(2..3),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::SwapGreed),
                },
            ],
        })
    );

    assert_eq!(
        parser("-isU:").parse_flags(),
        Ok(ast::Flags {
            span: span(0..4),
            items: vec![
                ast::FlagsItem {
                    span: span(0..1),
                    kind: ast::FlagsItemKind::Negation,
                },
                ast::FlagsItem {
                    span: span(1..2),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                },
                ast::FlagsItem {
                    span: span(2..3),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::DotMatchesNewLine),
                },
                ast::FlagsItem {
                    span: span(3..4),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::SwapGreed),
                },
            ],
        })
    );
    assert_eq!(
        parser("i-sU:").parse_flags(),
        Ok(ast::Flags {
            span: span(0..4),
            items: vec![
                ast::FlagsItem {
                    span: span(0..1),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::CaseInsensitive),
                },
                ast::FlagsItem {
                    span: span(1..2),
                    kind: ast::FlagsItemKind::Negation,
                },
                ast::FlagsItem {
                    span: span(2..3),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::DotMatchesNewLine),
                },
                ast::FlagsItem {
                    span: span(3..4),
                    kind: ast::FlagsItemKind::Flag(ast::Flag::SwapGreed),
                },
            ],
        })
    );

    assert_eq!(
        parser("isU").parse_flags().unwrap_err(),
        TestError {
            span: span(3..3),
            kind: ast::ErrorKind::FlagUnexpectedEof,
        }
    );
    assert_eq!(
        parser("isUa:").parse_flags().unwrap_err(),
        TestError {
            span: span(3..4),
            kind: ast::ErrorKind::FlagUnrecognized,
        }
    );
    assert_eq!(
        parser("isUi:").parse_flags().unwrap_err(),
        TestError {
            span: span(3..4),
            kind: ast::ErrorKind::FlagDuplicate {
                original: span(0..1)
            },
        }
    );
    assert_eq!(
        parser("i-sU-i:").parse_flags().unwrap_err(),
        TestError {
            span: span(4..5),
            kind: ast::ErrorKind::FlagRepeatedNegation {
                original: span(1..2),
            },
        }
    );
    assert_eq!(
        parser("-)").parse_flags().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::FlagDanglingNegation,
        }
    );
    assert_eq!(
        parser("i-)").parse_flags().unwrap_err(),
        TestError {
            span: span(1..2),
            kind: ast::ErrorKind::FlagDanglingNegation,
        }
    );
    assert_eq!(
        parser("iU-)").parse_flags().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::FlagDanglingNegation,
        }
    );
}

#[test]
fn parse_flag() {
    assert_eq!(parser("i").parse_flag(), Ok(ast::Flag::CaseInsensitive));
    assert_eq!(parser("m").parse_flag(), Ok(ast::Flag::MultiLine));
    assert_eq!(parser("s").parse_flag(), Ok(ast::Flag::DotMatchesNewLine));
    assert_eq!(parser("U").parse_flag(), Ok(ast::Flag::SwapGreed));
    assert_eq!(parser("u").parse_flag(), Ok(ast::Flag::Unicode));
    assert_eq!(parser("x").parse_flag(), Ok(ast::Flag::IgnoreWhitespace));

    assert_eq!(
        parser("a").parse_flag().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::FlagUnrecognized,
        }
    );
    assert_eq!(
        parser("☃").parse_flag().unwrap_err(),
        TestError {
            span: span_range("☃", 0..3),
            kind: ast::ErrorKind::FlagUnrecognized,
        }
    );
}

#[test]
fn parse_primitive_non_escape() {
    assert_eq!(
        parser(r".").parse_primitive(),
        Ok(Primitive::Dot(span(0..1)))
    );
    assert_eq!(
        parser(r"^").parse_primitive(),
        Ok(Primitive::Assertion(ast::Assertion {
            span: span(0..1),
            kind: ast::AssertionKind::StartLine,
        }))
    );
    assert_eq!(
        parser(r"$").parse_primitive(),
        Ok(Primitive::Assertion(ast::Assertion {
            span: span(0..1),
            kind: ast::AssertionKind::EndLine,
        }))
    );

    assert_eq!(
        parser(r"a").parse_primitive(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..1),
            kind: ast::LiteralKind::Verbatim,
            c: 'a',
        }))
    );
    assert_eq!(
        parser(r"|").parse_primitive(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..1),
            kind: ast::LiteralKind::Verbatim,
            c: '|',
        }))
    );
    assert_eq!(
        parser(r"☃").parse_primitive(),
        Ok(Primitive::Literal(ast::Literal {
            span: span_range("☃", 0..3),
            kind: ast::LiteralKind::Verbatim,
            c: '☃',
        }))
    );
}

#[test]
fn parse_escape() {
    assert_eq!(
        parser(r"\|").parse_primitive(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..2),
            kind: ast::LiteralKind::Punctuation,
            c: '|',
        }))
    );
    let specials = &[
        (r"\a", '\x07', ast::SpecialLiteralKind::Bell),
        (r"\f", '\x0C', ast::SpecialLiteralKind::FormFeed),
        (r"\t", '\t', ast::SpecialLiteralKind::Tab),
        (r"\n", '\n', ast::SpecialLiteralKind::LineFeed),
        (r"\r", '\r', ast::SpecialLiteralKind::CarriageReturn),
        (r"\v", '\x0B', ast::SpecialLiteralKind::VerticalTab),
    ];
    for &(pat, c, ref kind) in specials {
        assert_eq!(
            parser(pat).parse_primitive(),
            Ok(Primitive::Literal(ast::Literal {
                span: span(0..2),
                kind: ast::LiteralKind::Special(kind.clone()),
                c,
            }))
        );
    }
    assert_eq!(
        parser(r"\A").parse_primitive(),
        Ok(Primitive::Assertion(ast::Assertion {
            span: span(0..2),
            kind: ast::AssertionKind::StartText,
        }))
    );
    assert_eq!(
        parser(r"\z").parse_primitive(),
        Ok(Primitive::Assertion(ast::Assertion {
            span: span(0..2),
            kind: ast::AssertionKind::EndText,
        }))
    );
    assert_eq!(
        parser(r"\b").parse_primitive(),
        Ok(Primitive::Assertion(ast::Assertion {
            span: span(0..2),
            kind: ast::AssertionKind::WordBoundary,
        }))
    );
    assert_eq!(
        parser(r"\B").parse_primitive(),
        Ok(Primitive::Assertion(ast::Assertion {
            span: span(0..2),
            kind: ast::AssertionKind::NotWordBoundary,
        }))
    );

    assert_eq!(
        parser(r"\").parse_escape().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\y").parse_escape().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::EscapeUnrecognized,
        }
    );
}

#[test]
fn parse_unsupported_backreference() {
    assert_eq!(
        parser(r"\0").parse_escape().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::UnsupportedBackreference,
        }
    );
    assert_eq!(
        parser(r"\9").parse_escape().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::UnsupportedBackreference,
        }
    );
}

#[test]
fn parse_octal() {
    for i in 0..511 {
        let pat = format!(r"\{:o}", i);
        assert_eq!(
            parser_octal(&pat).parse_escape(),
            Ok(Primitive::Literal(ast::Literal {
                span: span(0..pat.len()),
                kind: ast::LiteralKind::Octal,
                c: ::std::char::from_u32(i).unwrap(),
            }))
        );
    }
    assert_eq!(
        parser_octal(r"\778").parse_escape(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..3),
            kind: ast::LiteralKind::Octal,
            c: '?',
        }))
    );
    assert_eq!(
        parser_octal(r"\7777").parse_escape(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..4),
            kind: ast::LiteralKind::Octal,
            c: '\u{01FF}',
        }))
    );
    assert_eq!(
        parser_octal(r"\778").parse(),
        Ok(Ast::Concat(ast::Concat {
            span: span(0..4),
            asts: vec![
                Ast::Literal(ast::Literal {
                    span: span(0..3),
                    kind: ast::LiteralKind::Octal,
                    c: '?',
                }),
                Ast::Literal(ast::Literal {
                    span: span(3..4),
                    kind: ast::LiteralKind::Verbatim,
                    c: '8',
                }),
            ],
        }))
    );
    assert_eq!(
        parser_octal(r"\7777").parse(),
        Ok(Ast::Concat(ast::Concat {
            span: span(0..5),
            asts: vec![
                Ast::Literal(ast::Literal {
                    span: span(0..4),
                    kind: ast::LiteralKind::Octal,
                    c: '\u{01FF}',
                }),
                Ast::Literal(ast::Literal {
                    span: span(4..5),
                    kind: ast::LiteralKind::Verbatim,
                    c: '7',
                }),
            ],
        }))
    );

    assert_eq!(
        parser_octal(r"\8").parse_escape().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::EscapeUnrecognized,
        }
    );
}

#[test]
fn parse_hex_two() {
    for i in 0..256 {
        let pat = format!(r"\x{:02x}", i);
        assert_eq!(
            parser(&pat).parse_escape(),
            Ok(Primitive::Literal(ast::Literal {
                span: span(0..pat.len()),
                kind: ast::LiteralKind::HexFixed(ast::HexLiteralKind::X),
                c: ::std::char::from_u32(i).unwrap(),
            }))
        );
    }

    assert_eq!(
        parser(r"\xF").parse_escape().unwrap_err(),
        TestError {
            span: span(3..3),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\xG").parse_escape().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\xFG").parse_escape().unwrap_err(),
        TestError {
            span: span(3..4),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
}

#[test]
fn parse_hex_four() {
    for i in 0..65536 {
        let c = match ::std::char::from_u32(i) {
            None => continue,
            Some(c) => c,
        };
        let pat = format!(r"\u{:04x}", i);
        assert_eq!(
            parser(&pat).parse_escape(),
            Ok(Primitive::Literal(ast::Literal {
                span: span(0..pat.len()),
                kind: ast::LiteralKind::HexFixed(ast::HexLiteralKind::UnicodeShort),
                c,
            }))
        );
    }

    assert_eq!(
        parser(r"\uF").parse_escape().unwrap_err(),
        TestError {
            span: span(3..3),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\uG").parse_escape().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\uFG").parse_escape().unwrap_err(),
        TestError {
            span: span(3..4),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\uFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(4..5),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\uFFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(5..6),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\uD800").parse_escape().unwrap_err(),
        TestError {
            span: span(2..6),
            kind: ast::ErrorKind::EscapeHexInvalid,
        }
    );
}

#[test]
fn parse_hex_eight() {
    for i in 0..65536 {
        let c = match ::std::char::from_u32(i) {
            None => continue,
            Some(c) => c,
        };
        let pat = format!(r"\U{:08x}", i);
        assert_eq!(
            parser(&pat).parse_escape(),
            Ok(Primitive::Literal(ast::Literal {
                span: span(0..pat.len()),
                kind: ast::LiteralKind::HexFixed(ast::HexLiteralKind::UnicodeLong),
                c,
            }))
        );
    }

    assert_eq!(
        parser(r"\UF").parse_escape().unwrap_err(),
        TestError {
            span: span(3..3),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\UG").parse_escape().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\UFG").parse_escape().unwrap_err(),
        TestError {
            span: span(3..4),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\UFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(4..5),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\UFFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(5..6),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\UFFFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(6..7),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\UFFFFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(7..8),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\UFFFFFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(8..9),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\UFFFFFFFG").parse_escape().unwrap_err(),
        TestError {
            span: span(9..10),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
}

#[test]
fn parse_hex_brace() {
    assert_eq!(
        parser(r"\u{26c4}").parse_escape(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..8),
            kind: ast::LiteralKind::HexBrace(ast::HexLiteralKind::UnicodeShort),
            c: '⛄',
        }))
    );
    assert_eq!(
        parser(r"\U{26c4}").parse_escape(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..8),
            kind: ast::LiteralKind::HexBrace(ast::HexLiteralKind::UnicodeLong),
            c: '⛄',
        }))
    );
    assert_eq!(
        parser(r"\x{26c4}").parse_escape(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..8),
            kind: ast::LiteralKind::HexBrace(ast::HexLiteralKind::X),
            c: '⛄',
        }))
    );
    assert_eq!(
        parser(r"\x{26C4}").parse_escape(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..8),
            kind: ast::LiteralKind::HexBrace(ast::HexLiteralKind::X),
            c: '⛄',
        }))
    );
    assert_eq!(
        parser(r"\x{10fFfF}").parse_escape(),
        Ok(Primitive::Literal(ast::Literal {
            span: span(0..10),
            kind: ast::LiteralKind::HexBrace(ast::HexLiteralKind::X),
            c: '\u{10FFFF}',
        }))
    );

    assert_eq!(
        parser(r"\x").parse_escape().unwrap_err(),
        TestError {
            span: span(2..2),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\x{").parse_escape().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\x{FF").parse_escape().unwrap_err(),
        TestError {
            span: span(2..5),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\x{}").parse_escape().unwrap_err(),
        TestError {
            span: span(2..4),
            kind: ast::ErrorKind::EscapeHexEmpty,
        }
    );
    assert_eq!(
        parser(r"\x{FGF}").parse_escape().unwrap_err(),
        TestError {
            span: span(4..5),
            kind: ast::ErrorKind::EscapeHexInvalidDigit,
        }
    );
    assert_eq!(
        parser(r"\x{FFFFFF}").parse_escape().unwrap_err(),
        TestError {
            span: span(3..9),
            kind: ast::ErrorKind::EscapeHexInvalid,
        }
    );
    assert_eq!(
        parser(r"\x{D800}").parse_escape().unwrap_err(),
        TestError {
            span: span(3..7),
            kind: ast::ErrorKind::EscapeHexInvalid,
        }
    );
    assert_eq!(
        parser(r"\x{FFFFFFFFF}").parse_escape().unwrap_err(),
        TestError {
            span: span(3..12),
            kind: ast::ErrorKind::EscapeHexInvalid,
        }
    );
}

#[test]
fn parse_decimal() {
    assert_eq!(parser("123").parse_decimal(), Ok(123));
    assert_eq!(parser("0").parse_decimal(), Ok(0));
    assert_eq!(parser("01").parse_decimal(), Ok(1));

    assert_eq!(
        parser("-1").parse_decimal().unwrap_err(),
        TestError {
            span: span(0..0),
            kind: ast::ErrorKind::DecimalEmpty
        }
    );
    assert_eq!(
        parser("").parse_decimal().unwrap_err(),
        TestError {
            span: span(0..0),
            kind: ast::ErrorKind::DecimalEmpty
        }
    );
    assert_eq!(
        parser("9999999999").parse_decimal().unwrap_err(),
        TestError {
            span: span(0..10),
            kind: ast::ErrorKind::DecimalInvalid,
        }
    );
}

#[test]
fn parse_set_class() {
    fn union(span: Span, items: Vec<ast::ClassSetItem>) -> ast::ClassSet {
        ast::ClassSet::union(ast::ClassSetUnion { span, items })
    }

    fn intersection(span: Span, lhs: ast::ClassSet, rhs: ast::ClassSet) -> ast::ClassSet {
        ast::ClassSet::BinaryOp(ast::ClassSetBinaryOp {
            span,
            kind: ast::ClassSetBinaryOpKind::Intersection,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn difference(span: Span, lhs: ast::ClassSet, rhs: ast::ClassSet) -> ast::ClassSet {
        ast::ClassSet::BinaryOp(ast::ClassSetBinaryOp {
            span,
            kind: ast::ClassSetBinaryOpKind::Difference,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn symdifference(span: Span, lhs: ast::ClassSet, rhs: ast::ClassSet) -> ast::ClassSet {
        ast::ClassSet::BinaryOp(ast::ClassSetBinaryOp {
            span,
            kind: ast::ClassSetBinaryOpKind::SymmetricDifference,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        })
    }

    fn itemset(item: ast::ClassSetItem) -> ast::ClassSet {
        ast::ClassSet::Item(item)
    }

    fn item_ascii(cls: ast::ClassAscii) -> ast::ClassSetItem {
        ast::ClassSetItem::Ascii(cls)
    }

    fn item_unicode(cls: ast::ClassUnicode) -> ast::ClassSetItem {
        ast::ClassSetItem::Unicode(cls)
    }

    fn item_perl(cls: ast::ClassPerl) -> ast::ClassSetItem {
        ast::ClassSetItem::Perl(cls)
    }

    fn item_bracket(cls: ast::ClassBracketed) -> ast::ClassSetItem {
        ast::ClassSetItem::Bracketed(Box::new(cls))
    }

    fn lit(span: Span, c: char) -> ast::ClassSetItem {
        ast::ClassSetItem::Literal(ast::Literal {
            span,
            kind: ast::LiteralKind::Verbatim,
            c,
        })
    }

    fn empty(span: Span) -> ast::ClassSetItem {
        ast::ClassSetItem::Empty(span)
    }

    fn range(span: Span, start: char, end: char) -> ast::ClassSetItem {
        let pos1 = Position {
            offset: span.start.offset + start.len_utf8(),
            column: span.start.column + 1,
            ..span.start
        };
        let pos2 = Position {
            offset: span.end.offset - end.len_utf8(),
            column: span.end.column - 1,
            ..span.end
        };
        ast::ClassSetItem::Range(ast::ClassSetRange {
            span,
            start: ast::Literal {
                span: Span { end: pos1, ..span },
                kind: ast::LiteralKind::Verbatim,
                c: start,
            },
            end: ast::Literal {
                span: Span {
                    start: pos2,
                    ..span
                },
                kind: ast::LiteralKind::Verbatim,
                c: end,
            },
        })
    }

    fn alnum(span: Span, negated: bool) -> ast::ClassAscii {
        ast::ClassAscii {
            span,
            kind: ast::ClassAsciiKind::Alnum,
            negated,
        }
    }

    fn lower(span: Span, negated: bool) -> ast::ClassAscii {
        ast::ClassAscii {
            span,
            kind: ast::ClassAsciiKind::Lower,
            negated,
        }
    }

    assert_eq!(
        parser("[[:alnum:]]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..11),
            negated: false,
            kind: itemset(item_ascii(alnum(span(1..10), false))),
        })))
    );
    assert_eq!(
        parser("[[[:alnum:]]]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..13),
            negated: false,
            kind: itemset(item_bracket(ast::ClassBracketed {
                span: span(1..12),
                negated: false,
                kind: itemset(item_ascii(alnum(span(2..11), false))),
            })),
        })))
    );
    assert_eq!(
        parser("[[:alnum:]&&[:lower:]]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..22),
            negated: false,
            kind: intersection(
                span(1..21),
                itemset(item_ascii(alnum(span(1..10), false))),
                itemset(item_ascii(lower(span(12..21), false))),
            ),
        })))
    );
    assert_eq!(
        parser("[[:alnum:]--[:lower:]]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..22),
            negated: false,
            kind: difference(
                span(1..21),
                itemset(item_ascii(alnum(span(1..10), false))),
                itemset(item_ascii(lower(span(12..21), false))),
            ),
        })))
    );
    assert_eq!(
        parser("[[:alnum:]~~[:lower:]]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..22),
            negated: false,
            kind: symdifference(
                span(1..21),
                itemset(item_ascii(alnum(span(1..10), false))),
                itemset(item_ascii(lower(span(12..21), false))),
            ),
        })))
    );

    assert_eq!(
        parser("[a]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..3),
            negated: false,
            kind: itemset(lit(span(1..2), 'a')),
        })))
    );
    assert_eq!(
        parser(r"[a\]]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..5),
            negated: false,
            kind: union(
                span(1..4),
                vec![
                    lit(span(1..2), 'a'),
                    ast::ClassSetItem::Literal(ast::Literal {
                        span: span(2..4),
                        kind: ast::LiteralKind::Punctuation,
                        c: ']',
                    }),
                ]
            ),
        })))
    );
    assert_eq!(
        parser(r"[a\-z]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..6),
            negated: false,
            kind: union(
                span(1..5),
                vec![
                    lit(span(1..2), 'a'),
                    ast::ClassSetItem::Literal(ast::Literal {
                        span: span(2..4),
                        kind: ast::LiteralKind::Punctuation,
                        c: '-',
                    }),
                    lit(span(4..5), 'z'),
                ]
            ),
        })))
    );
    assert_eq!(
        parser("[ab]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..4),
            negated: false,
            kind: union(
                span(1..3),
                vec![lit(span(1..2), 'a'), lit(span(2..3), 'b'),]
            ),
        })))
    );
    assert_eq!(
        parser("[a-]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..4),
            negated: false,
            kind: union(
                span(1..3),
                vec![lit(span(1..2), 'a'), lit(span(2..3), '-'),]
            ),
        })))
    );
    assert_eq!(
        parser("[-a]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..4),
            negated: false,
            kind: union(
                span(1..3),
                vec![lit(span(1..2), '-'), lit(span(2..3), 'a'),]
            ),
        })))
    );
    assert_eq!(
        parser(r"[\pL]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..5),
            negated: false,
            kind: itemset(item_unicode(ast::ClassUnicode {
                span: span(1..4),
                negated: false,
                kind: ast::ClassUnicodeKind::OneLetter('L'),
            })),
        })))
    );
    assert_eq!(
        parser(r"[\w]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..4),
            negated: false,
            kind: itemset(item_perl(ast::ClassPerl {
                span: span(1..3),
                kind: ast::ClassPerlKind::Word,
                negated: false,
            })),
        })))
    );
    assert_eq!(
        parser(r"[a\wz]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..6),
            negated: false,
            kind: union(
                span(1..5),
                vec![
                    lit(span(1..2), 'a'),
                    item_perl(ast::ClassPerl {
                        span: span(2..4),
                        kind: ast::ClassPerlKind::Word,
                        negated: false,
                    }),
                    lit(span(4..5), 'z'),
                ]
            ),
        })))
    );

    assert_eq!(
        parser("[a-z]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..5),
            negated: false,
            kind: itemset(range(span(1..4), 'a', 'z')),
        })))
    );
    assert_eq!(
        parser("[a-cx-z]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..8),
            negated: false,
            kind: union(
                span(1..7),
                vec![range(span(1..4), 'a', 'c'), range(span(4..7), 'x', 'z'),]
            ),
        })))
    );
    assert_eq!(
        parser(r"[\w&&a-cx-z]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..12),
            negated: false,
            kind: intersection(
                span(1..11),
                itemset(item_perl(ast::ClassPerl {
                    span: span(1..3),
                    kind: ast::ClassPerlKind::Word,
                    negated: false,
                })),
                union(
                    span(5..11),
                    vec![range(span(5..8), 'a', 'c'), range(span(8..11), 'x', 'z'),]
                ),
            ),
        })))
    );
    assert_eq!(
        parser(r"[a-cx-z&&\w]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..12),
            negated: false,
            kind: intersection(
                span(1..11),
                union(
                    span(1..7),
                    vec![range(span(1..4), 'a', 'c'), range(span(4..7), 'x', 'z'),]
                ),
                itemset(item_perl(ast::ClassPerl {
                    span: span(9..11),
                    kind: ast::ClassPerlKind::Word,
                    negated: false,
                })),
            ),
        })))
    );
    assert_eq!(
        parser(r"[a--b--c]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..9),
            negated: false,
            kind: difference(
                span(1..8),
                difference(
                    span(1..5),
                    itemset(lit(span(1..2), 'a')),
                    itemset(lit(span(4..5), 'b')),
                ),
                itemset(lit(span(7..8), 'c')),
            ),
        })))
    );
    assert_eq!(
        parser(r"[a~~b~~c]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..9),
            negated: false,
            kind: symdifference(
                span(1..8),
                symdifference(
                    span(1..5),
                    itemset(lit(span(1..2), 'a')),
                    itemset(lit(span(4..5), 'b')),
                ),
                itemset(lit(span(7..8), 'c')),
            ),
        })))
    );
    assert_eq!(
        parser(r"[\^&&^]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..7),
            negated: false,
            kind: intersection(
                span(1..6),
                itemset(ast::ClassSetItem::Literal(ast::Literal {
                    span: span(1..3),
                    kind: ast::LiteralKind::Punctuation,
                    c: '^',
                })),
                itemset(lit(span(5..6), '^')),
            ),
        })))
    );
    assert_eq!(
        parser(r"[\&&&&]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..7),
            negated: false,
            kind: intersection(
                span(1..6),
                itemset(ast::ClassSetItem::Literal(ast::Literal {
                    span: span(1..3),
                    kind: ast::LiteralKind::Punctuation,
                    c: '&',
                })),
                itemset(lit(span(5..6), '&')),
            ),
        })))
    );
    assert_eq!(
        parser(r"[&&&&]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..6),
            negated: false,
            kind: intersection(
                span(1..5),
                intersection(
                    span(1..3),
                    itemset(empty(span(1..1))),
                    itemset(empty(span(3..3))),
                ),
                itemset(empty(span(5..5))),
            ),
        })))
    );

    let pat = "[☃-⛄]";
    assert_eq!(
        parser(pat).parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span_range(pat, 0..9),
            negated: false,
            kind: itemset(ast::ClassSetItem::Range(ast::ClassSetRange {
                span: span_range(pat, 1..8),
                start: ast::Literal {
                    span: span_range(pat, 1..4),
                    kind: ast::LiteralKind::Verbatim,
                    c: '☃',
                },
                end: ast::Literal {
                    span: span_range(pat, 5..8),
                    kind: ast::LiteralKind::Verbatim,
                    c: '⛄',
                },
            })),
        })))
    );

    assert_eq!(
        parser(r"[]]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..3),
            negated: false,
            kind: itemset(lit(span(1..2), ']')),
        })))
    );
    assert_eq!(
        parser(r"[]\[]").parse(),
        Ok(Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
            span: span(0..5),
            negated: false,
            kind: union(
                span(1..4),
                vec![
                    lit(span(1..2), ']'),
                    ast::ClassSetItem::Literal(ast::Literal {
                        span: span(2..4),
                        kind: ast::LiteralKind::Punctuation,
                        c: '[',
                    }),
                ]
            ),
        })))
    );
    assert_eq!(
        parser(r"[\[]]").parse(),
        Ok(concat(
            0..5,
            vec![
                Ast::Class(ast::Class::Bracketed(ast::ClassBracketed {
                    span: span(0..4),
                    negated: false,
                    kind: itemset(ast::ClassSetItem::Literal(ast::Literal {
                        span: span(1..3),
                        kind: ast::LiteralKind::Punctuation,
                        c: '[',
                    })),
                })),
                Ast::Literal(ast::Literal {
                    span: span(4..5),
                    kind: ast::LiteralKind::Verbatim,
                    c: ']',
                }),
            ]
        ))
    );

    assert_eq!(
        parser("[").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser("[[").parse().unwrap_err(),
        TestError {
            span: span(1..2),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser("[[-]").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser("[[[:alnum:]").parse().unwrap_err(),
        TestError {
            span: span(1..2),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser(r"[\b]").parse().unwrap_err(),
        TestError {
            span: span(1..3),
            kind: ast::ErrorKind::ClassEscapeInvalid,
        }
    );
    assert_eq!(
        parser(r"[\w-a]").parse().unwrap_err(),
        TestError {
            span: span(1..3),
            kind: ast::ErrorKind::ClassRangeLiteral,
        }
    );
    assert_eq!(
        parser(r"[a-\w]").parse().unwrap_err(),
        TestError {
            span: span(3..5),
            kind: ast::ErrorKind::ClassRangeLiteral,
        }
    );
    assert_eq!(
        parser(r"[z-a]").parse().unwrap_err(),
        TestError {
            span: span(1..4),
            kind: ast::ErrorKind::ClassRangeInvalid,
        }
    );

    assert_eq!(
        parser_ignore_whitespace("[a ").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser_ignore_whitespace("[a- ").parse().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
}

#[test]
fn parse_set_class_open() {
    assert_eq!(parser("[a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..1),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(1..1),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(1..1),
            items: vec![],
        };
        Ok((set, union))
    });
    assert_eq!(parser_ignore_whitespace("[   a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..4),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(4..4),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(4..4),
            items: vec![],
        };
        Ok((set, union))
    });
    assert_eq!(parser("[^a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..2),
            negated: true,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(2..2),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(2..2),
            items: vec![],
        };
        Ok((set, union))
    });
    assert_eq!(parser_ignore_whitespace("[ ^ a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..4),
            negated: true,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(4..4),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(4..4),
            items: vec![],
        };
        Ok((set, union))
    });
    assert_eq!(parser("[-a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..2),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(1..1),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(1..2),
            items: vec![ast::ClassSetItem::Literal(ast::Literal {
                span: span(1..2),
                kind: ast::LiteralKind::Verbatim,
                c: '-',
            })],
        };
        Ok((set, union))
    });
    assert_eq!(parser_ignore_whitespace("[ - a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..4),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(2..2),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(2..3),
            items: vec![ast::ClassSetItem::Literal(ast::Literal {
                span: span(2..3),
                kind: ast::LiteralKind::Verbatim,
                c: '-',
            })],
        };
        Ok((set, union))
    });
    assert_eq!(parser("[^-a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..3),
            negated: true,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(2..2),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(2..3),
            items: vec![ast::ClassSetItem::Literal(ast::Literal {
                span: span(2..3),
                kind: ast::LiteralKind::Verbatim,
                c: '-',
            })],
        };
        Ok((set, union))
    });
    assert_eq!(parser("[--a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..3),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(1..1),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(1..3),
            items: vec![
                ast::ClassSetItem::Literal(ast::Literal {
                    span: span(1..2),
                    kind: ast::LiteralKind::Verbatim,
                    c: '-',
                }),
                ast::ClassSetItem::Literal(ast::Literal {
                    span: span(2..3),
                    kind: ast::LiteralKind::Verbatim,
                    c: '-',
                }),
            ],
        };
        Ok((set, union))
    });
    assert_eq!(parser("[]a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..2),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(1..1),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(1..2),
            items: vec![ast::ClassSetItem::Literal(ast::Literal {
                span: span(1..2),
                kind: ast::LiteralKind::Verbatim,
                c: ']',
            })],
        };
        Ok((set, union))
    });
    assert_eq!(parser_ignore_whitespace("[ ] a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..4),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(2..2),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(2..3),
            items: vec![ast::ClassSetItem::Literal(ast::Literal {
                span: span(2..3),
                kind: ast::LiteralKind::Verbatim,
                c: ']',
            })],
        };
        Ok((set, union))
    });
    assert_eq!(parser("[^]a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..3),
            negated: true,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(2..2),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(2..3),
            items: vec![ast::ClassSetItem::Literal(ast::Literal {
                span: span(2..3),
                kind: ast::LiteralKind::Verbatim,
                c: ']',
            })],
        };
        Ok((set, union))
    });
    assert_eq!(parser("[-]a]").parse_set_class_open(), {
        let set = ast::ClassBracketed {
            span: span(0..2),
            negated: false,
            kind: ast::ClassSet::union(ast::ClassSetUnion {
                span: span(1..1),
                items: vec![],
            }),
        };
        let union = ast::ClassSetUnion {
            span: span(1..2),
            items: vec![ast::ClassSetItem::Literal(ast::Literal {
                span: span(1..2),
                kind: ast::LiteralKind::Verbatim,
                c: '-',
            })],
        };
        Ok((set, union))
    });

    assert_eq!(
        parser("[").parse_set_class_open().unwrap_err(),
        TestError {
            span: span(0..1),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser_ignore_whitespace("[    ")
            .parse_set_class_open()
            .unwrap_err(),
        TestError {
            span: span(0..5),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser("[^").parse_set_class_open().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser("[]").parse_set_class_open().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser("[-").parse_set_class_open().unwrap_err(),
        TestError {
            span: span(0..2),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
    assert_eq!(
        parser("[--").parse_set_class_open().unwrap_err(),
        TestError {
            span: span(0..3),
            kind: ast::ErrorKind::ClassUnclosed,
        }
    );
}

#[test]
fn maybe_parse_ascii_class() {
    assert_eq!(
        parser(r"[:alnum:]").maybe_parse_ascii_class(),
        Some(ast::ClassAscii {
            span: span(0..9),
            kind: ast::ClassAsciiKind::Alnum,
            negated: false,
        })
    );
    assert_eq!(
        parser(r"[:alnum:]A").maybe_parse_ascii_class(),
        Some(ast::ClassAscii {
            span: span(0..9),
            kind: ast::ClassAsciiKind::Alnum,
            negated: false,
        })
    );
    assert_eq!(
        parser(r"[:^alnum:]").maybe_parse_ascii_class(),
        Some(ast::ClassAscii {
            span: span(0..10),
            kind: ast::ClassAsciiKind::Alnum,
            negated: true,
        })
    );

    let p = parser(r"[:");
    assert_eq!(p.maybe_parse_ascii_class(), None);
    assert_eq!(p.offset(), 0);

    let p = parser(r"[:^");
    assert_eq!(p.maybe_parse_ascii_class(), None);
    assert_eq!(p.offset(), 0);

    let p = parser(r"[^:alnum:]");
    assert_eq!(p.maybe_parse_ascii_class(), None);
    assert_eq!(p.offset(), 0);

    let p = parser(r"[:alnnum:]");
    assert_eq!(p.maybe_parse_ascii_class(), None);
    assert_eq!(p.offset(), 0);

    let p = parser(r"[:alnum]");
    assert_eq!(p.maybe_parse_ascii_class(), None);
    assert_eq!(p.offset(), 0);

    let p = parser(r"[:alnum:");
    assert_eq!(p.maybe_parse_ascii_class(), None);
    assert_eq!(p.offset(), 0);
}

#[test]
fn parse_unicode_class() {
    assert_eq!(
        parser(r"\pN").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..3),
            negated: false,
            kind: ast::ClassUnicodeKind::OneLetter('N'),
        }))
    );
    assert_eq!(
        parser(r"\PN").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..3),
            negated: true,
            kind: ast::ClassUnicodeKind::OneLetter('N'),
        }))
    );
    assert_eq!(
        parser(r"\p{N}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..5),
            negated: false,
            kind: ast::ClassUnicodeKind::Named(s("N")),
        }))
    );
    assert_eq!(
        parser(r"\P{N}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..5),
            negated: true,
            kind: ast::ClassUnicodeKind::Named(s("N")),
        }))
    );
    assert_eq!(
        parser(r"\p{Greek}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..9),
            negated: false,
            kind: ast::ClassUnicodeKind::Named(s("Greek")),
        }))
    );

    assert_eq!(
        parser(r"\p{scx:Katakana}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..16),
            negated: false,
            kind: ast::ClassUnicodeKind::NamedValue {
                op: ast::ClassUnicodeOpKind::Colon,
                name: s("scx"),
                value: s("Katakana"),
            },
        }))
    );
    assert_eq!(
        parser(r"\p{scx=Katakana}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..16),
            negated: false,
            kind: ast::ClassUnicodeKind::NamedValue {
                op: ast::ClassUnicodeOpKind::Equal,
                name: s("scx"),
                value: s("Katakana"),
            },
        }))
    );
    assert_eq!(
        parser(r"\p{scx!=Katakana}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..17),
            negated: false,
            kind: ast::ClassUnicodeKind::NamedValue {
                op: ast::ClassUnicodeOpKind::NotEqual,
                name: s("scx"),
                value: s("Katakana"),
            },
        }))
    );

    assert_eq!(
        parser(r"\p{:}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..5),
            negated: false,
            kind: ast::ClassUnicodeKind::NamedValue {
                op: ast::ClassUnicodeOpKind::Colon,
                name: s(""),
                value: s(""),
            },
        }))
    );
    assert_eq!(
        parser(r"\p{=}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..5),
            negated: false,
            kind: ast::ClassUnicodeKind::NamedValue {
                op: ast::ClassUnicodeOpKind::Equal,
                name: s(""),
                value: s(""),
            },
        }))
    );
    assert_eq!(
        parser(r"\p{!=}").parse_escape(),
        Ok(Primitive::Unicode(ast::ClassUnicode {
            span: span(0..6),
            negated: false,
            kind: ast::ClassUnicodeKind::NamedValue {
                op: ast::ClassUnicodeOpKind::NotEqual,
                name: s(""),
                value: s(""),
            },
        }))
    );

    assert_eq!(
        parser(r"\p").parse_escape().unwrap_err(),
        TestError {
            span: span(2..2),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\p{").parse_escape().unwrap_err(),
        TestError {
            span: span(3..3),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\p{N").parse_escape().unwrap_err(),
        TestError {
            span: span(4..4),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );
    assert_eq!(
        parser(r"\p{Greek").parse_escape().unwrap_err(),
        TestError {
            span: span(8..8),
            kind: ast::ErrorKind::EscapeUnexpectedEof,
        }
    );

    assert_eq!(
        parser(r"\pNz").parse(),
        Ok(Ast::Concat(ast::Concat {
            span: span(0..4),
            asts: vec![
                Ast::Class(ast::Class::Unicode(ast::ClassUnicode {
                    span: span(0..3),
                    negated: false,
                    kind: ast::ClassUnicodeKind::OneLetter('N'),
                })),
                Ast::Literal(ast::Literal {
                    span: span(3..4),
                    kind: ast::LiteralKind::Verbatim,
                    c: 'z',
                }),
            ],
        }))
    );
    assert_eq!(
        parser(r"\p{Greek}z").parse(),
        Ok(Ast::Concat(ast::Concat {
            span: span(0..10),
            asts: vec![
                Ast::Class(ast::Class::Unicode(ast::ClassUnicode {
                    span: span(0..9),
                    negated: false,
                    kind: ast::ClassUnicodeKind::Named(s("Greek")),
                })),
                Ast::Literal(ast::Literal {
                    span: span(9..10),
                    kind: ast::LiteralKind::Verbatim,
                    c: 'z',
                }),
            ],
        }))
    );
    assert_eq!(
        parser(r"\p\{").parse().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::UnicodeClassInvalid,
        }
    );
    assert_eq!(
        parser(r"\P\{").parse().unwrap_err(),
        TestError {
            span: span(2..3),
            kind: ast::ErrorKind::UnicodeClassInvalid,
        }
    );
}

#[test]
fn parse_perl_class() {
    assert_eq!(
        parser(r"\d").parse_escape(),
        Ok(Primitive::Perl(ast::ClassPerl {
            span: span(0..2),
            kind: ast::ClassPerlKind::Digit,
            negated: false,
        }))
    );
    assert_eq!(
        parser(r"\D").parse_escape(),
        Ok(Primitive::Perl(ast::ClassPerl {
            span: span(0..2),
            kind: ast::ClassPerlKind::Digit,
            negated: true,
        }))
    );
    assert_eq!(
        parser(r"\s").parse_escape(),
        Ok(Primitive::Perl(ast::ClassPerl {
            span: span(0..2),
            kind: ast::ClassPerlKind::Space,
            negated: false,
        }))
    );
    assert_eq!(
        parser(r"\S").parse_escape(),
        Ok(Primitive::Perl(ast::ClassPerl {
            span: span(0..2),
            kind: ast::ClassPerlKind::Space,
            negated: true,
        }))
    );
    assert_eq!(
        parser(r"\w").parse_escape(),
        Ok(Primitive::Perl(ast::ClassPerl {
            span: span(0..2),
            kind: ast::ClassPerlKind::Word,
            negated: false,
        }))
    );
    assert_eq!(
        parser(r"\W").parse_escape(),
        Ok(Primitive::Perl(ast::ClassPerl {
            span: span(0..2),
            kind: ast::ClassPerlKind::Word,
            negated: true,
        }))
    );

    assert_eq!(
        parser(r"\d").parse(),
        Ok(Ast::Class(ast::Class::Perl(ast::ClassPerl {
            span: span(0..2),
            kind: ast::ClassPerlKind::Digit,
            negated: false,
        })))
    );
    assert_eq!(
        parser(r"\dz").parse(),
        Ok(Ast::Concat(ast::Concat {
            span: span(0..3),
            asts: vec![
                Ast::Class(ast::Class::Perl(ast::ClassPerl {
                    span: span(0..2),
                    kind: ast::ClassPerlKind::Digit,
                    negated: false,
                })),
                Ast::Literal(ast::Literal {
                    span: span(2..3),
                    kind: ast::LiteralKind::Verbatim,
                    c: 'z',
                }),
            ],
        }))
    );
}

// This tests a bug fix where the nest limit checker wasn't decrementing
// its depth during post-traversal, which causes long regexes to trip
// the default limit too aggressively.
#[test]
fn regression_454_nest_too_big() {
    let pattern = r#"
        2(?:
          [45]\d{3}|
          7(?:
            1[0-267]|
            2[0-289]|
            3[0-29]|
            4[01]|
            5[1-3]|
            6[013]|
            7[0178]|
            91
          )|
          8(?:
            0[125]|
            [139][1-6]|
            2[0157-9]|
            41|
            6[1-35]|
            7[1-5]|
            8[1-8]|
            90
          )|
          9(?:
            0[0-2]|
            1[0-4]|
            2[568]|
            3[3-6]|
            5[5-7]|
            6[0167]|
            7[15]|
            8[0146-9]
          )
        )\d{4}
        "#;
    assert!(parser_nest_limit(pattern, 50).parse().is_ok());
}

// This tests that we treat a trailing `-` in a character class as a
// literal `-` even when whitespace mode is enabled and there is whitespace
// after the trailing `-`.
#[test]
fn regression_455_trailing_dash_ignore_whitespace() {
    assert!(parser("(?x)[ / - ]").parse().is_ok());
    assert!(parser("(?x)[ a - ]").parse().is_ok());
    assert!(parser(
        "(?x)[
            a
            - ]
        "
    )
    .parse()
    .is_ok());
    assert!(parser(
        "(?x)[
            a # wat
            - ]
        "
    )
    .parse()
    .is_ok());

    assert!(parser("(?x)[ / -").parse().is_err());
    assert!(parser("(?x)[ / - ").parse().is_err());
    assert!(parser(
        "(?x)[
            / -
        "
    )
    .parse()
    .is_err());
    assert!(parser(
        "(?x)[
            / - # wat
        "
    )
    .parse()
    .is_err());
}
