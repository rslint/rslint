use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::Parser;
use crate::parser::error::ParseDiagnosticType::CommaWithoutRightExpression;
use crate::span::Span;

impl<'a> Parser<'a> {
    /// Parses a single expression or a comma separated list of expressions such as `foo, bar`
    // TODO: recover from multiple erroneous commas too, a cheap way to do this may be to advance until a linebreak or other token
    pub fn parse_expr(&mut self, leading: Option<Span>) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        let mut first = true;
        let mut peeked;
        let mut exprs: Vec<Expr> = vec![];
        let mut whitespaces: Vec<LiteralWhitespace> = vec![];

        loop {
            let expr = if first {
                first = false;
                self.parse_assign_expr(Some(leading_whitespace.to_owned()))?
            } else {
                self.parse_assign_expr(None)?
            };

            exprs.push(expr);

            if self.cur_tok.token_type == TokenType::Comma {
                peeked = Some(self.cur_tok.token_type);
            } else {
                peeked = self.peek_while(|x| x.is_whitespace())?.map(|x| x.token_type);
            }
    
            if peeked == Some(TokenType::Comma) {
                let before = self.whitespace(true)?;
                let comma_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after = self.whitespace(false)?;

                whitespaces.push(LiteralWhitespace {
                    before,
                    after
                });

                let peeked_expr;
    
                if self.cur_tok.token_type.starts_expr() {
                    peeked_expr = Some(self.cur_tok.token_type);
                } else {
                    peeked_expr = self.peek_while(|x| x.is_whitespace())?.map(|x| x.token_type);
                }
    
                if peeked_expr.is_none() || !peeked_expr.unwrap().starts_expr() {
                    let err = self.error(CommaWithoutRightExpression, "Expected a second expression after a comma operator, but found none")
                        .primary(comma_span, "Expected a second expression following this")
                        .help("Help: The comma operator expects a left and right expression");
                    
                    self.errors.push(err);
                    break;
                }
    
                continue;
    
            } else { break; }
        }

        // If theres multiple expressions that means there were one or more commas
        if exprs.len() > 1 {
            return Ok(Expr::Sequence(SequenceExpr {
                span: self.span(exprs[0].span().start, exprs.last().unwrap().span().end),
                exprs,
                comma_whitespace: whitespaces,
            }));
        } else {
            return Ok(exprs.drain(..).next().unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr;
    use crate::parser::cst::expr::*;
    use crate::span;
    use crate::span::Span;
    use crate::parser::Parser;

    #[test]
    fn simple_single_comma() {
        assert_eq!(expr!("foo, 2"),
        Expr::Sequence(SequenceExpr {
            span: span!("foo, 2", "foo, 2"),
            exprs: vec![
                Expr::Identifier(LiteralExpr {
                    span: span!("foo, 2", "foo"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(3, 3),
                    }
                }),
                Expr::Number(LiteralExpr {
                    span: span!("foo, 2", "2"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(5, 5),
                        after: Span::new(6, 6)
                    }
                }),
            ],
            comma_whitespace: vec![
                LiteralWhitespace {
                    before: Span::new(3, 3),
                    after: Span::new(4, 5),
                }
            ]
        }))
    }

    #[test]
    fn single_expr_no_comma() {
        assert_eq!(expr!(" 2\n"),
        Expr::Number(LiteralExpr {
            span: Span::new(1, 2),
            whitespace: LiteralWhitespace {
                before: Span::new(0, 1),
                after: Span::new(2, 2)
            }
        }))
    }

    #[test]
    fn multiple_commas() {
        assert_eq!(expr!("new foo, bar, /aa/g"),
        Expr::Sequence(SequenceExpr {
            span: span!("new foo, bar, /aa/g", "new foo, bar, /aa/g"),
            exprs: vec![
                Expr::New(NewExpr {
                    span: span!("new foo, bar, /aa/g", "new foo"),
                    target: Box::new(Expr::Identifier(LiteralExpr {
                        span: span!("new foo, bar, /aa/g", "foo"),
                        whitespace: LiteralWhitespace {
                            before: Span::new(4, 4),
                            after: Span::new(7, 7)
                        }
                    })),
                    args: None,
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(3, 4)
                    }
                }),
                Expr::Identifier(LiteralExpr {
                    span: span!("new foo, bar, /aa/g", "bar"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(9, 9),
                        after: Span::new(12, 12)
                    }
                }),
                Expr::Regex(LiteralExpr {
                    span: span!("new foo, bar, /aa/g", "/aa/g"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(14, 14),
                        after: Span::new(19, 19)
                    }
                })
            ],
            comma_whitespace: vec![
                LiteralWhitespace {
                    before: Span::new(7, 7),
                    after: Span::new(8, 9)
                },
                LiteralWhitespace {
                    before: Span::new(12, 12),
                    after: Span::new(13, 14)
                }
            ]
        }))
    }

    #[test]
    fn invalid_comma_recovery() {
        let mut parser = Parser::with_source("foo,", "tests", false).unwrap();
        let expr = parser.parse_expr(None).unwrap();
        
        assert_eq!(expr,
        Expr::Identifier(LiteralExpr {
            span: span!("foo,", "foo"),
            whitespace: LiteralWhitespace {
                before: Span::new(0, 0),
                after: Span::new(3, 3)
            }
        }));

        assert_eq!(parser.errors.len(), 1);
    }
}