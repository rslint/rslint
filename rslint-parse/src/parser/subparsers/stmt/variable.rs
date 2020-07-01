use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::{AssignToken, TokenType};
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::peek;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_var_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Var,
            "parse_var_stmt expects the current token to be `var`"
        );

        let var_span = self.cur_tok.lexeme.to_owned();

        self.advance_lexer(false)?;
        let after_var = self.whitespace(false)?;
        let mut declarators: Vec<Declarator> = vec![];
        let mut comma_whitespaces: Vec<LiteralWhitespace> = vec![];
        let mut first = true;

        while first || peek!(self) == Some(TokenType::Comma) {
            if first {
                first = false;
            } else {
                let before = self.whitespace(true)?;
                self.advance_lexer(false)?;
                let after = self.whitespace(false)?;

                comma_whitespaces.push(LiteralWhitespace {
                    before,
                    after
                });
            }

            declarators.push(self.parse_var_declarator()?);
        }

        let semi = self.semi()?;

        // We can just keep parsing, despite a semicolon being required
        // TODO: see if this is "safe" to do
        if semi.is_none() {
            let err = self.error(ExpectedSemicolon, "An explicit semicolon was required after a variable declaration, but none was found")
                .primary(var_span.to_owned() + declarators.last().unwrap().span(), "A semicolon is required to end this statement");
            
            self.errors.push(err);
        }

        let semicolon = semi.unwrap_or(Semicolon::Implicit);
        Ok(Stmt::Variable(VarStmt {
            span: (var_span + declarators.last().unwrap().span()).extend(semicolon.offset()),
            declared: declarators,
            comma_whitespaces,
            semi: semicolon,
            var_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_var,
            },
        }))
    }

    fn parse_var_declarator(&mut self) -> Result<Declarator, ParserDiagnostic<'a>> {
        let before_ident = self.whitespace(true)?;
        let ident_span = self.cur_tok.lexeme.to_owned();

        if self.cur_tok.token_type != TokenType::Identifier {
            let err = self.error(UnexpectedToken, &format!("Expected an identifier for a variable declaration, but instead found `{}`", ident_span.content(self.source)))
                .primary(ident_span, "An identifier is expected here");
            
            return Err(err);
        }
        self.advance_lexer(false)?;
        let after_ident = self.whitespace(false)?;
        
        // Variable is being declared and defined
        if peek!(self) == Some(TokenType::AssignOp(AssignToken::Assign)) {
            let before_eq = self.whitespace(true)?;
            self.advance_lexer(false)?;
            let after_eq = self.whitespace(false)?;

            let value = Some(self.parse_assign_expr(None)?);

            let declarator = Declarator {
                span: ident_span.to_owned() + value.as_ref().map(|x| x.span()).unwrap().to_owned(),
                name: LiteralExpr {
                    span: ident_span,
                    whitespace: LiteralWhitespace {
                        before: before_ident,
                        after: after_ident,
                    },
                },
                initializer_whitespace: Some(LiteralWhitespace {
                    before: before_eq,
                    after: after_eq,
                }),
                value,
            };

            return Ok(declarator);
        }

        // Variable is being only declared
        Ok(Declarator {
            span: ident_span.to_owned(),
            name: LiteralExpr {
                span: ident_span,
                whitespace: LiteralWhitespace {
                    before: before_ident,
                    after: after_ident,
                },
            },
            initializer_whitespace: None,
            value: None,
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::cst::expr::*;
    use crate::parser::cst::stmt::*;
    use crate::span;
    use crate::span::Span;
    use crate::stmt;

    #[test]
    fn var_single_decl() {
        assert_eq!(
            stmt!(" var a = 6;"),
            Stmt::Variable(VarStmt {
                span: span!(" var a = 6;", "var a = 6;"),
                comma_whitespaces: vec![],
                var_whitespace:  LiteralWhitespace {
                    before: Span::new(0, 1),
                    after: Span::new(4, 5),
                },
                semi: Semicolon::Explicit(LiteralWhitespace {
                    before: Span::new(10, 10),
                    after: Span::new(11, 11),
                }),
                declared: vec![
                    Declarator {
                        span: span!(" var a = 6;", "a = 6"),
                        initializer_whitespace: Some(LiteralWhitespace {
                            before: Span::new(7, 7),
                            after: Span::new(8, 9),
                        }),
                        name: LiteralExpr {
                            span: Span::new(5, 6),
                            whitespace: LiteralWhitespace {
                                before: Span::new(5, 5),
                                after: Span::new(6, 7),
                            }
                        },
                        value: Some(Expr::Number(LiteralExpr {
                            span: span!(" var a = 6;", "6"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(9, 9),
                                after: Span::new(10, 10),
                            }
                        }))
                    }
                ]
            })
        )
    }

    #[test]
    fn var_single_decl_no_initializer() {
        assert_eq!(
            stmt!("var b"),
            Stmt::Variable(VarStmt {
                span: span!("var b", "var b"),
                var_whitespace:  LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 4),
                },
                semi: Semicolon::Implicit,
                comma_whitespaces: vec![],
                declared: vec![
                    Declarator {
                        span: span!("var b", "b"),
                        name: LiteralExpr {
                            span: span!("var b", "b"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(4, 4),
                                after: Span::new(5, 5),
                            }
                        },
                        value: None,
                        initializer_whitespace: None,
                    }
                ]
            })
        )
    }

    #[test]
    fn var_multiple_decl_no_initializers() {
        assert_eq!(
            stmt!("var b, c"),
            Stmt::Variable(VarStmt {
                span: span!("var b, c", "var b, c"),
                var_whitespace:  LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 4),
                },
                semi: Semicolon::Implicit,
                comma_whitespaces: vec![
                    LiteralWhitespace {
                        before: Span::new(5, 5),
                        after: Span::new(6, 7),
                    }
                ],
                declared: vec![
                    Declarator {
                        span: span!("var b, c", "b"),
                        name: LiteralExpr {
                            span: span!("var b, c", "b"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(4, 4),
                                after: Span::new(5, 5),
                            }
                        },
                        value: None,
                        initializer_whitespace: None,
                    },
                    Declarator {
                        span: span!("var b, c", "c"),
                        name: LiteralExpr {
                            span: span!("var b, c", "c"),
                            whitespace: LiteralWhitespace {
                                before: Span::new(7, 7),
                                after: Span::new(8, 8),
                            }
                        },
                        value: None,
                        initializer_whitespace: None,
                    }
                ]
            })
        )
    }
}