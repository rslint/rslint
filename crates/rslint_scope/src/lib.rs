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
use crossbeam_queue::SegQueue;
use dashmap::DashMap;
use datalog::{DatalogInner, DatalogTransaction};
use differential_datalog::{
    ddval::{DDValConvert, DDValue},
    program::{RelId, Update},
    DDlog,
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
use std::{iter, mem, ops::Deref, sync::Arc};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ScopeAnalyzer {
    #[serde(skip)]
    datalog: Arc<Datalog>,
    #[serde(skip)]
    registered_lints: Arc<DashMap<FileId, LintSet>>,
    #[serde(skip)]
    config_queue: Arc<SegQueue<Update<DDValue>>>,
}

impl ScopeAnalyzer {
    pub fn new(num_workers: usize) -> DatalogResult<Self> {
        tracing::trace!("creating a new ddlog instance");

        Ok(Self {
            datalog: Arc::new(Datalog::new(num_workers)?),
            registered_lints: Arc::new(DashMap::new()),
            config_queue: Arc::new(SegQueue::new()),
        })
    }

    pub fn shutdown(&self) {
        let _ = self.datalog.hddlog.stop();
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

    pub fn flush_config_queue(&self) -> DatalogResult<()> {
        if self.config_queue.is_empty() {
            return Ok(());
        }

        self.analyze_raw(iter::from_fn(|| self.config_queue.pop()).collect())
    }

    #[doc(hidden)]
    pub fn analyze_raw(&self, updates: Vec<Update<DDValue>>) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            let mut old_updates = trans.datalog.updates.borrow_mut();
            debug_assert!(old_updates.is_empty());
            *old_updates = updates;

            Ok(())
        })
    }

    #[doc(hidden)]
    pub fn analyze_raw_batch(
        &self,
        updates: impl IntoIterator<Item = Vec<Update<DDValue>>>,
    ) -> DatalogResult<()> {
        self.datalog.transaction(|trans| {
            let mut old_updates = trans.datalog.updates.borrow_mut();
            debug_assert!(old_updates.is_empty());

            for update in updates {
                old_updates.extend(update)
            }

            Ok(())
        })
    }

    pub fn no_unused_vars(&self, file: FileId, config: Option<NoUnusedVarsConfig>) {
        if self
            .registered_lints
            .get(&file)
            .map(|set| set.get_no_unused_vars() == config.is_some())
            .unwrap_or_default()
        {
            return;
        }

        self.config_queue
            .push(if let Some(config) = config.clone() {
                Update::InsertOrUpdate {
                    relid: Relations::config_EnableNoUnusedVars as RelId,
                    v: EnableNoUnusedVars {
                        file,
                        config: Ref::from(config),
                    }
                    .into_ddvalue(),
                }
            } else {
                Update::DeleteKey {
                    relid: Relations::config_EnableNoUnusedVars as RelId,
                    k: file.into_ddvalue(),
                }
            });

        self.registered_lints
            .entry(file)
            .or_insert(LintSet::new())
            .no_unused_vars(config.is_some());
    }

    pub fn no_undef(&self, file: FileId, config: Option<NoUndefConfig>) {
        if self
            .registered_lints
            .get(&file)
            .map(|set| set.get_no_undef() == config.is_some())
            .unwrap_or_default()
        {
            return;
        }

        self.config_queue
            .push(if let Some(config) = config.clone() {
                Update::InsertOrUpdate {
                    relid: Relations::config_EnableNoUndef as RelId,
                    v: EnableNoUndef {
                        file,
                        config: Ref::from(config),
                    }
                    .into_ddvalue(),
                }
            } else {
                Update::DeleteKey {
                    relid: Relations::config_EnableNoUndef as RelId,
                    k: file.into_ddvalue(),
                }
            });

        self.registered_lints
            .entry(file)
            .or_insert(LintSet::new())
            .no_undef(config.is_some());
    }

    pub fn no_unused_labels(&self, file: FileId, config: Option<NoUnusedLabelsConfig>) {
        if self
            .registered_lints
            .get(&file)
            .map(|set| set.get_no_unused_labels() == config.is_some())
            .unwrap_or_default()
        {
            return;
        }

        self.config_queue
            .push(if let Some(config) = config.clone() {
                Update::InsertOrUpdate {
                    relid: Relations::config_EnableNoUnusedLabels as RelId,
                    v: EnableNoUnusedLabels {
                        file,
                        config: Ref::from(config),
                    }
                    .into_ddvalue(),
                }
            } else {
                Update::DeleteKey {
                    relid: Relations::config_EnableNoUnusedLabels as RelId,
                    k: file.into_ddvalue(),
                }
            });

        self.registered_lints
            .entry(file)
            .or_insert(LintSet::new())
            .no_unused_labels(config.is_some());
    }

    pub fn no_typeof_undef(&self, file: FileId, config: Option<NoTypeofUndefConfig>) {
        if self
            .registered_lints
            .get(&file)
            .map(|set| set.get_no_typeof_undef() == config.is_some())
            .unwrap_or_default()
        {
            return;
        }

        self.config_queue
            .push(if let Some(config) = config.clone() {
                Update::InsertOrUpdate {
                    relid: Relations::config_EnableNoTypeofUndef as RelId,
                    v: EnableNoTypeofUndef {
                        file,
                        config: Ref::from(config),
                    }
                    .into_ddvalue(),
                }
            } else {
                Update::DeleteKey {
                    relid: Relations::config_EnableNoTypeofUndef as RelId,
                    k: file.into_ddvalue(),
                }
            });

        self.registered_lints
            .entry(file)
            .or_insert(LintSet::new())
            .no_typeof_undef(config.is_some());
    }

    pub fn no_use_before_def(&self, file: FileId, config: Option<NoUseBeforeDefConfig>) {
        if self
            .registered_lints
            .get(&file)
            .map(|set| set.get_no_use_before_def() == config.is_some())
            .unwrap_or_default()
        {
            return;
        }

        self.config_queue
            .push(if let Some(config) = config.clone() {
                Update::InsertOrUpdate {
                    relid: Relations::config_EnableNoUseBeforeDef as RelId,
                    v: EnableNoUseBeforeDef {
                        file,
                        config: Ref::from(config),
                    }
                    .into_ddvalue(),
                }
            } else {
                Update::DeleteKey {
                    relid: Relations::config_EnableNoUseBeforeDef as RelId,
                    k: file.into_ddvalue(),
                }
            });

        self.registered_lints
            .entry(file)
            .or_insert(LintSet::new())
            .no_use_before_def(config.is_some());
    }

    pub fn no_shadow(&self, file: FileId, config: Option<NoShadowConfig>) {
        if self
            .registered_lints
            .get(&file)
            .map(|set| set.get_no_shadow() == config.is_some())
            .unwrap_or_default()
        {
            return;
        }

        self.config_queue
            .push(if let Some(config) = config.clone() {
                Update::InsertOrUpdate {
                    relid: Relations::config_EnableNoShadow as RelId,
                    v: EnableNoShadow {
                        file,
                        config: Ref::from(config),
                    }
                    .into_ddvalue(),
                }
            } else {
                Update::DeleteKey {
                    relid: Relations::config_EnableNoShadow as RelId,
                    k: file.into_ddvalue(),
                }
            });

        self.registered_lints
            .entry(file)
            .or_insert(LintSet::new())
            .no_shadow(config.is_some());
    }
}

impl Deref for ScopeAnalyzer {
    type Target = Datalog;

    fn deref(&self) -> &Self::Target {
        &self.datalog
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
struct LintSet(u8);

impl LintSet {
    const NO_SHADOW: u8 = 0;
    const NO_USE_BEFORE_DEF: u8 = 1;
    const NO_TYPEOF_UNDEF: u8 = 2;
    const NO_UNUSED_LABELS: u8 = 3;
    const NO_UNDEF: u8 = 4;
    const NO_UNUSED_VARS: u8 = 5;

    const fn new() -> Self {
        Self(0)
    }

    fn get_no_shadow(&self) -> bool {
        get_bit(self.0, Self::NO_SHADOW)
    }

    fn no_shadow(&mut self, enabled: bool) {
        self.0 = set_bit(self.0, enabled, Self::NO_SHADOW)
    }

    fn get_no_use_before_def(&self) -> bool {
        get_bit(self.0, Self::NO_USE_BEFORE_DEF)
    }

    fn no_use_before_def(&mut self, enabled: bool) {
        self.0 = set_bit(self.0, enabled, Self::NO_USE_BEFORE_DEF)
    }

    fn get_no_typeof_undef(&self) -> bool {
        get_bit(self.0, Self::NO_TYPEOF_UNDEF)
    }

    fn no_typeof_undef(&mut self, enabled: bool) {
        self.0 = set_bit(self.0, enabled, Self::NO_TYPEOF_UNDEF)
    }

    fn get_no_unused_labels(&self) -> bool {
        get_bit(self.0, Self::NO_UNUSED_LABELS)
    }

    fn no_unused_labels(&mut self, enabled: bool) {
        self.0 = set_bit(self.0, enabled, Self::NO_UNUSED_LABELS)
    }

    fn get_no_undef(&self) -> bool {
        get_bit(self.0, Self::NO_UNDEF)
    }

    fn no_undef(&mut self, enabled: bool) {
        self.0 = set_bit(self.0, enabled, Self::NO_UNDEF)
    }

    fn get_no_unused_vars(&self) -> bool {
        get_bit(self.0, Self::NO_UNUSED_VARS)
    }

    fn no_unused_vars(&mut self, enabled: bool) {
        self.0 = set_bit(self.0, enabled, Self::NO_UNUSED_VARS)
    }
}

fn get_bit(number: u8, bit: u8) -> bool {
    ((number >> bit) & 1) > 0
}

fn set_bit(current: u8, value: bool, bit: u8) -> u8 {
    (current & !(1 << bit)) | ((value as u8) << bit)
}

#[doc(hidden)]
pub fn make_updates(file: FileId, syntax: &SyntaxNode) -> Vec<Update<DDValue>> {
    debug_assert!(
        syntax.is::<Script>() || syntax.is::<Module>(),
        "expected a Script or a Module to be analyzed",
    );

    let analyzer = AnalyzerInner;
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

    let inner = DatalogInner::new(file);
    let mut scope = DatalogTransaction::new(&inner).file(file, file_kind);
    for item in syntax.children().filter_map(|x| x.try_to::<ModuleItem>()) {
        if let Some(new_scope) = analyzer.visit(&scope, item) {
            scope = new_scope;
        }
    }

    let mut updates = inner.updates.borrow_mut();
    mem::take(&mut *updates)
}

#[test]
fn set_bit_test() {
    assert_eq!(get_bit(1, 0), true);
    assert_eq!(get_bit(0, 0), false);

    let num = 0;
    assert_eq!(get_bit(set_bit(num, true, 0), 0), true);
    assert_eq!(get_bit(set_bit(num, false, 0), 0), false);
    assert_eq!(get_bit(num, 7), false);
    assert_eq!(get_bit(set_bit(num, true, 7), 7), true);
    assert_eq!(get_bit(set_bit(num, false, 7), 7), false);
}

#[test]
fn lint_set_test() {
    let mut set = LintSet::new();

    assert_eq!(set.get_no_shadow(), false);
    set.no_shadow(true);
    assert_eq!(set.get_no_shadow(), true);
    set.no_shadow(true);
    assert_eq!(set.get_no_shadow(), true);
    set.no_shadow(false);
    assert_eq!(set.get_no_shadow(), false);
    set.no_shadow(false);
    assert_eq!(set.get_no_shadow(), false);

    assert_eq!(set.get_no_unused_vars(), false);
    set.no_unused_vars(true);
    assert_eq!(set.get_no_unused_vars(), true);
    set.no_unused_vars(true);
    assert_eq!(set.get_no_unused_vars(), true);
    set.no_unused_vars(false);
    assert_eq!(set.get_no_unused_vars(), false);
    set.no_unused_vars(false);
    assert_eq!(set.get_no_unused_vars(), false);
}
