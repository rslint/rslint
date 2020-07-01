use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::Parser;
use crate::parser::error::ParseDiagnosticType::UnexpectedToken;
use crate::peek;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        match self.cur_tok.token_type {
            TokenType::Var => self.parse_var_stmt(Some(leading_whitespace)),
            TokenType::BraceOpen => self.parse_block_stmt(Some(leading_whitespace)),
            TokenType::Semicolon => {
                let semi_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after = self.whitespace(false)?;

                Ok(Stmt::Empty(EmptyStmt {
                    span: semi_span,
                    semi_whitespace: LiteralWhitespace {
                        before: leading_whitespace,
                        after,
                    },
                }))
            }
            TokenType::If => self.parse_if_stmt(Some(leading_whitespace)),
            TokenType::Switch => self.parse_switch_stmt(Some(leading_whitespace)),
            TokenType::Throw => self.parse_throw_stmt(Some(leading_whitespace)),
            TokenType::While => self.parse_while_stmt(Some(leading_whitespace)),
            TokenType::Do => self.parse_do_while_stmt(Some(leading_whitespace)),
            TokenType::Break => self.parse_break_stmt(Some(leading_whitespace)),
            TokenType::Continue => self.parse_continue_stmt(Some(leading_whitespace)),
            TokenType::Return => self.parse_return_stmt(Some(leading_whitespace)),
            t if t.starts_expr() => self.parse_expr_stmt(Some(leading_whitespace)),

            _ => {
                let err = self.error(UnexpectedToken, &format!("Expected a statement or declaration, instead found `{}`", self.cur_tok.lexeme.content(self.source)))
                    .primary(self.cur_tok.lexeme.to_owned(), "Expected a statement or declaration here");

                return Err(err);
            },
        }
    }

    pub fn parse_stmt_list(
        &mut self,
        leading: Option<Span>,
        end: Option<&[TokenType]>,
    ) -> Result<Vec<Stmt>, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        let mut first = true;
        let mut stmts: Vec<Stmt> = vec![];

        while !end.unwrap_or(&[TokenType::EOF]).contains(&peek!(self).unwrap_or(TokenType::EOF)) {
            if first {
                first = false;
                stmts.push(self.parse_stmt(Some(leading_whitespace))?);
            } else {
                stmts.push(self.parse_stmt(None)?);
            }
        }
        Ok(stmts)
    }
}
