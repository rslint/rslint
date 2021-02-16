use crate::rule_prelude::*;
use ast::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow constant conditions which always yield one result.

    Constant conditions such as `if (true) {}` are almost always a mistake. Constant
    conditions always yield a single result which almost always ends up in unwanted behavior.
    This rule is aimed at catching those conditions in `if`, `do while`, `while`, and `for` statements, as well as
    conditional expressions.

    ## Incorrect Code Examples

    ```js
    if (true) {
        //    ^ this block is always used
    } else {
    //^^^^ this else block is unreachable
    }
    ```

    ```js
    // This loop endlessly runs
    for(foo = 5; 5; foo++) {

    }
    ```

    ## Correct Code Examples

    ```js
    if (foo) {
        /* */
    }
    ```
    */
    #[derive(Default)]
    NoConstantCondition,
    errors,
    tags(Recommended),
    "no-constant-condition"
}

#[typetag::serde]
impl CstRule for NoConstantCondition {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let mut notes = vec![];
        let cond = match node.kind() {
            IF_STMT | DO_WHILE_STMT | WHILE_STMT => {
                if let Some(cond) = node.children().find_map(|node| node.try_to::<Condition>()) {
                    if !util::is_const(cond.condition()?, true, &mut notes) {
                        return None;
                    }
                    cond.condition().unwrap()
                } else {
                    return None;
                }
            }
            COND_EXPR => {
                let cond = node.to::<CondExpr>().test()?;
                if !util::is_const(cond.clone(), true, &mut notes) {
                    return None;
                }
                cond
            }
            FOR_STMT => {
                let cond = node.to::<ForStmt>().test()?.expr()?;
                if !util::is_const(cond.clone(), true, &mut notes) {
                    return None;
                }
                cond
            }
            _ => return None,
        };

        let mut err = ctx.err(self.name(), "unexpected constant condition");
        if let Some(condition_value) = util::simple_bool_coerce(cond.clone()) {
            err = util::simple_const_condition_context(node.clone(), condition_value, err);
        } else {
            err = err.primary(cond.syntax(), "this condition always yields one result")
        }
        ctx.add_err(err);

        None
    }
}

rule_tests! {
    NoConstantCondition::default(),
    err: {
        "if(6) {}",
        "if(6 - 7 || 3 ? 7 && 2 : NaN + NaN || 2) {}",
        "if (true) {}",
        "if (NaN) {} else {}",
        "6 + 2 ? false : NaN",
        "false ? false : false ? false : false",
        "while (true) {}",
        "do { /* */ } while (NaN ? NaN : true)",
        "do { } while (NaN ? Infinity : true)"
    },
    ok: {
        "if (foo) {}",
        "if (false > foo) {} else {}",
        "if (foo ? NaN : Infinity) {}",
        "do {} while (foo + 6)",
        "for(var i = 5; foo; i++) {}"
    }
}
