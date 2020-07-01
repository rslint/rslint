use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    pub fn parse_switch_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Switch,
            "parse_switch_stmt expects the current token to be Switch"
        );

        self.state.in_switch_stmt = true;

        let switch_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_switch = self.whitespace(false)?;

        let open_paren_whitespace: LiteralWhitespace;
        let test: Expr;
        let before_open_paren = self.whitespace(true)?;
        if self.cur_tok.token_type != TokenType::ParenOpen {
            let err = self.error(ExpectedParen, "Expected an opening parenthesis between a `switch` and its test, but found none")
                .primary(switch_span, "Expected an opening parenthesis following this");
            
            // Same as if statement, `switch {` can cause a lot of issues for recovery, so for now we just return
            if self.cur_tok.token_type == TokenType::BraceOpen {
                return Err(err);
            }

            self.errors.push(err);

            open_paren_whitespace = LiteralWhitespace {
                before: before_open_paren,
                after: Span::new(before_open_paren.end, before_open_paren.end),
            };
            test = self.parse_expr(Some(before_open_paren))?;
        } else {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;
            open_paren_whitespace = LiteralWhitespace {
                before: before_open_paren,
                after,
            };
            test = self.parse_expr(None)?;
        }

        let before_close_paren = self.whitespace(true)?;
        let close_paren_whitespace: LiteralWhitespace;
        let before_open_brace: Span;

        if self.cur_tok.token_type != TokenType::ParenClose {
            let err = self.error(ExpectedParen, "Expected a closing parenthesis after a switch statement test, but found none")
                .primary(test.span().to_owned(), "This test must be enclosed in parentheses");
            
            self.errors.push(err);

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after: Span::new(before_close_paren.end, before_close_paren.end),
            };
            before_open_brace = before_close_paren;
        } else {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after,
            };
            before_open_brace = self.whitespace(true)?;
        }

        if self.cur_tok.token_type != TokenType::BraceOpen {
            let err = self.error(ExpectedBrace, "Expected a brace following a `switch` statement's test, but found none")
                .primary(self.cur_tok.lexeme.to_owned(), "Expected an opening brace here");

            return Err(err);
        }

        self.advance_lexer(false)?;
        let after_open_brace = self.whitespace(false)?;

        let cases = self.parse_case_list()?;

        let before_close_brace = self.whitespace(true)?;

        if self.cur_tok.token_type != TokenType::BraceClose {
            let err = self.error(ExpectedBrace, &format!("Expected a closing brace after a `switch` statement, instead found `{}`", self.cur_tok.lexeme.content(self.source)))
                .primary(self.cur_tok.lexeme.to_owned(), "Expected a closing brace here");
            
            return Err(err);
        }
        let close_brace_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_close_brace = self.whitespace(false)?;

        self.state.in_switch_stmt = false;

        return Ok(Stmt::Switch(SwitchStmt {
            span: switch_span + close_brace_span,
            switch_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_switch,
            },
            cases,
            open_paren_whitespace,
            close_paren_whitespace,
            test,
            open_brace_whitespace: LiteralWhitespace {
                before: before_open_brace,
                after: after_open_brace,
            },
            close_brace_whitespace: LiteralWhitespace {
                before: before_close_brace,
                after: after_close_brace,
            },
            
        }))
    }

    fn parse_case_list(&mut self) -> Result<Vec<Case>, ParserDiagnostic<'a>> {
        const ENDS_PREV_CASE: [TokenType; 4] = [TokenType::BraceClose, TokenType::Default, TokenType::Case, TokenType::EOF];

        let mut cases: Vec<Case> = vec![];

        loop {
            match peek!(self) {
                Some(TokenType::BraceClose) => return Ok(cases),
                Some(TokenType::Case) => {
                    let before = self.whitespace(true)?;
                    let case_span = self.cur_tok.lexeme.to_owned();
                    self.advance_lexer(false)?;
                    let after = self.whitespace(false)?;

                    // TODO: recover from `case :` and `case a { .. }`
                    let expr = self.parse_expr(None)?;

                    let before_colon = self.whitespace(true)?;
                    if self.cur_tok.token_type != TokenType::Colon {
                        let err = self.error(ExpectedColon, "A colon was expected following a `case` inside a `switch` statement, but none was found")
                            .primary(case_span, "A colon is required following this");

                        return Err(err);
                    }
                    let colon_span = self.cur_tok.lexeme.to_owned();
                    self.advance_lexer(false)?;
                    let after_colon = self.whitespace(false)?;

                    let cons = if ENDS_PREV_CASE.contains(&peek!(self).unwrap_or(TokenType::EOF)) {
                        vec![]
                    } else {
                        self.parse_stmt_list(None, Some(&ENDS_PREV_CASE))?
                    };
                    cases.push(Case {
                        span: case_span + colon_span,
                        default: false,
                        whitespace: LiteralWhitespace {
                            before,
                            after,
                        },
                        colon_whitespace: LiteralWhitespace {
                            before: before_colon,
                            after: after_colon,
                        },
                        test: Some(expr),
                        cons,
                    });
                },

                Some(TokenType::Default) => {
                    let mut should_discard = false;
                    let before = self.whitespace(true)?;
                    let default_span = self.cur_tok.lexeme.to_owned();
                    self.advance_lexer(false)?;
                    let after = self.whitespace(false)?;

                    // Duplicate `default` cases are invalid
                    if cases.iter().any(|case| case.default) {
                        let err = self.error(MultipleDefaults, "A `switch` statement may not contain multiple `default` cases")
                            .secondary(cases.iter().find(|case| case.default).unwrap().span, "First `default` case is defined here")
                            .primary(default_span, "Another `default` case here is invalid");
                        
                        self.errors.push(err);
                        should_discard = true;
                    }

                    let before_colon = self.whitespace(true)?;

                    if self.cur_tok.token_type != TokenType::Colon {
                        let err = self.error(ExpectedColon, "A colon was expected following a `default` inside a `switch` statement, but none was found")
                            .primary(default_span, "A colon is required following this");

                        return Err(err);
                    }

                    let colon_span = self.cur_tok.lexeme.to_owned();
                    self.advance_lexer(false)?;
                    let after_colon = self.whitespace(false)?;

                    let cons = if ENDS_PREV_CASE.contains(&peek!(self).unwrap_or(TokenType::EOF)) {
                        vec![]
                    } else {
                        self.parse_stmt_list(None, Some(&ENDS_PREV_CASE))?
                    };

                    if !should_discard {
                        cases.push(Case {
                            span: default_span + colon_span,
                            default: true,
                            whitespace: LiteralWhitespace {
                                before,
                                after,
                            },
                            colon_whitespace: LiteralWhitespace {
                                before: before_colon,
                                after: after_colon,
                            },
                            test: None,
                            cons,
                        });
                    }
                },

                _ => return Ok(cases),
            }
        }
    }
}
