mod analyzer;
mod datalog;
pub mod globals;
// pub mod scoping;
mod tests;

pub use ast;
pub use datalog::{Datalog, DatalogLint, DatalogResult};
pub use ddlog_std::{self, Ref};
pub use rslint_scoping_ddlog::typedefs::{
    ast::FileId,
    config::{
        NoShadowConfig, NoShadowHoisting, NoTypeofUndefConfig, NoUndefConfig, NoUnusedLabelsConfig,
        NoUnusedVarsConfig, NoUseBeforeDefConfig,
    },
    regex::{Regex, RegexSet},
};

use analyzer::{AnalyzerInner, Visit};
use config::{
    EnableNoShadow, EnableNoTypeofUndef, EnableNoUndef, EnableNoUnusedLabels, EnableNoUnusedVars,
    EnableNoUseBeforeDef,
};
use rslint_parser::{
    ast::{Module, ModuleItem, Script},
    SyntaxNode, SyntaxNodeExt,
};
use rslint_scoping_ddlog::{
    typedefs::ast::{FileKind, JSFlavor},
    Relations,
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

    pub fn analyze_batch(&self, files: &[(FileId, SyntaxNode)]) -> DatalogResult<()> {
        let span = tracing::info_span!("ddlog batch analyze");
        let _guard = span.enter();

        let analyzer = AnalyzerInner;
        self.datalog.transaction(|trans| {
            tracing::info!("starting ddlog batch with {} files", files.len());

            for (file, syntax) in files {
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

                let mut scope = trans.file(*file, file_kind);
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

    pub fn no_unused_vars(
        &self,
        file: FileId,
        config: Option<NoUnusedVarsConfig>,
    ) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            if let Some(config) = config {
                trans.insert_or_update(
                    Relations::config_EnableNoUnusedVars,
                    EnableNoUnusedVars {
                        file,
                        config: Ref::from(config),
                    },
                );
            } else {
                trans.delete_key(Relations::config_EnableNoUnusedVars, file);
            }

            Ok(())
        })
    }

    pub fn no_undef(&self, file: FileId, config: Option<NoUndefConfig>) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            if let Some(config) = config {
                trans.insert_or_update(
                    Relations::config_EnableNoUndef,
                    EnableNoUndef {
                        file,
                        config: Ref::from(config),
                    },
                );
            } else {
                trans.delete_key(Relations::config_EnableNoUndef, file);
            }

            Ok(())
        })
    }

    pub fn no_unused_labels(
        &self,
        file: FileId,
        config: Option<NoUnusedLabelsConfig>,
    ) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            if let Some(config) = config {
                trans.insert_or_update(
                    Relations::config_EnableNoUnusedLabels,
                    EnableNoUnusedLabels {
                        file,
                        config: Ref::from(config),
                    },
                );
            } else {
                trans.delete_key(Relations::config_EnableNoUnusedLabels, file);
            }

            Ok(())
        })
    }

    pub fn no_typeof_undef(
        &self,
        file: FileId,
        config: Option<NoTypeofUndefConfig>,
    ) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            if let Some(config) = config {
                trans.insert_or_update(
                    Relations::config_EnableNoTypeofUndef,
                    EnableNoTypeofUndef {
                        file,
                        config: Ref::from(config),
                    },
                );
            } else {
                trans.delete_key(Relations::config_EnableNoTypeofUndef, file);
            }

            Ok(())
        })
    }

    pub fn no_use_before_def(
        &self,
        file: FileId,
        config: Option<NoUseBeforeDefConfig>,
    ) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            if let Some(config) = config {
                trans.insert_or_update(
                    Relations::config_EnableNoUseBeforeDef,
                    EnableNoUseBeforeDef {
                        file,
                        config: Ref::from(config),
                    },
                );
            } else {
                trans.delete_key(Relations::config_EnableNoUseBeforeDef, file);
            }

            Ok(())
        })
    }

    pub fn no_shadow(&self, file: FileId, config: Option<NoShadowConfig>) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            if let Some(config) = config {
                trans.insert_or_update(
                    Relations::config_EnableNoShadow,
                    EnableNoShadow {
                        file,
                        config: Ref::from(config),
                    },
                );
            } else {
                trans.delete_key(Relations::config_EnableNoShadow, file);
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
