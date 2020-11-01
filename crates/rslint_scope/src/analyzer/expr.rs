use crate::{datalog::DatalogBuilder, visit::Visit, AnalyzerInner};
use rslint_core::rule_prelude::{
    ast::{Expr, Literal, LiteralKind},
    AstNode, SyntaxNodeExt,
};
use types::ExprId;

impl<'ddlog> Visit<'ddlog, Expr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, expr: Expr) -> Self::Output {
        match expr {
            Expr::Literal(literal) => self.visit(scope, literal),
            Expr::NameRef(name) => self.visit(scope, name),

            // FIXME: This is here so things can function before everything is 100%
            //        translatable into datalog, mostly for my sanity
            _ => 0,
        }
    }
}

impl<'ddlog> Visit<'ddlog, Literal> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, literal: Literal) -> Self::Output {
        let span = literal.syntax().trimmed_range();

        match literal.kind() {
            LiteralKind::Number(number) => scope.number(number, span),
            LiteralKind::BigInt(bigint) => scope.bigint(bigint, span),
            LiteralKind::String => {
                scope.string(literal.inner_string_text().unwrap().to_string(), span)
            }
            LiteralKind::Null => scope.null(span),
            LiteralKind::Bool(boolean) => scope.boolean(boolean, span),
            LiteralKind::Regex => scope.regex(span),
        }
    }
}
