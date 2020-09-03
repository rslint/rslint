//! General utilities to make linting easier.

use crate::rule_prelude::*;
use ast::*;
use rslint_parser::TextRange;

/// Expands an assignment to the returned value, e.g. `foo += 5` -> `foo + 5`, `foo = 6` -> `6`
///
/// # Panics
/// Panics if the expression does not have an operator.
pub fn get_assignment_expr_value(expr: AssignExpr) -> std::string::String {
    assert!(expr.op().is_some());

    let tok = expr.syntax().first_lossy_token().unwrap();
    let op_str = tok.text();
    println!("{:#?}", op_str[..op_str.len() - 1].to_string());

    if op_str == "=" {
        expr.rhs()
            .map(|e| e.syntax().trimmed_text().to_string())
            .unwrap_or_default()
    } else {
        format!(
            "{} {} {}",
            expr.lhs()
                .map(|e| e.syntax().trimmed_text().to_string())
                .unwrap_or_default(),
            op_str[..op_str.len() - 1].to_string(),
            expr.rhs()
                .map(|e| e.syntax().trimmed_text().to_string())
                .unwrap_or_default()
        )
    }
}

/// Attempt to check if a simple expression is always truthy or always falsey.
///
/// For example, `true`, `false`, and `foo = true`, this does not consider math ops like `0 + 0`.
pub fn simple_bool_coerce(condition: Expr) -> Option<bool> {
    match condition {
        Expr::Literal(lit) => {
            let coerced = match lit.kind() {
                LiteralKind::Bool(val) => val,
                LiteralKind::Null => false,
                // TODO: Handle this better once we can parse the number value
                LiteralKind::Number => return None,
                LiteralKind::String => !lit.inner_string_text().unwrap().is_empty(),
                LiteralKind::Regex => true,
            };
            Some(coerced)
        }
        Expr::ObjectExpr(_) | Expr::ArrayExpr(_) | Expr::FnExpr(_) => Some(true),
        Expr::AssignExpr(assign)
            if assign
                .op()
                .map(|op| op == AssignOp::Assign)
                .unwrap_or_default() =>
        {
            simple_bool_coerce(assign.rhs()?)
        }
        Expr::SequenceExpr(seqexpr) => simple_bool_coerce(seqexpr.exprs().last()?),
        Expr::Name(name) => match name.ident_token()?.text().as_str() {
            "NaN" | "undefined" => Some(false),
            "Infinity" => Some(true),
            _ => None,
        },
        Expr::BinExpr(binexpr) => match binexpr.op()? {
            BinOp::LogicalAnd => {
                Some(simple_bool_coerce(binexpr.lhs()?)? && simple_bool_coerce(binexpr.rhs()?)?)
            }
            BinOp::LogicalOr => {
                Some(simple_bool_coerce(binexpr.lhs()?)? || simple_bool_coerce(binexpr.rhs()?)?)
            }
            _ => None,
        },
        Expr::CondExpr(condexpr) => {
            if simple_bool_coerce(condexpr.test()?)? {
                simple_bool_coerce(condexpr.cons()?)
            } else {
                simple_bool_coerce(condexpr.alt()?)
            }
        }
        _ => None,
    }
}

/// Get the combined range of multiple nodes.
pub fn multi_node_range(mut nodes: impl Iterator<Item = SyntaxNode>) -> TextRange {
    TextRange::new(
        nodes
            .next()
            .map(|x| x.trimmed_range().start())
            .unwrap_or(0.into()),
        nodes
            .last()
            .map(|x| x.trimmed_range().end())
            .unwrap_or(0.into()),
    )
}
