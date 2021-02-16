use crate::rule_prelude::*;
use ast::Expr;
use SyntaxKind::*;

declare_lint! {
    /**
    Forbid the use of assignment expressions in conditions which may yield unwanted behavior.

    Assignment expressions return the value assigned:

    ```js
    let foo = 5;

    console.log(foo = 8); // 8
    console.log(foo += 4) // foo + 4 (12 in this case)
    ```

    Users often make a typo and end up using `=` instead of `==` or `===` in conditions in statements
    like `if`, `while`, `do_while`, and `for`. This is erroneous and is most likely unwanted behavior
    since the condition used will actually be the value assigned.

    ## Incorrect Code Examples

    ```js
    let foo = 5;

    if (foo = 6) {
    //      ^^^ assignments return the value assigned, therefore the condition checks `6`
    //          `6` is always truthy, therefore the if statement always runs even if we dont want it to.

    } else {}
    //^^^^ it makes this else unreachable

    foo // 6
    ```
    */
    #[serde(default)]
    NoCondAssign,
    errors,
    tags(Recommended),
    "no-cond-assign",
    /// Allow an assignment if they are enclosed in parentheses to allow
    /// things like reassigning a variable.
    pub allow_parens: bool
}

impl Default for NoCondAssign {
    fn default() -> Self {
        Self { allow_parens: true }
    }
}

const COND_CHECKED: [SyntaxKind; 5] = [IF_STMT, WHILE_STMT, DO_WHILE_STMT, FOR_STMT, COND_EXPR];

#[typetag::serde]
impl CstRule for NoCondAssign {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let cond = condition(node)?;
        if COND_CHECKED.contains(&node.kind()) && check(&cond, self.allow_parens) {
            let err = ctx
                .err(
                    self.name(),
                    format!(
                        "unexpected assignment inside a {} condition",
                        node.readable_stmt_name()
                    ),
                )
                .primary(
                    cond.syntax(),
                    "this condition results in unexpected behavior",
                )
                .suggestion(
                    cond.range(),
                    "try using `===` to compare instead",
                    "===",
                    Applicability::MaybeIncorrect,
                )
                .footer_note(format!(
                    "this makes the condition equivalent to `{}`",
                    color(&help_expr(cond.syntax()))
                ));

            ctx.add_err(err);
        }
        None
    }
}

fn condition(node: &SyntaxNode) -> Option<Expr> {
    if node.kind() == FOR_STMT {
        return node.to::<ast::ForStmt>().test()?.expr();
    }

    if node.kind() == COND_EXPR {
        return node.to::<ast::CondExpr>().test();
    }

    node.children()
        .find(|it| it.kind() == CONDITION)?
        .to::<ast::Condition>()
        .condition()
}

fn check(expr: &ast::Expr, allow_parens: bool) -> bool {
    match expr {
        Expr::AssignExpr(_) => {
            if expr
                .syntax()
                .parent()
                .map(|x| x.kind() == GROUPING_EXPR && allow_parens)
                .unwrap_or_default()
            {
                return false;
            }
            true
        }
        Expr::GroupingExpr(group) => {
            if let Some(inner) = group.inner() {
                return check(&inner, allow_parens);
            }
            false
        }
        Expr::BinExpr(bin) if bin.conditional() => {
            bin.lhs()
                .map(|e| check(&e, allow_parens))
                .unwrap_or_default()
                || bin
                    .rhs()
                    .map(|e| check(&e, allow_parens))
                    .unwrap_or_default()
        }
        _ => false,
    }
}

fn help_expr(expr: &SyntaxNode) -> String {
    match expr.kind() {
        ASSIGN_EXPR => util::get_assignment_expr_value(expr.to()),
        BIN_EXPR => {
            let expr = expr.to::<ast::BinExpr>();
            format!(
                "{} {} {}",
                expr.lhs()
                    .map(|e| help_expr(&e.syntax()))
                    .unwrap_or_default(),
                expr.op_token()
                    .map(|t| t.text().to_string())
                    .unwrap_or_default(),
                expr.rhs()
                    .map(|e| help_expr(&e.syntax()))
                    .unwrap_or_default()
            )
        }
        GROUPING_EXPR => format!(
            "({})",
            help_expr(&expr.to::<ast::GroupingExpr>().inner().unwrap().syntax())
        ),
        _ => expr.trimmed_text().to_string(),
    }
}

rule_tests! {
    NoCondAssign::default(),
    err: {
        "
        if (foo = 54) {}
        ",
        "
        while (foo = 1) {}
        ",
        "
        do { /* */ } while (bar = 1)
        ",
        "
        for(;foo = 4; bar) {}
        ",
        "if (bar = 5 ? foo : bar) {}"
    },
    ok: {}
}
