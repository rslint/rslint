mod expr;
mod stmt;
mod visit;

pub(crate) use visit::Visit;

use crate::DatalogBuilder;
use rslint_parser::{
    ast::{AstChildren, Name, Pattern},
    AstNode,
};
use types::{internment::Intern, Pattern as DatalogPattern};

pub(super) struct AnalyzerInner;

impl<'ddlog> Visit<'ddlog, Pattern> for AnalyzerInner {
    type Output = Intern<DatalogPattern>;

    fn visit(&self, _scope: &dyn DatalogBuilder<'ddlog>, pattern: Pattern) -> Self::Output {
        match pattern {
            Pattern::SinglePattern(single) => Intern::new(DatalogPattern {
                name: Intern::new(single.text()),
            }),

            // FIXME: Implement the rest of the patterns
            _ => Intern::new(DatalogPattern {
                name: Intern::new(String::from("TODO")),
            }),
        }
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<Pattern>> for AnalyzerInner {
    type Output = Vec<Intern<DatalogPattern>>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        patterns: AstChildren<Pattern>,
    ) -> Self::Output {
        patterns.map(|pattern| self.visit(scope, pattern)).collect()
    }
}

impl<'ddlog> Visit<'ddlog, Name> for AnalyzerInner {
    type Output = Intern<String>;

    fn visit(&self, _scope: &dyn DatalogBuilder<'ddlog>, name: Name) -> Self::Output {
        Intern::new(name.to_string())
    }
}
