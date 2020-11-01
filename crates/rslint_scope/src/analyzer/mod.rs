mod expr;
mod stmt;

use crate::{datalog::DatalogBuilder, visit::Visit};
use rslint_core::rule_prelude::{
    ast::{NameRef, Pattern},
    AstNode, SyntaxNodeExt,
};
use types::{
    internment::{self, Intern},
    ExprId, Pattern as DatalogPattern,
};

pub(super) struct AnalyzerInner;

impl AnalyzerInner {
    fn visit_pattern(&self, pattern: Pattern) -> Intern<DatalogPattern> {
        match pattern {
            Pattern::SinglePattern(single) => internment::intern(&DatalogPattern {
                name: internment::intern(&single.text()),
            }),

            // FIXME: Implement the rest of the patterns
            _ => internment::intern(&DatalogPattern {
                name: internment::intern(&String::from("TODO")),
            }),
        }
    }
}

impl<'ddlog> Visit<'ddlog, NameRef> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, name: NameRef) -> Self::Output {
        scope.name_ref(name.to_string(), name.syntax().trimmed_range())
    }
}
