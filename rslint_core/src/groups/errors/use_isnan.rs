use crate::rule_prelude::*;
use ast::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow incorrect comparisons against `NaN`.

    `NaN` is a special `Number` value used to represent "not a number" results in calculations.
    This value is specified in the IEEE Standard for Binary Floating-Point-Arithmetic.

    In JavaScript, `NaN` is unique, it is not equal to anything, including itself! therefore
    any comparisons to it will either always yield `true` or `false`. Therefore you should
    use `isNaN(/* num */)` instead to test if a value is `NaN`. This rule is aimed at removing this footgun.

    ## Invalid Code Examples

    ```ignore
    if (foo == NaN) {
        // unreachable
    }

    if (NaN != NaN) {
        // always runs
    }
    ```

    ## Correct Code Examples

    ```ignore
    if (isNaN(foo)) {
        /* */
    }

    if (!isNaN(foo)) {
        /* */
    }
    ```
    */
    #[serde(default)]
    UseIsnan,
    errors,
    "use-isnan",
    /// Switch statements use `===` internally to match an expression, therefore `switch (NaN)` and `case NaN` will never match.
    /// This rule disables uses like that which are always incorrect (false by default)
    pub enforce_for_switch_case: bool,
    /// Index functions like `indexOf` and `lastIndexOf` use `===` internally, therefore matching them against `NaN` will always
    /// yield `-1`. This option disallows using `indexOf(NaN)` and `lastIndexOf(NaN)` (false by default)
    pub enforce_for_index_of: bool
}

impl Default for UseIsnan {
    fn default() -> Self {
        Self {
            enforce_for_switch_case: true,
            enforce_for_index_of: false,
        }
    }
}

#[typetag::serde]
impl CstRule for UseIsnan {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        match node.kind() {
            BIN_EXPR => {
                let expr = node.to::<BinExpr>();
                if !expr.comparison() {
                    return None;
                }

                let opposite = if expr.lhs().filter(|e| e.text() == "NaN").is_some() {
                    expr.rhs()?
                } else if expr.rhs().filter(|e| e.text() == "NaN").is_some() {
                    expr.lhs()?
                } else {
                    return None;
                };
                let op = expr.op().unwrap();

                let always_text = if matches!(op, op!(!=) | op!(!==)) {
                    "true"
                } else {
                    "false"
                };

                let mut err = ctx
                    .err(
                        self.name(),
                        format!(
                            "comparing `{}` to `NaN` using `{}` will always return {}",
                            opposite.text(),
                            expr.op_token().unwrap().text(),
                            always_text
                        ),
                    )
                    .primary(expr.range(), "")
                    .note("note: `NaN` is not equal to anything including itself");

                // telling the user to use isNaN for `<`, `>`, etc is a bit misleading so we won't do it if that is the case
                if op == op!(==) || op == op!(===) {
                    err = err.note(format!(
                        "help: use `isNaN` instead: `{}`",
                        color(&format!("isNaN({})", opposite))
                    ))
                } else if op == op!(!=) || op == op!(!==) {
                    err = err.note(format!(
                        "help: use `isNaN` instead: `{}`",
                        color(&format!("!isNaN({})", opposite))
                    ))
                }

                ctx.add_err(err);
            }
            SWITCH_STMT if self.enforce_for_switch_case => {
                let stmt = node.to::<SwitchStmt>();
                let expr = stmt.test()?.condition()?;
                if expr.text() == "NaN" {
                    let err = ctx
                        .err(
                            self.name(),
                            "a switch statement with a test of `NaN` will never match",
                        )
                        .primary(expr.range(), "")
                        .note("note: `NaN` is not equal to anything including itself");

                    ctx.add_err(err);
                }
            }
            CASE_CLAUSE if self.enforce_for_switch_case => {
                let case = node.to::<CaseClause>();
                let expr = case.test()?;
                if expr.text() == "NaN" {
                    let err = ctx
                        .err(self.name(), "a case with a test of `NaN` will never match")
                        .primary(expr.range(), "")
                        .note("note: `NaN` is not equal to anything including itself");

                    ctx.add_err(err);
                }
            }
            CALL_EXPR if self.enforce_for_index_of => {
                let expr = node.to::<CallExpr>();
                let callee = expr.callee()?;
                if is_indexof_static_prop(&callee)
                    && expr.arguments()?.args().next()?.text() == "NaN"
                {
                    let err = ctx
                        .err(
                            "use-isnan",
                            "an index check with `NaN` will always return `-1`",
                        )
                        .primary(expr.range(), "")
                        .note("help: index checks use `===` internally, which will never match because `NaN` is not equal to anything");

                    ctx.add_err(err);
                }
            }
            _ => {}
        }
        None
    }
}

const INDEX_OF_NAMES: [&str; 2] = ["lastIndexOf", "indexOf"];

fn is_indexof_static_prop(expr: &Expr) -> bool {
    match expr {
        Expr::BracketExpr(brack_expr) => brack_expr
            .syntax()
            .try_to::<Literal>()
            .and_then(|l| l.inner_string_text())
            .filter(|text| INDEX_OF_NAMES.contains(&text.to_string().as_str()))
            .is_some(),
        Expr::DotExpr(dotexpr) => dotexpr
            .prop()
            .filter(|prop| INDEX_OF_NAMES.contains(&prop.to_string().as_str()))
            .is_some(),
        _ => false,
    }
}

// TODO: add a way to test rules with options
rule_tests! {
    UseIsnan::default(),
    err: {
        "123 == NaN;",
        "123 === NaN;",
        "NaN === \"abc\";",
        "NaN == \"abc\";",
        "123 != NaN;",
        "123 !== NaN;",
        "NaN !== \"abc\";",
        "NaN != \"abc\";",
        "NaN < \"abc\";",
        "\"abc\" < NaN;",
        "NaN > \"abc\";",
        "\"abc\" > NaN;",
        "NaN <= \"abc\";",
        "\"abc\" <= NaN;",
        "NaN >= \"abc\";",
        "\"abc\" >= NaN;"
    },
    ok: {
        "var x = NaN;",
        "isNaN(NaN) === true;",
        "isNaN(123) !== true;",
        "Number.isNaN(NaN) === true;",
        "Number.isNaN(123) !== true;",
        "foo(NaN + 1);",
        "foo(1 + NaN);",
        "foo(NaN - 1)",
        "foo(1 - NaN)",
        "foo(NaN * 2)",
        "foo(2 * NaN)",
        "foo(NaN / 2)",
        "foo(2 / NaN)",
        "var x; if (x = NaN) { }",
        "foo.indexOf(NaN)",
        "foo.lastIndexOf(NaN)",
    }
}
