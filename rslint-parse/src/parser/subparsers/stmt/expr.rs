use crate::diagnostic::ParserDiagnostic;
use crate::parser::cst::stmt::*;
use crate::parser::cst::expr::*;
use crate::parser::error::ParseDiagnosticType::*;
use crate::lexer::token::TokenType;
use crate::parser::Parser;
use crate::span::Span;
use crate::peek;

impl<'a> Parser<'a> {
    /// Parses an expression statement, also handles labelled statements
    pub fn parse_expr_stmt(&mut self, leading: Option<Span>) -> Result<Stmt, ParserDiagnostic> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        debug_assert!(
            self.cur_tok.token_type.starts_expr(),
            "parse_expr_stmt expects the current token to be the start to an expression"
        );

        let expr = self.parse_expr(Some(leading_whitespace))?;

        if let Expr::Identifier(_) = expr {
            // This is a labelled statement
            if peek!(self) == Some(TokenType::Colon) {
                let before_colon = self.whitespace(true)?;
                self.advance_lexer(false)?;
                let after_colon = self.whitespace(false)?;

                // Duplicate labels are an error
                let maybe_dup = self.state.labels.iter().find(|label| label.0 == expr.span().content(self.source));
                // This should really ignore the statement, but for now we will include it
                if maybe_dup.is_some() {
                    let label = maybe_dup.unwrap();
                    let err = self.error(DuplicateLabels, &format!("`{}` cannot be used as a statement label as it is already used", label.0))
                        .secondary(label.1, &format!("`{}` is first defined here", label.0))
                        .primary(expr.span().to_owned(), &format!("`{}` is redefined here", label.0));
                    
                    self.errors.push(err);
                } else {
                    self.state.labels.push((expr.span().content(self.source), expr.span().to_owned()));
                }

                let stmt = Box::new(self.parse_stmt(None)?);

                let label = if let Expr::Identifier(data) = expr {
                    data
                } else {
                    unreachable!();
                };

                return Ok(Stmt::Labelled(LabelledStmt {
                    span: label.span + stmt.span(),
                    label,
                    body: stmt,
                    colon_whitespace: LiteralWhitespace {
                        before: before_colon,
                        after: after_colon,
                    }
                }))
            }
        }
        let semi = self.semi()?;

        if semi.is_none() {
            let err = self.error(ExpectedSemicolon, "An explicit semicolon was required after an expression statement, but none was found")
                .primary(expr.span().to_owned(), "A semicolon is required to end this statement");
            
            self.errors.push(err);
        }

        let semicolon = semi.unwrap_or(Semicolon::Implicit);
        return Ok(Stmt::Expr(ExprStmt {
            span: expr.span().to_owned().extend(semicolon.offset()),
            expr,
            semi: semicolon,
        }));
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::cst::{expr::*, stmt::*};
    use crate::stmt;
    use crate::span;
    use crate::span::Span;

    #[test]
    fn expr_stmt() {
        assert_eq!(
            stmt!("5;"),
            Stmt::Expr(ExprStmt { 
                span: span!("5;", "5;"),
                expr: Expr::Number(LiteralExpr {
                    span: span!("5;", "5"),
                    whitespace: LiteralWhitespace {
                        before: Span::new(0, 0),
                        after: Span::new(1, 1),
                    }
                }),
                semi: Semicolon::Explicit(LiteralWhitespace {
                    before: Span::new(1, 1),
                    after: Span::new(2, 2),
                })
            })
        )
    }
}