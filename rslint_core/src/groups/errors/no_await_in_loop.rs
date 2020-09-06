use crate::rule_prelude::*;
use SyntaxKind::*;

declare_lint! {
    #[derive(Default)]
    NoAwaitInLoop,
    errors,
    "no-await-in-loop"
}

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

                if ancestor.is_loop() {
                    let err = ctx.err(self.name(), "Unexpected `await` in loop")
                        .primary(err_node.trimmed_range(), "this expression causes the loop to wait for the promise to resolve before continuing")
                        .note("note: the promises are resolved one after the other, not at the same time")
                        .note(format!("help: try adding the promises to an array, then resolving them all outside the loop with `{}`", color("Promise.all(/* promises */)")));

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
    ok: {}
}
