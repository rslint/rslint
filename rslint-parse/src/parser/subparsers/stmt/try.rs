use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::peek;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_try_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Try,
            "parse_try_stmt expects the current token to be Try"
        );
        let try_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_try = self.whitespace(false)?;

        let before_brace = self.whitespace(true)?;

        // Try statements only allow a block statement for some reason ¯\_(ツ)_/¯
        // TODO: we could possibly recover by still parsing a stmt and just mocking a block statement, or change the test type to be a generic `stmt`
        if self.cur_tok.token_type != TokenType::BraceOpen {
            let err = self
                .error(
                    ExpectedBrace,
                    &format!(
                        "Expected a block statement after a `try` statement, instead found `{}`",
                        self.cur_tok.lexeme.content(self.source)
                    ),
                )
                .primary(
                    self.cur_tok.lexeme.to_owned(),
                    "Expected an opening brace here",
                );

            return Err(err);
        }

        let test = if let Stmt::Block(data) = self.parse_block_stmt(Some(before_brace))? {
            data
        } else {
            unreachable!();
        };

        let mut handler: Option<CatchClause> = None;
        let mut finalizer: Option<BlockStmt> = None;
        let mut final_whitespace: Option<LiteralWhitespace> = None;

        if peek!(self) == Some(TokenType::Catch) {
            handler = Some(self.parse_catch_clause()?);
        }

        // A catch and a final by themselves are allowed, a catch then a finally is allowed too,
        // However, finally then catch isnt, so this is an easy way to handle all cases
        if peek!(self) == Some(TokenType::Finally) {
            let final_clause = self.parse_finally_clause()?;
            finalizer = Some(final_clause.1);
            final_whitespace = Some(final_clause.0);
        }

        let end = if finalizer.is_some() {
            finalizer.as_ref().unwrap().span.end
        } else if handler.is_some() {
            handler.as_ref().unwrap().span.end
        } else {
            test.span.end
        };

        Ok(Stmt::Try(TryStmt {
            span: Span::new(try_span.start, end),
            try_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_try,
            },
            test,
            handler,
            finalizer,
            final_whitespace,
        }))
    }

    fn parse_finally_clause(
        &mut self,
    ) -> Result<(LiteralWhitespace, BlockStmt), ParserDiagnostic> {
        let before_finally = self.whitespace(true)?;
        self.advance_lexer(false)?;
        let after_finally = self.whitespace(false)?;

        let before_block = self.whitespace(true)?;
        if self.cur_tok.token_type != TokenType::BraceOpen {
            let err = self
                .error(
                    ExpectedBrace,
                    &format!(
                        "Expected a block statement after a `try` statement, instead found `{}`",
                        self.cur_tok.lexeme.content(self.source)
                    ),
                )
                .primary(
                    self.cur_tok.lexeme.to_owned(),
                    "Expected an opening brace here",
                );

            return Err(err);
        }

        let body = if let Stmt::Block(data) = self.parse_block_stmt(Some(before_block))? {
            data
        } else {
            unreachable!();
        };

        Ok((
            LiteralWhitespace {
                before: before_finally,
                after: after_finally,
            },
            body,
        ))
    }

    fn parse_catch_clause(&mut self) -> Result<CatchClause, ParserDiagnostic> {
        let before_catch = self.whitespace(true)?;
        let catch_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_catch = self.whitespace(false)?;

        let before_open_paren = self.whitespace(true)?;
        let open_paren_whitespace: LiteralWhitespace;
        let before_param: Span;

        if self.cur_tok.token_type != TokenType::ParenOpen {
            let err = self
                .error(
                    ExpectedParen,
                    "Expected a parenthesis following a `catch` clause, but found none",
                )
                .primary(
                    self.cur_tok.lexeme.to_owned(),
                    "Expected an opening parenthesis here",
                );

            if self.cur_tok.token_type == TokenType::BraceOpen {
                return Err(err);
            }

            self.errors.push(err);

            open_paren_whitespace = LiteralWhitespace {
                before: before_open_paren,
                after: before_open_paren.end.into(),
            };

            before_param = before_open_paren;
        } else {
            self.advance_lexer(false)?;
            let after_open_paren = self.whitespace(false)?;

            open_paren_whitespace = LiteralWhitespace {
                before: before_open_paren,
                after: after_open_paren,
            };

            before_param = self.whitespace(true)?;
        }

        if self.cur_tok.token_type != TokenType::Identifier {
            let err = self.error(UnexpectedToken, &format!("The parameter to a `catch` clause must be an identifier, yet `{}` was found", self.cur_tok.lexeme.content(self.source)))
                .primary(self.cur_tok.lexeme.to_owned(), "Expected an identifier here");

            return Err(err);
        }

        let param_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let param = LiteralExpr {
            span: param_span,
            whitespace: LiteralWhitespace {
                before: before_param,
                after: self.whitespace(false)?,
            },
        };

        let before_close_paren = self.whitespace(true)?;
        let close_paren_whitespace: LiteralWhitespace;
        let before_body: Span;

        if self.cur_tok.token_type != TokenType::ParenClose {
            let err = self
                .error(
                    ExpectedParen,
                    "Expected a closing parenthesis following a `catch` parameter, but found none",
                )
                .primary(
                    self.cur_tok.lexeme.to_owned(),
                    "Expected a closing parenthesis here",
                );

            self.errors.push(err);
            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after: before_close_paren.end.into(),
            };

            before_body = before_close_paren;
        } else {
            self.advance_lexer(false)?;
            let after_close_paren = self.whitespace(false)?;

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after: after_close_paren,
            };

            before_body = self.whitespace(true)?;
        }

        if self.cur_tok.token_type != TokenType::BraceOpen {
            let err = self
                .error(
                    ExpectedBrace,
                    &format!(
                        "Expected a block statement after a `try` statement, instead found `{}`",
                        self.cur_tok.lexeme.content(self.source)
                    ),
                )
                .primary(
                    self.cur_tok.lexeme.to_owned(),
                    "Expected an opening brace here",
                );

            return Err(err);
        }

        let body = if let Stmt::Block(data) = self.parse_block_stmt(Some(before_body))? {
            data
        } else {
            unreachable!();
        };

        Ok(CatchClause {
            span: catch_span + body.span,
            catch_whitespace: LiteralWhitespace {
                before: before_catch,
                after: after_catch,
            },
            open_paren_whitespace,
            close_paren_whitespace,
            param,
            body,
        })
    }
}
