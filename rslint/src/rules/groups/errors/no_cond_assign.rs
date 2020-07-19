use crate::cst_rule;
use crate::diagnostic::DiagnosticBuilder;
use crate::visit::{Node, Visit};
use rslint_parse::parser::cst::expr::*;
use rslint_parse::parser::cst::stmt::*;

cst_rule! {
    "no-cond-assign",
    NoCondAssign
}

fn check_expr(ctx: &mut RuleContext, expr: &Expr) {
    if let Expr::Assign(ref assign) = expr {
        let err = DiagnosticBuilder::error(ctx.file_id, "no-cond-assign", "Unexpected assignment expression in a conditional statement")
        .primary(assign.span, "")
        .secondary(assign.right.span().to_owned(), "This value will be casted and used for the comparison")
        .help("Help: Did you mean to check for equality (`==` or `===`) instead of assigning to a variable (`=`) ?");

        ctx.diagnostics.push(err.into());
    }

    if let Expr::Binary(BinaryExpr { ref left, ref right, .. }) = expr {
        check_expr(ctx, &**left);
        check_expr(ctx, &**right);
    }
}

impl Visit for NoCondAssignVisitor<'_, '_> {
    fn visit_if_stmt(&mut self, stmt: &IfStmt, _: &dyn Node) {
        check_expr(&mut self.ctx, &stmt.condition);
        self.visit_expr(&stmt.condition, stmt as _);
        self.visit_stmt(&stmt.cons, stmt as _);
        if stmt.alt.is_some() {
            self.visit_stmt(stmt.alt.as_ref().unwrap(), stmt as _);
        }
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt, _: &dyn Node) {
        check_expr(&mut self.ctx, &stmt.condition);
        self.visit_expr(&stmt.condition, stmt as _);
        self.visit_stmt(&stmt.cons, stmt as _);
    }

    fn visit_do_while_stmt(&mut self, stmt: &DoWhileStmt, _: &dyn Node) {
        check_expr(&mut self.ctx, &stmt.condition);
        self.visit_expr(&stmt.condition, stmt as _);
        self.visit_stmt(&stmt.cons, stmt as _);
    }

    fn visit_for_stmt(&mut self, stmt: &ForStmt, _: &dyn Node) {
        if let Some(ref test) = stmt.test {
            check_expr(&mut self.ctx, test);
            self.visit_expr(test, stmt as _);
        }
        self.visit_stmt(&*stmt.body, stmt as _);
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_lint_err, assert_lint_ok};
    use crate::rules::groups::errors::no_cond_assign::NoCondAssign;

    #[test]
    fn no_cond_assign_err() {
        assert_lint_err! {
            NoCondAssign,
            "for (var i = 5; i = 5; i++) { /* */ }" => 16..21,
            "for (var i = 5; i == 5; i++) { for (var i = 5; i = 5; i++) { /* */ } }",
            "if (a = 5) {}" => 4..9,
            "
            if (a == 5)  {
                if (a = 6) {}
            }
            ",
            "
            if (a == 5)  {
                if (a = 6 || b = 7) {}
            }
            ",
            "
            if (a == 5)  {
                if (a == 6) {}
            } else {
                if (a = 5) {}
            }
            ",
            "while(a = 5) {}",
            "
            while (a == 5) {
                while(a = 5) {}
            }
            ",
            "do {} while (a === 5 || b = 5)",
            "
            do {
                for(var i = 5; i = 5; i++) {}
            } while (a == 6)
            ",
            "
            if (a == function() {
                while (b = 5) {

                }
            }) {}
            ",
        }
    }

    #[test]
    fn no_cond_assign_ok() {
        assert_lint_ok! {
            NoCondAssign,
            "if (a == 5 && a === 7) {}",
            "for(var i = 5; i == 5; i++) {}",
            "while (a == 56) {}",
            "do {} while (b == 5 && 12 == 4)",
            "
            if (function a() { while(b == 5) {} } == 6 || 8 && 7 == 3) {}
            "
        }
    }
}