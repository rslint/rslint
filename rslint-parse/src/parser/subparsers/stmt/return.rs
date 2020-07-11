use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    pub fn parse_return_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Return,
            "parse_return_stmt expects the current token to be Return"
        );

        let return_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_return = self.whitespace(false)?;
        let mut value = None;

        if self.cur_tok.token_type != TokenType::Linebreak && peek!(self).unwrap_or(TokenType::EOF).starts_expr() {
            value = Some(self.parse_expr(None)?);
        }
        let value_offset = value.as_ref().map(|x| x.span().size() + 1).unwrap_or(0);
        let semi = self.semi()?;

        if semi.is_none() {
            let err = self.error(ExpectedSemicolon, "An explicit semicolon was required after a return statement, but none was found")
                .primary(return_span.extend(value_offset), "A semicolon is required to end this statement");
            
            self.errors.push(err);
        }
        let semicolon = semi.unwrap_or(Semicolon::Implicit);

        let stmt_span = return_span.extend(value_offset + semicolon.offset());

        if !self.state.in_function {
            let err = self.error(InvalidReturn, "Return statements are not allowed outside of a function body")
                .primary(stmt_span, "Invalid in this context");

            self.errors.push(err);
        }

        Ok(Stmt::Return(ReturnStmt {
            span: stmt_span,
            return_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_return,
            },
            value,
            semi: semicolon,
        }))
    }
}