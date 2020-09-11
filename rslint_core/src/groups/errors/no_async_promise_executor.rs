use crate::rule_prelude::*;
use SyntaxKind::NEW_EXPR;
use rslint_parser::TextRange;

declare_lint! {
    /**
    Disallow async functions as promise executors. 

    Promise executors are special functions inside `new Promise()` constructors which take a `resolve` and 
    `reject` parameter to resolve or reject the promise. The function is a normal function therefore it could be 
    an async function. However this is usually wrong because: 
        - Any errors thrown by the function are lost.
        - It usually means the new promise is unnecessary. 

    ## Incorrect code examples 

    ```ignore
    let foo = new Promise(async (resolve, reject) => {
        doSomething(bar, (err, res)) => {
           /* */ 
        });
    });
    ```

    ```ignore
    let foo = new Promise(async function(resolve, reject) => {
        /* */
    });
    ```

    ## Correct code examples 

    Use a normal non-async function. 

    ```ignore
    let foo = new Promise(function(resolve, reject) => {
        /* */
    })
    ```
    */
    #[derive(Default)]
    NoAsyncPromiseExecutor,
    errors,
    "no-async-promise-executor"
}

#[typetag::serde]
impl CstRule for NoAsyncPromiseExecutor {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == NEW_EXPR && node.to::<ast::NewExpr>().object()?.syntax().text() == "Promise" {
            dbg!(&node);
            if let Some(range) = check_arg(node.to::<ast::NewExpr>().arguments()?.args().next()?) {
                let err = ctx.err(self.name(), "Don't use async functions for promise executors")
                    .primary(range, "")
                    .note("note: any errors thrown by the function will be lost");
                
                ctx.add_err(err);
            }
        }
        None
    }
}

fn check_arg(arg: ast::Expr) -> Option<TextRange> {
    Some(match dbg!(arg) {
        ast::Expr::FnExpr(func) if func.async_token().is_some() => func.syntax().trimmed_range(),
        ast::Expr::ArrowExpr(arrow) if arrow.async_token().is_some() => arrow.syntax().trimmed_range(),
        _ => return None,
    })
}

rule_tests! {
    NoAsyncPromiseExecutor::default(),
    err: {
        "new Promise(async () => {})",
        "new Promise(async function*() {})",
        "new Promise(async function() {}, foo)"
    },
    ok: {
        "new Promise(() => {})",
        "new Promise(function foo() {}, foo)"
    }
}
