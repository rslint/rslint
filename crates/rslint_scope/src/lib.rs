mod analyzer;
mod datalog;
pub mod globals;
pub mod scoping;
mod tests;

pub use datalog::{
    Datalog, DatalogBuilder, DatalogFunction, DatalogResult, DatalogScope, DatalogTransaction,
};

use analyzer::{AnalyzerInner, Visit};
use rslint_parser::{
    ast::{Module, ModuleItem, Script},
    SyntaxNode, SyntaxNodeExt,
};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use types::ast::{FileId, FileKind, JSFlavor};

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

    pub fn analyze(&self, file: FileId, syntax: &SyntaxNode) -> DatalogResult<()> {
        let analyzer = AnalyzerInner;

        self.datalog.transaction(|trans| {
            debug_assert!(
                syntax.is::<Script>() || syntax.is::<Module>(),
                "expected a Script or a Module to be analyzed",
            );

            let file_kind = if syntax.is::<Script>() {
                FileKind::JavaScript {
                    flavor: JSFlavor::Vanilla,
                }
            } else if syntax.is::<Module>() {
                FileKind::JavaScript {
                    flavor: JSFlavor::Module,
                }
            } else {
                FileKind::JavaScript {
                    flavor: JSFlavor::Vanilla,
                }
            };

            let mut scope = trans.file(file, file_kind);
            for item in syntax.children().filter_map(|x| x.try_to::<ModuleItem>()) {
                if let Some(new_scope) = analyzer.visit(&scope, item) {
                    scope = new_scope;
                }
            }

            Ok(())
        })
    }
}

impl Deref for ScopeAnalyzer {
    type Target = Datalog;

    fn deref(&self) -> &Self::Target {
        &self.datalog
    }
}
