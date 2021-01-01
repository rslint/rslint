mod builder;
mod derived_facts;
mod graphing;

pub use builder::DatalogBuilder;
pub use derived_facts::{DatalogLint, Outputs};

use crate::globals::JsGlobal;
use ast::{
    ClassId, ExprId, FileId, FileKind, FuncId, GlobalId, GlobalPriv, IPattern, ImportId, Increment,
    ScopeId, StmtId,
};
use ddlog_std::tuple2;
use differential_datalog::{
    ddval::{DDValConvert, DDValue},
    program::{IdxId, RelId, Update},
    DDlog, DeltaMap,
};
use inputs::{
    EveryScope, Expression, File as InputFile, FunctionArg, ImplicitGlobal, InputScope, Statement,
    UserGlobal,
};
use internment::Intern;
use rslint_scoping_ddlog::{api::HDDlog, Indexes, Relations, INPUT_RELIDMAP};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    fs::File,
    io::{self, Write},
    ops::Deref,
    path::Path,
    sync::{Mutex, MutexGuard},
};

// TODO: Make this runtime configurable
const DATALOG_WORKERS: usize = 1;

// TODO: Work on the internment situation, I don't like
//       having to allocate strings for idents
// TODO: Reduce the number of scopes generated as much as possible

pub type DatalogResult<T> = Result<T, String>;

#[derive(Debug)]
pub struct Datalog {
    hddlog: HDDlog,
    transaction_lock: Mutex<()>,
    outputs: Outputs,
}

unsafe impl Send for Datalog {}
unsafe impl Sync for Datalog {}

static_assertions::assert_impl_all!(Datalog: Send, Sync);

impl Datalog {
    pub fn new() -> DatalogResult<Self> {
        let (hddlog, _init_state) = HDDlog::run(DATALOG_WORKERS, false)?;
        let this = Self {
            hddlog,
            transaction_lock: Mutex::new(()),
            outputs: Outputs::new(),
        };

        Ok(this)
    }

    pub fn enable_profiling(&self, enable: bool) {
        tracing::info!(
            "{} ddlog profiling",
            if enable { "enabled" } else { "disabled" },
        );
        // self.hddlog.enable_timely_profiling(enable);
        self.hddlog.enable_cpu_profiling(enable);
    }

    pub fn record_commands<P>(&mut self, file: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let file = File::create(file.as_ref())?;
        self.hddlog.record_commands(&mut Some(Mutex::new(file)));

        Ok(())
    }

    pub fn ddlog_profile(&self) -> String {
        self.hddlog.profile()
    }

    pub fn outputs(&self) -> &Outputs {
        &self.outputs
    }

    pub fn reset(&self) -> DatalogResult<()> {
        tracing::info!("resetting ddlog instance");

        {
            let guard = TransactionGuard::new(&self.hddlog, &self.transaction_lock)?;
            // let handle = guard.handle.as_ref().unwrap();
            for relation in INPUT_RELIDMAP.keys().copied() {
                self.hddlog.clear_relation(relation as RelId)?;
            }

            guard.commit_dump_changes()?;
        }
        self.outputs.clear();

        Ok(())
    }

    // TODO: Make this take an iterator
    pub fn inject_globals(&self, globals: &[JsGlobal]) -> DatalogResult<()> {
        self.transaction(|trans| {
            tracing::trace!("injecting {} global variables", globals.len());

            for global in globals {
                trans.implicit_global(global);
            }

            Ok(())
        })
    }

    // TODO: Make this take an iterator
    pub fn inject_user_globals(&self, file: FileId, globals: &[JsGlobal]) -> DatalogResult<()> {
        self.transaction(|trans| {
            tracing::trace!("injecting {} global variables", globals.len());

            for global in globals {
                trans.user_global(file, global);
            }

            Ok(())
        })
    }

    pub fn dump_inputs<W>(&self, mut output: W) -> io::Result<()>
    where
        W: Write,
    {
        self.hddlog.dump_input_snapshot(&mut output)
    }

    // Note: Ddlog only allows one concurrent transaction, so all calls to this function
    //       will block until the previous completes
    // TODO: We can actually add to the transaction batch concurrently, but transactions
    //       themselves have to be synchronized in some fashion (barrier?)
    // TODO: This is a huge bottleneck, transactions need to be fixed somehow
    pub fn transaction<T, F>(&self, transaction: F) -> DatalogResult<T>
    where
        F: for<'trans> FnOnce(&mut DatalogTransaction<'trans>) -> DatalogResult<T>,
    {
        let span = tracing::info_span!("ddlog transaction");
        let _guard = span.enter();

        let inner = DatalogInner::new(FileId::new(0));
        let mut trans = DatalogTransaction::new(&inner)?;
        let result = transaction(&mut trans)?;

        let delta = {
            let span = tracing::info_span!("ddlog transaction lock");
            let _guard = span.enter();

            let guard = TransactionGuard::new(&self.hddlog, &self.transaction_lock)?;
            self.hddlog
                .apply_valupdates(inner.updates.borrow_mut().drain(..))?;

            guard.commit_dump_changes()?
        };
        self.outputs.batch_update(delta);

        Ok(result)
    }

    pub fn get_expr(&self, expr: ExprId, file: FileId) -> Option<Expression> {
        let query = self.query(
            Indexes::inputs_ExpressionById,
            Some(tuple2(expr, file).into_ddvalue()),
        );

        query
            .map_err(|err| tracing::error!("expression query error: {:?}", err))
            .ok()
            .and_then(|query| {
                if query.len() > 1 {
                    tracing::error!(
                        "more than one expression was returned from query: {:?}",
                        query,
                    );
                }

                query.into_iter().next()
            })
            .map(Expression::from_ddvalue)
    }

    pub fn get_stmt(&self, stmt: StmtId, file: FileId) -> Option<Statement> {
        let query = self.query(
            Indexes::inputs_StatementById,
            Some(tuple2(stmt, file).into_ddvalue()),
        );

        query
            .map_err(|err| tracing::error!("statement query error: {:?}", err))
            .ok()
            .and_then(|query| {
                if query.len() > 1 {
                    tracing::error!(
                        "more than one statement was returned from query: {:?}",
                        query,
                    );
                }

                query.into_iter().next()
            })
            .map(Statement::from_ddvalue)
    }

    pub(crate) fn query(
        &self,
        index: Indexes,
        key: Option<DDValue>,
    ) -> DatalogResult<BTreeSet<DDValue>> {
        if let Some(key) = key {
            self.hddlog.query_index(index as IdxId, key)
        } else {
            self.hddlog.dump_index(index as IdxId)
        }
    }

    pub fn purge_file(&self, file: FileId) -> DatalogResult<()> {
        fn delete_all(
            values: BTreeSet<DDValue>,
            relation: Relations,
        ) -> impl Iterator<Item = Update<DDValue>> {
            values.into_iter().map(move |value| Update::DeleteValue {
                relid: relation as RelId,
                v: value,
            })
        }

        let span = tracing::info_span!("purge file");
        let _guard = span.enter();
        tracing::trace!("purging file {}", file.id);

        let files = self.query(Indexes::inputs_FileById, Some(file.into_ddvalue()))?;
        let input_scopes =
            self.query(Indexes::inputs_InputScopeByFile, Some(file.into_ddvalue()))?;
        let every_scope =
            self.query(Indexes::inputs_EveryScopeByFile, Some(file.into_ddvalue()))?;
        let statements = self.query(Indexes::inputs_StatementByFile, Some(file.into_ddvalue()))?;
        let expressions =
            self.query(Indexes::inputs_ExpressionByFile, Some(file.into_ddvalue()))?;

        // TODO: More though deletion of all sub-relations, this should get rid of
        //       a decently large amount of data though
        let updates = delete_all(files, Relations::inputs_File)
            .chain(delete_all(input_scopes, Relations::inputs_InputScope))
            .chain(delete_all(every_scope, Relations::inputs_EveryScope))
            .chain(delete_all(statements, Relations::inputs_Statement))
            .chain(delete_all(expressions, Relations::inputs_Expression));

        let delta = {
            let span = tracing::info_span!("ddlog transaction lock");
            let _guard = span.enter();

            let guard = TransactionGuard::new(&self.hddlog, &self.transaction_lock)?;
            self.hddlog.apply_valupdates(updates)?;

            guard.commit_dump_changes()?
        };
        self.outputs.batch_update(delta);

        Ok(())
    }

    pub fn get_lints(&self, file: FileId) -> DatalogResult<Vec<DatalogLint>> {
        let span = tracing::info_span!("getting ddlog lints");
        let _guard = span.enter();

        let mut lints = Vec::with_capacity(20);

        lints.extend(self.outputs().no_undef.iter().filter_map(|usage| {
            if usage.key().file == file {
                Some(DatalogLint::NoUndef {
                    var: usage.key().name.clone(),
                    span: usage.key().span,
                    file: usage.key().file,
                })
            } else {
                None
            }
        }));

        lints.extend(self.outputs().no_unused_vars.iter().filter_map(|unused| {
            if unused.key().file == file {
                Some(DatalogLint::NoUnusedVars {
                    var: unused.key().name.clone(),
                    declared: unused.key().span,
                    file: unused.key().file,
                })
            } else {
                None
            }
        }));

        lints.extend(self.outputs().no_typeof_undef.iter().filter_map(|undef| {
            if undef.key().file != file {
                return None;
            }

            let whole_expr = self.get_expr(undef.key().whole_expr, file)?;
            let undefined_portion = self.get_expr(undef.key().undefined_expr, file)?;

            Some(DatalogLint::TypeofUndef {
                whole_expr: whole_expr.span,
                undefined_portion: undefined_portion.span,
                file: whole_expr.file,
            })
        }));

        lints.extend(self.outputs().use_before_def.iter().filter_map(|used| {
            if used.key().file == file {
                Some(DatalogLint::UseBeforeDef {
                    name: used.key().name.clone(),
                    used: used.key().used_in,
                    declared: used.key().declared_in,
                    file: used.key().file,
                })
            } else {
                None
            }
        }));

        lints.extend(self.outputs().no_shadow.iter().filter_map(|shadow| {
            if shadow.key().file == file {
                Some(DatalogLint::NoShadow {
                    variable: shadow.key().variable.clone(),
                    original: shadow.key().original.1,
                    shadow: shadow.key().shadower.1,
                    implicit: shadow.key().implicit,
                    file: shadow.key().file,
                })
            } else {
                None
            }
        }));

        lints.extend(self.outputs().no_unused_labels.iter().filter_map(|label| {
            if label.key().file == file {
                Some(DatalogLint::NoUnusedLabels {
                    label: label.key().label_name.data.clone(),
                    span: label.key().label_name.span,
                    file: label.key().file,
                })
            } else {
                None
            }
        }));

        Ok(lints)
    }
}

impl Default for Datalog {
    fn default() -> Self {
        Self::new().expect("failed to create ddlog instance")
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct DatalogInner {
    updates: RefCell<Vec<Update<DDValue>>>,
    file_id: Cell<FileId>,
    scope_id: Cell<ScopeId>,
    global_id: Cell<GlobalId>,
    import_id: Cell<ImportId>,
    class_id: Cell<ClassId>,
    function_id: Cell<FuncId>,
    statement_id: Cell<StmtId>,
    expression_id: Cell<ExprId>,
}

impl DatalogInner {
    fn new(file_id: FileId) -> Self {
        Self {
            updates: RefCell::new(Vec::with_capacity(100)),
            file_id: Cell::new(file_id),
            scope_id: Cell::new(ScopeId::new(0)),
            global_id: Cell::new(GlobalId::new(0)),
            import_id: Cell::new(ImportId::new(0)),
            class_id: Cell::new(ClassId::new(0)),
            function_id: Cell::new(FuncId::new(0)),
            statement_id: Cell::new(StmtId::new(0)),
            expression_id: Cell::new(ExprId::new(0)),
        }
    }

    fn inc_scope(&self) -> ScopeId {
        self.scope_id.inc()
    }

    fn inc_global(&self) -> GlobalId {
        self.global_id.inc()
    }

    fn inc_import(&self) -> ImportId {
        self.import_id.inc()
    }

    fn inc_class(&self) -> ClassId {
        self.class_id.inc()
    }

    fn inc_function(&self) -> FuncId {
        self.function_id.inc()
    }

    fn inc_statement(&self) -> StmtId {
        self.statement_id.inc()
    }

    fn inc_expression(&self) -> ExprId {
        self.expression_id.inc()
    }

    fn file_id(&self) -> FileId {
        self.file_id.get()
    }

    pub fn insert<V>(&self, relation: Relations, val: V) -> &Self
    where
        V: DDValConvert,
    {
        self.updates.borrow_mut().push(Update::Insert {
            relid: relation as RelId,
            v: val.into_ddvalue(),
        });

        self
    }

    pub fn insert_or_update<V>(&self, relation: Relations, val: V) -> &Self
    where
        V: DDValConvert,
    {
        self.updates.borrow_mut().push(Update::InsertOrUpdate {
            relid: relation as RelId,
            v: val.into_ddvalue(),
        });

        self
    }

    pub fn delete_key<V>(&self, relation: Relations, val: V) -> &Self
    where
        V: DDValConvert,
    {
        self.updates.borrow_mut().push(Update::DeleteKey {
            relid: relation as RelId,
            k: val.into_ddvalue(),
        });

        self
    }
}

pub struct DatalogTransaction<'ddlog> {
    datalog: &'ddlog DatalogInner,
}

impl<'ddlog> DatalogTransaction<'ddlog> {
    const fn new(datalog: &'ddlog DatalogInner) -> DatalogResult<Self> {
        Ok(Self { datalog })
    }

    pub fn file(&self, file_id: FileId, kind: FileKind) -> DatalogScope<'ddlog> {
        self.datalog.file_id.set(file_id);
        self.datalog.scope_id.set(ScopeId::new(0));
        self.datalog.global_id.set(GlobalId::new(0));
        self.datalog.import_id.set(ImportId::new(0));
        self.datalog.class_id.set(ClassId::new(0));
        self.datalog.function_id.set(FuncId::new(0));
        self.datalog.statement_id.set(StmtId::new(0));
        self.datalog.expression_id.set(ExprId::new(0));

        let scope_id = self.datalog.inc_scope();
        self.datalog
            .insert(
                Relations::inputs_File,
                InputFile {
                    id: file_id,
                    kind,
                    top_level_scope: scope_id,
                },
            )
            .insert(
                Relations::inputs_InputScope,
                InputScope {
                    parent: scope_id,
                    child: scope_id,
                    file: file_id,
                },
            )
            .insert(
                Relations::inputs_EveryScope,
                EveryScope {
                    scope: scope_id,
                    file: file_id,
                },
            );

        DatalogScope {
            datalog: self.datalog,
            scope_id,
        }
    }

    pub fn scope(&self) -> DatalogScope<'ddlog> {
        let scope_id = self.datalog.inc_scope();
        self.datalog
            .insert(
                Relations::inputs_InputScope,
                InputScope {
                    parent: scope_id,
                    child: scope_id,
                    file: self.datalog.file_id(),
                },
            )
            .insert(
                Relations::inputs_EveryScope,
                EveryScope {
                    scope: scope_id,
                    file: self.datalog.file_id(),
                },
            );

        DatalogScope {
            datalog: self.datalog,
            scope_id,
        }
    }

    // TODO: Fully integrate global info into ddlog
    fn implicit_global(&self, global: &JsGlobal) -> GlobalId {
        let id = self.datalog.inc_global();
        self.datalog.insert(
            Relations::inputs_ImplicitGlobal,
            ImplicitGlobal {
                id: GlobalId { id: id.id },
                name: Intern::new(global.name.to_string()),
                privileges: if global.writeable {
                    GlobalPriv::ReadWriteGlobal
                } else {
                    GlobalPriv::ReadonlyGlobal
                },
            },
        );

        id
    }

    // TODO: Fully integrate global info into ddlog
    fn user_global(&self, file: FileId, global: &JsGlobal) -> GlobalId {
        let id = self.datalog.inc_global();
        self.datalog.insert(
            Relations::inputs_UserGlobal,
            UserGlobal {
                id: GlobalId { id: id.id },
                file,
                name: Intern::new(global.name.to_string()),
                privileges: if global.writeable {
                    GlobalPriv::ReadWriteGlobal
                } else {
                    GlobalPriv::ReadonlyGlobal
                },
            },
        );

        id
    }
}

impl<'ddlog> Deref for DatalogTransaction<'ddlog> {
    type Target = DatalogInner;

    fn deref(&self) -> &Self::Target {
        self.datalog
    }
}

#[derive(Clone)]
#[must_use]
pub struct DatalogFunction<'ddlog> {
    datalog: &'ddlog DatalogInner,
    func_id: FuncId,
    body_scope: ScopeId,
}

impl<'ddlog> DatalogFunction<'ddlog> {
    pub fn func_id(&self) -> FuncId {
        self.func_id
    }

    pub fn argument(&self, pattern: IPattern, implicit: bool) {
        self.datalog.insert(
            Relations::inputs_FunctionArg,
            FunctionArg {
                file: self.file_id(),
                parent_func: self.func_id(),
                pattern,
                implicit,
            },
        );
    }
}

impl<'ddlog> DatalogBuilder<'ddlog> for DatalogFunction<'ddlog> {
    fn datalog(&self) -> &'ddlog DatalogInner {
        self.datalog
    }

    fn scope_id(&self) -> ScopeId {
        self.body_scope
    }
}

#[derive(Clone)]
#[must_use]
pub struct DatalogScope<'ddlog> {
    datalog: &'ddlog DatalogInner,
    scope_id: ScopeId,
}

impl<'ddlog> DatalogScope<'ddlog> {
    pub fn scope_id(&self) -> ScopeId {
        self.scope_id
    }
}

impl<'ddlog> DatalogBuilder<'ddlog> for DatalogScope<'ddlog> {
    fn datalog(&self) -> &'ddlog DatalogInner {
        self.datalog
    }

    fn scope_id(&self) -> ScopeId {
        self.scope_id
    }
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
fn dump_delta(delta: &differential_datalog::DeltaMap<DDValue>) {
    for (rel, changes) in delta.iter() {
        println!(
            "Changes to relation {}",
            rslint_scoping_ddlog::relid2name(*rel).unwrap()
        );

        for (val, weight) in changes.iter() {
            if *weight == 1 {
                println!(">> {} {:+}", val, weight);
            }
        }

        if !changes.is_empty() {
            println!();
        }
    }
}

struct TransactionGuard<'a> {
    hddlog: &'a HDDlog,
    // handle: Option<TransactionHandle>,
    committed: bool,
    _lock: MutexGuard<'a, ()>,
}

impl<'a> TransactionGuard<'a> {
    pub fn new(hddlog: &'a HDDlog, lock: &'a Mutex<()>) -> DatalogResult<Self> {
        let _lock = lock.lock().expect("failed to lock transaction");
        hddlog.transaction_start()?;

        Ok(Self {
            hddlog,
            // handle: Some(hddlog.transaction_start()?),
            committed: false,
            _lock,
        })
    }

    pub fn commit_dump_changes(mut self) -> DatalogResult<DeltaMap<DDValue>> {
        // let delta = self
        //     .hddlog
        //     .transaction_commit_dump_changes(self.handle.take().unwrap())?;
        let delta = self.hddlog.transaction_commit_dump_changes()?;
        self.committed = true;

        Ok(delta)
    }
}

impl Drop for TransactionGuard<'_> {
    fn drop(&mut self) {
        // if let Some(handle) = self.handle.take() {
        //     if let Err(err) = self.hddlog.transaction_rollback(handle) {
        //         eprintln!("failed to rollback transaction: {}", err);
        //     }
        // }

        if !self.committed {
            if let Err(err) = self.hddlog.transaction_rollback() {
                eprintln!("failed to rollback transaction: {}", err);
            }
        }
    }
}
