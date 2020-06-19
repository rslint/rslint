use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek_or;

impl<'a> Parser<'a> {
    pub fn parse_member_or_new_expr(
        &mut self,
        leading_whitespace: Option<Span>,
        new_expr: bool,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_ws = if leading_whitespace.is_some() {
            leading_whitespace.unwrap()
        } else {
            self.whitespace(true)?
        };
        let start = self.cur_tok.lexeme.start;

        // Parse `new foo(bar)` or `new foo`
        if self.cur_tok.token_type == TokenType::New {
            self.advance_lexer(false)?;
            let after_new = self.whitespace(false)?;
            // TODO: handle `new.target` for ES6

            let expr = self.parse_member_or_new_expr(None, new_expr)?;
            let expr_span = expr.span();
            
            if !new_expr || peek_or!(self, [TokenType::ParenOpen]) == Some(TokenType::ParenOpen) {
                let mut args = None;
                if peek_or!(self, [TokenType::ParenOpen]) == Some(TokenType::ParenOpen) {
                    args = Some(self.parse_args(None)?);
                }

                let new_expr = Expr::New(NewExpr {
                    span: self.span(start, expr_span.end),
                    target: Box::new(expr),
                    args,
                    whitespace: LiteralWhitespace {
                        after: after_new,
                        before: leading_ws,
                    },
                });

                return self.parse_suffixes(new_expr, true);
            }

            return Ok(Expr::New(NewExpr {
                span: self.span(start, expr_span.end),
                target: Box::new(expr),
                args: None,
                whitespace: LiteralWhitespace {
                    after: after_new,
                    before: leading_ws,
                },
            }));
        } else {
            let target = self.parse_primary_expr(Some(leading_ws))?;
            self.parse_suffixes(target, true)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cst::expr::*;
    use crate::parser::Parser;
    use crate::span::Span;

    #[test]
    fn member_expr_with_suffix() {
        let res = Parser::with_source("361 \n.bar\n", "tests", true)
            .unwrap()
            .parse_member_or_new_expr(None, false)
            .unwrap();
        assert_eq!(
            res,
            Expr::Member(MemberExpr {
                span: Span::new(0, 9),
                object: Box::new(Expr::Number(LiteralExpr {
                    span: Span::new(0, 3),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(3, 4)
                    }
                })),
                property: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(6, 9),
                    whitespace: LiteralWhitespace {
                        before: Span::new(6, 6),
                        after: Span::new(9, 9)
                    }
                })),
                whitespace: LiteralWhitespace {
                    before: Span::new(4, 5),
                    after: Span::new(6, 6)
                }
            })
        )
    }

    #[test]
    fn new_expr_with_member_suffixes() {
        let res = Parser::with_source(" new \n foo. bar", "tests", true)
            .unwrap()
            .parse_member_or_new_expr(None, true);
        assert_eq!(
            res,
            Ok(Expr::New(NewExpr {
                span: Span::new(1, 15),
                target: Box::new(Expr::Member(MemberExpr {
                    span: Span::new(7, 15),
                    object: Box::new(Expr::Identifier(LiteralExpr {
                        span: Span::new(7, 10),
                        whitespace: LiteralWhitespace {
                            before: Span::new(5, 7),
                            after: Span::new(10, 10),
                        }
                    })),
                    property: Box::new(Expr::Identifier(LiteralExpr {
                        span: Span::new(12, 15),
                        whitespace: LiteralWhitespace {
                            before: Span::new(12, 12),
                            after: Span::new(15, 15),
                        }
                    })),
                    whitespace: LiteralWhitespace {
                        before: Span::new(10, 10),
                        after: Span::new(11, 12),
                    }
                })),
                args: None,
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 1),
                    after: Span::new(4, 5),
                }
            }))
        )
    }

    #[test]
    fn new_expr_without_suffixes() {
        let res = Parser::with_source("new foo", "tests", true)
            .unwrap()
            .parse_member_or_new_expr(None, true);
        assert_eq!(
            res,
            Ok(Expr::New(NewExpr {
                span: Span::new(0, 7),
                target: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(4, 7),
                    whitespace: LiteralWhitespace {
                        before: Span::new(4, 4),
                        after: Span::new(7, 7)
                    }
                })),
                args: None,
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 4),
                },
            }))
        )
    }

    #[test]
    fn member_expr_without_suffixes() {
        let res = Parser::with_source("foo", "tests", true)
            .unwrap()
            .parse_member_or_new_expr(None, false);
        assert_eq!(
            res,
            Ok(Expr::Identifier(LiteralExpr {
                span: Span::new(0, 3),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 3)
                }
            }))
        )
    }

    #[test]
    fn recursive_new_expr() {
        let res = Parser::with_source("new new foo", "tests", true)
            .unwrap()
            .parse_member_or_new_expr(None, false);
        assert_eq!(res,
        Ok(Expr::New(NewExpr {
            span: Span::new(0, 11),
            target: Box::new(Expr::New(NewExpr {
                span: Span::new(4, 11),
                target: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(8, 11),
                    whitespace: LiteralWhitespace {
                        before: Span::new(8, 8),
                        after: Span::new(11, 11)
                    }
                })),
                args: None,
                whitespace: LiteralWhitespace {
                    before: Span::new(4, 4),
                    after: Span::new(7, 8)
                }
            })),
            args: None,
            whitespace: LiteralWhitespace {
                before: Span::new(0, 0),
                after: Span::new(3, 4)
            }
        })))
    }
}
