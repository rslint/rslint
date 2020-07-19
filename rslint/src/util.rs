use rslint_parse::lexer::token::*;
use rslint_parse::parser::cst::expr::*;

/// Try to coerce a simple constant expression to a bool, this includes literals and logical expressions
/// with simple const left and right values, this is used for more helpful errors about the impact of const conditions
pub fn simple_bool_cast(expr: &Expr, source: &str) -> Option<bool> {
    match expr {
        Expr::Array(_) | Expr::Object(_) | Expr::Regex(_) | Expr::True(_) | Expr::Function(_) => {
            Some(true)
        }
        Expr::Identifier(ident) => match ident.span.content(source) {
            "NaN" | "undefined" => Some(false),
            "Infinity" => Some(true),
            _ => None,
        },
        Expr::Null(_) | Expr::False(_) => Some(false),
        Expr::Number(num) => Some(num.span.content(source) != "0"),
        Expr::String(string) => Some(string.span.size() == 2),
        Expr::Unary(unexpr) => match unexpr.op {
            TokenType::Void => Some(false),
            TokenType::LogicalNot => Some(!simple_bool_cast(&unexpr.object, source)?),
            TokenType::Typeof => Some(true),
            _ => None,
        },
        Expr::Binary(binexpr) => match binexpr.op {
            TokenType::BinOp(BinToken::LogicalAnd) => Some(
                simple_bool_cast(&binexpr.left, source)?
                    && simple_bool_cast(&binexpr.right, source)?,
            ),
            TokenType::BinOp(BinToken::LogicalOr) => Some(
                simple_bool_cast(&binexpr.left, source)?
                    || simple_bool_cast(&binexpr.right, source)?,
            ),
            _ => None,
        },
        Expr::Conditional(condexpr) => {
            if simple_bool_cast(&condexpr.condition, source)? {
                Some(simple_bool_cast(&condexpr.if_true, source)?)
            } else {
                Some(simple_bool_cast(&condexpr.if_false, source)?)
            }
        }
        _ => None,
    }
}

/// Returns true if the identifier is a builtin constant variable such as NaN, Infinity, and undefined
pub fn is_const_ident(ident: &LiteralExpr, source: &str) -> bool {
    const IDENTS: [&'static str; 3] = ["NaN", "Infinity", "undefined"];
    IDENTS.contains(&ident.span.content(source))
}
