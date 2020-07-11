use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    pub fn parse_binary_expr(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };
        let left = self.parse_unary_expr(Some(leading_whitespace))?;
        self.parse_binary_expression_recursive(left, 0)
    }

    fn parse_binary_expression_recursive(
        &mut self,
        left: Expr,
        min_precedence: u8,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let peeked = peek!(self);

        match peeked {
            Some(TokenType::In) if !self.state.no_in => {},
            Some(TokenType::Instanceof) | Some(TokenType::BinOp(_)) => {}
            _ => return Ok(left),
        }

        if peeked.unwrap().precedence().unwrap() <= min_precedence {
            return Ok(left);
        }

        let before = self.whitespace(true)?;
        let op = self.cur_tok.token_type;
        self.advance_lexer(false)?;
        let after = self.whitespace(false)?;

        let right = Box::new({
            let left_but_actually_left_of_right = self.parse_unary_expr(None)?;
            self.parse_binary_expression_recursive(
                left_but_actually_left_of_right,
                op.precedence().unwrap(),
            )?
        });

        let expr = Expr::Binary(BinaryExpr {
            span: self.span(left.span().start, right.span().end),
            left: Box::new(left),
            right,
            op,
            whitespace: LiteralWhitespace {
                before,
                after,
            },
        });

        Ok(self.parse_binary_expression_recursive(expr, min_precedence)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::expr;
    use crate::lexer::token::*;
    use crate::parser::cst::expr::*;
    use crate::span;
    use crate::span::Span;

    #[test]
    fn simple_single_binary_expr() {
        assert_eq!(
            expr!("1 + 2"),
            Expr::Binary(BinaryExpr {
                span: span!("1 + 2", "1 + 2"),
                left: Box::new(Expr::Number(LiteralExpr {
                    span: span!("1 + 2", "1"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(1, 2),
                    }
                })),
                right: Box::new(Expr::Number(LiteralExpr {
                    span: span!("1 + 2", "2"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(4, 4),
                        after: Span::new(5, 5),
                    }
                })),
                op: TokenType::BinOp(BinToken::Add),
                whitespace: LiteralWhitespace {
                    before: Span::new(2, 2),
                    after: Span::new(3, 4)
                }
            })
        )
    }

    #[test]
    fn no_binop() {
        assert_eq!(
            expr!("foo.bar"),
            Expr::Member(MemberExpr {
                span: span!("foo.bar", "foo.bar"),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo.bar", "foo"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(3, 3),
                    }
                })),
                property: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo.bar", "bar"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(4, 4),
                        after: Span::new(7, 7),
                    }
                })),
                whitespace: LiteralWhitespace {
                    before: Span::new(3, 3),
                    after: Span::new(4, 4)
                }
            })
        )
    }

    #[test]
    fn precedence() {
        /* Multiply has a higher precedence, therefore it should be the branch, not the root.
         *
         *      BinAdd
         *    /       \
         *   1        BinMultiply
         *           /           \
         *          2             4
         */
        assert_eq!(
            expr!("1 + 2 * 4"),
            Expr::Binary(BinaryExpr {
                span: span!("1 + 2 * 4", "1 + 2 * 4"),
                left: Box::new(Expr::Number(LiteralExpr {
                    span: span!("1 + 2 * 4", "1"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(1, 2),
                    }
                })),
                right: Box::new(Expr::Binary(BinaryExpr {
                    span: span!("1 + 2 * 4", "2 * 4"),
                    left: Box::new(Expr::Number(LiteralExpr {
                        span: span!("1 + 2 * 4", "2"),
                        whitespace: LiteralWhitespace {
                            before: Span::new(4, 4),
                            after: Span::new(5, 6),
                        }
                    })),
                    right: Box::new(Expr::Number(LiteralExpr {
                        span: span!("1 + 2 * 4", "4"),
                        whitespace: LiteralWhitespace {
                            before: Span::new(8, 8),
                            after: Span::new(9, 9),
                        }
                    })),
                    op: TokenType::BinOp(BinToken::Multiply),
                    whitespace: LiteralWhitespace {
                        before: Span::new(6, 6),
                        after: Span::new(7, 8),
                    }
                })),
                op: TokenType::BinOp(BinToken::Add),
                whitespace: LiteralWhitespace {
                    before: Span::new(2, 2),
                    after: Span::new(3, 4),
                }
            })
        )
    }

    #[test]
    fn associativity() {
        assert_eq!(
            expr!("1 + 2 + 3"),
            Expr::Binary(BinaryExpr {
                span: span!("1 + 2 + 3", "1 + 2 + 3"),
                left: Box::new(Expr::Binary(BinaryExpr {
                    span: span!("1 + 2 + 3", "1 + 2"),
                    left: Box::new(Expr::Number(LiteralExpr {
                        span: span!("1 + 2 + 3", "1"),
                        whitespace: LiteralWhitespace {
                            before: Span::new(0, 0),
                            after: Span::new(1, 2),
                        }
                    })),
                    right: Box::new(Expr::Number(LiteralExpr {
                        span: span!("1 + 2 + 3", "2"),
                        whitespace: LiteralWhitespace {
                            before: Span::new(4, 4),
                            after: Span::new(5, 6),
                        }
                    })),
                    op: TokenType::BinOp(BinToken::Add),
                    whitespace: LiteralWhitespace {
                        before: Span::new(2, 2),
                        after: Span::new(3, 4),
                    }
                })),
                right: Box::new(Expr::Number(LiteralExpr {
                    span: span!("1 + 2 + 3", "3"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(8, 8),
                        after: Span::new(9, 9),
                    }
                })),
                op: TokenType::BinOp(BinToken::Add),
                whitespace: LiteralWhitespace {
                    before: Span::new(6, 6),
                    after: Span::new(7, 8)
                }
            })
        )
    }
}
