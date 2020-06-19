use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;

impl<'a> Parser<'a> {
    /// An IdentifierName production may include a keyword too, productions such as `new foo().let` are valid.
    pub fn parse_identifier_name(
        &mut self,
        leading_whitespace: Option<Span>,
    ) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_ws = if leading_whitespace.is_some() {
            leading_whitespace.unwrap()
        } else {
            self.whitespace(true)?
        };

        if self.cur_tok.token_type != TokenType::Identifier && !self.cur_tok.token_type.is_keyword()
        {
            // Although `{}.6` or `{}."a"` is invalid, we can recover from this by assuming the user meant to get the property with square bracket notation
            if [TokenType::LiteralNumber, TokenType::LiteralString]
                .contains(&self.cur_tok.token_type)
            {
                let error = self
                    .error(
                        ExpectedIdentifier,
                        &format!(
                            "Expected an identifier name, found a {}",
                            stringify!(self.cur_tok.token_type)
                        ),
                    )
                    .primary(
                        self.cur_tok.lexeme.to_owned(),
                        "Invalid as an identifier name",
                    );

                self.errors.push(error);
            } else {
                self.discard_recover(
                    Some("Expected an identifier, but found an extraneous token"),
                    |t| t != &TokenType::Identifier && !t.is_keyword(),
                )?;
            }
        }

        let span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let trailing_whitespace = self.whitespace(false)?;
        Ok(Expr::Identifier(LiteralExpr {
            span,
            whitespace: LiteralWhitespace {
                before: leading_ws,
                after: trailing_whitespace,
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cst::expr::*;
    use crate::parser::Parser;
    use crate::span::Span;

    #[test]
    fn identifier_name() {
        assert_eq!(
            Parser::with_source(" \nbeans \n\n", "tests", true)
                .unwrap()
                .parse_identifier_name(None)
                .unwrap(),
            Expr::Identifier(LiteralExpr {
                span: Span::new(2, 7),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 2),
                    after: Span::new(7, 8)
                }
            })
        )
    }

    #[test]
    fn identifier_name_with_keyword() {
        assert_eq!(
            Parser::with_source(" \nclass \n\n", "tests", true)
                .unwrap()
                .parse_identifier_name(None)
                .unwrap(),
            Expr::Identifier(LiteralExpr {
                span: Span::new(2, 7),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 2),
                    after: Span::new(7, 8)
                }
            })
        )
    }

    #[test]
    fn identifier_name_string_recovery() {
        let mut parser = Parser::with_source(" \n'yee' \n\n", "tests", true).unwrap();
        assert_eq!(
            parser.parse_identifier_name(None).unwrap(),
            Expr::Identifier(LiteralExpr {
                span: Span::new(2, 7),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 2),
                    after: Span::new(7, 8)
                }
            })
        );
        assert_eq!(parser.errors.len(), 1);
    }

    #[test]
    fn identifier_name_number_recovery() {
        let mut parser = Parser::with_source(" \n12345 \n\n", "tests", true).unwrap();
        assert_eq!(
            parser.parse_identifier_name(None).unwrap(),
            Expr::Identifier(LiteralExpr {
                span: Span::new(2, 7),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 2),
                    after: Span::new(7, 8)
                }
            })
        );
        assert_eq!(parser.errors.len(), 1);
    }

    #[should_panic]
    #[test]
    fn invalid_identifier_name() {
        Parser::with_source(" \n/aaa/g \n\n", "tests", false)
            .unwrap()
            .parse_identifier_name(None)
            .unwrap();
    }
}
