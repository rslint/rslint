use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    pub fn parse_break_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Break,
            "parse_break_stmt expects the current token to be Break"
        );

        let break_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_break = self.whitespace(false)?;

        let label = self.parse_optional_label()?;

        let semi = self.semi()?;

        let label_offset = label.as_ref().map_or(0, |l| l.span.size() + 1);
        if semi.is_none() {
            let err = self.error(ExpectedSemicolon, "An explicit semicolon was required after a break statement, but none was found")
                .primary(break_span.extend(label_offset), "A semicolon is required to end this statement");
            
            self.errors.push(err);
        }

        let semicolon = semi.unwrap_or(Semicolon::Implicit);
        let stmt_span = break_span.extend(label_offset).extend(semicolon.offset());

        // Currently we just issue an error and still parse, perhaps we should reconsider this later on
        if !self.state.in_switch_stmt && !self.state.in_iteration_stmt {
            let err = self.error(InvalidBreak, "`break` statements may not appear outside of `switch` or iteration statements")
                .primary(stmt_span, "Invalid in this context");

            self.errors.push(err);
        }

        Ok(Stmt::Break(BreakStmt {
            span: stmt_span,
            break_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_break,
            },
            label,
            semi: semicolon,
        }))
    }

    pub fn parse_continue_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert_eq!(
            self.cur_tok.token_type,
            TokenType::Continue,
            "parse_continue_stmt expects the current token to be Continue"
        );

        let continue_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_continue = self.whitespace(false)?;

        let label = self.parse_optional_label()?;

        let semi = self.semi()?;

        let label_offset = label.as_ref().map_or(0, |l| l.span.size() + 1);
        if semi.is_none() {
            let err = self.error(ExpectedSemicolon, "An explicit semicolon was required after a continue statement, but none was found")
                .primary(continue_span.extend(label_offset), "A semicolon is required to end this statement");
            
            self.errors.push(err);
        }

        let semicolon = semi.unwrap_or(Semicolon::Implicit);
        let stmt_span = continue_span.extend(label_offset).extend(semicolon.offset());

        // Currently we just issue an error and still parse, perhaps we should reconsider this later on
        if !self.state.in_iteration_stmt {
            let err = self.error(InvalidContinue, "`continue` statements may not appear outside of an iteration statement")
                .primary(stmt_span, "Invalid in this context");

            self.errors.push(err);
        }

        Ok(Stmt::Continue(ContinueStmt {
            span: stmt_span,
            continue_whitespace: LiteralWhitespace {
                before: leading_whitespace,
                after: after_continue,
            },
            label,
            semi: semicolon,
        }))
    }

    fn parse_optional_label(&mut self) -> Result<Option<LiteralExpr>, ParserDiagnostic<'a>> {
        if self.cur_tok.token_type == TokenType::Linebreak || peek!(self) != Some(TokenType::Identifier) {
            return Ok(None);
        }

        let before_label = self.whitespace(true)?;
        let label_span = self.cur_tok.lexeme.to_owned();
        self.advance_lexer(false)?;
        let after_label = self.whitespace(false)?;

        let label_str = label_span.content(self.source);
        // It is an early error if no label was defined with a name used in a labelled `break` or `continue`
        if !self.state.labels.iter().any(|l| l.0 == label_str) {
            let err = self.error(InvalidLabel, &format!("The statement label `{}` is used but never defined", label_str))
                .primary(label_span, "This label is undefined");
            
            self.errors.push(err);
        }

        Ok(Some(LiteralExpr {
            span: label_span,
            whitespace: LiteralWhitespace {
                before: before_label,
                after: after_label,
            }
        }))
    }
}