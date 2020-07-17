use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_block_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::BraceOpen,
            "parse_block_stmt expects the current token to be BraceOpen"
        );

        let open_brace_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_open_brace = self.whitespace(false)?;

        let mut stmts = vec![];

        loop {
            let loop_leading_whitespace = self.whitespace(true)?;

            if self.cur_tok.token_type == TokenType::BraceClose {
                let close_brace_span = self.cur_tok.lexeme.to_owned();
                self.advance_lexer(false)?;
                let after_close_brace = self.whitespace(false)?;

                return Ok(Stmt::Block(BlockStmt {
                    span: open_brace_span + close_brace_span,
                    stmts,
                    open_brace_whitespace: LiteralWhitespace {
                        before: leading_whitespace,
                        after: after_open_brace,
                    },
                    close_brace_whitespace: LiteralWhitespace {
                        before: loop_leading_whitespace,
                        after: after_close_brace,
                    }
                }));
            }

            if self.cur_tok.token_type == TokenType::EOF {
                let err = self.error(UnterminatedBlock, "Expected a closing brace for block statement, got end of input")
                    .secondary(open_brace_span, "Block statement begins here")
                    .primary(self.cur_tok.lexeme.to_owned(), "Input ends here");
                
                return Err(err);
            }

            stmts.push(self.parse_stmt(Some(loop_leading_whitespace))?);
        }
    }
}