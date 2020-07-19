use crate::cst_rule;
use crate::diagnostic::DiagnosticBuilder;
use crate::visit::{Node, Visit};
use rslint_parse::lexer::token::*;
use rslint_parse::parser::cst::expr::*;
use rslint_parse::parser::cst::stmt::*;
use crate::util::{is_const_ident, simple_bool_cast};

cst_rule! {
    "no-constant-condition",
    NoConstantCondition
}

// check if an expression is constant and return a second bool saying if the expr is truthy or falsey
fn is_const(expr: &Expr, source: &str, bool_pos: bool) -> bool {
    match expr {
        Expr::Regex(_)
        | Expr::Object(_)
        | Expr::Function(_)
        | Expr::Null(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::False(_)
        | Expr::True(_) => true,
        Expr::Identifier(ident) => is_const_ident(ident, source),
        Expr::Conditional(condexpr) => {
            is_const(&condexpr.condition, source, true)
        }
        Expr::Array(array) => array.exprs.iter().all(|expr| {
            expr.as_ref()
                .map(|x| is_const(&x, source, false))
                .unwrap_or(true)
        }),
        Expr::Unary(unary) => {
            if unary.op == TokenType::Void {
                true
            } else {
                (unary.op == TokenType::Typeof && bool_pos) || is_const(&unary.object, source, true)
            }
        }
        Expr::Binary(binexpr) => match binexpr.op {
            TokenType::BinOp(BinToken::LogicalOr) | TokenType::BinOp(BinToken::LogicalAnd) => {
                let left_const = is_const(&binexpr.left, source, bool_pos);
                let right_const = is_const(&binexpr.right, source, bool_pos);

                // TODO: Handle OR cases with a right const value, this needs truthy value handling so its slightly more complex
                (left_const && right_const)
                    || short_circuits(&binexpr.left, source, binexpr.op)
                    || short_circuits(&binexpr.right, source, binexpr.op)
            }
            _ => {
                is_const(&binexpr.left, source, false)
                    && is_const(&binexpr.right, source, false)
                    && binexpr.op != TokenType::In
            }
        },
        Expr::Assign(assignexpr) => {
            assignexpr.op == TokenType::AssignOp(AssignToken::Assign)
                && is_const(&assignexpr.right, source, bool_pos)
        }
        Expr::Sequence(seqexp) => seqexp
            .exprs
            .iter()
            .all(|expr| is_const(expr, source, bool_pos)),

        _ => false,
    }
}

fn short_circuits(expr: &Expr, source: &str, op: TokenType) -> bool {
    match expr {
        Expr::Regex(_)
        | Expr::Object(_)
        | Expr::Function(_)
        | Expr::Null(_)
        | Expr::String(_)
        | Expr::Number(_)
        | Expr::False(_)
        | Expr::True(_) => {
            (op == TokenType::BinOp(BinToken::LogicalOr) && expr.span().content(source) == "true")
                || (op == TokenType::BinOp(BinToken::LogicalAnd)
                    && expr.span().content(source) == "false")
        }
        Expr::Unary(unexpr) => {
            op == TokenType::BinOp(BinToken::LogicalAnd) && unexpr.op == TokenType::Void
        }
        Expr::Binary(binexpr) => match binexpr.op {
            TokenType::BinOp(BinToken::LogicalAnd) | TokenType::BinOp(BinToken::LogicalOr) => {
                short_circuits(&binexpr.left, source, op)
                    || short_circuits(&binexpr.right, source, op)
            }
            _ => false,
        },
        _ => false,
    }
}

impl Visit for NoConstantConditionVisitor<'_, '_> {
    fn visit_conditional_expr(&mut self, condexpr: &ConditionalExpr, _: &dyn Node) {
        if is_const(&*condexpr.condition, self.ctx.file_source, true) {
            let mut err = DiagnosticBuilder::error(self.ctx.file_id, "no-constant-condition", "Unexpected constant condition in conditional expression");

            // If we can easily deduce whether the condition is truthy or falsey we can offer more context around why the condition is bad
            // This does not factor in math operations like 5 - 5, since that would pretty much require an interpreter and number parsing
            if let Some(cast) = simple_bool_cast(&*condexpr.condition, self.ctx.file_source) {
                if cast {
                    err = err.secondary(condexpr.if_true.span().to_owned(), "...which means this expression is always returned")
                        .primary(condexpr.span, "this expression is always truthy...");
                } else {
                    err = err.secondary(condexpr.if_false.span().to_owned(), "...which means this expression is always returned")
                        .primary(condexpr.span, "this expression is always falsey...");
                }
            } else {
                err = err.primary(condexpr.span, "this expression is always yields one result");
            }
            
            self.ctx.diagnostics.push(err.into());
        }

        self.visit_expr(&condexpr.condition, condexpr as _);
        self.visit_expr(&condexpr.if_true, condexpr as _);
        self.visit_expr(&condexpr.if_false, condexpr as _);
    }

    fn visit_if_stmt(&mut self, ifstmt: &IfStmt, _: &dyn Node) {
        if is_const(&ifstmt.condition, self.ctx.file_source, true) {
            let mut err = DiagnosticBuilder::error(self.ctx.file_id, "no-constant-condition", "Unexpected constant condition in if statement");
            
            if let Some(cast) = simple_bool_cast(&ifstmt.condition, self.ctx.file_source) {
                if cast && ifstmt.alt.is_some() {
                    err = err.secondary(ifstmt.alt.as_ref().unwrap().span(), "...which makes this `else` unreachable")
                        .primary(ifstmt.condition.span().to_owned(), "this expression is always truthy...");
                } else if !cast {
                    err = err.secondary(ifstmt.cons.span(), "...which makes this statement unreachable")
                        .primary(ifstmt.condition.span().to_owned(), "this expression is always falsey...");
                } else {
                    err = err.primary(ifstmt.condition.span().to_owned(), "this expression is always truthy");
                }
            } else {
                err = err.primary(ifstmt.condition.span().to_owned(), "this expression is always yields one result");
            }

            self.ctx.diagnostics.push(err.into());
        }

        self.visit_expr(&ifstmt.condition, ifstmt as _);
        self.visit_stmt(&ifstmt.cons, ifstmt as _);
        self.visit_opt_stmt(ifstmt.alt.as_ref(), ifstmt as _);
    }

    fn visit_while_stmt(&mut self, whilestmt: &WhileStmt, _: &dyn Node) {
        if is_const(&whilestmt.condition, self.ctx.file_source, true) {
            let mut err = DiagnosticBuilder::error(self.ctx.file_id, "no-constant-condition", "Unexpected constant condition in while statement");
            
            if let Some(cast) = simple_bool_cast(&whilestmt.condition, self.ctx.file_source) {
                if cast {
                    err = err.secondary(whilestmt.cons.span(), "...which makes this infinitely loop")
                        .primary(whilestmt.condition.span().to_owned(), "this expression is always truthy");
                } else {
                    err = err.secondary(whilestmt.cons.span(), "...which makes this loop unreachable")
                    .primary(whilestmt.condition.span().to_owned(), "this expression is always falsey");
                }
            } else {
                err = err.primary(whilestmt.condition.span().to_owned(), "this expression is always yields one result");
            }

            self.ctx.diagnostics.push(err.into());
        }

        self.visit_expr(&whilestmt.condition, whilestmt as _);
        self.visit_stmt(&whilestmt.cons, whilestmt as _);
    }

    fn visit_do_while_stmt(&mut self, dowhile: &DoWhileStmt, _: &dyn Node) {
        if is_const(&dowhile.condition, self.ctx.file_source, true) {
            let mut err = DiagnosticBuilder::error(self.ctx.file_id, "no-constant-condition", "Unexpected constant condition in do while statement");
            
            if let Some(cast) = simple_bool_cast(&dowhile.condition, self.ctx.file_source) {
                if cast {
                    err = err.secondary(dowhile.cons.span(), "...which makes this infinitely loop")
                        .primary(dowhile.condition.span().to_owned(), "this expression is always truthy...");
                } else {
                    err = err.secondary(dowhile.cons.span(), "...which makes this loop only run once")
                    .primary(dowhile.condition.span().to_owned(), "this expression is always falsey...");
                }
            } else {
                err = err.primary(dowhile.condition.span().to_owned(), "this expression is always yields one result");
            }

            self.ctx.diagnostics.push(err.into());
        }

        self.visit_expr(&dowhile.condition, dowhile as _);
        self.visit_stmt(&dowhile.cons, dowhile as _);
    }

    fn visit_for_stmt(&mut self, forstmt: &ForStmt, _: &dyn Node) {
        if let Some(ref test) = forstmt.test {
            if is_const(test, self.ctx.file_source, true) {
                let mut err = DiagnosticBuilder::error(self.ctx.file_id, "no-constant-condition", "Unexpected constant condition in for loop");
            
                if let Some(cast) = simple_bool_cast(test, self.ctx.file_source) {
                    if cast {
                        err = err.secondary(forstmt.body.span(), "...which makes this infinitely loop")
                            .primary(test.span().to_owned(), "this expression is always truthy...");
                    } else {
                        err = err.secondary(forstmt.body.span(), "...which makes this loop unreachable")
                        .primary(test.span().to_owned(), "this expression is always falsey...");
                    }
                } else {
                    err = err.primary(test.span().to_owned(), "this expression is always yields one result");
                }

                self.ctx.diagnostics.push(err.into());
            }
        }

        self.visit_opt_for_stmt_init(forstmt.init.as_ref(), forstmt as _);
        self.visit_opt_expr(forstmt.test.as_ref(), forstmt as _);
        self.visit_opt_expr(forstmt.update.as_ref(), forstmt as _);
        self.visit_stmt(&*forstmt.body, forstmt as _);
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_lint_err, assert_lint_ok};
    use crate::rules::groups::errors::no_constant_condition::NoConstantCondition;

    #[test]
    fn no_constant_condition_err() {
        assert_lint_err! {
            NoConstantCondition,
            "if(6) {}",
            "if(6 - 7 || 3 ? 7 && 2 : NaN + NaN || 2) {}",
            "if (true) {}",
            "if (NaN) {} else {}",
            "6 + 2 ? false : NaN",
            "false ? false : false ? false : false",
            "while (true) {}",
            "do { /* */ } while (NaN ? NaN : true)",
            "do { } while (NaN ? Infinity : true)",
        }
    }

    #[test]
    fn no_constant_condition_ok() {
        assert_lint_ok! {
            NoConstantCondition,
            "if (foo) {}",
            "if (false > foo) {} else {}",
            "if (foo ? NaN : Infinity) {}",
            "do {} while (foo + 6)",
            "for(var i = 5; foo; i++) {}",
        }
    }
}