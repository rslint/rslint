mod builder;
mod derived_facts;

pub use builder::DatalogBuilder;
pub use derived_facts::{DatalogLint, Outputs};

use crate::globals::JsGlobal;
use differential_datalog::{
    ddval::{DDValConvert, DDValue},
    program::{IdxId, RelId, Update},
    record::Record,
    DDlog,
};
use rslint_scoping_ddlog::{api::HDDlog, Indexes, Relations, INPUT_RELIDMAP};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    sync::Mutex,
};
use types::{
    ast::{
        ClassId, ExprId, FileId, FileKind, FuncId, GlobalId, GlobalPriv, IPattern, ImportId,
        Increment, ScopeId, StmtId,
    },
    ddlog_std::tuple2,
    inputs::{EveryScope, Expression, File as InputFile, FunctionArg, ImplicitGlobal, InputScope},
    internment::Intern,
};

// TODO: Make this runtime configurable
const DATALOG_WORKERS: usize = 2;

// TODO: Work on the internment situation, I don't like
//       having to allocate strings for idents

pub type DatalogResult<T> = Result<T, String>;

#[derive(Debug)]
pub struct Datalog {
    hddlog: HDDlog,
    transaction_lock: Mutex<()>,
    outputs: Outputs,
}

impl Datalog {
    pub fn new() -> DatalogResult<Self> {
        let (hddlog, _init_state) =
            HDDlog::run(DATALOG_WORKERS, false, |_: usize, _: &Record, _: isize| {})?;
        let this = Self {
            hddlog,
            transaction_lock: Mutex::new(()),
            outputs: Outputs::new(),
        };

        Ok(this)
    }

    pub fn outputs(&self) -> &Outputs {
        &self.outputs
    }

    pub fn reset(&self) -> DatalogResult<()> {
        self.transaction(|_trans| {
            for relation in INPUT_RELIDMAP.keys().copied() {
                self.hddlog.clear_relation(relation as RelId)?;
            }

            Ok(())
        })?;

        self.outputs.clear();

        Ok(())
    }

    // TODO: Make this take an iterator
    pub fn inject_globals(&self, file: FileId, globals: &[JsGlobal]) -> DatalogResult<()> {
        self.transaction(|trans| {
            for global in globals {
                trans.implicit_global(file, global);
            }

            Ok(())
        })
    }

    // FIXME: Make this only apply to a single file or remove it
    pub fn clear_globals(&self) -> DatalogResult<()> {
        let _transaction_guard = self.transaction_lock.lock().unwrap();

        self.hddlog.transaction_start()?;
        self.hddlog
            .clear_relation(Relations::inputs_ImplicitGlobal as RelId)?;

        self.hddlog.transaction_commit()
    }

    pub fn dump_inputs(&self) -> DatalogResult<String> {
        let mut inputs = Vec::new();
        self.hddlog.dump_input_snapshot(&mut inputs).unwrap();

        Ok(String::from_utf8(inputs).unwrap())
    }

    // Note: Ddlog only allows one concurrent transaction, so all calls to this function
    //       will block until the previous completes
    // TODO: We can actually add to the transaction batch concurrently, but transactions
    //       themselves have to be synchronized in some fashion (barrier?)
    pub fn transaction<T, F>(&self, transaction: F) -> DatalogResult<T>
    where
        F: for<'trans> FnOnce(&mut DatalogTransaction<'trans>) -> DatalogResult<T>,
    {
        let inner = DatalogInner::new(FileId::new(0));
        let mut trans = DatalogTransaction::new(&inner)?;
        let result = transaction(&mut trans)?;

        let delta = {
            let _transaction_guard = self.transaction_lock.lock().unwrap();

            self.hddlog.transaction_start()?;
            self.hddlog
                .apply_valupdates(inner.updates.borrow_mut().drain(..))?;

            self.hddlog.transaction_commit_dump_changes()?
        };
        self.outputs.batch_update(delta);

        Ok(result)
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

    pub fn get_lints(&self, file: FileId) -> DatalogResult<Vec<DatalogLint>> {
        let mut lints = Vec::new();

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

        lints.extend(self.outputs().unused_variables.iter().filter_map(|unused| {
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

        lints.extend(self.outputs().typeof_undef.iter().filter_map(|undef| {
            if undef.key().file != file {
                return None;
            }

            let whole_expr = self
                .query(
                    Indexes::inputs_ExpressionById,
                    Some(tuple2(undef.key().whole_expr, file).into_ddvalue()),
                )
                .ok()?
                .into_iter()
                .next()
                .map(|expr| unsafe { Expression::from_ddvalue(expr) })?;

            let undefined_portion = self
                .query(
                    Indexes::inputs_ExpressionById,
                    Some(tuple2(undef.key().undefined_expr, file).into_ddvalue()),
                )
                .ok()?
                .into_iter()
                .next()
                .map(|expr| unsafe { Expression::from_ddvalue(expr) })?
                .span;

            Some(DatalogLint::TypeofUndef {
                whole_expr: whole_expr.span,
                undefined_portion,
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

    fn insert<V>(&self, relation: Relations, val: V) -> &Self
    where
        V: DDValConvert,
    {
        self.updates.borrow_mut().push(Update::Insert {
            relid: relation as RelId,
            v: val.into_ddvalue(),
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
    fn implicit_global(&self, file: FileId, global: &JsGlobal) -> GlobalId {
        let id = self.datalog.inc_global();
        self.datalog.insert(
            Relations::inputs_ImplicitGlobal,
            ImplicitGlobal {
                id: GlobalId { id: id.id },
                name: Intern::new(global.name.to_string()),
                file,
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

#[derive(Clone)]
#[must_use]
pub struct DatalogFunction<'ddlog> {
    datalog: &'ddlog DatalogInner,
    func_id: FuncId,
    scope_id: ScopeId,
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
        self.scope_id
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
