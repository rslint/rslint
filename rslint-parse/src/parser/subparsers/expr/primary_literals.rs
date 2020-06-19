use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::peek_or;
use crate::span::Span;

// I decided to not include this logic in the primary expr file since there is a lot of error recovery logic.
// primary literals (object and array) need a lot of tests too.
impl<'a> Parser<'a> {
    pub fn parse_array_literal(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::BracketOpen,
            "parse_array_literal expects the current token to be a bracket"
        );

        let open_bracket_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let afterening_bracket = self.whitespace(false)?;

        // Arrays may have undefined elements declared, e.g. `[,,,,,,foo,,,,]`
        let mut exprs: Vec<Option<Expr>> = vec![];
        let mut comma_whitespaces: Vec<LiteralWhitespace> = vec![];

        loop {
            let loop_leading_whitespace = self.whitespace(true)?;

            if self.done() {
                let err = self.error(UnterminatedObjectLiteral, "Unterminated array literal")
                    .secondary(open_bracket_span, "Array literal starts here")
                    .primary(self.cur_tok.lexeme.to_owned(), "File ends here");
                
                return Err(err);
            }

            match self.cur_tok.token_type {
                TokenType::BracketClose => {
                    self.advance_lexer(false)?;
                    let after_closing_bracket = self.whitespace(false)?;

                    return Ok(Expr::Array(ArrayExpr {
                        span: Span::new(open_bracket_span.start, after_closing_bracket.end),
                        exprs,
                        comma_whitespaces,
                        opening_bracket_whitespace: LiteralWhitespace {
                            before: leading_whitespace,
                            after: afterening_bracket,
                        },
                        closing_bracket_whitespace: LiteralWhitespace {
                            before: loop_leading_whitespace,
                            after: after_closing_bracket,
                        },
                    }));
                }

                TokenType::Comma => {
                    self.advance_lexer(false)?;
                    let after = self.whitespace(false)?;

                    comma_whitespaces.push(LiteralWhitespace {
                        before: loop_leading_whitespace,
                        after,
                    });
                    exprs.push(None);
                }

                t if t.starts_expr() => {
                    let expr = self.parse_assign_expr(Some(loop_leading_whitespace))?;

                    match peek_or!(self) {
                        Some(TokenType::Comma) => {
                            let before_comma = self.whitespace(true)?;
                            self.advance_lexer(false)?;
                            let after_comma = self.whitespace(false)?;

                            comma_whitespaces.push(LiteralWhitespace {
                                before: before_comma,
                                after: after_comma,
                            });
                        }

                        Some(t) if t != TokenType::BracketClose => {
                            // Insert an implicit comma to recover from the error.
                            // This assumes the leading whitespace of the comma is 0 in length and on the end span of the expr.
                            // The comma has all of the trailing whitespace of the expr.
                            // The error wont be reported here, it will be reported in the next iteration

                            let before = Span::new(expr.span().end, expr.span().end);
                            let after = Span::new(expr.span().end, self.cur_tok.lexeme.start);
                            comma_whitespaces.push(LiteralWhitespace {
                                before,
                                after,
                            });

                            // We want to avoid skipping over an expression because we can recover and parse it in the next iteration
                            if !t.starts_expr() {
                                self.whitespace(true)?;
                                self.advance_lexer(false)?;
                                self.whitespace(false)?;
                            }

                            let err = self.error(ExpectedComma, "Expected a comma or closing bracket after an array element, but found an unexpected token")
                                .primary(expr.span().to_owned(), "Expected a comma or closing bracket after this");

                            self.errors.push(err);
                        }
                        _ => {}
                    }
                    exprs.push(Some(expr));
                }

                _ => {
                    let err_span = self.cur_tok.lexeme.to_owned();
                    let err_source = self.cur_tok.lexeme.content(self.source);
                    self.advance_lexer(false)?;
                    let err = self
                        .error(
                            UnexpectedToken,
                            &format!(
                                "Expected a comma, expression, or closing bracket but found `{}`",
                                err_source
                            ),
                        )
                        .primary(err_span, "Unexpected");

                    self.errors.push(err);
                }
            }
        }
    }

    pub fn parse_object_literal(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::BraceOpen,
            "parse_object_literal expects the current token to be a brace"
        );

        let open_brace_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let afteren_brace_span = self.whitespace(false)?;
        let mut props: Vec<ObjProp> = vec![];
        let mut comma_whitespaces: Vec<LiteralWhitespace> = vec![];

        loop {
            let loop_leading_whitespace = self.whitespace(true)?;

            if self.done() {
                let err = self.error(UnterminatedObjectLiteral, "Unterminated object literal")
                    .secondary(open_brace_span, "Object literal starts here")
                    .primary(self.cur_tok.lexeme.to_owned(), "File ends here");
                
                return Err(err);
            }

            if self.cur_tok.token_type == TokenType::BraceClose {
                let close_brace_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after_close_brace = self.whitespace(false)?;

                return Ok(Expr::Object(Object {
                    span: Span::new(open_brace_span.start, close_brace_span.end),
                    props,
                    comma_whitespaces,
                    open_brace_whitespace: LiteralWhitespace {
                        before: leading_whitespace,
                        after: afteren_brace_span,
                    },
                    close_brace_whitespace: LiteralWhitespace {
                        before: loop_leading_whitespace,
                        after: after_close_brace,
                    },
                }));
            }

            if [
                TokenType::Identifier,
                TokenType::LiteralString,
                TokenType::LiteralNumber,
            ]
            .contains(&self.cur_tok.token_type)
                || self.cur_tok.token_type.is_keyword()
            {
                let prop = self.parse_object_property(Some(loop_leading_whitespace))?;
                let potential_comma_span = Span::new(prop.value.span().end, prop.value.span().end);
                let prop_span = prop.span.to_owned();
                props.push(prop);

                match peek_or!(self) {
                    Some(TokenType::Comma) => {
                        let before = self.whitespace(true)?;
                        self.advance_lexer(false)?;
                        let after = self.whitespace(false)?;

                        comma_whitespaces.push(LiteralWhitespace {
                            before,
                            after,
                        });
                    }
                    // Recover from `{a: b c: d}`, the comma's whitespace will be the end of the property value
                    Some(t) if t != TokenType::BraceClose => {
                        let err = self
                            .error(
                                ExpectedComma,
                                "Expected a comma between object properties, but found none",
                            )
                            .primary(prop_span, "A comma is required following this property");

                        self.errors.push(err);
                        comma_whitespaces.push(LiteralWhitespace {
                            before: potential_comma_span.to_owned(),
                            after: potential_comma_span,
                        });
                    }
                    _ => {}
                }
                continue;
            }
            println!("a");
            let unexpected = self.cur_tok.lexeme.to_owned();
            self.discard_recover(Some("Expected an expression or a closing brace in object literal, but encountered an unexpected token"), |t| !t.starts_expr() && t != &TokenType::BraceClose)?;
            let err = self.error(UnexpectedToken, "Expected an expression or a closing brace in object literal, but encountered an unexpected token")
                .primary(unexpected, "Unexpected");

            self.errors.push(err);
        }
    }

    fn parse_object_property(
        &mut self,
        leading: Option<Span>,
    ) -> Result<ObjProp, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        // TODO: get and set props (needs stmt parsing)

        // property name can only be identifier, string, or number, see: https://www.ecma-international.org/ecma-262/5.1/#sec-11.1.5
        // TODO: clean this up a bit, it's very ugly right now
        if ![
            TokenType::Identifier,
            TokenType::LiteralString,
            TokenType::LiteralNumber,
        ]
        .contains(&self.cur_tok.token_type)
            && !self.cur_tok.token_type.is_keyword()
        {
            let err = self.error(ExpectedObjectKey, &format!("Expected an identifier, string, or number for an object property key, but found `{}`", self.cur_tok.lexeme.content(self.source)))
                .primary(self.cur_tok.lexeme.to_owned(), "Unexpected");
            return Err(err);
        } else {
            let key = if self.cur_tok.token_type == TokenType::Identifier
                || self.cur_tok.token_type.is_keyword()
            {
                self.parse_identifier_name(None)?
            } else {
                self.parse_primary_expr(Some(leading_whitespace))?
            };

            match peek_or!(self) {
                // Recover from `{ a b }` by assuming a colon was there, the whitespace for before and after will be the end of the key's span
                Some(t) if t.starts_expr() => {
                    let err = self
                        .error(
                            MissingColonAfterKey,
                            "Missing a colon between an object key and its value",
                        )
                        .primary(
                            key.span().to_owned(),
                            "A colon is required following this key",
                        );

                    self.errors.push(err);

                    let value = self.parse_assign_expr(None)?;
                    let colon_whitespace = self.span(key.span().end, key.span().end);

                    return Ok(ObjProp {
                        span: self.span(key.span().start, value.span().end),
                        key: Box::new(key),
                        value: Box::new(value),
                        whitespace: LiteralWhitespace {
                            before: colon_whitespace.to_owned(),
                            after: colon_whitespace,
                        },
                    });
                }

                Some(TokenType::Colon) => {
                    let before_colon = self.whitespace(true)?;
                    self.advance_lexer(false)?;
                    let after_colon = self.whitespace(false)?;

                    if peek_or!(self).map(|t| t.starts_expr()).is_none() {
                        self.discard_recover(
                            Some("Expected a value following an object key, but found none"),
                            |t| !t.starts_expr(),
                        )?;
                    }

                    let value = self.parse_assign_expr(None)?;

                    return Ok(ObjProp {
                        span: self.span(key.span().start, value.span().end),
                        key: Box::new(key),
                        value: Box::new(value),
                        whitespace: LiteralWhitespace {
                            before: before_colon,
                            after: after_colon,
                        },
                    });
                }

                _ => {
                    let err = self
                        .error(
                            MissingColonAfterKey,
                            "Missing a colon between an object key and its value",
                        )
                        .primary(
                            key.span().to_owned(),
                            "A colon is required following this key",
                        );

                    return Err(err);
                }
            }
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
    fn object_literal() {
        assert_eq!(
            expr!("{a: 5, b: 7,}"),
            Expr::Object(Object {
                span: span!("{a: 5, b: 7,}", "{a: 5, b: 7,}"),
                props: vec![
                    ObjProp {
                        span: span!("{a: 5, b: 7,}", "a: 5"),
                        key: Box::new(Expr::Identifier(LiteralExpr {
                            span: span!("{a: 5, b: 7,}", "a"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(1, 1),
                                after: Span::new(2, 2),
                            }
                        })),
                        value: Box::new(Expr::Number(LiteralExpr {
                            span: span!("{a: 5, b: 7,}", "5"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(4, 4),
                                after: Span::new(5, 5),
                            }
                        })),
                        whitespace: LiteralWhitespace {
                            before: Span::new(2, 2),
                            after: Span::new(3, 4),
                        }
                    },
                    ObjProp {
                        span: span!("{a: 5, b: 7,}", "b: 7"),
                        key: Box::new(Expr::Identifier(LiteralExpr {
                            span: span!("{a: 5, b: 7,}", "b"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(7, 7),
                                after: Span::new(8, 8),
                            }
                        })),
                        value: Box::new(Expr::Number(LiteralExpr {
                            span: span!("{a: 5, b: 7,}", "7"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(10, 10),
                                after: Span::new(11, 11),
                            }
                        })),
                        whitespace: LiteralWhitespace {
                            before: Span::new(8, 8),
                            after: Span::new(9, 10),
                        }
                    }
                ],
                comma_whitespaces: vec![
                    LiteralWhitespace {
                        before: Span::new(5, 5),
                        after: Span::new(6, 7),
                    },
                    LiteralWhitespace {
                        before: Span::new(11, 11),
                        after: Span::new(12, 12),
                    },
                ],
                open_brace_whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(1, 1),
                },
                close_brace_whitespace: LiteralWhitespace {
                    before: Span::new(12, 12),
                    after: Span::new(13, 13),
                }
            }),
        )
    }

    #[test]
    fn object_literal_error_recovery() {
        assert_eq!(
            expr!("{a  5  b: 7,}"),
            Expr::Object(Object {
                span: span!("{a  5  b: 7,}", "{a  5  b: 7,}"),
                props: vec![
                    ObjProp {
                        span: span!("{a  5  b: 7,}", "a  5"),
                        key: Box::new(Expr::Identifier(LiteralExpr {
                            span: span!("{a  5  b: 7,}", "a"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(1, 1),
                                after: Span::new(2, 4),
                            }
                        })),
                        value: Box::new(Expr::Number(LiteralExpr {
                            span: span!("{a  5  b: 7,}", "5"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(4, 4),
                                after: Span::new(5, 7),
                            }
                        })),
                        whitespace: LiteralWhitespace {
                            before: Span::new(2, 2),
                            after: Span::new(2, 2),
                        }
                    },
                    ObjProp {
                        span: span!("{a  5  b: 7,}", "b: 7"),
                        key: Box::new(Expr::Identifier(LiteralExpr {
                            span: span!("{a  5  b: 7,}", "b"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(7, 7),
                                after: Span::new(8, 8),
                            }
                        })),
                        value: Box::new(Expr::Number(LiteralExpr {
                            span: span!("{a  5  b: 7,}", "7"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(10, 10),
                                after: Span::new(11, 11),
                            }
                        })),
                        whitespace: LiteralWhitespace {
                            before: Span::new(8, 8),
                            after: Span::new(9, 10),
                        }
                    }
                ],
                comma_whitespaces: vec![
                    LiteralWhitespace {
                        before: Span::new(5, 5),
                        after: Span::new(5, 5),
                    },
                    LiteralWhitespace {
                        before: Span::new(11, 11),
                        after: Span::new(12, 12),
                    },
                ],
                open_brace_whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(1, 1),
                },
                close_brace_whitespace: LiteralWhitespace {
                    before: Span::new(12, 12),
                    after: Span::new(13, 13),
                }
            }),
        )
    }

    #[test]
    // fn object_literal_unexpected_token() {
    //     let mut parser = Parser::with_source("{a: b ]}")
    // }

    #[test]
    fn array_literal() {
        assert_eq!(
            expr!("[ 2, a]"),
            Expr::Array(ArrayExpr {
                span: span!("[ 2, a]", "[ 2, a]"),
                exprs: vec![
                    Some(Expr::Number(LiteralExpr {
                        span: span!("[ 2, a]", "2"),
                        whitespace: LiteralWhitespace {
                            before: Span::new(2, 2),
                            after: Span::new(3, 3)
                        }
                    })),
                    Some(Expr::Identifier(LiteralExpr {
                        span: span!("[ 2, a]", "a"),
                        whitespace: LiteralWhitespace {
                            before: Span::new(5, 5),
                            after: Span::new(6, 6),
                        }
                    })),
                ],
                comma_whitespaces: vec![LiteralWhitespace {
                    before: Span::new(3, 3),
                    after: Span::new(4, 5),
                }],
                opening_bracket_whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(1, 2),
                },
                closing_bracket_whitespace: LiteralWhitespace {
                    before: Span::new(6, 6),
                    after: Span::new(7, 7)
                }
            }),
        )
    }

    #[test]
    fn empty_array_literal() {
        assert_eq!(
            expr!("[]"),
            Expr::Array(ArrayExpr {
                span: span!("[]", "[]"),
                exprs: vec![],
                comma_whitespaces: vec![],
                opening_bracket_whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(1, 1),
                },
                closing_bracket_whitespace: LiteralWhitespace {
                    before: Span::new(1, 1),
                    after: Span::new(2, 2),
                }
            })
        )
    }

    #[test]
    fn array_literal_invalid_token_error_recovery() {
        let mut parser = Parser::with_source("[; 2,]", "tests", true).unwrap();
        let expr = parser.parse_expr(None).unwrap();

        assert_eq!(
            expr,
            Expr::Array(ArrayExpr {
                span: span!("[; 2,]", "[; 2,]"),
                exprs: vec![Some(Expr::Number(LiteralExpr {
                    span: span!("[; 2,]", "2"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(2, 3),
                        after: Span::new(4, 4),
                    }
                }))],
                comma_whitespaces: vec![LiteralWhitespace {
                    before: Span::new(4, 4),
                    after: Span::new(5, 5),
                }],
                opening_bracket_whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(1, 1),
                },
                closing_bracket_whitespace: LiteralWhitespace {
                    before: Span::new(5, 5),
                    after: Span::new(6, 6)
                }
            })
        )
    }
}
