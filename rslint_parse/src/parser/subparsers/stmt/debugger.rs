use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_debugger_stmt(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Stmt, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Debugger,
            "parse_debugger_stmt expects the current token to be Debugger"
        );

        let debugger_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after = self.whitespace(false)?;

        let semi = self.semi()?;

        if semi.is_none() {
            let err = self.error(ExpectedSemicolon, "An explicit semicolon was required after a debugger statement, but none was found")
                .primary(debugger_span.to_owned(), "A semicolon is required to end this statement");

            self.errors.push(err);
        }

        let semicolon = semi.unwrap_or(Semicolon::Implicit);
        Ok(Stmt::Debugger(Debugger {
            span: debugger_span.extend(semicolon.offset()),
            whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after,
            },
            semi: semicolon,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cst::expr::*;
    use crate::parser::cst::stmt::*;
    use crate::parser::Parser;
    use crate::span;
    use crate::span::Span;
    use crate::stmt;

    #[test]
    fn debugger_stmt() {
        assert_eq!(
            stmt!(" debugger; "),
            Stmt::Debugger(Debugger {
                span: span!(" debugger; ", "debugger;"),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 1),
                    after: Span::new(9, 9),
                },
                semi: Semicolon::Explicit(LiteralWhitespace {
                    before: Span::new(9, 9),
                    after: Span::new(10, 11)
                })
            }),
        )
    }

    #[test]
    fn debugger_no_semi() {
        let mut parser = Parser::with_source(" debugger await", "tests", true).unwrap();
        let stmt = parser.parse_stmt().unwrap();

        assert_eq!(
            stmt,
            Stmt::Debugger(Debugger {
                span: span!(" debugger await", "debugger"),
                whitespace: LiteralWhitespace {
                    before: Span::new(0, 1),
                    after: Span::new(9, 10),
                },
                semi: Semicolon::Implicit
            }),
        );

        assert_eq!(parser.errors.len(), 1)
    }
}
