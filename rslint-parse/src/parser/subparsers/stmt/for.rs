use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::peek;
use crate::span::Span;

impl<'a> Parser<'a> {
    // This takes care of `for (...)`, `for (foo in bar)`, and *eventually* `for (foo of bar)`
    pub fn parse_for_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::For,
            "parse_for_stmt expects the current token to be For"
        );

        let for_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_for = self.whitespace(false)?;
        let open_paren_whitespace: LiteralWhitespace;
        let before_init = self.whitespace(true)?;

        if self.cur_tok.token_type != TokenType::ParenOpen {
            let err = self.error(ExpectedParen, "Expected an opening parenthesis after a `for` loop declaration, but found none")
                .primary(self.cur_tok.lexeme.to_owned(), "Expected an opening parenthesis here");

            self.errors.push(err);

            open_paren_whitespace = LiteralWhitespace {
                before: before_init,
                after: before_init.end.into(),
            };
        } else {
            self.advance_lexer(false)?;
            let after_open_paren = self.whitespace(false)?;
            open_paren_whitespace = LiteralWhitespace {
                before: before_init,
                after: after_open_paren,
            };
        }

        let (init, init_semicolon_whitespace, for_in_loop) = if self.cur_tok.token_type == TokenType::Semicolon {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            (
                None,
                LiteralWhitespace {
                    before: before_init,
                    after,
                },
                false,
            )
        } else {
            let ret = self.parse_for_init(before_init)?;
            (Some(ret.0), ret.1, ret.2)
        };

        // handle for in loops
        if for_in_loop {
            let before_in = self.whitespace(true)?;
            self.advance_lexer(false)?;
            let after_in = self.whitespace(false)?;

            let right = self.parse_expr(None)?;
            let before_close_paren = self.whitespace(true)?;
            let close_paren_whitespace: LiteralWhitespace;
            let body: Box<Stmt>;

            if self.cur_tok.token_type != TokenType::ParenClose {
                let err = self.error(ExpectedParen, "Expected a closing parenthesis after a `for...in` loop declaration, but found none")
                    .primary(self.cur_tok.lexeme.to_owned(), "Expected a closing parenthesis here");

                self.errors.push(err);
                
                close_paren_whitespace = LiteralWhitespace {
                    before: before_close_paren,
                    after: before_close_paren.end.into()
                };
                body = Box::new(self.parse_stmt(Some(before_close_paren))?);
            } else {
                self.advance_lexer(false)?;
                let after_close_paren = self.whitespace(false)?;

                close_paren_whitespace = LiteralWhitespace {
                    before: before_close_paren,
                    after: after_close_paren,
                };
                body = Box::new(self.parse_stmt(None)?); 
            }

            return Ok(Stmt::ForIn(ForInStmt {
                span: for_span + body.span(),
                left: init.unwrap(),
                right,
                open_paren_whitespace,
                close_paren_whitespace,
                in_whitespace: LiteralWhitespace {
                    before: before_in,
                    after: after_in,
                },
                body,
                for_whitespace: LiteralWhitespace {
                    before: leading_whitespace,
                    after: after_for,
                },
            }))
        }

        let mut test: Option<Expr> = None;
        let test_semicolon_whitespace;

        if peek!(self) == Some(TokenType::Semicolon) {
            let before = self.whitespace(true)?;
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            test_semicolon_whitespace = LiteralWhitespace { before, after };
        } else {
            if self.cur_tok.token_type == TokenType::ParenClose {
                let err = self
                    .error(
                        UnexpectedToken,
                        "Expected a test and update expression in a `for` loop, but found none",
                    )
                    .primary(
                        for_span + self.cur_tok.lexeme,
                        "This loop requires a test and update expression",
                    )
                    .secondary(
                        Span::from(self.cur_tok.lexeme.start - 1),
                        "Help: insert a semicolon here if you want an infinite loop",
                    );
                self.errors.push(err);
            }

            test_semicolon_whitespace = LiteralWhitespace {
                before: self.cur_tok.lexeme.end.into(),
                after: self.cur_tok.lexeme.end.into(),
            };

            test = Some(self.parse_expr(None)?);
        }

        let mut update = None;
        let before_update = self.whitespace(true)?;
        let close_paren_whitespace: LiteralWhitespace;

        if self.cur_tok.token_type == TokenType::ParenClose {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            close_paren_whitespace = LiteralWhitespace {
                before: before_update,
                after,
            };
        } else {
            update = Some(self.parse_expr(Some(before_update))?);

            if peek!(self) != Some(TokenType::ParenClose) {
                let err = self.error(ExpectedParen, "Expected a closing parenthesis after a `for` loop declaration, but found none")
                    .primary(update.as_ref().map(|x| x.span()).unwrap_or(&Span::from(test_semicolon_whitespace.after.end)).to_owned(), "Expected a closing parenthesis following this");

                self.errors.push(err);
            }

            let before = self.whitespace(true)?;
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            close_paren_whitespace = LiteralWhitespace { before, after };
        }

        let body = Box::new(self.parse_stmt(None)?);

        Ok(Stmt::For(ForStmt {
            span: for_span + body.span(),
            for_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_for,
            },
            open_paren_whitespace,
            close_paren_whitespace,
            init,
            test,
            update,
            body,
            init_semicolon_whitespace,
            test_semicolon_whitespace,
        }))
    }

    fn parse_for_init(
        &mut self,
        leading: Span,
    ) -> Result<(ForStmtInit, LiteralWhitespace, bool), ParserDiagnostic> {
        self.state.no_in = true;
        match self.cur_tok.token_type {
            TokenType::Var => {
                // We can cleverly use the var subparser to handle this, then if the semicolon is explicit, we unwrap
                // the whitespace for it, and return it, or if its implicit we return None
                let mut stmt = if let Stmt::Variable(data) = self.parse_var_stmt(Some(leading))? {
                    data
                } else {
                    unreachable!();
                };

                let semi_whitespace: LiteralWhitespace;

                if peek!(self) == Some(TokenType::In) {
                    // token by token, this matches closer with a standard for loop with a semi after the var, perhaps we should
                    // instead continue parsing like that and issue an unexpected token error after that?, however that would be a worse recovery
                    if let Semicolon::Explicit(data) = stmt.semi {
                        let err = self.error(UnexpectedToken, "`for...in` statements cannot have a semicolon after the variable declaration")
                            .primary(Span::from(data.before.end), "Remove this semicolon");

                        self.errors.push(err);
                        stmt.semi = Semicolon::Implicit;
                        stmt.span.end -= 1;
                    }

                    // The whitespace is ignored in for in statements, so we can just return a zeroed whitespace
                    return Ok((
                        ForStmtInit::Var(stmt),
                        LiteralWhitespace {
                            before: 0.into(),
                            after: 0.into(),
                        },
                        true,
                    ));
                }

                if let Semicolon::Explicit(data) = stmt.semi {
                    // In this case the semi actually belongs to the for statement
                    stmt.semi = Semicolon::Implicit;
                    stmt.span.end -= 1;
                    semi_whitespace = data;
                } else {
                    let err = self.error(ExpectedSemicolon, "Expected a semicolon after a `for` statement initializer, but found none")
                        .primary(stmt.span, "A semicolon is required after this");

                    self.errors.push(err);
                    let last = stmt.declared.last().expect("Tried to unwrap the last declarator to a var stmt, but somehow there wasn't one");
                    let after_semi = if last.value.is_some() {
                        Span::new(
                            last.value.as_ref().unwrap().span().end,
                            self.cur_tok.lexeme.start,
                        )
                    } else {
                        last.name.whitespace.after
                    };

                    semi_whitespace = LiteralWhitespace {
                        before: stmt.span.end.into(),
                        after: after_semi,
                    }
                }

                self.state.no_in = false;
                Ok((ForStmtInit::Var(stmt), semi_whitespace, false))
            }

            _ => {
                let expr = self.parse_expr(Some(leading))?;
                let semi_whitespace: LiteralWhitespace;

                if peek!(self) == Some(TokenType::In) {
                    return Ok((ForStmtInit::Expr(expr), LiteralWhitespace {
                        before: 0.into(),
                        after: 0.into(),
                    }, true));
                }

                if peek!(self) == Some(TokenType::Semicolon) {
                    let before = self.whitespace(true)?;
                    self.advance_lexer(false)?;
                    let after = self.whitespace(false)?;

                    semi_whitespace = LiteralWhitespace { before, after };
                } else {
                    let err = self.error(ExpectedSemicolon, "Expected a semicolon after a `for` statement initializer, but found none")
                        .primary(expr.span().to_owned(), "A semicolon is required after this");

                    self.errors.push(err);
                    semi_whitespace = LiteralWhitespace {
                        before: expr.span().end.into(),
                        after: Span::new(expr.span().end, self.cur_tok.lexeme.start),
                    };
                }

                self.state.no_in = false;
                Ok((ForStmtInit::Expr(expr), semi_whitespace, false))
            }
        }
    }
}
