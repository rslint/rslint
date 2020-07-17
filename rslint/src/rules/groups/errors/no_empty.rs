use crate::cst_rule;
use crate::diagnostic::DiagnosticBuilder;
use crate::visit::{Node, Visit};
use rslint_parse::parser::cst::declaration::FunctionDecl;
use rslint_parse::parser::cst::stmt::*;
use std::any::TypeId;

cst_rule! {
    "no-empty",
    NoEmpty
}

impl Visit for NoEmptyVisitor<'_, '_> {
    fn visit_empty_stmt(&mut self, empty: &EmptyStmt, _parent: &dyn Node) {
        let err = DiagnosticBuilder::error(
            self.ctx.file_id,
            "no-empty",
            "Empty statements are not allowed",
        )
        .primary(empty.span, "");

        self.ctx.diagnostics.push(err.into())
    }

    fn visit_block_stmt(&mut self, block: &BlockStmt, parent: &dyn Node) {
        if block.stmts.len() == 0
            && !(block.open_brace_whitespace.after + block.close_brace_whitespace.before)
                .contains_comments(self.ctx.file_source)
            // Empty functions should not error
            && !parent.as_any().is::<FunctionDecl>()
        {
            let mut err = DiagnosticBuilder::error(
                self.ctx.file_id,
                "no-empty",
                "Empty block statements are not allowed",
            )
            .primary(block.span, "");

            if parent.type_id() == TypeId::of::<TryStmt>() {
                let try_stmt = parent.as_any().downcast_ref::<TryStmt>().unwrap();
                // This could also be the finalizer block stmt, in which case it could mean the handler is in fact reachable, so we do this
                // to prevent an erroneous warning
                if try_stmt.handler.is_some() && block.span == try_stmt.test.span {
                    err = err.secondary(
                        try_stmt.handler.as_ref().unwrap().span,
                        "This handler is unreachable",
                    );
                }
            }

            self.ctx.diagnostics.push(err.into())
        } else {
            for stmt in &block.stmts {
                self.visit_stmt(stmt, parent);
            }
        }
    }

    fn visit_switch_stmt(&mut self, switch: &SwitchStmt, _parent: &dyn Node) {
        if switch.cases.len() == 0
            && !(switch.open_brace_whitespace.after + switch.close_brace_whitespace.before)
                .contains_comments(self.ctx.file_source)
        {
            let err = DiagnosticBuilder::error(
                self.ctx.file_id,
                "no-empty",
                "Empty switch statements are not allowed",
            )
            .primary(switch.span, "");

            self.ctx.diagnostics.push(err.into());
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_lint_ok, assert_lint_err};
    use crate::rules::groups::errors::no_empty::NoEmpty;

    #[test]
    fn no_empty_err() {
        assert_lint_err! {
            NoEmpty,
            "{}" => 0..2,
            "try { }" => 4..7,
            "try { /* a */ } catch(e) { }",
            "try { /* a */ } finally { }",
            "switch(a) {}",
            ";",
            "{{}}",
        }
    }

    #[test]
    fn no_empty_ok() {
        assert_lint_ok! {
            NoEmpty,
            "{ /* */ }",
            "{\n /* */}",
            "{/* */\n}",
            "function a() {}",
            "try { /* \n*/ } catch (e) { /* \n*/ } finally { /* */\n }",
            "switch (a) { /* */ }",
            "switch (a) { \n case 5: }"
        }
    }
}
