use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::ConditionalWithoutColon;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    /// Parses a conditional (ternary) expression.  
    /// If the colon is missing, for recovery the parser assumes the colon directly touches the alternate expression  
    /// ```js
    ///           // v Colon assumed to be here, colon has no trailing whitespace
    /// expr ? cond  alt
    ///         // ^^ conditional still consumes valid trailing whitespace  
    /// ```
    pub fn parse_conditional_expr(
        &mut self,
        leading_whitespace: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_ws = if leading_whitespace.is_some() {
            leading_whitespace.unwrap()
        } else {
            self.whitespace(true)?
        };

        let condition = self.parse_binary_expr(Some(leading_ws))?;

        if peek!(self) == Some(TokenType::QuestionMark) {
            let before_qmark = self.whitespace(true)?;
            let qmark_span = self.cur_tok.lexeme.to_owned();
            self.advance_lexer(false)?;
            let after_qmark = self.whitespace(false)?;

            let if_true = Box::new(self.parse_assign_expr(None)?);

            let before_colon = self.whitespace(true)?;
            let after_colon;

            // Recover by assuming a colon is there
            if self.cur_tok.token_type != TokenType::Colon {
                let err = self
                    .error(
                        ConditionalWithoutColon,
                        "Invalid conditional expression missing a colon",
                    )
                    .primary(
                        if_true.span().to_owned(),
                        "Expected a colon with an alternate expression following this",
                    )
                    .secondary(qmark_span, "Conditional expression begins here");

                self.errors.push(err);
                after_colon = self.span(before_colon.end, before_colon.end);
            } else {
                self.advance_lexer(false)?;
                after_colon = self.whitespace(false)?;
            }

            let if_false = Box::new(self.parse_assign_expr(None)?);

            let span = self.span(condition.span().start, if_false.span().end);

            Ok(Expr::Conditional(ConditionalExpr {
                span,
                condition: Box::new(condition),
                if_true,
                if_false,
                whitespace: ConditionalWhitespace {
                    before_qmark,
                    after_qmark,
                    before_colon,
                    after_colon,
                },
            }))
        } else {
            Ok(condition)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::expr;
    use crate::parser::cst::expr::*;
    use crate::span;
    use crate::span::Span;

    #[test]
    fn conditional_simple() {
        assert_eq!(
            expr!("foo ? bar : baz"),
            Expr::Conditional(ConditionalExpr {
                span: span!("foo ? bar : baz", "foo ? bar : baz"),
                condition: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo ? bar : baz", "foo"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(3, 4),
                    }
                })),
                if_true: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo ? bar : baz", "bar"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(6, 6),
                        after: Span::new(9, 10),
                    }
                })),
                if_false: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo ? bar : baz", "baz"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(12, 12),
                        after: Span::new(15, 15)
                    }
                })),
                whitespace: ConditionalWhitespace {
                    before_qmark: Span::new(4, 4),
                    after_qmark: Span::new(5, 6),
                    before_colon: Span::new(10, 10),
                    after_colon: Span::new(11, 12),
                }
            })
        );
    }

    #[test]
    fn conditional_no_whitespace() {
        assert_eq!(
            expr!("foo?bar:baz"),
            Expr::Conditional(ConditionalExpr {
                span: span!("foo?bar:baz", "foo?bar:baz"),
                condition: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo?bar:baz", "foo"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(3, 3),
                    }
                })),
                if_true: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo?bar:baz", "bar"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(4, 4),
                        after: Span::new(7, 7),
                    }
                })),
                if_false: Box::new(Expr::Identifier(LiteralExpr {
                    span: span!("foo?bar:baz", "baz"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(8, 8),
                        after: Span::new(11, 11)
                    }
                })),
                whitespace: ConditionalWhitespace {
                    before_qmark: Span::new(3, 3),
                    after_qmark: Span::new(4, 4),
                    before_colon: Span::new(7, 7),
                    after_colon: Span::new(8, 8),
                }
            })
        );
    }

    #[test]
    fn primary_as_conditional() {
        assert_eq!(
            expr!("o"),
            Expr::Identifier(LiteralExpr {
                span: span!("o", "o"),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(1, 1)
                }
            })
        );
    }
}
