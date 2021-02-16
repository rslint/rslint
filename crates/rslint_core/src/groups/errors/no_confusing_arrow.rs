use crate::rule_prelude::*;
use ast::{ArrowExpr, Expr};

declare_lint! {
    /**
    Disallow arrow functions where they could be confused with comparisons.

    Arrow functions (`=>`) are similar in syntax to some comparison operators (`>`, `<`, `<=`, and `>=`).
    This rule warns against using the arrow function syntax in places where it could be confused with
    a comparison operator

    Here's an example where the usage of `=>` could be confusing:

    ```js
    // The intent is not clear
    var x = a => 1 ? 2 : 3;
    // Did the author mean this
    var x = function (a) { return 1 ? 2 : 3 };
    // Or this
    var x = a >= 1 ? 2 : 3;
    ```

    ## Incorrect Code Examples

    ```js
    var x = a => 1 ? 2 : 3;
    var x = (a) => 1 ? 2 : 3;
    ```
    */
    #[serde(default)]
    NoConfusingArrow,
    errors,
    tags(Recommended),
    "no-confusing-arrow",
    /// Relaxes the rule and accepts parenthesis as a valid "confusion-preventing" syntax.
    /// `true` by default.
    pub allow_parens: bool
}

impl Default for NoConfusingArrow {
    fn default() -> Self {
        Self { allow_parens: true }
    }
}

#[typetag::serde]
impl CstRule for NoConfusingArrow {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let function_stmt = node.try_to::<ArrowExpr>()?;
        let expr = function_stmt.body()?.syntax().try_to::<Expr>()?;

        if is_conditional(&expr) && !(self.allow_parens && is_parenthesised(&expr)) {
            let diagnostic = ctx
                .err(
                    self.name(),
                    "arrow function in ternary expression could be mistaken for a comparison",
                )
                .primary(
                    function_stmt.syntax(),
                    "it could be confused with a comparison operator",
                );

            ctx.add_err(diagnostic);
        }

        None
    }
}

fn is_conditional(expr: &Expr) -> bool {
    match expr {
        Expr::CondExpr(_) => true,
        Expr::GroupingExpr(group) => group.inner().map_or(false, |e| is_conditional(&e)),
        _ => false,
    }
}

fn is_parenthesised(expr: &Expr) -> bool {
    matches!(expr, Expr::GroupingExpr(_))
}

rule_tests! {
    NoConfusingArrow::default(),
    err: {
        "a => 1 ? 2 : 3",
        "var x = a => 1 ? 2 : 3",
        "var x = (a) => 1 ? 2 : 3",
    },
    ok: {
        "a => { return 1 ? 2 : 3; }",
        "var x = a => { return 1 ? 2 : 3; }",
        "var x = (a) => { return 1 ? 2 : 3; }",
        "var x = a => (1 ? 2 : 3)",
    }
}

rule_tests! {
    allow_parens_false_valid,
    allow_parens_false_invalid,
    NoConfusingArrow {
        allow_parens: false,
    },
    err: {
        "a => 1 ? 2 : 3",
        "var x = a => 1 ? 2 : 3",
        "var x = (a) => 1 ? 2 : 3",
        "var x = a => (1 ? 2 : 3)",
    },
    ok: {
        "a => { return 1 ? 2 : 3; }",
        "var x = a => { return 1 ? 2 : 3; }",
        "var x = (a) => { return 1 ? 2 : 3; }",
    }
}
