mod expr;
mod module;
mod stmt;
mod visit;

pub(crate) use visit::Visit;

use crate::DatalogBuilder;
use rslint_parser::{
    ast::{AstChildren, Name, ObjectPatternProp, Pattern},
    AstNode,
};
use types::{
    ast::{
        IPattern, Name as DatalogName, ObjectPatternProp as DatalogObjectPatternProp,
        Pattern as DatalogPattern, Spanned,
    },
    internment::Intern,
};

pub(super) struct AnalyzerInner;

impl<'ddlog> Visit<'ddlog, Name> for AnalyzerInner {
    type Output = Spanned<DatalogName>;

    fn visit(&self, _scope: &dyn DatalogBuilder<'ddlog>, name: Name) -> Self::Output {
        Spanned::new(Intern::new(name.to_string()), name.range())
    }
}

impl<'ddlog> Visit<'ddlog, Pattern> for AnalyzerInner {
    type Output = IPattern;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, pattern: Pattern) -> Self::Output {
        Intern::new(match pattern {
            Pattern::SinglePattern(single) => DatalogPattern::SinglePattern {
                name: self.visit(scope, single.name()).into(),
            },
            Pattern::RestPattern(rest) => DatalogPattern::RestPattern {
                rest: self.visit(scope, rest.pat()).into(),
            },
            Pattern::AssignPattern(assign) => DatalogPattern::AssignPattern {
                key: self.visit(scope, assign.key()).into(),
                value: self.visit(scope, assign.value()).into(),
            },
            Pattern::ObjectPattern(object) => DatalogPattern::ObjectPattern {
                props: self.visit(scope, object.elements()).into(),
            },
            Pattern::ArrayPattern(array) => DatalogPattern::ArrayPattern {
                elems: self.visit(scope, array.elements()).into(),
            },
        })
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

impl<'ddlog> Visit<'ddlog, ObjectPatternProp> for AnalyzerInner {
    type Output = Intern<DatalogObjectPatternProp>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, prop: ObjectPatternProp) -> Self::Output {
        Intern::new(match prop {
            ObjectPatternProp::AssignPattern(assign) => {
                DatalogObjectPatternProp::ObjAssignPattern {
                    assign_key: self.visit(scope, assign.key()).into(),
                    assign_value: self.visit(scope, assign.value()).into(),
                }
            }
            ObjectPatternProp::KeyValuePattern(kv) => {
                DatalogObjectPatternProp::ObjKeyValuePattern {
                    key: self.visit(scope, kv.key()).into(),
                    value: self.visit(scope, kv.value()).into(),
                }
            }
            ObjectPatternProp::RestPattern(rest) => DatalogObjectPatternProp::ObjRestPattern {
                rest: self.visit(scope, rest.pat()).into(),
            },
            ObjectPatternProp::SinglePattern(single) => {
                DatalogObjectPatternProp::ObjSinglePattern {
                    name: self.visit(scope, single.name()).into(),
                }
            }
        })
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<ObjectPatternProp>> for AnalyzerInner {
    type Output = Vec<Intern<DatalogObjectPatternProp>>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        properties: AstChildren<ObjectPatternProp>,
    ) -> Self::Output {
        properties
            .map(|property| self.visit(scope, property))
            .collect()
    }
}
