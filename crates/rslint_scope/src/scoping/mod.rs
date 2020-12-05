use crate::datalog::Datalog;
use differential_datalog::ddval::DDValConvert;
use rslint_parser::ast::{AstNode, Expr, Stmt};
use rslint_scoping_ddlog::Indexes;
use std::sync::Arc;
use types::{
    ast::Span,
    ddlog_std::{tuple2, tuple3},
    inputs::{Expression, InputScope, Statement},
    internment::Intern,
    name_in_scope::NameInScope,
    scopes::ScopeFamily,
};

pub use types::ast::{ExprId, FileId, ScopeId, StmtId};

#[derive(Debug, Clone)]
pub struct ProgramInfo {
    datalog: Arc<Datalog>,
}

impl ProgramInfo {
    pub fn new(datalog: Arc<Datalog>) -> Self {
        Self { datalog }
    }

    pub fn stmt(&self, stmt: &Stmt, file: FileId) -> Option<StmtInfo> {
        // TODO: Log errors if they occur
        let query = self.datalog.query(
            Indexes::inputs_StatementBySpan,
            Some(tuple2(Span::from(stmt.range()), file).into_ddvalue()),
        );

        query
            .ok()
            // TODO: Log error if there's more than one value
            .and_then(|query| query.into_iter().next())
            .map(Statement::from_ddvalue)
            .map(|stmt| StmtInfo {
                id: stmt.id,
                file: stmt.file,
                scope: stmt.scope,
            })
    }

    pub fn expr(&self, expr: &Expr, file: FileId) -> Option<ExprInfo> {
        // TODO: Log errors if they occur
        let query = self.datalog.query(
            Indexes::inputs_ExpressionBySpan,
            Some(tuple2(Span::from(expr.range()), file).into_ddvalue()),
        );

        query
            .ok()
            // TODO: Log error if there's more than one value
            .and_then(|query| query.into_iter().next())
            .map(Expression::from_ddvalue)
            .map(|expr| ExprInfo {
                id: expr.id,
                file: expr.file,
                scope: expr.scope,
            })
    }

    pub fn scope(&self, scope: ScopeId, file: FileId) -> ScopeInfo<'_> {
        ScopeInfo {
            handle: self,
            scope,
            file,
        }
    }
}

pub struct ScopeInfo<'a> {
    handle: &'a ProgramInfo,
    scope: ScopeId,
    file: FileId,
}

impl<'a> ScopeInfo<'a> {
    pub fn parent(&self) -> Option<ScopeId> {
        // TODO: Log errors if they occur
        let query = self.handle.datalog.query(
            Indexes::inputs_InputScopeByChild,
            Some(tuple2(self.scope, self.file).into_ddvalue()),
        );

        query
            .ok()
            // TODO: Log error if there's more than one value
            .and_then(|query| query.into_iter().next())
            .map(|scope| InputScope::from_ddvalue(scope).parent)
    }

    pub fn children(&self) -> Option<Vec<ScopeId>> {
        // TODO: Log errors if they occur
        let query = self.handle.datalog.query(
            Indexes::scopes_ScopeFamilyByParent,
            Some(tuple2(self.scope, self.file).into_ddvalue()),
        );

        query.ok().map(|query| {
            query
                .into_iter()
                .map(|scope| ScopeFamily::from_ddvalue(scope).child)
                .collect()
        })
    }

    pub fn contains(&self, name: &str) -> bool {
        // TODO: Log errors if they occur
        let query = self.handle.datalog.query(
            Indexes::name_in_scope_Index_VariableInScope,
            Some(tuple3(self.file, self.scope, Intern::new(name.to_owned())).into_ddvalue()),
        );

        query.map_or(false, |query| !query.is_empty())
    }

    pub fn variable(&self, name: &str) -> Option<NameInScope> {
        // TODO: Log errors if they occur
        let query = self.handle.datalog.query(
            Indexes::name_in_scope_Index_VariableInScope,
            Some(tuple3(self.file, self.scope, Intern::new(name.to_owned())).into_ddvalue()),
        );

        query
            .ok()
            .and_then(|vars| vars.into_iter().next())
            .map(NameInScope::from_ddvalue)
    }
}

pub struct ExprInfo {
    id: ExprId,
    file: FileId,
    scope: ScopeId,
}

impl ExprInfo {
    pub const fn id(&self) -> ExprId {
        self.id
    }

    pub const fn file(&self) -> FileId {
        self.file
    }

    pub const fn scope(&self) -> ScopeId {
        self.scope
    }
}

pub struct StmtInfo {
    id: StmtId,
    file: FileId,
    scope: ScopeId,
}

impl StmtInfo {
    pub const fn id(&self) -> StmtId {
        self.id
    }

    pub const fn file(&self) -> FileId {
        self.file
    }

    pub const fn scope(&self) -> ScopeId {
        self.scope
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{datalog::DatalogBuilder, AnalyzerInner, ScopeAnalyzer, Visit};
    use rslint_parser::{ast::Stmt, parse_expr, parse_text, SyntaxNodeExt};
    use types::{
        ast::{FileKind, JSFlavor},
        config::Config,
    };

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
        let query_expr = info.expr(&expr, FileId::new(0)).unwrap();

        assert_eq!(query_expr.id, expr_id);
        assert_eq!(query_expr.scope, parent_scope);
    }

    #[test]
    fn scope_relations() {
        let datalog = Arc::new(Datalog::new().unwrap());

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
            let scope = info.scope(id, FileId::new(0));
            let query_children = scope.children().unwrap();
            let parent = scope.parent().unwrap();

            assert_eq!(parent, top_id);
            assert_eq!(num_children as usize, query_children.len());
            assert!(children.iter().all(|child| query_children.contains(child)));
        }
    }

    #[test]
    fn var_in_scope() {
        let datalog = Arc::new(Datalog::new().unwrap());

        let (empty, filled) = datalog
            .transaction(|trans| {
                let file = trans.file(
                    FileId::new(0),
                    FileKind::JavaScript {
                        flavor: JSFlavor::Vanilla,
                    },
                    Config::default(),
                );

                // let foo;
                let (_stmt, filled) = AnalyzerInner.visit(
                    &file,
                    parse_text("let foo;", 0)
                        .syntax()
                        .first_child()
                        .unwrap()
                        .to::<Stmt>(),
                );

                Ok((file.scope_id(), filled.scope_id()))
            })
            .unwrap();

        let info = ProgramInfo::new(datalog);
        let empty = info.scope(empty, FileId::new(0));
        assert!(!empty.contains("foo"));

        let filled = info.scope(filled, FileId::new(0));
        assert!(filled.contains("foo"));
    }
}
