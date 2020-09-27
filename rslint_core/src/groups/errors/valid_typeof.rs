use crate::rule_prelude::*;
use ast::{BinExpr, Expr, Literal, UnaryExpr};

declare_lint! {
    /**
    Enforce the use of valid string literals in a `typeof` comparison.

    ## Invalid Code Examples
    ```ignore
    typeof foo === "strnig"
    typeof foo == "undefimed"
    typeof bar != "nunber"
    typeof bar !== "fucntion"
    ```
    */
    #[derive(Default)]
    ValidTypeof,
    errors,
    "valid-typeof",

    #[serde(default)]
    pub require_string_literals: bool,
}

const VALID_TYPES: [&str; 8] = [
    "undefined",
    "object",
    "boolean",
    "number",
    "string",
    "function",
    "symbol",
    "bigint",
];

#[typetag::serde]
impl CstRule for ValidTypeof {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let expr = node.try_to::<BinExpr>()?;
        if !expr.comparison() {
            return None;
        }
        let (lhs, rhs) = (expr.lhs()?, expr.rhs()?);

        let cmp_value = if is_typeof_expr(&lhs) {
            rhs
        } else if is_typeof_expr(&rhs) {
            lhs
        } else {
            return None;
        };

        let str_literal = cmp_value
            .syntax()
            .try_to::<Literal>()
            .and_then(|lit| Some((lit.inner_string_text()?, lit.range())));

        let (literal, literal_range) = if self.require_string_literals {
            if let Some(lit) = str_literal {
                lit
            } else if is_typeof_expr(&cmp_value) {
                return None;
            } else {
                let err = ctx
                    .err(self.name(), "invalid typeof comparison value")
                    .primary(cmp_value.range(), "");
                ctx.add_err(err);
                return None;
            }
        } else {
            str_literal?
        };

        if !VALID_TYPES.iter().any(|ty| *ty == literal) {
            let literal = String::from(literal);
            let suggestion =
                util::find_best_match_for_name(VALID_TYPES.iter().copied(), &literal, None);

            let err = ctx
                .err(self.name(), "invalid typeof comparison value")
                .primary(literal_range, "");

            let err = if let Some(suggestion) = suggestion {
                err.note(format!(
                    "help: a type with a similair name exists: `{}`",
                    suggestion
                ))
            } else {
                err
            };

            ctx.add_err(err);
        }
        None
    }
}

fn is_typeof_expr(expr: &Expr) -> bool {
    expr.syntax()
        .try_to::<UnaryExpr>()
        .filter(|expr| expr.op() == Some(op!(typeof)))
        .is_some()
}

rule_tests! {
    ValidTypeof::default(),
    err: {
        r#"typeof foo === "strnig""#,
        r#"typeof foo == "undefimed""#,
        r#"typeof bar != "nunber""#,
        r#"typeof bar !== "fucntion""#
    },
    ok: {
        r#"typeof foo === "string""#,
        r#"typeof bar == "undefined""#,
        "typeof foo === baz",
        "typeof foo === 4",
        "typeof bar === typeof qux"
    }
}
