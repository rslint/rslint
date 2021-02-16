use crate::rule_prelude::*;

declare_lint! {
    /**
    Disallow the use of debugger statements.

    `debugger` statements are used to tell the environment executing the code to start an appropriate
    debugger. These statements are rendered useless by modern IDEs which have built in breakpoint support.
    Having them in production code is erroneous as it will tell the browser to stop running and open a debugger.

    ## Invalid Code Examples

    ```js
    function doSomething() {
        debugger;
        doSomethingElse();
    }
    ```
    */
    #[derive(Default)]
    NoDebugger,
    errors,
    tags(Recommended),
    "no-debugger"
}

#[typetag::serde]
impl CstRule for NoDebugger {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == SyntaxKind::DEBUGGER_STMT {
            let err = ctx
                .err(self.name(), "debugger statements are not allowed")
                .primary(node, "");

            ctx.add_err(err);
        }
        None
    }
}

rule_tests! {
    NoDebugger::default(),
    err: {
        "debugger",
        "debugger;"
    },
    ok: {}
}
