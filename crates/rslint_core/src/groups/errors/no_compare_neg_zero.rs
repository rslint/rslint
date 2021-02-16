use crate::rule_prelude::*;

declare_lint! {
    /**
    Disallow comparison against `-0` which yields unexpected behavior.

    Comparison against `-0` causes unwanted behavior because it passes for both `-0` and `+0`.
    That is, `x == -0` and `x == +0` both pass under the same circumstances. If a user wishes
    to compare against `-0` they should use `Object.is(x, -0)`.

    ## Incorrect Code Examples

    ```js
    if (x === -0) {
           // ^^ this comparison works for both -0 and +0
    }
    ```

    ## Correct code examples

    ```js
    if (x === 0) {
        /* */
    }
    ```

    ```js
    if (Object.is(x, -0)) {
        /* */
    }
    ```
    */
    #[derive(Default)]
    NoCompareNegZero,
    errors,
    tags(Recommended),
    "no-compare-neg-zero"
}

#[typetag::serde]
impl CstRule for NoCompareNegZero {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.try_to::<ast::BinExpr>()?.comparison() {
            let bin = node.to::<ast::BinExpr>();
            let op = bin.op_token().unwrap();
            if let Some(expr) = bin
                .lhs()
                .filter(|e| unsafe_comparison(e))
                .and_then(|_| bin.rhs())
            {
                issue_err(expr, ctx, op.clone(), node);
            }
            if let Some(expr) = bin
                .rhs()
                .filter(|e| unsafe_comparison(e))
                .and_then(|_| bin.lhs())
            {
                issue_err(expr, ctx, op, node)
            }
        }
        None
    }
}

fn issue_err(expr: ast::Expr, ctx: &mut RuleCtx, op: SyntaxToken, parent: &SyntaxNode) {
    let err = ctx
        .err(
            "no-compare-neg-zero",
            format!(
                "comparison against `-0` with `{}` yields unexpected behavior",
                expr.text()
            ),
        )
        .primary(
            op,
            "...because this comparison passes for both `-0` and `+0`",
        )
        .suggestion(
            parent,
            "try using `Object.is` instead",
            format!("Object.is({}, -0)", expr.syntax().text()),
            Applicability::MaybeIncorrect,
        );

    ctx.fix()
        .replace(parent, format!("Object.is({}, -0)", expr.text()));

    ctx.add_err(err);
}

fn unsafe_comparison(expr: &ast::Expr) -> bool {
    expr.syntax().text() == "-0"
}

rule_tests! {
    NoCompareNegZero::default(),
    err: {
        "x == -0",
        "x != -0",
        "x === -0",
        "-0 === -0",
        "-0 == x",
        "-0 >= 1",
        "x < -0",
        "x !== -0"
    },
    ok: {
        "x === 0",
        "0 === 0",
        "Object.is(x, -0)"
    }
}
