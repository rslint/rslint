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

    ```js
    if (foo == NaN) {
        // unreachable
    }

    if (NaN != NaN) {
        // always runs
    }
    ```

    ## Correct Code Examples

    ```js
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
    tags(Recommended),
    "use-isnan",
    /// Switch statements use `===` internally to match an expression, therefore `switch (NaN)` and `case NaN` will never match.
    /// This rule disables uses like that which are always incorrect (true by default)
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

                let mut err = if opposite.text().len() <= 20 {
                    ctx.err(
                        self.name(),
                        format!(
                            "comparing `{}` to `NaN` using `{}` will always return {}",
                            opposite.text(),
                            expr.op_token().unwrap().text(),
                            always_text
                        ),
                    )
                } else {
                    ctx.err(
                        self.name(),
                        format!(
                            "comparisons to `NaN` with `{}` will always return {}",
                            expr.op_token().unwrap().text(),
                            always_text
                        ),
                    )
                }
                .primary(expr.range(), "")
                .footer_note("`NaN` is not equal to anything including itself");

                // telling the user to use isNaN for `<`, `>`, etc is a bit misleading so we won't do it if that is the case
                if op == op!(==) || op == op!(===) {
                    err = err.suggestion(
                        expr.range(),
                        "use `isNaN` instead",
                        format!("isNaN({})", opposite),
                        Applicability::Always,
                    );
                } else if op == op!(!=) || op == op!(!==) {
                    err = err.suggestion(
                        expr.range(),
                        "use `isNaN` instead",
                        format!("!isNaN({})", opposite),
                        Applicability::Always,
                    );
                }

                ctx.add_err(err);
            }
            SWITCH_STMT if self.enforce_for_switch_case => {
                // TODO: a suggestion for this
                let stmt = node.to::<SwitchStmt>();
                let expr = stmt.test()?.condition()?;
                if expr.text() == "NaN" {
                    let err = ctx
                        .err(
                            self.name(),
                            "a switch statement with a test of `NaN` will never match",
                        )
                        .primary(expr.range(), "")
                        .footer_note("`NaN` is not equal to anything including itself");

                    ctx.add_err(err);
                }
            }
            CASE_CLAUSE if self.enforce_for_switch_case => {
                // TODO: suggestion for this
                let case = node.to::<CaseClause>();
                let expr = case.test()?;
                if expr.text() == "NaN" {
                    let err = ctx
                        .err(self.name(), "a case with a test of `NaN` will never match")
                        .primary(expr.range(), "")
                        .footer_note("`NaN` is not equal to anything including itself");

                    ctx.add_err(err);
                }
            }
            CALL_EXPR if self.enforce_for_index_of => {
                // TODO: suggestion for this
                let expr = node.to::<CallExpr>();
                let callee = expr.callee()?;
                let node = callee.syntax();
                // rustfmt puts the last call's args each on a new line for some reason which is very ugly
                #[rustfmt::skip]
                let is_index_call =
                    node.structural_lossy_token_eq(&["Array", ".", "prototype", ".", "indexOf"])
                        || node.structural_lossy_token_eq(&["Array", ".", "prototype", ".", "lastIndexOf"])
                        || node.structural_lossy_token_eq(&["String", ".", "prototype", ".", "indexOf"])
                        || node.structural_lossy_token_eq(&["String", ".", "prototype", ".", "lastIndexOf"]);

                let second_arg_is_nan = expr
                    .arguments()
                    .map(|a| a.args().nth(1).filter(|x| x.text() == "NaN"))
                    .flatten()
                    .is_some();

                if (is_indexof_static_prop(&callee)
                    && expr.arguments()?.args().next()?.text() == "NaN"
                    && !is_index_call)
                    || (is_index_call && second_arg_is_nan)
                {
                    let err = ctx
                        .err(
                            self.name(),
                            "an index check with `NaN` will always return `-1`",
                        )
                        .primary(expr.range(), "")
                        .footer_help("index checks use `===` internally, which will never match because `NaN` is not equal to anything");

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

rule_tests! {
    indexof_ok,
    indexof_err,
    UseIsnan {
        enforce_for_index_of: true,
        enforce_for_switch_case: false
    },
    err: {
        "Array.prototype.indexOf(foo, NaN)",
        "Array.prototype.lastIndexOf(foo, NaN)",
        "String.prototype.indexOf(foo, NaN)",
        "String.prototype.lastIndexOf(foo, NaN)",
    },
    ok: {
        "Array.prototype.indexOf(NaN)",
        "Array.prototype.lastIndexOf(NaN)",
        "String.prototype.indexOf(NaN)",
        "String.prototype.lastIndexOf(NaN)",
    }
}
