//! General utilities to make linting easier.

use crate::rule_prelude::*;
use ast::*;
use rslint_parser::TextRange;
use std::borrow::Borrow;
use std::cmp;
use SyntaxKind::*;

/// Expands an assignment to the returned value, e.g. `foo += 5` -> `foo + 5`, `foo = 6` -> `6`
///
/// # Panics
/// Panics if the expression does not have an operator.
pub fn get_assignment_expr_value(expr: AssignExpr) -> std::string::String {
    assert!(expr.op().is_some());

    let tok = expr.syntax().first_lossy_token().unwrap();
    let op_str = tok.text();

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
                LiteralKind::Number(num) => num != 0.0,
                LiteralKind::BigInt(bigint) => bigint != 0.into(),
                LiteralKind::String => !lit.inner_string_text().unwrap().is_empty(),
                LiteralKind::Regex => true,
            };
            Some(coerced)
        }
        Expr::Template(tpl) => {
            if tpl.quasis().any(|t| !t.text().is_empty()) {
                Some(true)
            } else if tpl.syntax().text().len() == 2.into() {
                Some(false)
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
        Expr::NameRef(name) => match name.ident_token()?.text().as_str() {
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
            .unwrap_or_else(|| 0.into()),
        nodes
            .last()
            .map(|x| x.trimmed_range().end())
            .unwrap_or_else(|| 0.into()),
    )
}

/// Whether this is a predefined constant identifier such as NaN and undefined
pub fn is_const_ident(ident: SyntaxToken) -> bool {
    ["NaN", "Infinity", "undefined"].contains(&&*ident.text().to_string())
}

/// Check whether an expr is constant and is always falsey or truthy
pub fn is_const(expr: Expr, boolean_pos: bool, notes: &mut Vec<&str>) -> bool {
    match expr {
        Expr::Literal(_) | Expr::ObjectExpr(_) | Expr::FnExpr(_) | Expr::ArrowExpr(_) => true,
        Expr::NameRef(name) => util::is_const_ident(name.ident_token().unwrap()),
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
            let not_const = array.syntax().parent().map_or(false, |p| {
                p.kind() == BIN_EXPR && p.to::<BinExpr>().op() == Some(BinOp::Plus)
            });

            if not_const {
                array.elements().all(|elem| {
                    if let ExprOrSpread::Expr(expr) = elem {
                        is_const(expr, boolean_pos, notes)
                    } else {
                        false
                    }
                })
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
            if binexpr.conditional() {
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
            } else {
                let lhs_const = binexpr
                    .lhs()
                    .map_or(false, |expr| is_const(expr, false, notes));
                let rhs_const = binexpr
                    .rhs()
                    .map_or(false, |expr| is_const(expr, false, notes));

                lhs_const && rhs_const && binexpr.op() != Some(op!(in))
            }
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
                if let Some(alt) = stmt.alt() {
                    diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always truthy...",
                        )
                        .secondary(
                            alt.syntax().trimmed_range(),
                            "...which makes this unreachable",
                        );
                } else if let Some(cons) = stmt.cons() {
                    diagnostic = diagnostic
                        .primary(
                            stmt.condition().unwrap().syntax().trimmed_range(),
                            "this condition is always truthy...",
                        )
                        .secondary(
                            cons.syntax().trimmed_range(),
                            "...which makes this always run",
                        );
                } else {
                    diagnostic = diagnostic.primary(
                        stmt.condition().unwrap().syntax().trimmed_range(),
                        "this condition consistently yields one result",
                    )
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

/// Get the range represented by a list of tokens.
///
/// # Panics
///
/// Panics if the items is an empty iterator.
pub fn token_list_range<I>(items: I) -> TextRange
where
    I: IntoIterator,
    I::Item: Borrow<SyntaxToken>,
{
    let collection = items
        .into_iter()
        .map(|x| x.borrow().clone())
        .collect::<Vec<_>>();
    let start = collection
        .first()
        .expect("Empty token list")
        .text_range()
        .start();
    let end = collection
        .last()
        .expect("Empty token list")
        .text_range()
        .end();
    TextRange::new(start, end)
}

/// Compare two lists of tokens by comparing their underlying string value.
// Note: two generics is so right is not constrained to be the same type as left
pub fn string_token_eq<L, R>(left: L, right: R) -> bool
where
    L: IntoIterator,
    R: IntoIterator,
    L::Item: Borrow<SyntaxToken>,
    R::Item: Borrow<SyntaxToken>,
{
    let left_vec: Vec<L::Item> = left.into_iter().collect();
    let right_vec: Vec<R::Item> = right.into_iter().collect();

    if left_vec.len() != right_vec.len() {
        return false;
    }
    left_vec
        .into_iter()
        .zip(right_vec.into_iter())
        .all(|(l, r)| l.borrow().to_string() == r.borrow().to_string())
}

/// Find the Levenshtein distance between two strings
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    if a.is_empty() {
        return b.chars().count();
    } else if b.is_empty() {
        return a.chars().count();
    }

    let mut dcol: Vec<_> = (0..=b.len()).collect();
    let mut t_last = 0;

    for (i, sc) in a.chars().enumerate() {
        let mut current = i;
        dcol[0] = current + 1;

        for (j, tc) in b.chars().enumerate() {
            let next = dcol[j + 1];
            if sc == tc {
                dcol[j + 1] = current;
            } else {
                dcol[j + 1] = cmp::min(current, next);
                dcol[j + 1] = cmp::min(dcol[j + 1], dcol[j]) + 1;
            }
            current = next;
            t_last = j;
        }
    }
    dcol[t_last + 1]
}

/// Find the best match for a string in an iterator of strings based on levenshtein distance.
///
/// This considers a case insensitive match and the levenshtein distance with a cutoff.
/// This is taken from [rustc's implementation](https://github.com/rust-lang/rust/blob/master/compiler/rustc_ast/src/util/lev_distance.rs)
pub fn find_best_match_for_name<'a>(
    iter_names: impl Iterator<Item = &'a str>,
    lookup: &str,
    dist: impl Into<Option<usize>>,
) -> Option<&'a str> {
    let max_dist = dist
        .into()
        .map_or_else(|| cmp::max(lookup.len(), 3) / 3, |d| d);
    let name_vec = iter_names.collect::<Vec<_>>();

    let (case_insensitive_match, levenshtein_match) = name_vec
        .iter()
        .filter_map(|&name| {
            let dist = levenshtein_distance(lookup, name);
            if dist <= max_dist {
                Some((name, dist))
            } else {
                None
            }
        })
        // Here we are collecting the next structure:
        // (case_insensitive_match, (levenshtein_match, levenshtein_distance))
        .fold((None, None), |result, (candidate, dist)| {
            (
                if candidate.to_uppercase() == lookup.to_uppercase() {
                    Some(candidate)
                } else {
                    result.0
                },
                match result.1 {
                    None => Some((candidate, dist)),
                    Some((c, d)) => Some(if dist < d { (candidate, dist) } else { (c, d) }),
                },
            )
        });

    // Priority of matches:
    // 1. Exact case insensitive match
    // 2. Levenshtein distance match
    // 3. Sorted word match
    if let Some(candidate) = case_insensitive_match {
        Some(candidate)
    } else if levenshtein_match.is_some() {
        levenshtein_match.map(|x| x.0)
    } else {
        find_match_by_sorted_words(name_vec, lookup)
    }
}

fn find_match_by_sorted_words<'a>(iter_names: Vec<&'a str>, lookup: &str) -> Option<&'a str> {
    iter_names.iter().fold(None, |result, candidate| {
        if sort_by_words(&candidate) == sort_by_words(lookup) {
            Some(candidate)
        } else {
            result
        }
    })
}

fn sort_by_words(name: &str) -> std::string::String {
    let mut split_words: Vec<&str> = name.split('_').collect();
    split_words.sort_unstable();
    split_words.join("_")
}

/// Check if this is either a Call expression with the callee of `name`,
/// or if this is a New expression with a callee of `name`.
/// e.g. `Boolean()` or `new Boolean()`
pub fn constructor_or_call_with_callee(
    node: impl Borrow<SyntaxNode>,
    name: impl AsRef<str>,
) -> bool {
    let node = node.borrow();
    match node.kind() {
        NEW_EXPR | CALL_EXPR => node.children().any(|child| child.text() == name.as_ref()),
        _ => false,
    }
}

/// Get the first enclosing function of a node, this does not consider if the node itself is a function.
pub fn outer_function(node: impl Borrow<SyntaxNode>) -> Option<SyntaxNode> {
    node.borrow()
        .ancestors()
        .skip(1)
        .find(|ancestor| matches!(ancestor.kind(), ARROW_EXPR | FN_DECL | FN_EXPR))
}
