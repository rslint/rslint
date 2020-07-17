use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_with_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::With,
            "parse_with_stmt expects the current token to be `with`"
        );

        let with_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_with = self.whitespace(false)?;

        let open_paren_whitespace;
        let object;
        let before_open_paren = self.whitespace(true)?;

        if self.cur_tok.token_type != TokenType::ParenOpen {
            let err = self.error(ExpectedParen, "Expected an opening parenthesis following a `with` statement declaration, but found none")
                .primary(self.cur_tok.lexeme.to_owned(), "Expected an opening parenthesis here");

            self.errors.push(err);

            open_paren_whitespace = LiteralWhitespace {
                before: before_open_paren,
                after: before_open_paren.end.into()
            };

            object = self.parse_expr(Some(before_open_paren))?;
        } else {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            open_paren_whitespace = LiteralWhitespace {
                before: before_open_paren,
                after,
            };

            object = self.parse_expr(None)?;
        }
        
        let close_paren_whitespace;
        let body;
        let before_close_paren = self.whitespace(true)?;

        if self.cur_tok.token_type != TokenType::ParenClose {
            let err = self.error(ExpectedParen, "Expected a closing parenthesis following a `with` statement object, but found none")
                .primary(self.cur_tok.lexeme.to_owned(), "Expected a closing parenthesis here");

            self.errors.push(err);

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after: before_close_paren.end.into()
            };

            body = self.parse_stmt(Some(before_close_paren))?;
        } else {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after,
            };

            body = self.parse_stmt(None)?;
        }

        if self.state.strict.is_some() {
            let err = self.error(DisallowedStatement, "`with` statements are not allowed in strict mode code")
                .primary(with_span + body.span(), "This statement is not allowed");

            self.errors.push(err);
        }

        Ok(Stmt::With(WithStmt {
            span: with_span + body.span(),
            with_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_with,
            },
            open_paren_whitespace,
            close_paren_whitespace,
            object,
            body: Box::new(body),
        }))
    }
}