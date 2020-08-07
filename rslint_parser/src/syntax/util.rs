//! General utility functions for parsing and error checking.

use crate::{
    ast::{Expr, GroupingExpr},
    SyntaxKind::*,
    *,
};

/// Check if assignment to an expression is invalid and report an error if so.
///
/// For example: `++true` is invalid.
pub fn check_assign_target(p: &mut Parser, target: &Expr) {
    let err = p
        .err_builder(&format!(
            "Invalid assignment to `{:?}`",
            target.syntax().text()
        ))
        .primary(
            target.syntax().text_range(),
            "This expression cannot be assigned to",
        );

    match target.syntax().kind() {
        NAME | ARRAY_EXPR | DOT_EXPR => {}
        GROUPING_EXPR => {
            let inner = GroupingExpr::cast(target.syntax().to_owned())
                .unwrap()
                .inner();
            if let Some(inner) = inner {
                check_assign_target(p, &inner)
            }
        }
        _ => p.error(err),
    }
}

/// Get the precedence of the current operator
pub fn current_precedence(p: &Parser) -> Option<u8> {
    Some(match p.cur() {
        T![||] => 1,
        T![&&] => 2,
        T![|] => 3,
        T![^] => 4,
        T![&] => 5,
        T![==] | T![!=] | T![===] | T![!==] => 6,
        T![>] | T![>=] | T![<] | T![<=] => 7,
        T![<<] | T![>>] | T![>>>] => 8,
        T![+] | T![-] => 9,
        T![*] | T![/] => 10,
        T![%] => 11,
        _ => return None
    })
}
