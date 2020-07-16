use crate::cst_rule;
use crate::diagnostic::DiagnosticBuilder;
use crate::visit::{Node, Visit};
use rslint_parse::lexer::token::{BinToken, BinToken::*, TokenType};
use rslint_parse::parser::cst::expr::*;

cst_rule! {
    "no-compare-neg-zero",
    NoCompareNegZero
}

const CHECKED_BIN: [BinToken; 8] = [
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equality,
    StrictEquality,
    Inequality,
    StrictInequality,
];

fn get_neg_zero<'a>(left: &'a Expr, right: &'a Expr, source: &str) -> Option<&'a Expr> {
    if let Expr::Unary(UnaryExpr { op: TokenType::BinOp(BinToken::Subtract), ref object, ..}) = left {
        if object.span().content(source) == "0" {
            return Some(left)
        }
        None
    } else {
        if let Expr::Unary(UnaryExpr { op: TokenType::BinOp(BinToken::Subtract), ref object, ..}) = right {
            if object.span().content(source) == "0" {
                return Some(right)
            }
            None
        } else {
            None
        }
    }
}

impl Visit for NoCompareNegZeroVisitor<'_, '_> {
    fn visit_binary_expr(&mut self, expr: &BinaryExpr, _: &dyn Node) {
        match expr.op {
            TokenType::BinOp(ref tok) if CHECKED_BIN.contains(tok) => {
                let neg = get_neg_zero(&expr.left, &expr.right, self.ctx.file_source);
                if neg.is_some() {
                    let err_expr = neg.unwrap();
                    let span = if err_expr == &*expr.left {
                        expr.left.span()
                    } else {
                        expr.right.span()
                    };

                    let err = DiagnosticBuilder::error(
                        self.ctx.file_id,
                        "no-compare-neg-zero",
                        "Comparison against `-0` is not allowed",
                    )
                    .primary(expr.span, "")
                    .secondary(span.to_owned(), "Help: Did you mean to compare against `0`?")
                    .help("Note: If you meant to exactly compare against `-0` you should use `Object.is(x, -0)`");
    
                    self.ctx.diagnostics.push(err.into());
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_lint_err, assert_lint_ok};
    use crate::rules::groups::errors::no_compare_neg_zero::NoCompareNegZero;

    #[test]
    fn no_compare_neg_zero_err() {
        assert_lint_err! {
            NoCompareNegZero,
            "0 == -0" => 0..7,
            "-0 === -0" => 0..9,
            "\"aa\" === -0",
            "foo <= -0",
            "foo >= -0",
            "if (x === -0) {}" => 4..12,
            "var a = x === -0;",
        };
    }

    #[test]
    fn no_compare_neg_zero_ok() {
        assert_lint_ok! {
            NoCompareNegZero,
            "0 == 0",
            "-0 ",
        }
    }
}