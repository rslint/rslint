use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;

pub static PRIMARY_EXPR_ACCEPTABLE: [TokenType; 9] = [
    TokenType::LiteralRegEx,
    TokenType::LiteralString,
    TokenType::LiteralNumber,
    TokenType::Null,
    TokenType::True,
    TokenType::False,
    TokenType::Identifier,
    TokenType::This,
    TokenType::InvalidToken,
];

impl<'a> Parser<'a> {
    /// Parses a primary expression, expects the current token to be a whitespace or a potential primary expr token.  
    pub fn parse_primary_expr(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };
        let cur_lexeme = self.cur_tok.lexeme.to_owned();

        if self.lexer_done && self.cur_tok.is_whitespace() {
            return Err(self
                .error(
                    ExpectedExpression,
                    "Expected an expression before end of file",
                )
                .primary(cur_lexeme, "Expected an expression following this"));
        }

        if self.cur_tok.token_type == TokenType::BracketOpen {
            return self.parse_array_literal(Some(leading_whitespace));
        }

        if self.cur_tok.token_type == TokenType::BraceOpen {
            return self.parse_object_literal(Some(leading_whitespace));
        }

        if self.cur_tok.token_type == TokenType::ParenOpen {
            let open_paren_span = self.cur_tok.lexeme.to_owned();
            self.advance_lexer(false)?;
            let open_paren_trailing = self.whitespace(false)?;

            let grouped = self.parse_expr(None)?;

            let before_close_paren = self.whitespace(true)?;

            if self.cur_tok.token_type != TokenType::ParenClose {
                let err_tok = self.cur_tok.lexeme.to_owned();
                self.discard_recover(None, |t| *t != TokenType::BracketClose)
                    .map_err(|_| {
                        self.error(
                            UnmatchedBracket,
                            "Expected a closing parenthesis but found none",
                        )
                        .secondary(
                            open_paren_span.to_owned(),
                            "Grouping expression begins here",
                        )
                        .primary(err_tok.to_owned(), "Expected a closing parenthesis here")
                    })?;

                let err = self
                    .error(
                        UnmatchedBracket,
                        "Expected a closing parenthesis but found none",
                    )
                    .secondary(
                        open_paren_span.to_owned(),
                        "Grouping expression begins here",
                    )
                    .primary(err_tok, "Expected a closing parenthesis here");
                self.errors.push(err);
            }

            let close_paren_span = self.cur_tok.lexeme.to_owned();
            self.advance_lexer(false)?;
            let close_paren_trailing = self.whitespace(false)?;

            return Ok(Expr::Grouping(GroupingExpr {
                span: Span::new(open_paren_span.start, close_paren_span.end),
                expr: Box::new(grouped),
                opening_paren_whitespace: OperatorWhitespace {
                    before_op: leading_whitespace,
                    after_op: open_paren_trailing,
                },
                closing_paren_whitespace: OperatorWhitespace {
                    before_op: before_close_paren,
                    after_op: close_paren_trailing,
                },
            }));
        }

        if !PRIMARY_EXPR_ACCEPTABLE.contains(&self.cur_tok.token_type) {
            self.discard_recover(Some("Unexpected token, expected an expression"), |x| {
                !PRIMARY_EXPR_ACCEPTABLE.contains(&x)
            })?;
        }

        let expr_kind = match self.cur_tok.token_type {
            TokenType::LiteralRegEx => Expr::Regex,
            TokenType::LiteralString => Expr::String,
            TokenType::LiteralNumber => Expr::Number,
            TokenType::Null => Expr::Null,
            TokenType::True => Expr::True,
            TokenType::False => Expr::False,
            TokenType::InvalidToken | TokenType::Identifier => Expr::Identifier,
            TokenType::This => Expr::This,
            _ => unreachable!(),
        };
        let expr_tok = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let expr = expr_kind(LiteralExpr {
            span: expr_tok,
            whitespace: ExprWhitespace {
                before: leading_whitespace,
                after: self.whitespace(false)?,
            },
        });

        Ok(expr)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cst::expr::*;
    use crate::parser::Parser;
    use crate::span::Span;
    use crate::expr;

    #[test]
    fn errors_on_unterminated_string() {
        assert!(Parser::with_source(" \"a ", "tests", true)
            .unwrap()
            .parse_primary_expr(None)
            .is_err());
    }

    #[test]
    fn this_expr() {
        assert_eq!(
            Parser::with_source(" this ", "tests", true)
                .unwrap()
                .parse_primary_expr(None),
            Ok(Expr::This(LiteralExpr {
                span: Span::new(1, 5),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 1),
                    after: Span::new(5, 6),
                }
            }))
        )
    }

    #[test]
    fn invalid_token() {
        assert_eq!(
            Parser::with_source("  152aa   ", "tests", true)
                .unwrap()
                .parse_primary_expr(None),
            Ok(Expr::Identifier(LiteralExpr {
                span: Span::new(2, 7),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 2),
                    after: Span::new(7, 10),
                }
            }))
        );
    }

    #[test]
    fn primary_expr_leading_whitespace_with_linebreaks() {
        assert_eq!(
            Parser::with_source("\n\n \n \r\n 'yee haw' ", "tests", true)
                .unwrap()
                .parse_primary_expr(None),
            Ok(Expr::String(LiteralExpr {
                span: Span::new(8, 17),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 8),
                    after: Span::new(17, 18),
                }
            }))
        );
    }

    #[test]
    fn primary_expr_trailing_whitespace_with_linebreaks() {
        assert_eq!(
            Parser::with_source("  \n'oi' \n  ", "tests", true)
                .unwrap()
                .parse_primary_expr(None),
            Ok(Expr::String(LiteralExpr {
                span: Span::new(3, 7),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 3),
                    after: Span::new(7, 8),
                }
            }))
        );
    }

    #[test]
    fn no_whitespace() {
        assert_eq!(
            Parser::with_source("\"a\"", "tests", true)
                .unwrap()
                .parse_primary_expr(None),
            Ok(Expr::String(LiteralExpr {
                span: Span::new(0, 3),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 3)
                }
            }))
        )
    }

    #[test]
    #[should_panic]
    fn expected_expression() {
        expr!(";");
    }
}
