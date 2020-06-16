use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::{
    ExpectedComma, ExpectedIdentifier, UnmatchedBracket,
};
use crate::parser::Parser;
use crate::peek_or;
use crate::span::Span;

impl<'a> Parser<'a> {
    /// Recursively parses suffixes  
    ///  
    /// Suffix ::  
    ///   Deref
    ///   Arguments
    pub fn parse_suffixes(
        &mut self,
        object: Expr,
        no_call: bool,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        match peek_or!(self, [TokenType::ParenOpen, TokenType::Period]) {
            Some(TokenType::Period) => {
                let before_dot = self.whitespace(true)?;
                let dot_span = self.cur_tok.lexeme.to_owned();
                // Go to the next token after the dot
                self.advance_lexer(false)?;
                let after_dot = self.whitespace(false)?;

                // We could use parse_identifier_name here but this allows us to recover and to offer a more helpful error message
                if !self.cur_tok.token_type.is_identifier_name() {
                    let peeked = self.peek_while(|t| {
                        [TokenType::Whitespace, TokenType::Linebreak].contains(&t.token_type)
                    })?;

                    if peeked
                        .filter(|t| t.token_type.is_identifier_name())
                        .is_none()
                    {
                        let err = self
                            .error(
                                ExpectedIdentifier,
                                "Expected an identifier after a dot suffix but found none",
                            )
                            .primary(dot_span, "Expected an identifier following this expression");

                        self.errors.push(err);
                        // Recover by ignoring the member expression
                        return Ok(object);
                    }
                }

                let before_ident = self.whitespace(true)?;
                let ident_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after_ident = self.whitespace(false)?;

                let identifier = Expr::Identifier(LiteralExpr {
                    span: ident_span,
                    whitespace: ExprWhitespace {
                        before: before_ident,
                        after: after_ident,
                    },
                });

                let end = identifier.span().end;

                self.parse_suffixes(
                    Expr::Member(MemberExpr {
                        span: self.span(object.span().start, end),
                        object: Box::new(object),
                        property: Box::new(identifier),
                        whitespace: MemberExprWhitespace {
                            before_dot,
                            after_dot,
                        },
                    }),
                    no_call,
                )
            }

            Some(TokenType::ParenOpen) if !no_call => {
                let arguments = self.parse_args(None)?;
                self.parse_suffixes(
                    Expr::Call(CallExpr {
                        span: self.span(object.span().start, arguments.span.end),
                        arguments,
                        callee: Box::new(object),
                    }),
                    no_call,
                )
            }

            Some(TokenType::BracketOpen) => {
                let before_op = self.whitespace(true)?;
                // It is impossible for the advanced token to be anything other than an opening bracket
                let open_bracket_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after_op = self.whitespace(false)?;

                let property = self.parse_expr(None)?;

                let before_closing_bracket = self.whitespace(true)?;

                if self.cur_tok.token_type != TokenType::BracketClose {
                    let err_tok = self.cur_tok.lexeme.to_owned();
                    self.discard_recover(None, |t| *t != TokenType::BracketClose)
                        .map_err(|_| {
                            self.error(
                                UnmatchedBracket,
                                "Expected a closing bracket but found none",
                            )
                            .secondary(
                                open_bracket_span.to_owned(),
                                "Property access begins here",
                            )
                            .primary(err_tok.to_owned(), "Expected a closing square bracket here")
                        })?;

                    let err = self
                        .error(
                            UnmatchedBracket,
                            "Expected a closing bracket but found none",
                        )
                        .secondary(open_bracket_span, "Property access begins here")
                        .primary(err_tok, "Expected a closing square bracket here");
                    self.errors.push(err);
                }
                let close_bracket_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after_closing_bracket = self.whitespace(false)?;

                self.parse_suffixes(
                    Expr::Bracket(BracketExpr {
                        span: self.span(object.span().start, close_bracket_span.end),
                        object: Box::new(object),
                        property: Box::new(property),
                        opening_bracket_whitespace: OperatorWhitespace {
                            before_op,
                            after_op,
                        },
                        closing_bracket_whitespace: OperatorWhitespace {
                            before_op: before_closing_bracket,
                            after_op: after_closing_bracket,
                        },
                    }),
                    no_call,
                )
            }

            _ => Ok(object),
        }
    }

    /// Parse arguments to a call such as `foo(bar)` or `new foo(bar, foo,)`.  
    /// The function assumes the token after the leading whitespace is a parentheses.  
    /// If the token after the whitespace is not a parentheses, it is an internal parser error.
    pub fn parse_args(&mut self, leading: Option<Span>) -> Result<Arguments, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::ParenOpen,
            "parse_args() assumes the token after the whitespace is a ParenOpen"
        );
        let paren_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_paren = self.whitespace(false)?;

        let mut exprs: Vec<Expr> = vec![];
        let mut whitespaces = vec![];
        let mut first = true;

        // TODO: there is definitely a better way to do this
        loop {
            let loop_leading_whitespace = self.whitespace(true)?;
            if self.cur_tok.token_type == TokenType::ParenClose {
                self.advance_lexer(false)?;
                let after_close_paren = self.whitespace(false)?;

                return Ok(Arguments {
                    span: self.span(paren_span.start, after_close_paren.start),
                    arguments: exprs,
                    comma_whitespaces: whitespaces,
                    open_paren_whitespace: OperatorWhitespace {
                        before_op: leading_whitespace,
                        after_op: after_paren,
                    },
                    close_paren_whitespace: OperatorWhitespace {
                        before_op: loop_leading_whitespace,
                        after_op: after_close_paren,
                    },
                });
            }

            if first {
                first = false;
            } else {
                if self.cur_tok.token_type != TokenType::Comma {
                    // We can recover and issue a better error message for `(foo bar)`
                    // This assumes the comma was right after the previous expression with no leading and trailing whitespace
                    if self.cur_tok.token_type.starts_expr() {
                        let last = exprs.last()
                            .expect("parse_args expected a previous expr to recover, but none was found somehow")
                            .span()
                            .to_owned();

                        whitespaces.push(OperatorWhitespace {
                            before_op: self.span(last.end, last.end),
                            after_op: self.span(last.end, last.end),
                        });

                        let expr = self.parse_assign_expr(Some(loop_leading_whitespace))?;

                        let err = self
                            .error(ExpectedComma, "Expected a comma between separate arguments")
                            .primary(last.end..expr.span().start, "A comma is required here");

                        exprs.push(expr);
                        self.errors.push(err);
                        continue;
                    }

                    let potential_err_span = self.cur_tok.lexeme.to_owned();

                    self.advance_lexer(false)?;
                    let after_comma = self.whitespace(false)?;

                    whitespaces.push(OperatorWhitespace {
                        before_op: loop_leading_whitespace.to_owned(),
                        after_op: after_comma,
                    });

                    if peek_or!(self, [TokenType::ParenClose]) == Some(TokenType::ParenClose) {
                        let before_paren = self.whitespace(true)?;
                        self.advance_lexer(false)?;
                        let after_close_paren = self.whitespace(false)?;

                        return Ok(Arguments {
                            span: self.span(paren_span.start, after_close_paren.start),
                            arguments: exprs,
                            comma_whitespaces: whitespaces,
                            open_paren_whitespace: OperatorWhitespace {
                                before_op: leading_whitespace,
                                after_op: after_paren,
                            },
                            close_paren_whitespace: OperatorWhitespace {
                                before_op: before_paren,
                                after_op: after_close_paren,
                            },
                        });
                    }

                    let err = self
                        .error(ExpectedComma, "Expected a comma in argument list")
                        .primary(potential_err_span, "Expected a comma here");
                    return Err(err);
                }

                self.advance_lexer(false)?;
                let after_comma = self.whitespace(false)?;

                whitespaces.push(OperatorWhitespace {
                    before_op: loop_leading_whitespace.to_owned(),
                    after_op: after_comma,
                });

                if peek_or!(self, [TokenType::ParenClose]) == Some(TokenType::ParenClose) {
                    let before_paren = self.whitespace(true)?;
                    self.advance_lexer(false)?;
                    let after_close_paren = self.whitespace(false)?;

                    return Ok(Arguments {
                        span: self.span(paren_span.start, after_close_paren.start),
                        arguments: exprs,
                        comma_whitespaces: whitespaces,
                        open_paren_whitespace: OperatorWhitespace {
                            before_op: leading_whitespace,
                            after_op: after_paren,
                        },
                        close_paren_whitespace: OperatorWhitespace {
                            before_op: before_paren,
                            after_op: after_close_paren,
                        },
                    });
                }

                exprs.push(self.parse_assign_expr(None)?);
                continue;
            }
            exprs.push(self.parse_assign_expr(Some(loop_leading_whitespace))?);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr;
    use crate::parser::cst::expr::*;
    use crate::parser::Parser;
    use crate::span;
    use crate::span::Span;

    #[test]
    fn dot_deref_suffixes() {
        let mut parser = Parser::with_source(" a\n . \n\n b. a", "tests", true).unwrap();
        let expr = parser.parse_primary_expr(None).unwrap();
        let member = parser.parse_suffixes(expr, false);
        assert_eq!(
            member,
            Ok(Expr::Member(MemberExpr {
                span: Span::new(1, 13),
                object: Box::new(Expr::Member(MemberExpr {
                    span: Span::new(1, 10),
                    object: Box::new(Expr::Identifier(LiteralExpr {
                        span: Span::new(1, 2),
                        whitespace: ExprWhitespace {
                            before: Span::new(0, 1),
                            after: Span::new(2, 2),
                        }
                    })),
                    property: Box::new(Expr::Identifier(LiteralExpr {
                        span: Span::new(9, 10),
                        whitespace: ExprWhitespace {
                            before: Span::new(6, 9),
                            after: Span::new(10, 10),
                        }
                    })),
                    whitespace: MemberExprWhitespace {
                        before_dot: Span::new(2, 4),
                        after_dot: Span::new(5, 6),
                    }
                })),
                property: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(12, 13),
                    whitespace: ExprWhitespace {
                        before: Span::new(12, 12),
                        after: Span::new(13, 13),
                    }
                })),
                whitespace: MemberExprWhitespace {
                    before_dot: Span::new(10, 10),
                    after_dot: Span::new(11, 12),
                }
            }),)
        )
    }

    #[test]
    fn attempt_to_parse_suffixes_without_suffixes() {
        let mut parser = Parser::with_source("foo  ", "tests", true).unwrap();
        let expr = parser.parse_primary_expr(None).unwrap();
        let member = parser.parse_suffixes(expr, false);

        assert_eq!(
            member,
            Ok(Expr::Identifier(LiteralExpr {
                span: Span::new(0, 3),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 5)
                }
            }))
        )
    }

    #[test]
    fn suffixes_with_identifier_following() {
        let mut parser = Parser::with_source("foo  bar", "tests", true).unwrap();
        let expr = parser.parse_primary_expr(None).unwrap();
        let member = parser.parse_suffixes(expr, false);

        assert_eq!(
            member,
            Ok(Expr::Identifier(LiteralExpr {
                span: Span::new(0, 3),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 5)
                }
            }))
        )
    }

    #[test]
    fn arguments_no_trailing() {
        let mut parser = Parser::with_source("(foo, bar ) ", "tests", true).unwrap();
        let args = parser.parse_args(None).unwrap();

        assert_eq!(
            args,
            Arguments {
                span: span!("(foo, bar ) ", "(foo, bar )"),
                arguments: vec![
                    Expr::Identifier(LiteralExpr {
                        span: span!("(foo, bar ) ", "foo"),
                        whitespace: ExprWhitespace {
                            before: Span::new(1, 1),
                            after: Span::new(4, 4),
                        }
                    }),
                    Expr::Identifier(LiteralExpr {
                        span: span!("(foo, bar ) ", "bar"),
                        whitespace: ExprWhitespace {
                            before: Span::new(6, 6),
                            after: Span::new(9, 10),
                        }
                    }),
                ],
                open_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 0),
                    after_op: Span::new(1, 1),
                },
                close_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(10, 10),
                    after_op: Span::new(11, 12),
                },
                comma_whitespaces: vec![OperatorWhitespace {
                    before_op: Span::new(4, 4),
                    after_op: Span::new(5, 6),
                }]
            }
        )
    }

    #[test]
    fn empty_arguments() {
        let mut parser = Parser::with_source(" ( \n) ", "tests", true).unwrap();
        let args = parser.parse_args(None).unwrap();

        assert_eq!(
            args,
            Arguments {
                span: span!(" ( \n) ", "( \n)"),
                arguments: vec![],
                open_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 1),
                    after_op: Span::new(2, 3),
                },
                close_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(3, 4),
                    after_op: Span::new(5, 6),
                },
                comma_whitespaces: vec![]
            }
        )
    }

    #[test]
    fn single_argument() {
        let mut parser = Parser::with_source("(/a/g)", "tests", true).unwrap();
        let args = parser.parse_args(None).unwrap();

        assert_eq!(
            args,
            Arguments {
                span: span!("(/a/g)", "(/a/g)"),
                arguments: vec![Expr::Regex(LiteralExpr {
                    span: span!("(/a/g)", "/a/g"),
                    whitespace: ExprWhitespace {
                        before: Span::new(1, 1),
                        after: Span::new(5, 5)
                    }
                })],
                open_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 0),
                    after_op: Span::new(1, 1),
                },
                close_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(5, 5),
                    after_op: Span::new(6, 6),
                },
                comma_whitespaces: vec![]
            }
        )
    }

    #[test]
    fn trailing_comma_args() {
        let mut parser = Parser::with_source("(foo,)", "tests", true).unwrap();
        let args = parser.parse_args(None).unwrap();

        assert_eq!(
            args,
            Arguments {
                span: span!("(foo,)", "(foo,)"),
                arguments: vec![Expr::Identifier(LiteralExpr {
                    span: span!("(foo,)", "foo"),
                    whitespace: ExprWhitespace {
                        before: Span::new(1, 1),
                        after: Span::new(4, 4),
                    }
                })],
                open_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 0),
                    after_op: Span::new(1, 1),
                },
                close_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(5, 5),
                    after_op: Span::new(6, 6),
                },
                comma_whitespaces: vec![OperatorWhitespace {
                    before_op: Span::new(4, 4),
                    after_op: Span::new(5, 5)
                }]
            }
        )
    }

    #[test]
    fn no_comma_recovery() {
        let mut parser = Parser::with_source("(foo bar)", "tests", true).unwrap();
        let args = parser.parse_args(None).unwrap();

        assert_eq!(
            args,
            Arguments {
                span: span!("(foo bar)", "(foo bar)"),
                arguments: vec![
                    Expr::Identifier(LiteralExpr {
                        span: span!("(foo bar)", "foo"),
                        whitespace: ExprWhitespace {
                            before: Span::new(1, 1),
                            after: Span::new(4, 5),
                        }
                    }),
                    Expr::Identifier(LiteralExpr {
                        span: span!("(foo bar)", "bar"),
                        whitespace: ExprWhitespace {
                            before: Span::new(5, 5),
                            after: Span::new(8, 8),
                        }
                    }),
                ],
                open_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 0),
                    after_op: Span::new(1, 1),
                },
                close_paren_whitespace: OperatorWhitespace {
                    before_op: Span::new(8, 8),
                    after_op: Span::new(9, 9),
                },
                comma_whitespaces: vec![OperatorWhitespace {
                    before_op: Span::new(4, 4),
                    after_op: Span::new(4, 4),
                }]
            }
        );
        assert_eq!(parser.errors.len(), 1)
    }

    #[test]
    fn bracket_suffix() {
        assert_eq!(
            expr!(" foo ['bar'] "),
            Expr::Bracket(BracketExpr {
                span: span!(" foo ['bar'] ", "foo ['bar']"),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!(" foo ['bar'] ", "foo"),
                    whitespace: ExprWhitespace {
                        before: Span::new(0, 1),
                        after: Span::new(4, 5),
                    }
                })),
                property: Box::new(Expr::String(LiteralExpr {
                    span: span!(" foo ['bar'] ", "'bar'"),
                    whitespace: ExprWhitespace {
                        before: Span::new(6, 6),
                        after: Span::new(11, 11),
                    }
                })),
                opening_bracket_whitespace: OperatorWhitespace {
                    before_op: Span::new(5, 5),
                    after_op: Span::new(6, 6),
                },
                closing_bracket_whitespace: OperatorWhitespace {
                    before_op: Span::new(11, 11),
                    after_op: Span::new(12, 13),
                }
            })
        )
    }

    #[test]
    fn bracket_suffix_error_recovery() {
        let mut parser = Parser::with_source("foo[bar;[]", "tests", true).unwrap();
        let expr = parser.parse_expr(None).unwrap();

        assert_eq!(
            expr,
            Expr::Bracket(BracketExpr {
                span: span!("foo[bar;[]", "foo[bar;[]"),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo[bar;[]", "foo"),
                    whitespace: ExprWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(3, 3),
                    }
                })),
                property: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo[bar;[]", "bar"),
                    whitespace: ExprWhitespace {
                        before: Span::new(4, 4),
                        after: Span::new(7, 7),
                    }
                })),
                opening_bracket_whitespace: OperatorWhitespace {
                    before_op: Span::new(3, 3),
                    after_op: Span::new(4, 4),
                },
                closing_bracket_whitespace: OperatorWhitespace {
                    before_op: Span::new(7, 7),
                    after_op: Span::new(10, 10),
                }
            })
        );
        assert_eq!(parser.errors.len(), 1);
    }
}
