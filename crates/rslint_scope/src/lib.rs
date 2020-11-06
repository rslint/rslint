mod analyzer;
mod datalog;
pub mod globals;
mod tests;

pub use datalog::{
    Datalog, DatalogBuilder, DatalogFunction, DatalogResult, DatalogScope, DatalogTransaction,
};

use analyzer::AnalyzerInner;
use analyzer::Visit;
use rslint_core::{CstRule, Rule, RuleCtx};
use rslint_parser::{ast::Stmt, SyntaxNode, SyntaxNodeExt};
use serde::{Deserialize, Serialize};
use types::{InvalidNameUse, VarUseBeforeDeclaration};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeAnalyzer {
    #[serde(skip)]
    datalog: Datalog,
}

impl ScopeAnalyzer {
    pub fn new() -> DatalogResult<Self> {
        Ok(Self {
            datalog: Datalog::new()?,
        })
    }

    pub fn analyze(&self, syntax: &SyntaxNode, ctx: &mut RuleCtx) -> DatalogResult<()> {
        self.analyze_inner(syntax)?;

        for InvalidNameUse { name, span, .. } in self.datalog.invalid_name_uses(None)? {
            let error = ctx
                .err(
                    "datalog-scoping",
                    format!("cannot find value `{}` in this scope", name),
                )
                .primary(span, "not found in this scope".to_owned());

            ctx.add_err(error);
        }

        for VarUseBeforeDeclaration {
            name,
            used_in,
            declared_in,
        } in self.datalog.var_use_before_declaration(None)?
        {
            let error = ctx
                .err(
                    "datalog-scoping",
                    format!("used the variable `{}` before it was declared", name),
                )
                .primary(used_in, "used here (value will be undefined)".to_owned())
                .secondary(declared_in, "declared here".to_owned());

            ctx.add_err(error);
        }

        Ok(())
    }

    fn analyze_inner(&self, syntax: &SyntaxNode) -> DatalogResult<()> {
        let analyzer = AnalyzerInner;

        self.datalog.transaction(|trans| {
            let mut scope = trans.scope();
            for stmt in syntax.children().filter_map(|node| node.try_to::<Stmt>()) {
                if let (_stmt_id, Some(new_scope)) = analyzer.visit(&scope, stmt) {
                    scope = new_scope;
                }
            }

            Ok(())
        })
    }
}

impl Rule for ScopeAnalyzer {
    fn name(&self) -> &'static str {
        "scope-analysis"
    }

    fn group(&self) -> &'static str {
        "errors"
    }
}

#[typetag::serde]
impl CstRule for ScopeAnalyzer {
    fn check_root(&self, root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if let Err(err) = self.analyze(root, ctx) {
            eprintln!("Datalog error: {:?}", err);
        }

        Some(())
    }
}
