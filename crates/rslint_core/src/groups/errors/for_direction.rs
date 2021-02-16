use crate::rule_prelude::*;
use ast::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow for loops which update their counter in the wrong direction.

    A for loop with a counter may update its value in the wrong direction. that is to say, if i made
    a counter with a value of `0`, if the for statement checked if `counter < 10` and the update went `counter--`,
    that loop would be infinite. This is because `counter` will never be smaller than `10` because `counter--` always
    yields a value smaller than 10. A for loop which does this is almost always a bug because it is either
    unreachable or infinite.

    ## Incorrect Code Examples

    ```js
    for (var i = 0; i < 10; i--) {
        /* infinite loop */
    }
    ```

    ```js
    for (var i = 10; i >= 20; i++) {
        /* unreachable */
    }
    ```

    ## Correct Code Examples

    ```js
    for (var i = 0; i < 10; i++) {

    }
    ```
    */
    #[derive(Default)]
    ForDirection,
    errors,
    tags(Recommended),
    "for-direction"
}

#[typetag::serde]
impl CstRule for ForDirection {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if let Some(test) = node
            .try_to::<ForStmt>()
            .and_then(|f| f.test())
            .and_then(|test| test.expr())
        {
            let for_stmt = node.to::<ForStmt>();
            if for_stmt.update().is_some()
                && test.syntax().try_to::<BinExpr>()?.lhs()?.syntax().kind() == NAME_REF
            {
                let test_bin = test.syntax().to::<BinExpr>();
                if test_bin.rhs().is_none() || for_stmt.init().is_none() {
                    return None;
                }

                let counter = test_bin.lhs().unwrap().syntax().to::<NameRef>();
                let op = test_bin.op()?;

                let wrong_direction = if op == BinOp::LessThan || op == BinOp::LessThanOrEqual {
                    -1
                } else if op == BinOp::GreaterThan || op == BinOp::GreaterThanOrEqual {
                    1
                } else {
                    return None;
                };

                if let Some(direction) = update_direction(&for_stmt, &counter) {
                    if direction == wrong_direction {
                        throw_err(for_stmt, &counter, ctx);
                    }
                }
            }
        }
        None
    }
}

fn update_direction(for_stmt: &ForStmt, counter: &NameRef) -> Option<i8> {
    let update = for_stmt.update()?.syntax().first_child()?;
    match update.kind() {
        UNARY_EXPR => {
            let expr = update.to::<UnaryExpr>();
            if expr.expr()?.syntax().try_to::<NameRef>()?.syntax().text() == counter.syntax().text()
            {
                let op = expr.op().unwrap();
                Some(if op == UnaryOp::Increment { 1 } else { -1 })
            } else {
                None
            }
        }
        ASSIGN_EXPR => assign_direction(update.to(), counter),
        _ => None,
    }
}

fn assign_direction(assign: AssignExpr, counter: &NameRef) -> Option<i8> {
    if assign.lhs()?.syntax().text() == counter.syntax().text() {
        match assign.op()? {
            AssignOp::AddAssign => maybe_negate_direction(assign.rhs()?, 1),
            AssignOp::SubtractAssign => maybe_negate_direction(assign.rhs()?, -1),
            _ => Some(0),
        }
    } else {
        None
    }
}

fn maybe_negate_direction(rhs: Expr, direction: i8) -> Option<i8> {
    Some(match rhs {
        Expr::UnaryExpr(unexpr) => {
            if unexpr.op()? == UnaryOp::Minus {
                -direction
            } else {
                direction
            }
        }
        Expr::NameRef(_) => 0,
        _ => direction,
    })
}

// TODO: we can say if the loop is unreachable once we have number parsing
fn throw_err(for_stmt: ForStmt, counter: &NameRef, ctx: &mut RuleCtx) {
    let bin = for_stmt
        .test()
        .unwrap()
        .syntax()
        .first_child()
        .unwrap()
        .to::<BinExpr>();
    let lhs = bin.lhs().unwrap().syntax().trimmed_text();
    let rhs = bin.rhs().unwrap().syntax().clone();
    let op = bin.op().unwrap();

    if let Some(lit) = rhs
        .try_to::<Literal>()
        .filter(|literal| literal.is_number())
    {
        if try_offer_context(&for_stmt, counter, op, lit, ctx).is_some() {
            return;
        }
    }

    let err = ctx
        .err(
            "for-direction",
            "For loop is updating the counter in the wrong direction",
        )
        .secondary(
            for_stmt.test().unwrap().range(),
            format!(
                "this test is checking if `{}` is {} `{}`...",
                lhs,
                lt_gt_name(op),
                rhs
            ),
        )
        .primary(
            for_stmt.update().unwrap().range(),
            format!(
                "...but `{}` is updating in the same direction",
                for_stmt.update().unwrap().syntax().trimmed_text()
            ),
        );

    ctx.add_err(err);
}

fn lt_gt_name(op: BinOp) -> &'static str {
    match op {
        BinOp::LessThan => "less than",
        BinOp::LessThanOrEqual => "less than or equal to",
        BinOp::GreaterThan => "greater than",
        BinOp::GreaterThanOrEqual => "greater than or equal to",
        _ => unreachable!(),
    }
}

/// try to offer even more context around the error if we know the initial numeric value of the counter
fn try_offer_context(
    for_stmt: &ForStmt,
    counter: &NameRef,
    op: BinOp,
    checked_value: Literal,
    ctx: &mut RuleCtx,
) -> Option<()> {
    let init = for_stmt.init()?;

    let initial_value = match init.inner().unwrap() {
        ForHead::Decl(decl) => {
            let decl = decl.declared().find(|declarator| {
                declarator.pattern().map_or(false, |pat| {
                    if let Pattern::SinglePattern(single) = pat {
                        single.syntax().text() == counter.syntax().text()
                    } else {
                        false
                    }
                })
            })?;
            decl.value()?
        }
        ForHead::Expr(Expr::AssignExpr(assign)) => {
            assign.lhs().and_then(|lhs| {
                if let PatternOrExpr::Expr(Expr::NameRef(name)) = lhs {
                    Some(name).filter(|name| name.syntax().text() == counter.syntax().text())
                } else {
                    None
                }
            })?;
            assign.rhs()?
        }
        _ => return None,
    };

    let mut err = ctx.err(
        "for-direction",
        "For loop is updating the counter in the wrong direction",
    );

    if let Some(LiteralKind::Number(num)) = initial_value
        .syntax()
        .try_to::<Literal>()
        .map(|lit| lit.kind())
    {
        if is_initially_unreachable(num, checked_value.as_number().unwrap(), op) {
            err = err
                .secondary(
                    init.syntax().trimmed_range(),
                    format!(
                        "{} is first declared as `{}`...",
                        counter.syntax().text(),
                        initial_value.syntax().text()
                    ),
                )
                .secondary(
                    for_stmt.test().unwrap().range(),
                    format!(
                        "...which makes this test unreachable because `{}` is not {} `{}`...",
                        initial_value.syntax().text(),
                        lt_gt_name(op),
                        checked_value.syntax().text()
                    ),
                )
                .primary(
                    for_stmt.update().unwrap().range(),
                    "...and this update will never make it true",
                );
        } else {
            err = err
                .secondary(
                    init.syntax(),
                    format!(
                        "{} is first declared as `{}`...",
                        counter.syntax().text(),
                        initial_value.syntax().text()
                    ),
                )
                .secondary(
                    for_stmt.test().unwrap().range(),
                    format!(
                        "...which makes this test always true because `{}` is always {} `{}`...",
                        initial_value.syntax().text(),
                        lt_gt_name(op),
                        checked_value.syntax().text()
                    ),
                )
                .primary(
                    for_stmt.update().unwrap().range(),
                    "...and this update will never make the condition false",
                );
        }
        ctx.add_err(err);
        return Some(());
    }
    None
}

fn is_initially_unreachable(initial_value: f64, checked_value: f64, op: BinOp) -> bool {
    match op {
        BinOp::LessThan => initial_value >= checked_value,
        BinOp::LessThanOrEqual => initial_value > checked_value,
        BinOp::GreaterThan => initial_value <= checked_value,
        BinOp::GreaterThanOrEqual => initial_value < checked_value,
        _ => unreachable!(),
    }
}

rule_tests! {
    ForDirection::default(),
    err: {
        "for (var i = 0; i < 10; i--) {}",
        "for(let i = 0; i < 2; i--) {}",
        "for(let i = 0; i <= 2; i += -1) {}",
        "for(let i = 2; i >= 0; i -= -1) {}",
        "for(let i = 0; i < 2; i -= 1) {}",
        "for(let i = 2; i > 2; i++) {}",
        "for(let i = 2; i > 2; i += 1) {}",
        "for(let i = 5n; i < 2; i--) {}"
    },
    ok: {
        "for (var i = 0; i < 10; i++) {}",
        "for(let i = 2; i > 2; i -= 1) {}",
        "for(let i = 2; i >= 0; i -= 1) {}",
        "for(let i = 2; i > 2; i += -1) {}",
        "for(let i = 2; i >= 0; i += -1) {}",
        "for(let i = 0; i < 3;) {}",
        "for(let i = 5; i < 2; i |= 2) {}",
        "for(let i = 5n; i < 2n; i &= 2) {}"
    }
}
