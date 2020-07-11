use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::peek;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_while_stmt(
        &mut self,
        leading: Option<Span>,
    ) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::While,
            "parse_while_stmt expects the current token to be While"
        );

        let while_span = self.cur_tok.lexeme.to_owned();

        let (while_whitespace, open_paren_whitespace, close_paren_whitespace, condition, _) =
            self.parse_while_start(Some(leading_whitespace))?;

        let cons = self.parse_stmt(None)?;

        return Ok(Stmt::While(WhileStmt {
            span: while_span + cons.span(),
            while_whitespace,
            open_paren_whitespace,
            close_paren_whitespace,
            cons: Box::new(cons),
            condition,
        }));
    }

    pub fn parse_do_while_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Do,
            "parse_do_while_stmt expects the current token to be Do"
        );

        let do_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_do = self.whitespace(false)?;

        let cons = Box::new(self.parse_stmt(None)?);

        let (while_whitespace, open_paren_whitespace, close_paren_whitespace, condition, end) =
            self.parse_while_start(Some(leading_whitespace))?;
        
        return Ok(Stmt::DoWhile(DoWhileStmt {
            span: do_span.extend(end),
            close_paren_whitespace,
            open_paren_whitespace,
            while_whitespace,
            do_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_do,
            },
            condition,
            cons,
        }));
    }

    // parses a `while (expression)` production, this is to allow us to reuse the code for `do { ... } while ( ... )` and `while ( ... ) { ... }`
    fn parse_while_start(
        &mut self,
        leading: Option<Span>,
    ) -> Result<
        (
            LiteralWhitespace,
            LiteralWhitespace,
            LiteralWhitespace,
            Expr,
            usize,
        ),
        ParserDiagnostic<'a>,
    > {
        let before_while = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        if self.cur_tok.token_type != TokenType::While {
            let err = self
                .error(
                    UnexpectedToken,
                    &format!(
                        "Expected a `while` condition, but instead found `{}`",
                        self.cur_tok.lexeme.content(self.source)
                    ),
                )
                .primary(
                    self.cur_tok.lexeme.to_owned(),
                    "Expected a `while` condition here",
                );

            return Err(err);
        }

        let while_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_while = self.whitespace(false)?;

        let before_paren = self.whitespace(true)?;
        let open_paren_whitespace: LiteralWhitespace;
        let condition: Expr;

        if self.cur_tok.token_type != TokenType::ParenOpen {
            let err = self.error(ExpectedParen, "Expected an opening parenthesis between a `while` and its condition, but found none")
                .primary(while_span, "An opening parenthesis is expected following this");

            // Just like if, `while {` causes problems for error recovery, so we return for now
            if self.cur_tok.token_type == TokenType::BraceOpen {
                return Err(err);
            }
            self.errors.push(err);

            open_paren_whitespace = LiteralWhitespace {
                before: before_paren,
                after: Span::new(before_paren.end, before_paren.end),
            };
            condition = self.parse_expr(Some(before_paren))?;
        } else {
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;
            open_paren_whitespace = LiteralWhitespace {
                before: before_paren,
                after,
            };
            condition = self.parse_expr(None)?;
        }

        let close_paren_whitespace: LiteralWhitespace;
        let end: usize;

        // avoid consuming leading whitespace so in a `do {} while {}` we dont consume the leading whitespace of the next statement if the while is erroneous
        if peek!(self) != Some(TokenType::ParenClose) {
            let before_close_paren =
                Span::new(self.cur_tok.lexeme.start, self.cur_tok.lexeme.start);
            let err = self
                .error(
                    ExpectedParen,
                    "Expected a closing parenthesis after a `while` condition, but found none",
                )
                .primary(
                    condition.span().to_owned(),
                    "This condition must be enclosed in parentheses",
                );

            self.errors.push(err);

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after: Span::new(before_close_paren.end, before_close_paren.end),
            };

            end = condition.span().end;
        } else {
            let before_close_paren = self.whitespace(true)?;
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            close_paren_whitespace = LiteralWhitespace {
                before: before_close_paren,
                after,
            };

            end = after.start;
        }

        // The while whitespace, the open paren whitespace, the close paren whitespace, the condition of the production, and the end of the production
        return Ok((
            LiteralWhitespace {
                before: before_while,
                after: after_while,
            },
            open_paren_whitespace,
            close_paren_whitespace,
            condition,
            end,
        ));
    }
}
