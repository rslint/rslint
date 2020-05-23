use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::{BinToken, TokenType};
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_unary_expr(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        match self.cur_tok.token_type {
            t @ TokenType::Increment | t @ TokenType::Decrement => {
                let start = self.cur_tok.lexeme.start;
                // Advance over the token
                self.advance_lexer(false)?;
                let after_op = self.whitespace(false)?;
                let object = self.parse_unary_expr(None)?;
                let end = object.span().end;

                if !object.is_valid_assign_target() {
                    let err = self
                        .error(
                            InvalidTargetExpression,
                            &format!("Invalid left hand side expression for prefix {:?}", t),
                        )
                        .secondary(
                            start..start + 2,
                            &format!("Prefix {:?} operation used here", t),
                        )
                        .primary(
                            object.span().to_owned(),
                            "Not a valid expression for the operator",
                        );
                    self.errors.push(err);
                }

                return Ok(Expr::Update(UpdateExpr {
                    span: Span::new(start, end),
                    prefix: true,
                    object: Box::new(object),
                    op: t,
                    whitespace: OperatorWhitespace {
                        before_op: leading_whitespace,
                        after_op,
                    },
                }));
            }

            t @ TokenType::Delete
            | t @ TokenType::Void
            | t @ TokenType::Typeof
            | t @ TokenType::BinOp(BinToken::Add)
            | t @ TokenType::BinOp(BinToken::Subtract)
            | t @ TokenType::BitwiseNot
            | t @ TokenType::LogicalNot => {
                let start = self.cur_tok.lexeme.start;
                self.advance_lexer(false)?;
                let after_op = self.whitespace(false)?;
                let object = self.parse_unary_expr(None)?;
                let end = object.span().end;
                // TODO: Handle strict mode delete
                return Ok(Expr::Unary(UnaryExpr {
                    span: Span::new(start, end),
                    object: Box::new(object),
                    op: t,
                    whitespace: OperatorWhitespace {
                        before_op: leading_whitespace,
                        after_op,
                    },
                }));
            }

            _ => {}
        }

        let object = self.parse_member_or_new_expr(Some(leading_whitespace), true)?;
        let start = object.span().start;
        let mut had_linebreak = self.cur_tok.token_type == TokenType::Linebreak;

        let next: Option<TokenType>;
        if self.cur_tok.token_type != TokenType::Increment
            && self.cur_tok.token_type != TokenType::Decrement
        {
            loop {
                match self.peek_lexer()?.map(|x| x.token_type) {
                    Some(TokenType::Whitespace) => continue,
                    Some(TokenType::Linebreak) => {
                        had_linebreak = true;
                        continue;
                    }
                    t @ _ => {
                        next = t;
                        break;
                    }
                }
            }
            self.lexer.reset();

            if next != Some(TokenType::Increment) && next != Some(TokenType::Decrement) {
                return Ok(object);
            }
        }

        // A linebreak between an expr and a postfix update is not allowed, therefore we need to return here
        if had_linebreak {
            return Ok(object);
        }

        let before_op = self.whitespace(true)?;
        let op_span = self.cur_tok.lexeme.to_owned();
        let op = self.cur_tok.token_type;
        let end = self.cur_tok.lexeme.end;
        self.advance_lexer(false)?;
        let after_op = self.whitespace(false)?;

        if !object.is_valid_assign_target() {
            let err = self
                .error(
                    InvalidTargetExpression,
                    &format!("Invalid left hand side expression for postfix {:?}", op),
                )
                .secondary(op_span, &format!("Postfix {:?} used here", op))
                .primary(
                    object.span().to_owned(),
                    "Not a valid expression for the operator",
                );
            self.errors.push(err);
        }

        Ok(Expr::Update(UpdateExpr {
            span: Span::new(start, end),
            prefix: false,
            object: Box::new(object),
            op,
            whitespace: OperatorWhitespace {
                before_op,
                after_op,
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::token::TokenType;
    use crate::parser::cst::expr::*;
    use crate::parser::Parser;
    use crate::span::Span;

    #[test]
    fn unary_prefix_update() {
        let mut parser = Parser::with_source("--foo \n++5", "tests", true).unwrap();
        let first = parser.parse_unary_expr(None);
        let second = parser.parse_unary_expr(None);
        assert_eq!(
            first,
            Ok(Expr::Update(UpdateExpr {
                span: Span::new(0, 5),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(2, 5),
                    whitespace: ExprWhitespace {
                        before: Span::new(2, 2),
                        after: Span::new(5, 6),
                    }
                })),
                prefix: true,
                op: TokenType::Decrement,
                whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 0),
                    after_op: Span::new(2, 2)
                }
            }))
        );
        assert_eq!(
            second,
            Ok(Expr::Update(UpdateExpr {
                span: Span::new(7, 10),
                object: Box::new(Expr::Number(LiteralExpr {
                    span: Span::new(9, 10),
                    whitespace: ExprWhitespace {
                        before: Span::new(9, 9),
                        after: Span::new(10, 10),
                    }
                })),
                prefix: true,
                op: TokenType::Increment,
                whitespace: OperatorWhitespace {
                    before_op: Span::new(6, 7),
                    after_op: Span::new(9, 9)
                }
            }))
        );
    }

    #[test]
    fn postfix_unary_valid_target() {
        let mut parser = Parser::with_source("mark++", "tests", true).unwrap();
        let res = parser.parse_unary_expr(None).unwrap();
        assert_eq!(
            res,
            Expr::Update(UpdateExpr {
                span: Span::new(0, 6),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(0, 4),
                    whitespace: ExprWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(4, 4),
                    }
                })),
                prefix: false,
                op: TokenType::Increment,
                whitespace: OperatorWhitespace {
                    before_op: Span::new(4, 4),
                    after_op: Span::new(6, 6),
                }
            })
        );
    }

    #[test]
    fn postfix_unary_with_whitespace() {
        let mut parser = Parser::with_source("\nmk -- \n\n", "tests", true).unwrap();
        let res = parser.parse_unary_expr(None).unwrap();
        assert_eq!(
            res,
            Expr::Update(UpdateExpr {
                span: Span::new(1, 6),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(1, 3),
                    whitespace: ExprWhitespace {
                        before: Span::new(0, 1),
                        after: Span::new(3, 4)
                    }
                })),
                prefix: false,
                op: TokenType::Decrement,
                whitespace: OperatorWhitespace {
                    before_op: Span::new(4, 4),
                    after_op: Span::new(6, 7),
                }
            })
        )
    }

    #[test]
    fn postfix_unary_invalid_target() {
        let mut parser = Parser::with_source("true++", "tests", true).unwrap();
        let res = parser.parse_unary_expr(None).unwrap();
        assert_eq!(
            res,
            Expr::True(LiteralExpr {
                span: Span::new(0, 4),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(4, 4),
                }
            })
        );
        assert_eq!(parser.errors.len(), 1);
    }

    #[test]
    fn prefix_update_valid_target() {
        let mut parser = Parser::with_source(" ++ foo ", "tests", true).unwrap();
        let res = parser.parse_unary_expr(None).unwrap();
        assert_eq!(
            res,
            Expr::Update(UpdateExpr {
                span: Span::new(1, 7),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(4, 7),
                    whitespace: ExprWhitespace {
                        before: Span::new(4, 4),
                        after: Span::new(7, 8)
                    }
                })),
                prefix: true,
                op: TokenType::Increment,
                whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 1),
                    after_op: Span::new(3, 4),
                }
            })
        )
    }

    #[test]
    fn prefix_unary() {
        let mut parser = Parser::with_source("delete the_world", "tests", true).unwrap();
        let res = parser.parse_unary_expr(None).unwrap();
        assert_eq!(
            res,
            Expr::Unary(UnaryExpr {
                span: Span::new(0, 16),
                object: Box::new(Expr::Identifier(LiteralExpr {
                    span: Span::new(7, 16),
                    whitespace: ExprWhitespace {
                        before: Span::new(7, 7),
                        after: Span::new(16, 16)
                    }
                })),
                op: TokenType::Delete,
                whitespace: OperatorWhitespace {
                    before_op: Span::new(0, 0),
                    after_op: Span::new(6, 7)
                }
            })
        )
    }
}
