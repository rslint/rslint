use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::declaration::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::{peek_token, peek};
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_function_declaration(
        &mut self,
        leading: Option<Span>,
    ) -> Result<FunctionDecl, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Function,
            "parse_function_declaration expects the current token to be Function"
        );

        let old_state = self.state.in_function;
        self.state.in_function = true;

        let function_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_function = self.whitespace(false)?;
        let mut name = None;

        if [Some(TokenType::InvalidToken), Some(TokenType::Identifier)].contains(&peek!(self)) {
            let before = self.whitespace(true)?;
            let span = self.cur_tok.lexeme.to_owned();
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            name = Some(LiteralExpr {
                span,
                whitespace: LiteralWhitespace {
                    before,
                    after
                }
            });
        }

        if peek!(self) != Some(TokenType::ParenOpen) {
            self.whitespace(true)?;
            let err = self.error(ExpectedParen, &format!("Expected an opening parenthesis for function arguments, but found `{}`", self.cur_tok.lexeme.content(self.source)))
                .primary(self.cur_tok.lexeme.to_owned(), "Expected an opening parenthesis here");

            // we use parse_args to parse arguments, currently this doesnt allow recovering from missing parentheses
            return Err(err);
        }

        let parameters = self.parse_args(None)?.to_parameters(self);
        let old_strict = self.state.strict;
        let open_brace_whitespace;
        let body;


        if peek!(self) != Some(TokenType::BraceOpen) {
            let before = self.whitespace(true)?;
            let err = self.error(ExpectedBrace, &format!("Expected an opening brace after a function declaration, but found `{}`", self.cur_tok.lexeme.content(self.source)))
                .primary(self.cur_tok.lexeme.to_owned(), "Expected an opening brace here");

            self.errors.push(err);
            open_brace_whitespace = LiteralWhitespace {
                before,
                after: before.end.into()
            };

            body = self.parse_stmt_decl_list(Some(before), Some(&[TokenType::EOF, TokenType::BraceClose]), true)?;
        } else {
            let before = self.whitespace(true)?;
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            open_brace_whitespace = LiteralWhitespace {
                before,
                after,
            };

            body = self.parse_stmt_decl_list(None, Some(&[TokenType::EOF, TokenType::BraceClose]), true)?;
        }

        if self.state.strict.is_some() {
            parameters.verify_strict_mode(self, name.as_ref().map(|x| x.span.content(self.source)));
        }

        // If strict mode was declared in the body we need to clear it as its not global strict mode
        if old_strict.is_none() && self.state.strict.is_some() {
            self.state.strict = None;
        }

        // Check for redundant use strict directive
        if old_strict.is_some() && old_strict != self.state.strict {
            let msg = if name.is_some() {
                format!("The strict mode declaration in the body of function `{}` is redundant, as the outer scope is already in strict mode", name.as_ref().unwrap().span.content(self.source))
            } else {
                "Strict mode declaration is redundant as the outer scope is already in strict mode".to_string()
            };

            let err = self.warning(RedundantUseStrict, &msg)
                .secondary(old_strict.unwrap(), "Strict mode is first declared here")
                .primary(self.state.strict.unwrap(), "This strict mode declaration is redundant");

            self.errors.push(err);
        }

        let close_brace_whitespace;
        let end;

        if peek!(self) != Some(TokenType::BraceClose) {
            let start = self.cur_tok.lexeme.start;
            end = start;
            let span = peek_token!(self).as_ref().unwrap().lexeme.to_owned();
            let err = self.error(ExpectedBrace, "Expected a closing brace after a function body, but found none")
                .primary(Span::new(function_span.start, span.start), "Expected a closing brace to end this declaration");
            
            self.errors.push(err);
            close_brace_whitespace = LiteralWhitespace {
                before: Span::new(start, span.start),
                after: span.start.into()
            };
        } else {
            let before = self.whitespace(true)?;
            end = self.cur_tok.lexeme.end;
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            close_brace_whitespace = LiteralWhitespace {
                before,
                after
            };
        }

        if !old_state { 
            self.state.in_function = false;
        }

        Ok(FunctionDecl {
            span: Span::new(function_span.start, end),
            function_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_function,
            },
            name,
            parameters,
            open_brace_whitespace,
            close_brace_whitespace,
            body,
        })
    }
}
