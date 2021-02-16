use crate::rule_prelude::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Forbid the use of unsafe control flow statements in try and catch blocks.

    JavaScript suspends any running control flow statements inside of `try` and `catch` blocks until
    `finally` is done executing. This means that any control statements such as `return`, `throw`, `break`,
    and `continue` which are used inside of a `finally` will override any control statements in `try` and `catch`.
    This is almost always unexpected behavior.

    ## Incorrect Code Examples

    ```js
    // We expect 10 to be returned, but 5 is actually returned
    function foo() {
        try {
            return 10;
        //  ^^^^^^^^^ this statement is executed, but actually returning is paused...
        } finally {
            return 5;
        //  ^^^^^^^^^ ...finally is executed, and this statement returns from the function, **the previous is ignored**
        }
    }
    foo() // 5
    ```

    Throwing errors inside try statements

    ```js
    // We expect an error to be thrown, then 5 to be returned, but the error is not thrown
    function foo() {
        try {
            throw new Error("bar");
        //  ^^^^^^^^^^^^^^^^^^^^^^^ this statement is executed but throwing the error is paused...
        } finally {
            return 5;
        //  ^^^^^^^^^ ...we expect the error to be thrown and then for 5 to be returned,
        //  but 5 is returned early, **the error is not thrown**.
        }
    }
    foo() // 5
    ```
    */
    #[derive(Default)]
    NoUnsafeFinally,
    errors,
    tags(Recommended),
    "no-unsafe-finally"
}

pub const CONTROL_FLOW_STMT: [SyntaxKind; 4] = [BREAK_STMT, CONTINUE_STMT, THROW_STMT, RETURN_STMT];

#[typetag::serde]
impl CstRule for NoUnsafeFinally {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if CONTROL_FLOW_STMT.contains(&node.kind())
            && node.parent()?.parent()?.is::<ast::Finalizer>()
        {
            self.output(node, ctx);
        }
        None
    }
}

impl NoUnsafeFinally {
    fn output(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let parent = if node.parent()?.kind() == FINALIZER {
            node.parent()?
        } else {
            node.parent()?.parent()?
        };

        let try_stmt = parent.parent()?.to::<ast::TryStmt>();

        let err = if let Some(control) = try_stmt
            .test()?
            .syntax()
            .children()
            .find(|it| CONTROL_FLOW_STMT.contains(&it.kind()))
        {
            let err = ctx.err(
                self.name(),
                format!(
                    "Unsafe usage of a {} inside of a Try statement",
                    node.readable_stmt_name()
                ),
            );

            let get_kind = |kind: SyntaxKind| match kind {
                RETURN_STMT => "returning from the block",
                CONTINUE_STMT => "continuing the loop",
                THROW_STMT => "throwing this error",
                BREAK_STMT => "breaking from the loop",
                _ => unreachable!(),
            };

            err.secondary(
                control.clone(),
                format!(
                    "{} is paused until the `finally` block is done executing...",
                    get_kind(control.kind())
                ),
            )
            .primary(
                node,
                format!(
                    "...however, {} exits the statement altogether",
                    get_kind(node.kind())
                ),
            )
            .primary(
                node,
                format!("which makes `{}` never finish running", control),
            )
        } else {
            ctx.err(
                self.name(),
                format!(
                    "Unsafe usage of a {} inside of a Try statement",
                    node.readable_stmt_name()
                ),
            )
            .primary(
                node,
                "this statement abruptly ends execution, yielding unwanted behavior",
            )
        };

        ctx.add_err(err);
        None
    }
}

rule_tests! {
    NoUnsafeFinally::default(),
    err: {
        "
        try {
            throw A;
        } finally {
            return;
        }
        ",
        "
        try {
            throw new Error();
        } catch {

        } finally {
            continue;
        }
        ",
        /// ignore
        "
        try {
            {}
        } finally {
            try {} finally {
                return 5;
            }
        }
        "
    },
    ok: {
        "
        try {
            throw A;
        } finally {
            if (false) {
                return true;
            }
        }
        "
    }
}
