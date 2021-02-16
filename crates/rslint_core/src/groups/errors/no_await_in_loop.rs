use crate::rule_prelude::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow await inside of loops.

    You may want to `await` a promise until it is fulfilled or rejected, inside of loops. In such cases, to take
    full advantage of concurrency, you should __not__ `await` the promise in every iteration, otherwise your async
    operations will be executed serially.
    Generally it is recommended that you create all promises, then use `Promise.all` for them. This way your async
    operations will be performed concurrently.

    ## Incorrect Code Exapmles

    ```js
    async function foo(xs) {
        const results = [];
        for (const x of xs) {
            // iteration does not proceed until `bar(x)` completes
            results.push(await bar(x));
        }
        return baz(results);
    }
    ```

    ## Correct Code Examples

    ```js
    async function foo(xs) {
        const results = [];
        for (const x of xs) {
            // push a promise to the array; it does not prevent the iteration
            results.push(bar(x));
        }
        // we wait for all the promises concurrently
        return baz(await Promise.all(results));
    }
    ```
    */
    #[derive(Default)]
    NoAwaitInLoop,
    errors,
    tags(Recommended),
    "no-await-in-loop"
}

#[typetag::serde]
impl CstRule for NoAwaitInLoop {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if let Some(err_node) = node.children().find(|node| node.kind() == AWAIT_EXPR) {
            for ancestor in node.ancestors() {
                match ancestor.kind() {
                    FN_DECL | FN_EXPR | ARROW_EXPR => return None,
                    FOR_OF_STMT if ancestor.to::<ast::ForOfStmt>().await_token().is_some() => {
                        return None
                    }
                    _ => {}
                }

                if ancestor.is_loop()
                    && ancestor
                        .child_with_ast::<ast::Stmt>()?
                        .range()
                        .contains_range(node.text_range())
                {
                    let err = ctx.err(self.name(), "Unexpected `await` in loop")
                        .primary(err_node, "this expression causes the loop to wait for the promise to resolve before continuing")
                        .footer_note("the promises are resolved one after the other, not at the same time")
                        .footer_help(format!("try adding the promises to an array, then resolving them all outside the loop using `{}`", color("Promise.all(/* promises */)")));

                    ctx.add_err(err);
                    return None;
                }
            }
        }
        None
    }
}

rule_tests! {
    NoAwaitInLoop::default(),
    err: {
        "
        async function foo() {
            const res = [];
            for(var i = 1; i < 20; i++) {
                res.push(await i);
            }
        }
        ",
        "
        async () => {
            while(true) {
                await i;
            }
        }
        "
    },
    ok: {
        "
        for (let i of await foo) {}
        "
    }
}
