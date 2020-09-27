use crate::rule_prelude::*;
use ast::{BinExpr, Expr, UnaryExpr};
use rslint_parser::{SyntaxText, TextRange};

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

        let (literal, range) = get_type_literal(expr.lhs(), expr.rhs())?;

        if !VALID_TYPES.iter().any(|ty| *ty == literal) {
            let literal = String::from(literal);
            let suggestion =
                util::find_best_match_for_name(VALID_TYPES.iter().copied(), &literal, None);

            let err = ctx
                .err(self.name(), "invalid typeof comparison value")
                .primary(range, "");

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

fn get_type_literal(lhs: Option<Expr>, rhs: Option<Expr>) -> Option<(SyntaxText, TextRange)> {
    fn inner(expr: Expr) -> Option<(SyntaxText, TextRange)> {
        if let Expr::Literal(lit) = expr {
            lit.inner_string_text().map(|text| (text, lit.range()))
        } else {
            None
        }
    }

    if is_typeof_expr(lhs.as_ref()) {
        inner(rhs?)
    } else if is_typeof_expr(rhs.as_ref()) {
        inner(lhs?)
    } else {
        None
    }
}

fn is_typeof_expr(expr: Option<&Expr>) -> bool {
    expr.and_then(|e| e.syntax().try_to::<UnaryExpr>())
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
    "typeof bar === typeof qux"
  }
}
