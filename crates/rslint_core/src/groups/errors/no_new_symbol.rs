use crate::rule_prelude::*;
use ast::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow constructing `Symbol` using `new`.

    `Symbol` shouldn't be constructed using `new` keyword since it results in a `TypeError`, instead
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
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == NEW_EXPR {
            let new_expr = node.to::<NewExpr>();

            if new_expr.object()?.syntax().text() == "Symbol" {
                let err = ctx
                    .err(self.name(), "`Symbol` cannot be called as a constructor.")
                    .primary(node, "")
                    .suggestion(
                        node,
                        "help: call it as a function instead",
                        "Symbol()",
                        Applicability::MaybeIncorrect,
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
