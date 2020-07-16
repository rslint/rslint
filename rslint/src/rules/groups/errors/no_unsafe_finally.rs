use crate::cst_rule;
use crate::diagnostic::DiagnosticBuilder;
use crate::visit::{Node, Visit};
use rslint_parse::parser::cst::stmt::*;

cst_rule! {
    "no-unsafe-finally",
    NoUnsafeFinally
}

impl Visit for NoUnsafeFinallyVisitor<'_, '_> {
    fn visit_try_stmt(&mut self, try_stmt: &TryStmt, parent: &dyn Node) {
        if let Some(ref finalizer) = try_stmt.finalizer {
            let control = finalizer.stmts.iter().find(|stmt| match stmt {
                Stmt::Return(_) | Stmt::Throw(_) | Stmt::Break(_) | Stmt::Continue(_) => true,
                _ => false,
            });

            if let Some(control_stmt) = control {
                let builder = DiagnosticBuilder::error(
                    self.ctx.file_id,
                    "no-unsafe-finally",
                    "Control statements at the top level of a `try` statement `finally` are unsafe",
                ).primary(control_stmt.span(), "This statement is unsafe and will yield unexpected behavior");

                self.ctx.diagnostics.push(builder.into());
            }
        }

        // We need to still recurse through and visit other statements
        self.visit_block_stmt(&try_stmt.test, parent);
        if let Some(ref handler) = try_stmt.handler {
            self.visit_catch_clause(handler, parent);
        }
        if let Some(ref finalizer ) = try_stmt.finalizer {
            self.visit_block_stmt(finalizer, parent);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_lint_err, assert_lint_ok};
    use crate::rules::groups::errors::no_unsafe_finally::NoUnsafeFinally;

    #[test]
    fn no_unsafe_finally_err() {
        assert_lint_err! {
            NoUnsafeFinally,
            "
            try {
                /* do something */
                return a;
            } finally {
                return b;
            }
            ",
            "
            a: try {
                /* do something */
                return a;
            } finally {
                break a;
            }
            ",
            "
            b: try {
                /* do something */
                return a;
            } finally {
                continue b;
            }
            ",
            "
            try {
                /* do something */
                return a;
            } finally {
                throw b;
            }
            ",
            "
            try {
                /* do something */
                return a;
            } finally {
                try {
                    /* */
                } finally {
                    return a;
                }
            }
            ",
            "
            try {
                /* do something */
                return a;
            } catch(e) {
                try {
                    /* */
                } finally {
                    return a;
                }
            } finally {
                /* */
            }
            "
        }
    }

    #[test]
    fn no_unsafe_finally_ok() {
        assert_lint_ok! {
            NoUnsafeFinally,
            "
            try {
                /* */
            } finally {
                /* */
            }
            ",
            "
            try {
                //
            } finally {
                if (a) {
                    return b;
                }
            }
            ",
            "
            try {
                /* */
            } finally {
                try {
                    try {
                        return a;
                    } finally {
                        if (a) {
                            return b;
                        }
                    }
                } finally {

                }
            }
            "
        }
    }
}