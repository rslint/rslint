use crate::rule_prelude::*;
use ast::*;
use SyntaxKind::*;

declare_lint! {
    /**
    `Symbol` shouldn't be constructed using `new` keyword, instead
    it should be called as a function.

    ## Incorrect code examples

    ```js
    // This call results in TypeError
    const fooSymbol = new Symbol("foo"); 
    ```

    ## Correct code examples

    ```js
    const fooSymbol = Symbol("foo");
    ```
    */
    #[derive(Default)]
    NoNewSymbol,
    errors,
    "no-new-symbol",
}

#[typetag::serde]
impl CstRule for NoNewSymbol {
    #[allow(clippy::blocks_in_if_conditions)]
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == NEW_EXPR
        {
            let new_expr = node.to::<NewExpr>();

            if new_expr.object()?.syntax().text() == "Symbol" {
                let err = ctx
                    .err(
                        self.name(),
                        "`Symbol` cannot be called as a constructor.",
                    ).primary(
                        new_expr.new_token().unwrap().text_range(),
                        "this operator is redundant...",
                    );

                ctx.add_err(err);
            }
        }
        None
    }
}

rule_tests! {
    NoNewSymbol::default(),
    err: {
        "
        new Symbol()
        ",
    },
    ok: {
        "
        Symbol()
        ",
        "
        new SomeClass()
        "
    }
}
