use crate::rule_prelude::*;
use ast::{BinExpr, Expr, UnaryOp};
use rslint_parser::{SyntaxText, TextRange};
use SyntaxKind::*;

declare_lint! {
    /**
    Enforces the to use valid string literals in a `typeof` comparison.

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
    "valid-typeof"
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
        if node.kind() == BIN_EXPR {
            let expr = node.to::<BinExpr>();
            let (literal, range) = get_type_literal(expr.lhs(), expr.rhs())?;

            if !VALID_TYPES.iter().any(|ty| *ty == literal) {
                let literal = String::from(literal);
                let suggestion = VALID_TYPES
                    .iter()
                    .map(|ty| (*ty, strsim::levenshtein(ty, &literal)))
                    .filter(|(_, d)| *d < 4)
                    .next();

                let err = ctx.err(self.name(), "Invalid typeof comparison value");

                let err = if let Some((suggestion, _)) = suggestion {
                    err.primary(
                        range,
                        format!("help: a type with a similair name exists: `{}`", suggestion),
                    )
                } else {
                    err.primary(range, "invalid type for typeof comparison")
                };

                ctx.add_err(err);
            }
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
    if let Some(Expr::UnaryExpr(unary)) = expr {
        if let Some(UnaryOp::Typeof) = unary.op() {
            true
        } else {
            false
        }
    } else {
        false
    }
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
