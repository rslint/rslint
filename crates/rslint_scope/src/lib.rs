mod analyzer;
mod datalog;
pub mod globals;
// pub mod scoping;
mod tests;

pub use ast::{self, FileId};
pub use config::{Config, NoShadowHoisting};
pub use datalog::{Datalog, DatalogLint, DatalogResult};
pub use ddlog_std;

use analyzer::{AnalyzerInner, Visit};
use ast::{FileKind, JSFlavor};
use rslint_parser::{
    ast::{Module, ModuleItem, Script},
    SyntaxNode, SyntaxNodeExt,
};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScopeAnalyzer {
    #[serde(skip)]
    datalog: Arc<Datalog>,
}

impl ScopeAnalyzer {
    pub fn new() -> DatalogResult<Self> {
        tracing::trace!("creating a new ddlog instance");

        Ok(Self {
            datalog: Arc::new(Datalog::new()?),
        })
    }

    pub fn analyze_batch(&self, files: &[(FileId, SyntaxNode, Config)]) -> DatalogResult<()> {
        let span = tracing::info_span!("ddlog batch analyze");
        let _guard = span.enter();

        let analyzer = AnalyzerInner;
        self.datalog.transaction(|trans| {
            tracing::info!("starting ddlog batch with {} files", files.len());

            for (file, syntax, config) in files {
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

                let mut scope = trans.file(*file, file_kind, config.clone());
                for item in syntax.children().filter_map(|x| x.try_to::<ModuleItem>()) {
                    if let Some(new_scope) = analyzer.visit(&scope, item) {
                        scope = new_scope;
                    }
                }
            }

            tracing::info!("finished ddlog batch with {} files", files.len());
            Ok(())
        })
    }

    pub fn analyze(&self, file: FileId, syntax: &SyntaxNode, config: Config) -> DatalogResult<()> {
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

            let mut scope = trans.file(file, file_kind, config);
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
