mod derived_facts;

pub use derived_facts::Outputs;

use crate::globals::JsGlobal;
use differential_datalog::{
    ddval::{DDValConvert, DDValue},
    int::Int,
    program::{IdxId, RelId, Update},
    record::Record,
    DDlog, DeltaMap,
};
use rslint_parser::{BigInt, TextRange};
use rslint_scoping_ddlog::{api::HDDlog, relid2name, Indexes, Relations, INPUT_RELIDMAP};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    fs::File,
    mem,
    sync::{Arc, Mutex},
};
use types::{
    ast::{
        ArrayElement, AssignOperand, BinOperand, ClassId, ExprId, ExprKind, ForInit, FuncId,
        GlobalId, IClassElement, IPattern, ImportClause, ImportId, Increment, LitKind, Name,
        Pattern, PropertyKey, PropertyVal, Scope, Spanned, StmtId, StmtKind, SwitchClause,
        TryHandler, UnaryOperand,
    },
    ddlog_std::Either,
    inputs::{
        Array, Arrow, ArrowParam, Assign, Await, BinOp, BracketAccess, Break, Call, Class,
        ClassExpr, ConstDecl, Continue, DoWhile, DotAccess, ExprBigInt, ExprBool, ExprNumber,
        ExprString, Expression, For, ForIn, Function, FunctionArg, If, ImplicitGlobal, ImportDecl,
        InlineFunc, InlineFuncParam, InputScope, Label, LetDecl, NameRef, New, Property, Return,
        Statement, Switch, SwitchCase, Template, Ternary, Throw, Try, UnaryOp, VarDecl, While,
        With, Yield,
    },
    internment::Intern,
};

// TODO: Work on the internment situation, I don't like
//       having to allocate strings for idents

pub type DatalogResult<T> = Result<T, String>;

#[derive(Debug, Clone)]
pub struct Datalog {
    datalog: Arc<Mutex<DatalogInner>>,
    outputs: Outputs,
}

impl Datalog {
    pub fn new() -> DatalogResult<Self> {
        let (hddlog, _init_state) = HDDlog::run(2, false, |_: usize, _: &Record, _: isize| {})?;
        let this = Self {
            datalog: Arc::new(Mutex::new(DatalogInner::new(hddlog))),
            outputs: Outputs::new(),
        };

        Ok(this)
    }

    pub fn outputs(&self) -> &Outputs {
        &self.outputs
    }

    pub fn with_replay_file(&self, file: File) {
        self.datalog
            .lock()
            .unwrap()
            .hddlog
            .record_commands(&mut Some(Mutex::new(file)));
    }

    pub fn reset(&self) -> DatalogResult<()> {
        self.transaction(|trans| {
            for relation in INPUT_RELIDMAP.keys().copied() {
                trans.datalog.hddlog.clear_relation(relation as RelId)?;
            }

            Ok(())
        })?;

        self.outputs.clear();

        Ok(())
    }

    // TODO: Make this take an iterator
    pub fn inject_globals(&self, globals: &[JsGlobal]) -> DatalogResult<()> {
        self.transaction(|trans| {
            for global in globals {
                trans.implicit_global(global);
            }

            Ok(())
        })
    }

    pub fn clear_globals(&self) -> DatalogResult<()> {
        self.transaction(|trans| {
            trans
                .datalog
                .hddlog
                .clear_relation(Relations::inputs_ImplicitGlobal as RelId)?;

            Ok(())
        })
    }

    pub fn dump_inputs(&self) -> DatalogResult<String> {
        let mut inputs = Vec::new();
        self.datalog
            .lock()
            .unwrap()
            .hddlog
            .dump_input_snapshot(&mut inputs)
            .unwrap();

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
        let datalog = self
            .datalog
            .lock()
            .expect("failed to lock datalog for transaction");

        let mut trans = DatalogTransaction::new(&*datalog)?;
        let result = transaction(&mut trans)?;
        self.outputs.batch_update(trans.commit()?);

        Ok(result)
    }

    pub(crate) fn query(
        &self,
        index: Indexes,
        key: Option<DDValue>,
    ) -> DatalogResult<BTreeSet<DDValue>> {
        let ddlog = self.datalog.lock().unwrap();
        if let Some(key) = key {
            ddlog.hddlog.query_index(index as IdxId, key)
        } else {
            ddlog.hddlog.dump_index(index as IdxId)
        }
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
    hddlog: HDDlog,
    updates: RefCell<Vec<Update<DDValue>>>,
    scope_id: Cell<Scope>,
    global_id: Cell<GlobalId>,
    import_id: Cell<ImportId>,
    class_id: Cell<ClassId>,
    function_id: Cell<FuncId>,
    statement_id: Cell<StmtId>,
    expression_id: Cell<ExprId>,
}

impl DatalogInner {
    fn new(hddlog: HDDlog) -> Self {
        Self {
            hddlog,
            updates: RefCell::new(Vec::with_capacity(100)),
            scope_id: Cell::new(Scope::new(0)),
            global_id: Cell::new(GlobalId::new(0)),
            import_id: Cell::new(ImportId::new(0)),
            class_id: Cell::new(ClassId::new(0)),
            function_id: Cell::new(FuncId::new(0)),
            statement_id: Cell::new(StmtId::new(0)),
            expression_id: Cell::new(ExprId::new(0)),
        }
    }

    fn inc_scope(&self) -> Scope {
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

    fn insert<V>(&self, relation: RelId, val: V) -> &Self
    where
        V: DDValConvert,
    {
        self.updates.borrow_mut().push(Update::Insert {
            relid: relation,
            v: val.into_ddvalue(),
        });

        self
    }
}

pub struct DatalogTransaction<'ddlog> {
    datalog: &'ddlog DatalogInner,
}

impl<'ddlog> DatalogTransaction<'ddlog> {
    fn new(datalog: &'ddlog DatalogInner) -> DatalogResult<Self> {
        datalog.hddlog.transaction_start()?;

        Ok(Self { datalog })
    }

    pub fn scope(&self) -> DatalogScope<'ddlog> {
        let scope_id = self.datalog.inc_scope();
        self.datalog
            .insert(
                Relations::inputs_InputScope as RelId,
                InputScope {
                    parent: scope_id,
                    child: scope_id,
                },
            )
            .insert(Relations::inputs_EveryScope as RelId, scope_id);

        DatalogScope {
            datalog: self.datalog,
            scope_id,
        }
    }

    // TODO: Fully integrate global info into ddlog
    fn implicit_global(&self, global: &JsGlobal) -> GlobalId {
        let id = self.datalog.inc_global();
        self.datalog.insert(
            Relations::inputs_ImplicitGlobal as RelId,
            ImplicitGlobal {
                id,
                name: Intern::new(global.name.to_string()),
            },
        );

        id
    }

    pub fn commit(self) -> DatalogResult<DeltaMap<DDValue>> {
        let updates = mem::take(&mut *self.datalog.updates.borrow_mut());
        self.datalog.hddlog.apply_valupdates(updates.into_iter())?;

        let delta = self.datalog.hddlog.transaction_commit_dump_changes()?;

        // #[cfg(debug_assertions)]
        // {
        //     println!("== start transaction ==");
        //     dump_delta(&delta);
        //     println!("==  end transaction  ==\n\n");
        // }

        Ok(delta)
    }
}

pub trait DatalogBuilder<'ddlog> {
    fn scope_id(&self) -> Scope;

    fn datalog(&self) -> &'ddlog DatalogInner;

    fn scope(&self) -> DatalogScope<'ddlog> {
        let parent = self.scope_id();
        let child = self.datalog().inc_scope();
        debug_assert_ne!(parent, child);

        self.datalog()
            .insert(
                Relations::inputs_InputScope as RelId,
                InputScope { parent, child },
            )
            .insert(Relations::inputs_EveryScope as RelId, child);

        DatalogScope {
            datalog: self.datalog(),
            scope_id: child,
        }
    }

    fn next_function_id(&self) -> FuncId {
        self.datalog().inc_function()
    }

    fn next_expr_id(&self) -> ExprId {
        self.datalog().inc_expression()
    }

    // TODO: Fully integrate global info into ddlog
    fn implicit_global(&self, global: &JsGlobal) -> GlobalId {
        let id = self.datalog().inc_global();
        self.datalog().insert(
            Relations::inputs_ImplicitGlobal as RelId,
            ImplicitGlobal {
                id,
                name: Intern::new(global.name.to_string()),
            },
        );

        id
    }

    fn decl_function(
        &self,
        id: FuncId,
        name: Option<Spanned<Name>>,
    ) -> (DatalogFunction<'ddlog>, DatalogScope<'ddlog>) {
        let body = self.scope();
        self.datalog().insert(
            Relations::inputs_Function as RelId,
            Function {
                id,
                name: name.into(),
                scope: self.scope_id(),
                body: body.scope_id(),
            },
        );

        (
            DatalogFunction {
                datalog: self.datalog(),
                func_id: id,
                scope_id: body.scope_id(),
            },
            body,
        )
    }

    fn decl_let(
        &self,
        pattern: Option<Intern<Pattern>>,
        value: Option<ExprId>,
        span: TextRange,
    ) -> (StmtId, DatalogScope<'ddlog>) {
        let scope = self.scope();
        let stmt_id = {
            let datalog = scope.datalog();
            let stmt_id = datalog.inc_statement();

            datalog
                .insert(
                    Relations::inputs_LetDecl as RelId,
                    LetDecl {
                        stmt_id,
                        pattern: pattern.into(),
                        value: value.into(),
                    },
                )
                .insert(
                    Relations::inputs_Statement as RelId,
                    Statement {
                        id: stmt_id,
                        kind: StmtKind::StmtLetDecl,
                        scope: scope.scope_id(),
                        span: span.into(),
                    },
                );

            stmt_id
        };

        (stmt_id, scope)
    }

    fn decl_const(
        &self,
        pattern: Option<Intern<Pattern>>,
        value: Option<ExprId>,
        span: TextRange,
    ) -> (StmtId, DatalogScope<'ddlog>) {
        let scope = self.scope();
        let stmt_id = {
            let datalog = scope.datalog();
            let stmt_id = datalog.inc_statement();

            datalog
                .insert(
                    Relations::inputs_ConstDecl as RelId,
                    ConstDecl {
                        stmt_id,
                        pattern: pattern.into(),
                        value: value.into(),
                    },
                )
                .insert(
                    Relations::inputs_Statement as RelId,
                    Statement {
                        id: stmt_id,
                        kind: StmtKind::StmtConstDecl,
                        scope: scope.scope_id(),
                        span: span.into(),
                    },
                );

            stmt_id
        };

        (stmt_id, scope)
    }

    fn decl_var(
        &self,
        pattern: Option<Intern<Pattern>>,
        value: Option<ExprId>,
        span: TextRange,
    ) -> (StmtId, DatalogScope<'ddlog>) {
        let scope = self.scope();
        let stmt_id = {
            let datalog = scope.datalog();
            let stmt_id = datalog.inc_statement();

            datalog
                .insert(
                    Relations::inputs_VarDecl as RelId,
                    VarDecl {
                        stmt_id,
                        pattern: pattern.into(),
                        value: value.into(),
                    },
                )
                .insert(
                    Relations::inputs_Statement as RelId,
                    Statement {
                        id: stmt_id,
                        kind: StmtKind::StmtVarDecl,
                        scope: scope.scope_id(),
                        span: span.into(),
                    },
                );

            stmt_id
        };

        (stmt_id, scope)
    }

    fn ret(&self, value: Option<ExprId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Return as RelId,
                Return {
                    stmt_id,
                    value: value.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtReturn,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn if_stmt(
        &self,
        cond: Option<ExprId>,
        if_body: Option<StmtId>,
        else_body: Option<StmtId>,
        span: TextRange,
    ) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_If as RelId,
                If {
                    stmt_id,
                    cond: cond.into(),
                    if_body: if_body.into(),
                    else_body: else_body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtIf,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn brk(&self, label: Option<Name>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Break as RelId,
                Break {
                    stmt_id,
                    label: label.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtBreak,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn do_while(&self, body: Option<StmtId>, cond: Option<ExprId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_DoWhile as RelId,
                DoWhile {
                    stmt_id,
                    body: body.into(),
                    cond: cond.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtDoWhile,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn while_stmt(&self, cond: Option<ExprId>, body: Option<StmtId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_While as RelId,
                While {
                    stmt_id,
                    cond: cond.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtWhile,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn for_stmt(
        &self,
        init: Option<ForInit>,
        test: Option<ExprId>,
        update: Option<ExprId>,
        body: Option<StmtId>,
        span: TextRange,
    ) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_For as RelId,
                For {
                    stmt_id,
                    init: init.into(),
                    test: test.into(),
                    update: update.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtFor,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn for_in(
        &self,
        elem: Option<ForInit>,
        collection: Option<ExprId>,
        body: Option<StmtId>,
        span: TextRange,
    ) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_ForIn as RelId,
                ForIn {
                    stmt_id,
                    elem: elem.into(),
                    collection: collection.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtForIn,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn cont(&self, label: Option<Name>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Continue as RelId,
                Continue {
                    stmt_id,
                    label: label.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtContinue,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn with(&self, cond: Option<ExprId>, body: Option<StmtId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_With as RelId,
                With {
                    stmt_id,
                    cond: cond.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtWith,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn label(&self, name: Option<Spanned<Name>>, body: Option<StmtId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Label as RelId,
                Label {
                    stmt_id,
                    name: name.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtLabel,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn switch(
        &self,
        test: Option<ExprId>,
        cases: Vec<(SwitchClause, Option<StmtId>)>,
        span: TextRange,
    ) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Switch as RelId,
                Switch {
                    stmt_id,
                    test: test.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtSwitch,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for (case, body) in cases {
            datalog.insert(
                Relations::inputs_SwitchCase as RelId,
                SwitchCase {
                    stmt_id,
                    case,
                    body: body.into(),
                },
            );
        }

        stmt_id
    }

    fn throw(&self, exception: Option<ExprId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Throw as RelId,
                Throw {
                    stmt_id,
                    exception: exception.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtThrow,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn try_stmt(
        &self,
        body: Option<StmtId>,
        handler: TryHandler,
        finalizer: Option<StmtId>,
        span: TextRange,
    ) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Try as RelId,
                Try {
                    stmt_id,
                    body: body.into(),
                    handler,
                    finalizer: finalizer.into(),
                },
            )
            .insert(
                Relations::inputs_Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtTry,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn debugger(&self, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog.insert(
            Relations::inputs_Statement as RelId,
            Statement {
                id: stmt_id,
                kind: StmtKind::StmtDebugger,
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        stmt_id
    }

    fn stmt_expr(&self, expr: Option<ExprId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog.insert(
            Relations::inputs_Statement as RelId,
            Statement {
                id: stmt_id,
                kind: StmtKind::StmtExpr {
                    expr_id: expr.into(),
                },
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        stmt_id
    }

    fn empty(&self, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog.insert(
            Relations::inputs_Statement as RelId,
            Statement {
                id: stmt_id,
                kind: StmtKind::StmtEmpty,
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        stmt_id
    }

    fn number(&self, number: f64, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_ExprNumber as RelId,
                ExprNumber {
                    expr_id,
                    value: number.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprLit {
                        kind: LitKind::LitNumber,
                    },
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn bigint(&self, bigint: BigInt, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_ExprNumber as RelId,
                ExprBigInt {
                    expr_id,
                    value: Int::from_bigint(bigint),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprLit {
                        kind: LitKind::LitBigInt,
                    },
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn string(&self, value: Name, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_ExprString as RelId,
                ExprString { expr_id, value },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprLit {
                        kind: LitKind::LitString,
                    },
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn null(&self, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprLit {
                    kind: LitKind::LitNull,
                },
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn boolean(&self, boolean: bool, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_ExprBool as RelId,
                ExprBool {
                    expr_id,
                    value: boolean,
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprLit {
                        kind: LitKind::LitBool,
                    },
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    // TODO: Do we need to take in the regex literal?
    fn regex(&self, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprLit {
                    kind: LitKind::LitRegex,
                },
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn name_ref(&self, value: Name, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_NameRef as RelId,
                NameRef { expr_id, value },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprNameRef,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn yield_expr(&self, value: Option<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Yield as RelId,
                Yield {
                    expr_id,
                    value: value.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprYield,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn await_expr(&self, value: Option<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Await as RelId,
                Await {
                    expr_id,
                    value: value.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprAwait,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn arrow(
        &self,
        body: Option<Either<ExprId, StmtId>>,
        params: Vec<IPattern>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Arrow as RelId,
                Arrow {
                    expr_id,
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprArrow,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for param in params {
            datalog.insert(
                Relations::inputs_ArrowParam as RelId,
                ArrowParam { expr_id, param },
            );
        }

        expr_id
    }

    fn unary(&self, op: Option<UnaryOperand>, expr: Option<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_UnaryOp as RelId,
                UnaryOp {
                    expr_id,
                    op: op.into(),
                    expr: expr.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprUnaryOp,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn bin(
        &self,
        op: Option<BinOperand>,
        lhs: Option<ExprId>,
        rhs: Option<ExprId>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_BinOp as RelId,
                BinOp {
                    expr_id,
                    op: op.into(),
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprBinOp,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn ternary(
        &self,
        test: Option<ExprId>,
        true_val: Option<ExprId>,
        false_val: Option<ExprId>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Ternary as RelId,
                Ternary {
                    expr_id,
                    test: test.into(),
                    true_val: true_val.into(),
                    false_val: false_val.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprTernary,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn this(&self, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprThis,
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn template(&self, tag: Option<ExprId>, elements: Vec<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Template as RelId,
                Template {
                    expr_id,
                    tag: tag.into(),
                    elements: elements.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprTemplate,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn array(&self, elements: Vec<ArrayElement>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Array as RelId,
                Array {
                    expr_id,
                    elements: elements.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprArray,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn object(
        &self,
        properties: Vec<(Option<PropertyKey>, PropertyVal)>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        for (key, val) in properties {
            datalog.insert(
                Relations::inputs_Property as RelId,
                Property {
                    expr_id,
                    key: key.into(),
                    val: Some(val).into(),
                },
            );
        }

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprObject,
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn grouping(&self, inner: Option<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprGrouping {
                    inner: inner.into(),
                },
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn bracket(&self, object: Option<ExprId>, prop: Option<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_BracketAccess as RelId,
                BracketAccess {
                    expr_id,
                    object: object.into(),
                    prop: prop.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprBracket,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn dot(&self, object: Option<ExprId>, prop: Option<Spanned<Name>>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_DotAccess as RelId,
                DotAccess {
                    expr_id,
                    object: object.into(),
                    prop: prop.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprDot,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn new(&self, object: Option<ExprId>, args: Option<Vec<ExprId>>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_New as RelId,
                New {
                    expr_id,
                    object: object.into(),
                    args: args.map(Into::into).into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprNew,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn call(&self, callee: Option<ExprId>, args: Option<Vec<ExprId>>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Call as RelId,
                Call {
                    expr_id,
                    callee: callee.into(),
                    args: args.map(Into::into).into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprCall,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn assign(
        &self,
        lhs: Option<Either<IPattern, ExprId>>,
        rhs: Option<ExprId>,
        op: Option<AssignOperand>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Assign as RelId,
                Assign {
                    expr_id,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                    op: op.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprAssign,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn sequence(&self, exprs: Vec<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprSequence {
                    exprs: exprs.into(),
                },
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn new_target(&self, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprNewTarget,
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn import_meta(&self, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprImportMeta,
                scope: self.scope_id(),
                span: span.into(),
            },
        );

        expr_id
    }

    fn fn_expr(
        &self,
        name: Option<Spanned<Name>>,
        params: Vec<IPattern>,
        body: Option<StmtId>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_InlineFunc as RelId,
                InlineFunc {
                    expr_id,
                    name: name.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprInlineFunc,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for param in params {
            datalog.insert(
                Relations::inputs_InlineFuncParam as RelId,
                InlineFuncParam { expr_id, param },
            );
        }

        expr_id
    }

    fn super_call(&self, args: Option<Vec<ExprId>>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprSuperCall {
                    args: args.map(Into::into).into(),
                },
                scope: self.scope_id(),
                span: span.into(),
            },
        );
        expr_id
    }

    fn import_call(&self, arg: Option<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression as RelId,
            Expression {
                id: expr_id,
                kind: ExprKind::ExprImportCall { arg: arg.into() },
                scope: self.scope_id(),
                span: span.into(),
            },
        );
        expr_id
    }

    fn class_expr(&self, elements: Option<Vec<IClassElement>>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_ClassExpr as RelId,
                ClassExpr {
                    expr_id,
                    elements: elements.map(Into::into).into(),
                },
            )
            .insert(
                Relations::inputs_Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprClass,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn class_decl(
        &self,
        name: Option<Spanned<Name>>,
        parent: Option<ExprId>,
        elements: Option<Vec<IClassElement>>,
    ) -> (ClassId, DatalogScope<'ddlog>) {
        let scope = self.scope();
        let id = {
            let datalog = self.datalog();
            let id = datalog.inc_class();

            datalog.insert(
                Relations::inputs_Class as RelId,
                Class {
                    id,
                    name: name.into(),
                    parent: parent.into(),
                    elements: elements.map(Into::into).into(),
                    scope: self.scope_id(),
                },
            );

            id
        };

        (id, scope)
    }

    fn import_decl(&self, clauses: Vec<ImportClause>) {
        let datalog = self.datalog();
        let id = datalog.inc_import();

        for clause in clauses {
            datalog.insert(
                Relations::inputs_ImportDecl as RelId,
                ImportDecl { id, clause },
            );
        }
    }
}

#[derive(Clone)]
#[must_use]
pub struct DatalogFunction<'ddlog> {
    datalog: &'ddlog DatalogInner,
    func_id: FuncId,
    scope_id: Scope,
}

impl<'ddlog> DatalogFunction<'ddlog> {
    pub fn func_id(&self) -> FuncId {
        self.func_id
    }

    pub fn argument(&self, pattern: Intern<Pattern>) {
        self.datalog.insert(
            Relations::inputs_FunctionArg as RelId,
            FunctionArg {
                parent_func: self.func_id(),
                pattern,
            },
        );
    }
}

impl<'ddlog> DatalogBuilder<'ddlog> for DatalogFunction<'ddlog> {
    fn datalog(&self) -> &'ddlog DatalogInner {
        self.datalog
    }

    fn scope_id(&self) -> Scope {
        self.scope_id
    }
}

#[derive(Clone)]
#[must_use]
pub struct DatalogScope<'ddlog> {
    datalog: &'ddlog DatalogInner,
    scope_id: Scope,
}

impl<'ddlog> DatalogScope<'ddlog> {
    pub fn scope_id(&self) -> Scope {
        self.scope_id
    }
}

impl<'ddlog> DatalogBuilder<'ddlog> for DatalogScope<'ddlog> {
    fn datalog(&self) -> &'ddlog DatalogInner {
        self.datalog
    }

    fn scope_id(&self) -> Scope {
        self.scope_id
    }
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
fn dump_delta(delta: &DeltaMap<DDValue>) {
    for (rel, changes) in delta.iter() {
        println!("Changes to relation {}", relid2name(*rel).unwrap());

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
