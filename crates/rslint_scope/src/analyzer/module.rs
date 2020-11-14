use crate::{
    datalog::{DatalogBuilder, DatalogScope},
    AnalyzerInner, Visit,
};
use rslint_parser::ast::{
    AstChildren, ExportDecl, ExportNamed, ImportClause, ImportDecl, ModuleItem, NamedImports,
};
use types::ast::{ImportClause as DatalogImportClause, NamedImport as DatalogNamedImport, StmtId};

impl<'ddlog> Visit<'ddlog, ModuleItem> for AnalyzerInner {
    type Output = Option<DatalogScope<'ddlog>>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, item: ModuleItem) -> Self::Output {
        match item {
            ModuleItem::ImportDecl(import) => {
                self.visit(scope, import);
                None
            }
            ModuleItem::ExportNamed(export) => {
                self.visit(scope, export);
                None
            }
            ModuleItem::ExportDefaultDecl(_export) => {
                // self.visit(scope, export);
                None
            }
            ModuleItem::ExportDefaultExpr(_export) => {
                // self.visit(scope, export);
                None
            }
            ModuleItem::ExportWildcard(_export) => {
                // self.visit(scope, export);
                None
            }
            ModuleItem::ExportDecl(export) => self.visit(scope, export).map(|(_id, scope)| scope),
            ModuleItem::Stmt(stmt) => Some(self.visit(scope, stmt).1),
        }
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<ModuleItem>> for AnalyzerInner {
    type Output = DatalogScope<'ddlog>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        items: AstChildren<ModuleItem>,
    ) -> Self::Output {
        let mut scope = scope.scope();
        for item in items {
            if let Some(new_scope) = self.visit(&scope, item) {
                scope = new_scope;
            }
        }

        scope
    }
}

impl<'ddlog> Visit<'ddlog, ImportDecl> for AnalyzerInner {
    type Output = ();

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, import: ImportDecl) -> Self::Output {
        let clauses = self.visit(scope, import.imports());
        scope.import_decl(clauses);
    }
}

impl<'ddlog> Visit<'ddlog, ImportClause> for AnalyzerInner {
    type Output = DatalogImportClause;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, clause: ImportClause) -> Self::Output {
        match clause {
            ImportClause::WildcardImport(wildcard) => DatalogImportClause::WildcardImport {
                alias: self.visit(scope, wildcard.alias()).into(),
            },
            ImportClause::NamedImports(named) => DatalogImportClause::GroupedImport {
                imports: self.visit(scope, named).into(),
            },
            ImportClause::Name(name) => DatalogImportClause::SingleImport {
                name: self.visit(scope, name),
            },
        }
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<ImportClause>> for AnalyzerInner {
    type Output = Vec<DatalogImportClause>;

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        imports: AstChildren<ImportClause>,
    ) -> Self::Output {
        imports.map(|import| self.visit(scope, import)).collect()
    }
}

impl<'ddlog> Visit<'ddlog, NamedImports> for AnalyzerInner {
    type Output = Vec<DatalogNamedImport>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, imports: NamedImports) -> Self::Output {
        imports
            .specifiers()
            .map(|spec| DatalogNamedImport {
                name: self.visit(scope, spec.name()).into(),
                alias: self.visit(scope, spec.alias()).into(),
            })
            .collect()
    }
}

impl<'ddlog> Visit<'ddlog, ExportDecl> for AnalyzerInner {
    type Output = Option<(Option<StmtId>, DatalogScope<'ddlog>)>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, export: ExportDecl) -> Self::Output {
        self.visit(scope, export.decl().map(|decl| (decl, true)))
    }
}

impl<'ddlog> Visit<'ddlog, ExportNamed> for AnalyzerInner {
    type Output = ();

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, export: ExportNamed) -> Self::Output {
        for specifier in export.specifiers() {
            let name = specifier.name().map(|name| self.visit(scope, name));
            let alias = specifier.alias().map(|alias| self.visit(scope, alias));

            scope.export_named(name, alias);
        }
    }
}
