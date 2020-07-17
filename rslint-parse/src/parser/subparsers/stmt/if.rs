use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    pub fn parse_if_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::If,
            "parse_if_stmt expects the current token to be If"
        );

        let if_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_if = self.whitespace(false)?;

        let before_paren = self.whitespace(true)?;
        let open_paren_whitespace: LiteralWhitespace;
        let condition: Expr;
        let mut error: Option<ParserDiagnostic> = None;

        // recover from `if a` by ignoring the need for an open paren and instead reporting an error and parsing the expression
        if self.cur_tok.token_type != TokenType::ParenOpen {
            let mut err = self.error(ExpectedParen, "Expected an opening parenthesis between an `if` and the condition, but found none")
                .primary(self.cur_tok.lexeme.to_owned(), "An opening parenthesis is expected here");
            
            // `if {}` causes a lot of issues since it will be recovered by interpreting it as an object literal,
            // This causes a lot of random errors, so we just return if this is the case
            // TODO: handle this more gracefully
            if self.cur_tok.token_type == TokenType::BraceOpen {
                return Err(err);
            }

            open_paren_whitespace = LiteralWhitespace {
                before: before_paren,
                after: Span::from(before_paren.end),
            };
            condition = self.parse_expr(Some(before_paren))?;
            err = err.secondary(Span::from(condition.span().start - 1), "Help: insert a `(` here");
            error = Some(err);
        } else {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;
            open_paren_whitespace = LiteralWhitespace {
                before: before_paren,
                after,
            };
            condition = self.parse_expr(None)?;
        }

        let before_close_paren = self.whitespace(true)?;
        let close_paren_whitespace: LiteralWhitespace;
        let cons: Stmt;

        if self.cur_tok.token_type != TokenType::ParenClose {
            let mut err = self.error(ExpectedParen, "Expected a closing parenthesis after an if statement condition, but found none")
                .primary(condition.span().to_owned(), "This condition must be enclosed in parentheses")
                .secondary(Span::from(condition.span().end), "Help: insert a `)` here");
            
            // We can offer an overall more "complete" error if both parentheses are missing
            if error.is_some() {
                let cond_str = condition.span().content(self.source);
                err = self.error(MissingParentheses, "Missing parentheses around an `if` statement condition")
                    .primary(condition.span().to_owned(), "This condition must be encased in parentheses")
                    .help(&format!("Help: convert the condition to `({})`", cond_str));
            }
            self.errors.push(err);

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after: Span::new(before_close_paren.end, before_close_paren.end),
            };
            cons = self.parse_stmt(Some(before_close_paren))?;
        } else {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            if error.is_some() {
                self.errors.push(error.unwrap());
            }
            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after,
            };
            cons = self.parse_stmt(None)?;
        }

        // Span is used to report erroneous `else` blocks
        let mut alt: (Option<Stmt>, Span) = (None, Span::new(0, 0));
        let mut else_whitespace: Option<LiteralWhitespace> = None;
        let mut first = true;

        // We do this in a loop so we can loop over erroneous `else` blocks
        loop {
            if peek!(self) == Some(TokenType::Else) {
                // This is a valid `else` block
                if first {
                    first = false;
                    let before_else = self.whitespace(true)?;
                    let else_span = self.cur_tok.lexeme.to_owned();
                    self.advance_lexer(false)?;
                    let after_else = self.whitespace(false)?;

                    let alt_stmt = self.parse_stmt(None)?;
                    alt = (Some(alt_stmt), else_span);
                    else_whitespace = Some(LiteralWhitespace {
                        before: before_else,
                        after: after_else,
                    });
                    continue;
                }

                // Else block is erroneous
                self.whitespace(true)?;
                let err_else_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                self.whitespace(false)?;
                // Parse but ignore the following statement
                self.parse_stmt(None)?;

                let err = self.error(MultipleElseBlocks, "An `if` statement may not contain multiple `else` blocks")
                    .secondary(alt.1, "First `else` block is defined here")
                    .primary(err_else_span, "This `else` block is invalid");
                
                self.errors.push(err);
            } else {
                let end = if alt.0.is_some() { alt.0.as_ref().unwrap().span().end } else { cons.span().end };
                return Ok(Stmt::If(IfStmt {
                    span: if_span.extend(end),
                    cons: Box::new(cons),
                    if_whitespace: LiteralWhitespace {
                        before: leading_whitespace,
                        after: after_if,
                    },
                    open_paren_whitespace,
                    close_paren_whitespace,
                    condition,
                    else_whitespace,
                    alt: alt.0.map(Box::new)
                }));
            }
        }
    }
}