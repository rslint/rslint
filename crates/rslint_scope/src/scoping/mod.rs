use crate::datalog::Datalog;
use differential_datalog::ddval::DDValConvert;
use rslint_parser::ast::{AstNode, Expr};
use rslint_scoping_ddlog::Indexes;
use types::{
    ast::{ExprKind, Span},
    ddlog_std::tuple2,
    inputs::{Expression, InputScope},
    internment::Intern,
    ChildScope,
};

pub use types::ast::{ExprId, Scope};

#[derive(Debug, Clone)]
pub struct ProgramInfo {
    datalog: Datalog,
}

impl ProgramInfo {
    pub fn new(datalog: Datalog) -> Self {
        Self { datalog }
    }

    pub fn expr(&self, expr: &Expr) -> Option<ExprInfo> {
        // TODO: Log errors if they occur
        let query = self.datalog.query(
            Indexes::inputs_ExpressionBySpan,
            Some(Span::from(expr.range()).into_ddvalue()),
        );

        query
            .ok()
            // TODO: Log error if there's more than one value
            .and_then(|query| query.into_iter().next())
            .map(|expr| unsafe { Expression::from_ddvalue(expr) })
            .map(Into::into)
    }

    pub fn scope(&self, scope: Scope) -> ScopeInfo<'_> {
        ScopeInfo {
            handle: self,
            scope,
        }
    }
}

pub struct ScopeInfo<'a> {
    handle: &'a ProgramInfo,
    scope: Scope,
}

impl<'a> ScopeInfo<'a> {
    pub fn parent(&self) -> Option<Scope> {
        // TODO: Log errors if they occur
        let query = self.handle.datalog.query(
            Indexes::inputs_InputScopeByChild,
            Some(self.scope.into_ddvalue()),
        );

        query
            .ok()
            // TODO: Log error if there's more than one value
            .and_then(|query| query.into_iter().next())
            .map(|scope| unsafe { InputScope::from_ddvalue(scope).parent })
    }

    pub fn children(&self) -> Option<Vec<Scope>> {
        // TODO: Log errors if they occur
        let query = self
            .handle
            .datalog
            .query(Indexes::ChildScopeByParent, Some(self.scope.into_ddvalue()));

        query.ok().map(|query| {
            query
                .into_iter()
                .map(|scope| unsafe { ChildScope::from_ddvalue(scope).child })
                .collect()
        })
    }

    pub fn has_in_scope(&self, name: &str) -> bool {
        // TODO: Log errors if they occur
        let query = self.handle.datalog.query(
            Indexes::Index_VariableInScope,
            Some(tuple2(self.scope, Intern::new(name.to_owned())).into_ddvalue()),
        );

        query.map_or(false, |query| !query.is_empty())
    }
}

pub struct ExprInfo {
    pub id: ExprId,
    _kind: ExprKind,
    pub scope: Scope,
}

impl From<Expression> for ExprInfo {
    fn from(expr: Expression) -> Self {
        Self {
            id: expr.id,
            _kind: expr.kind,
            scope: expr.scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{datalog::DatalogBuilder, AnalyzerInner, ScopeAnalyzer, Visit};
    use rslint_parser::{parse_expr, TextRange};
    use types::ast::{LitKind, Pattern, Spanned};

    #[test]
    fn get_expr() {
        let expr = parse_expr("1", 0).tree();
        let analyzer = ScopeAnalyzer::new().unwrap();

        let (expr_id, parent_scope) = analyzer
            .transaction(|trans| {
                let analyzer = AnalyzerInner;
                let scope = trans.scope();

                let id = analyzer.visit(&scope, expr.clone());

                Ok((id, scope.scope_id()))
            })
            .unwrap();

        let info = ProgramInfo::new(analyzer.datalog);
        let query_expr = info.expr(&expr).unwrap();

        assert_eq!(query_expr.id, expr_id);
        assert_eq!(query_expr.scope, parent_scope);
        assert_eq!(
            query_expr._kind,
            ExprKind::ExprLit {
                kind: LitKind::LitNumber,
            },
        );
    }

    #[test]
    fn scope_relations() {
        let datalog = Datalog::new().unwrap();

        let mut ids = Vec::new();
        let top_id = datalog
            .transaction(|trans| {
                let top = trans.scope();

                for num_children in [0, 1, 2, 3, 10, 50].iter().copied() {
                    let scope = top.scope();
                    let children: Vec<_> = (0..num_children)
                        .map(|_| scope.scope().scope_id())
                        .collect();

                    ids.push((scope.scope_id(), num_children, children));
                }
                let _ = trans.scope().scope().scope();

                Ok(top.scope_id())
            })
            .unwrap();

        let info = ProgramInfo::new(datalog);
        for (id, num_children, children) in ids {
            let scope = info.scope(id);
            let query_children = scope.children().unwrap();
            let parent = scope.parent().unwrap();

            assert_eq!(parent, top_id);
            assert_eq!(num_children as usize, query_children.len());
            assert!(children.iter().all(|child| query_children.contains(child)));
        }
    }

    #[test]
    fn var_in_scope() {
        let datalog = Datalog::new().unwrap();

        let (empty, filled) = datalog
            .transaction(|trans| {
                let empty = trans.scope();

                // let foo;
                let (_stmt, filled) = empty.decl_let(
                    Some(Intern::new(Pattern::SinglePattern {
                        name: Some(Spanned::new(Intern::new("foo".to_owned()), 4..7u32)).into(),
                    })),
                    None,
                    TextRange::new(0.into(), 8.into()),
                );

                Ok((empty.scope_id(), filled.scope_id()))
            })
            .unwrap();

        let info = ProgramInfo::new(datalog);
        let empty = info.scope(empty);
        assert!(!empty.has_in_scope("foo"));

        let filled = info.scope(filled);
        assert!(filled.has_in_scope("foo"));
    }
}
