use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::ExpectedIdentifier;
use crate::parser::Parser;
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
        object_start: usize,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let peeked = self
            .peek_while(|t| [TokenType::Whitespace, TokenType::Linebreak].contains(&t.token_type))?
            .map(|t| t.token_type);

        match peeked {
            // This is required to correctly parse if the current token is a period
            t if self.cur_tok.token_type == TokenType::Period || t == Some(TokenType::Period) => {
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
                        span: Span::new(object_start, end),
                        object: Box::new(object),
                        property: Box::new(identifier),
                        whitespace: MemberExprWhitespace {
                            before_dot,
                            after_dot,
                        },
                    }),
                    object_start,
                )
            }

            _ => {
                self.lexer.reset();
                Ok(object)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cst::expr::*;
    use crate::parser::Parser;
    use crate::span::Span;

    #[test]
    fn dot_deref_suffixes() {
        let mut parser = Parser::with_source(" a\n . \n\n b. a", "tests", true).unwrap();
        let expr = parser.parse_primary_expr(None).unwrap();
        let member = parser.parse_suffixes(expr, 1);
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
        let member = parser.parse_suffixes(expr, 0);

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
        let member = parser.parse_suffixes(expr, 0);

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
}
