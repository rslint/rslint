use crate::parser::error::ParseDiagnosticType::InvalidTargetExpression;
use crate::diagnostic::ParserDiagnostic;
use crate::lexer::token::{TokenType, AssignToken};
use crate::parser::cst::expr::*;
use crate::parser::Parser;
use crate::span::Span;

impl<'a> Parser<'a> {
    pub fn parse_assign_expr(&mut self, leading: Option<Span>) -> Result<Expr, ParserDiagnostic<'a>> {
        let leading_whitespace = if leading.is_none() {
            self.whitespace(true)?
        } else {
            leading.unwrap()
        };

        let target = self.parse_conditional_expr(Some(leading_whitespace))?;

        // It is *technically* wrong to parse an assignment expression if the target is not a LHS expression.
        // However, for the purposes of error recovery, we will still parse it.
        // This is safe because no token will be consumed if the next token is not an AssignToken.
        // TODO: maybe rethink this choice
        self.parse_assign_expr_recursive(target)
    }

    pub fn parse_assign_expr_recursive(&mut self, target: Expr) -> Result<Expr, ParserDiagnostic<'a>> {
        let before_op;
        let op: AssignToken;

        if let TokenType::AssignOp(tok) = self.cur_tok.token_type {
            before_op = self.span(self.cur_tok.lexeme.start, self.cur_tok.lexeme.start);
            op = tok;
        } else {
            // This is to avoid the case of `OTHERTOKEN ASSIGNTOKEN`, peeking would skip over `OTHERTOKEN` and use `ASSIGNTOKEN`
            // There may be a better way of doing this
            match &self.cur_tok {
                t if t.is_whitespace() => {},
                _ => return Ok(target)
            }
            let peeked = self.peek_while(|x| x.is_whitespace())?.map(|x| x.token_type);
            
            if let Some(TokenType::AssignOp(kind)) = peeked {
                before_op = self.whitespace(true)?;
                op = kind;
            } else {
                return Ok(target);
            }
        }
        self.advance_lexer(false)?;

        let after_op = self.whitespace(false)?;

        if !target.is_valid_assign_target() {
            let err = self.error(InvalidTargetExpression, &format!("Invalid assignment target for `{}`", op.to_string()))
                .primary(target.span().to_owned(), "Not a valid assignment target");
            
            self.errors.push(err);
        }

        let right = self.parse_assign_expr(None)?;

        Ok(Expr::Assign(AssignmentExpr {
            span: self.span(target.span().start, right.span().end),
            left: Box::new(target),
            right: Box::new(right),
            op: TokenType::AssignOp(op),
            whitespace: OperatorWhitespace {
                before_op,
                after_op
            }
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::expr;
    use crate::lexer::token::*;
    use crate::parser::cst::expr::*;
    use crate::span;
    use crate::span::Span;

    #[test]
    fn simple_assignment() {
        assert_eq!(expr!("foo += bar"),
        Expr::Assign(AssignmentExpr {
            span: span!("foo += bar", "foo += bar"),
            left: Box::new(Expr::Identifier(LiteralExpr {
                span: span!("foo += bar", "foo"),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(3, 4),
                }
            })),
            right: Box::new(Expr::Identifier(LiteralExpr {
                span: span!("foo += bar", "bar"),
                whitespace: ExprWhitespace {
                    before: Span::new(7, 7),
                    after: Span::new(10, 10),
                }
            })),
            op: TokenType::AssignOp(AssignToken::AddAssign),
            whitespace: OperatorWhitespace {
                before_op: Span::new(4, 4),
                after_op: Span::new(6, 7)
            }
        }))
    }

    #[test]
    fn invalid_assign_target() {
        assert_eq!(expr!("true += false"),
        Expr::Assign(AssignmentExpr {
            span: span!("true += false", "true += false"),
            left: Box::new(Expr::True(LiteralExpr {
                span: span!("true += false", "true"),
                whitespace: ExprWhitespace {
                    before: Span::new(0, 0),
                    after: Span::new(4, 5)
                }
            })),
            right: Box::new(Expr::False(LiteralExpr {
                span: span!("true += false", "false"),
                whitespace: ExprWhitespace {
                    before: Span::new(8, 8),
                    after: Span::new(13, 13),
                }
            })),
            op: TokenType::AssignOp(AssignToken::AddAssign),
            whitespace: OperatorWhitespace {
                before_op: Span::new(5, 5),
                after_op: Span::new(7, 8)
            }
        }))
    }
}