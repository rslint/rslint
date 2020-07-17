use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::{peek_token, peek};
use crate::span::Span;

// I decided to not include this logic in the primary expr file since there is a lot of error recovery logic.
// primary literals (object and array) need a lot of tests too.
impl<'a> Parser<'a> {
    pub fn parse_array_literal(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic> {
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

            if self.cur_tok.token_type == TokenType::EOF {
                let err = self
                    .error(UnterminatedObjectLiteral, "Unterminated array literal")
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

                    match peek!(self) {
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
                            comma_whitespaces.push(LiteralWhitespace { before, after });

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
    ) -> Result<Expr, ParserDiagnostic> {
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
                let err = self
                    .error(UnterminatedObjectLiteral, "Unterminated object literal")
                    .secondary(open_brace_span, "Object literal starts here")
                    .primary(self.cur_tok.lexeme.to_owned(), "File ends here");

                return Err(err);
            }

            if self.cur_tok.token_type == TokenType::BraceClose {
                let close_brace_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after_close_brace = self.whitespace(false)?;

                return Ok(Expr::Object(ObjectExpr {
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
                let potential_comma_span = Span::new(prop.span().end, prop.span().end);
                let prop_span = prop.span().to_owned();
                props.push(prop);

                match peek!(self) {
                    Some(TokenType::Comma) => {
                        let before = self.whitespace(true)?;
                        self.advance_lexer(false)?;
                        let after = self.whitespace(false)?;

                        comma_whitespaces.push(LiteralWhitespace { before, after });
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
    ) -> Result<ObjProp, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        // property name can only be identifier, string, or number, see: https://www.ecma-international.org/ecma-262/5.1/#sec-11.1.5
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

            if let Expr::Identifier(LiteralExpr { span, whitespace }) = key.to_owned() {
                if ["get", "set"].contains(&span.content(self.source)) && peek!(self) != Some(TokenType::Colon) {
                    let setter = span.content(self.source) == "set";
                    let string = if setter { "setter" } else { "getter" };
                    let before_key = self.whitespace(true)?;

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
                    }

                    let prop_key = if self.cur_tok.token_type == TokenType::Identifier
                        || self.cur_tok.token_type.is_keyword()
                    {
                        self.parse_identifier_name(Some(before_key))?
                    } else {
                        self.parse_primary_expr(Some(before_key))?
                    };

                    // is_valid_assign_target checks if the parser is in strict mode and emits an error for `eval` and `arguments`
                    // which means we can simply reuse that function and ignore the return
                    prop_key.is_valid_assign_target(self);

                    if peek!(self) != Some(TokenType::ParenOpen) {
                        self.whitespace(true)?;
                        let err = self.error(ExpectedParen, &format!("Expected an opening parenthesis after an object {}, but found none", string))
                            .primary(self.cur_tok.lexeme.to_owned(), "Expected an opening parenthesis here");

                        return Err(err);
                    }

                    let mut args = self.parse_args(None)?;
                    let mut argument = None;

                    if args.arguments.len() != 0 && !setter {
                        let err = self
                            .error(
                                InvalidComputedPropertyArgs,
                                "Object getters cannot take any arguments",
                            )
                            .primary(args.span, "These arguments must be empty");

                        self.errors.push(err);
                    }

                    if args.arguments.len() != 1 && setter {
                        let err = self
                            .error(
                                InvalidComputedPropertyArgs,
                                "Object setters must take a single argument",
                            )
                            .primary(args.span, "Setters are required to take a single argument");

                        self.errors.push(err);
                    }

                    if setter && args.arguments.len() > 0 {
                        if let Expr::Identifier(ident) = args.arguments.first().unwrap() {
                            argument = Some(ident.to_owned());
                        } else {
                            let err = self.error(InvalidComputedPropertyArgs, "The argument to an object setter must be an identifier")
                                .primary(args.arguments.first().unwrap().span().to_owned(), "Expected an identifier here");

                            self.errors.push(err);
                            args.arguments = vec![];
                        }
                    }

                    let open_brace_whitespace;
                    let body;

                    if peek!(self) != Some(TokenType::BraceOpen) {
                        let before = self.whitespace(true)?;
                        let err = self.error(ExpectedBrace, &format!("Expected an opening brace after an object {}, but found `{}`", string, self.cur_tok.lexeme.content(self.source)))
                            .primary(self.cur_tok.lexeme.to_owned(), "Expected an opening brace here");

                        self.errors.push(err);
                        open_brace_whitespace = LiteralWhitespace {
                            before,
                            after: before.end.into()
                        };

                        body = self.parse_stmt_decl_list(Some(before), Some(&[TokenType::EOF, TokenType::BraceClose]), true)?;
                    } else {
                        let before = self.whitespace(true)?;
                        self.advance_lexer(false)?;
                        let after = self.whitespace(false)?;

                        open_brace_whitespace = LiteralWhitespace {
                            before,
                            after,
                        };

                        body = self.parse_stmt_decl_list(None, Some(&[TokenType::EOF, TokenType::BraceClose]), true)?;
                    }

                    let close_brace_whitespace;
                    let end;

                    if peek!(self) != Some(TokenType::BraceClose) {
                        let start = self.cur_tok.lexeme.start;
                        end = start;
                        let span = peek_token!(self).as_ref().unwrap().lexeme.to_owned();
                        let err = self.error(ExpectedBrace, &format!("Expected a closing brace after an object {} body, but found none", string))
                            .primary(Span::new(open_brace_whitespace.before.end, span.start), "Expected a closing brace to close this block");
                        
                        self.errors.push(err);
                        close_brace_whitespace = LiteralWhitespace {
                            before: Span::new(start, span.start),
                            after: span.start.into()
                        };
                    } else {
                        let before = self.whitespace(true)?;
                        end = self.cur_tok.lexeme.end;
                        self.advance_lexer(false)?;
                        let after = self.whitespace(false)?;

                        close_brace_whitespace = LiteralWhitespace {
                            before,
                            after
                        };
                    }

                    let computed_prop = ComputedObjProp {
                        span: Span::new(span.start, end),
                        open_paren_whitespace: args.open_paren_whitespace,
                        close_paren_whitespace: args.close_paren_whitespace,
                        argument,
                        open_brace_whitespace,
                        close_brace_whitespace,
                        identifier_whitespace: whitespace,
                        key: Box::new(prop_key),
                        body,
                    };

                    return Ok(if setter {
                        ObjProp::Setter(computed_prop)
                    } else {
                        ObjProp::Getter(computed_prop)
                    });
                }
            }

            match peek!(self) {
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

                    return Ok(ObjProp::Literal(LiteralObjProp {
                        span: self.span(key.span().start, value.span().end),
                        key: Box::new(key),
                        value: Box::new(value),
                        whitespace: LiteralWhitespace {
                            before: colon_whitespace.to_owned(),
                            after: colon_whitespace,
                        },
                    }));
                }

                Some(TokenType::Colon) => {
                    let before_colon = self.whitespace(true)?;
                    self.advance_lexer(false)?;
                    let after_colon = self.whitespace(false)?;

                    if peek!(self).map(|t| t.starts_expr()).is_none() {
                        self.discard_recover(
                            Some("Expected a value following an object key, but found none"),
                            |t| !t.starts_expr(),
                        )?;
                    }

                    let value = self.parse_assign_expr(None)?;

                    return Ok(ObjProp::Literal(LiteralObjProp {
                        span: self.span(key.span().start, value.span().end),
                        key: Box::new(key),
                        value: Box::new(value),
                        whitespace: LiteralWhitespace {
                            before: before_colon,
                            after: after_colon,
                        },
                    }));
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
            Expr::Object(ObjectExpr {
                span: span!("{a: 5, b: 7,}", "{a: 5, b: 7,}"),
                props: vec![
                    ObjProp::Literal(LiteralObjProp {
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
                    }),
                    ObjProp::Literal(LiteralObjProp {
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
                    })
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
            Expr::Object(ObjectExpr {
                span: span!("{a  5  b: 7,}", "{a  5  b: 7,}"),
                props: vec![
                    ObjProp::Literal(LiteralObjProp {
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
                    }),
                    ObjProp::Literal(LiteralObjProp {
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
                    })
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
        let mut parser = Parser::with_source("[; 2,]", 0, true).unwrap();
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
