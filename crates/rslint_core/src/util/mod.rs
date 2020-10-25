//! General utilities to make linting easier.

mod const_exprs;
mod style;

pub use const_exprs::*;
pub use style::*;

use crate::rule_prelude::*;
use ast::*;
use rslint_parser::TextRange;
use std::borrow::Borrow;
use std::cmp;
use std::cmp::{Eq, Ord, Reverse};
use std::collections::{BinaryHeap, HashMap};
use std::hash::Hash;
use SyntaxKind::*;

// rustfmt panics on this function for me
#[rustfmt::skip]
pub fn most_frequent<T>(items: Vec<T>) -> T
where
    T: Hash + Eq + Ord + Clone,
{
    let mut map = HashMap::new();
    for x in items {
        *map.entry(x).or_insert(0) += 1;
    }

    let mut heap = BinaryHeap::with_capacity(2);
    for (x, count) in map.into_iter() {
        heap.push(Reverse((count, x)));
        if heap.len() > 1 {
            heap.pop();
        }
    }
    // TODO: remove this clone
    heap.into_sorted_vec()[0].0.1.to_owned()
}

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
