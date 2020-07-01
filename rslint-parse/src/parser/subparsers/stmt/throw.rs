use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    pub fn parse_throw_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Throw,
            "parse_throw_stmt expects the current token to be Throw"
        );

        let throw_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_throw = self.whitespace(false)?;

        // A linebreak between the `throw` and the exception is not allowed
        // We could probably still parse, but this would go against the ECMA spec, so caution should be taken
        if self.cur_tok.token_type == TokenType::Linebreak {
            let mut err = self.error(ExpectedExpression, "Expected an expression following a `throw` statement, but found none")
                .primary(throw_span, "Expected an expression to throw following this");
            
            if peek!(self).unwrap_or(TokenType::EOF).starts_expr() {
                let err_expr = self.peek_while(|t| t.is_whitespace())?.unwrap();
                err = err.secondary(err_expr.lexeme, "Help: if you meant to throw this expression, remove the linebreak(s) before it");
            }

            return Err(err);
        }

        let arg = self.parse_expr(None)?;

        let semi = self.semi()?;

        if semi.is_none() {
            let err = self.error(ExpectedSemicolon, "An explicit semicolon was required after a throw statement, but none was found")
                .primary(throw_span + arg.span().to_owned(), "A semicolon is required to end this statement");
            
            self.errors.push(err);
        }

        let semicolon = semi.unwrap_or(Semicolon::Implicit);
        return Ok(Stmt::Throw(ThrowStmt {
            span: throw_span.extend(semicolon.offset()),
            arg,
            semi: semicolon,
            throw_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_throw,
            }
        }))
    }
}