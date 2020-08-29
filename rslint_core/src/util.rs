//! General utilities to make linting easier. 

use crate::rule_prelude::*;

/// Expands an assignment to the returned value, e.g. `foo += 5` -> `foo + 5`, `foo = 6` -> `6`
/// 
/// # Panics 
/// Panics if the expression does not have an operator. 
pub fn get_assignment_expr_value(expr: ast::AssignExpr) -> String {
    assert!(expr.op().is_some());

    let tok = expr.syntax().first_lossy_token().unwrap();
    let op_str = tok.text();
    println!("{:#?}", op_str[..op_str.len() - 1].to_string());

    if op_str == "=" {
        expr.rhs().map(|e| e.syntax().trimmed_text().to_string()).unwrap_or_default()
    } else {
        format!(
            "{} {} {}",
            expr.lhs().map(|e| e.syntax().trimmed_text().to_string()).unwrap_or_default(),
            op_str[..op_str.len() - 1].to_string(),
            expr.rhs().map(|e| e.syntax().trimmed_text().to_string()).unwrap_or_default()
        )
    }
}
