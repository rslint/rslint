mod analyzer;
mod datalog;
pub mod globals;

pub use datalog::{
    Datalog, DatalogBuilder, DatalogFunction, DatalogResult, DatalogScope, DatalogTransaction,
    DerivedFacts,
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
        let analyzer = AnalyzerInner;

        let facts = self.datalog.transaction(|trans| {
            let scope = trans.scope();
            for stmt in syntax.children().filter_map(|node| node.try_to::<Stmt>()) {
                analyzer.visit(&scope, stmt);
            }

            Ok(())
        })?;

        for InvalidNameUse { name, span, .. } in facts.invalid_name_uses {
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
        } in facts.var_use_before_decl
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
