use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::TokenType;
use crate::parser::cst::expr::*;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    /// Parse a left hand side expression, which includes call, dot, and bracket suffix expressions.
    pub fn parse_lhs_expr(&mut self, leading_whitespace: Option<Span>) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_ws = if leading_whitespace.is_some() {
            leading_whitespace.unwrap()
        } else {
            self.whitespace(true)?
        };

        let callee = self.parse_member_or_new_expr(Some(leading_ws), true)?;

        if peek!(self, [TokenType::ParenOpen]) == Some(TokenType::ParenOpen) {
            let arguments = self.parse_args(None)?;
            let expr = Expr::Call(CallExpr {
                span: self.span(callee.span().start, arguments.span.end),
                arguments,
                callee: Box::new(callee),
            });

            return self.parse_suffixes(expr, false);
        }

        Ok(callee)
    }
}
