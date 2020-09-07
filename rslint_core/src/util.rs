//! General utilities to make linting easier.

use crate::rule_prelude::*;
use ast::*;
use rslint_parser::TextRange;
use SyntaxKind::*;

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
        Expr::Template(tpl) => {
            if tpl.quasis().any(|t| !t.text().is_empty()) {
                return Some(true);
            } else if tpl.syntax().text().len() == 2.into() {
                return Some(false);
            } else {
                None
            }
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

/// Whether this is a predefined constant identifier such as NaN and undefined
pub fn is_const_ident(ident: Name) -> bool {
    ["NaN", "Infinity", "undefined"].contains(&&*ident.syntax().text().to_string())
}

/// Check whether an expr is constant and is always falsey or truthy
pub fn is_const(expr: Expr, boolean_pos: bool, notes: &mut Vec<&str>) -> bool {
    match expr {
        Expr::Literal(_) | Expr::ObjectExpr(_) | Expr::FnExpr(_) | Expr::ArrowExpr(_) => true,
        Expr::Name(name) => util::is_const_ident(name),
        Expr::Template(tpl) => {
            // If any of the template's string elements are not empty, then the template is always truthy like a non empty string
            (boolean_pos && tpl.quasis().any(|t| !t.text().is_empty()))
                || tpl
                    .elements()
                    .all(|e| e.expr().map_or(false, |e| is_const(e, boolean_pos, notes)))
        }
        Expr::GroupingExpr(group) => group
            .inner()
            .map_or(true, |e| is_const(e, boolean_pos, notes)),
        Expr::ArrayExpr(array) => {
            if array.syntax().parent().map_or(false, |p| {
                p.kind() == BIN_EXPR && p.to::<BinExpr>().op() == Some(BinOp::Plus)
            }) {
                return array.elements().all(|elem| {
                    if let ExprOrSpread::Expr(expr) = elem {
                        is_const(expr, boolean_pos, notes)
                    } else {
                        false
                    }
                });
            } else {
                true
            }
        }
        Expr::UnaryExpr(unexpr) => {
            if unexpr.op() == Some(UnaryOp::Void) {
                notes.push("note: void always returns `undefined`, which makes the expression always falsey");
                true
            } else {
                (unexpr.op() == Some(UnaryOp::Typeof) && boolean_pos)
                    || unexpr
                        .expr()
                        .map_or(true, |e| is_const(e, boolean_pos, notes))
            }
        }
        // TODO: Handle more cases which require knowing if the right value is const
        Expr::BinExpr(binexpr) => {
            let lhs_const = binexpr
                .lhs()
                .map_or(false, |expr| is_const(expr, boolean_pos, notes));
            let rhs_const = binexpr
                .rhs()
                .map_or(false, |expr| is_const(expr, boolean_pos, notes));

            let lhs_short_circuits = binexpr.lhs().map_or(false, |expr| {
                binexpr.op().map_or(false, |op| short_circuits(expr, op))
            });
            let rhs_short_circuits = binexpr.rhs().map_or(false, |expr| {
                binexpr.op().map_or(false, |op| short_circuits(expr, op))
            });

            (lhs_const && rhs_const) || lhs_short_circuits || rhs_short_circuits
        }
        Expr::AssignExpr(assignexpr) => {
            if assignexpr.op() == Some(AssignOp::Assign) && assignexpr.rhs().is_some() {
                is_const(assignexpr.rhs().unwrap(), boolean_pos, notes)
            } else {
                false
            }
        }
        Expr::SequenceExpr(seqexpr) => {
            is_const(seqexpr.exprs().last().unwrap(), boolean_pos, notes)
        }
        _ => false,
    }
}

fn short_circuits(expr: Expr, op: BinOp) -> bool {
    match expr {
        Expr::Literal(lit) => {
            if let LiteralKind::Bool(b) = lit.kind() {
                match op {
                    BinOp::LogicalOr => b,
                    BinOp::LogicalAnd => !b,
                    _ => false,
                }
            } else {
                false
            }
        }
        Expr::UnaryExpr(unexpr) => {
            op == BinOp::LogicalAnd && unexpr.op().map_or(false, |op| op == UnaryOp::Void)
        }
        Expr::BinExpr(binexpr) => {
            if binexpr.conditional() {
                binexpr
                    .lhs()
                    .map_or(false, |expr| short_circuits(expr, binexpr.op().unwrap()))
                    || binexpr
                        .rhs()
                        .map_or(false, |expr| short_circuits(expr, binexpr.op().unwrap()))
            } else {
                false
            }
        }
        _ => false,
    }
}

/// Issue more context around the effects of a constant condition on a node.
///
/// For example, if the statement is an if statement and the condition value is false,
/// that means the if statement cons is unreachable and the else statement always triggers.
/// This function adds labels which illustrate that.
/// You should get the `condition_value` from [`simple_bool_coerce`].
/// This method does nothing if the node is not a cond expr, if, while, do while, or for stmt
pub fn simple_const_condition_context(
    parent: SyntaxNode,
    condition_value: bool,
    mut diagnostic: DiagnosticBuilder,
) -> DiagnosticBuilder {
    // TODO: we can likely clean this up a lot
    match parent.kind() {
        COND_EXPR => {
            let condexpr = parent.to::<CondExpr>();
            if condition_value && condexpr.cons().is_some() {
                diagnostic = diagnostic
                    .primary(
                        condexpr.test().unwrap().syntax().trimmed_range(),
                        "this expression is always truthy...",
                    )
                    .secondary(
                        condexpr.cons().unwrap().syntax().trimmed_range(),
                        "...which means this expression is always returned",
                    );
            } else if !condition_value && condexpr.alt().is_some() {
                diagnostic = diagnostic
                    .primary(
                        condexpr.test().unwrap().syntax().trimmed_range(),
                        "this expression is always falsey...",
                    )
                    .secondary(
                        condexpr.alt().unwrap().syntax().trimmed_range(),
                        "...which means this expression is always returned",
                    );
            }
        }
        IF_STMT => {
            let stmt = parent.to::<IfStmt>();
            if condition_value {
                if let Some(cons) = stmt.cons() {
                    diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always truthy...",
                        )
                        .secondary(
                            cons.syntax().trimmed_range(),
                            "...which makes this unreachable",
                        );
                } else {
                    if let Some(alt) = stmt.alt() {
                        diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always truthy...",
                        )
                        .secondary(
                            alt.syntax().trimmed_range(),
                            "...which makes this always run",
                        );
                    } else {
                        diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition consistently yields one result",
                        )
                    }
                }
            } else if !condition_value && stmt.alt().is_some() {
                diagnostic = diagnostic
                    .primary(
                        stmt.condition().unwrap().syntax().trimmed_range(),
                        "this condition is always falsey...",
                    )
                    .secondary(
                        stmt.alt().unwrap().syntax().trimmed_range(),
                        "...which makes this unreachable",
                    );
            }
        }
        WHILE_STMT => {
            let stmt = parent.to::<WhileStmt>();
            if let Some(cons) = stmt.cons().map(|stmt| stmt.syntax().clone()) {
                if condition_value {
                    diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always truthy...",
                        )
                        .secondary(cons.trimmed_range(), "...which makes this infinitely loop");
                } else {
                    diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always falsey...",
                        )
                        .secondary(cons.trimmed_range(), "...which makes this loop never run");
                }
            }
        }
        DO_WHILE_STMT => {
            let stmt = parent.to::<DoWhileStmt>();
            if let Some(cons) = stmt.cons().map(|stmt| stmt.syntax().clone()) {
                if condition_value {
                    diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always truthy...",
                        )
                        .secondary(cons.trimmed_range(), "...which makes this infinitely loop");
                } else {
                    diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always falsey...",
                        )
                        .secondary(
                            cons.trimmed_range(),
                            "...which makes this loop only run once",
                        );
                }
            }
        }
        FOR_STMT => {
            let stmt = parent.to::<ForStmt>();
            if let Some(cons) = stmt.cons().map(|stmt| stmt.syntax().clone()) {
                if condition_value {
                    diagnostic = diagnostic
                        .primary(
                            stmt.test().unwrap().syntax().trimmed_range(),
                            "this test condition is always truthy...",
                        )
                        .secondary(cons.trimmed_range(), "...which makes this infinitely loop");
                } else {
                    diagnostic = diagnostic
                        .primary(
                            stmt.test().unwrap().syntax().trimmed_range(),
                            "this test condition is always falsey...",
                        )
                        .secondary(cons.trimmed_range(), "...which makes this loop never run");
                }
            }
        }
        _ => {}
    }
    diagnostic
}
