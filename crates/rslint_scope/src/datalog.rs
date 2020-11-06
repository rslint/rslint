use crate::globals::JsGlobal;
use differential_datalog::{
    ddval::{DDValConvert, DDValue},
    int::Int,
    program::{IdxId, RelId, Update},
    record::Record,
    DDlog, DeltaMap,
};
use rslint_parser::{BigInt, TextRange};
use rslint_scoping_ddlog::{api::HDDlog, relid2name, Indexes, Relations};
use std::{
    cell::{Cell, RefCell},
    mem,
    sync::{Arc, Mutex},
};
use types::{ddlog_std::Either, internment::Intern, *};

// TODO: Work on the internment situation, I don't like
//       having to allocate strings for idents

pub type DatalogResult<T> = Result<T, String>;

#[derive(Debug, Clone)]
pub struct DerivedFacts {
    pub invalid_name_uses: Vec<InvalidNameUse>,
    pub var_use_before_decl: Vec<VarUseBeforeDeclaration>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Weight {
    Insert,
    Delete,
}

impl From<isize> for Weight {
    fn from(weight: isize) -> Self {
        match weight {
            1 => Self::Insert,
            -1 => Self::Delete,

            invalid => unreachable!("invalid weight given: {}", invalid),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Datalog {
    datalog: Arc<Mutex<DatalogInner>>,
}

impl Datalog {
    pub fn new() -> DatalogResult<Self> {
        let (hddlog, init_state) = HDDlog::run(2, false, |_: usize, _: &Record, _: isize| {})?;
        let this = Self {
            datalog: Arc::new(Mutex::new(DatalogInner::new(hddlog))),
        };
        this.update(init_state);

        Ok(this)
    }

    // TODO: Make this take an iterator
    pub fn inject_globals(&self, globals: &[JsGlobal]) -> DatalogResult<DerivedFacts> {
        self.transaction(|trans| {
            for global in globals {
                trans.implicit_global(global);
            }

            Ok(())
        })
    }

    pub fn clear_globals(&self) -> DatalogResult<DerivedFacts> {
        self.transaction(|trans| {
            trans
                .datalog
                .hddlog
                .clear_relation(Relations::ImplicitGlobal as RelId)?;

            Ok(())
        })
    }

    pub fn variables_for_scope(&self, scope: Scope) -> DatalogResult<Vec<Name>> {
        let mut vars = Vec::new();

        self.transaction(|trans| {
            vars.extend(
                trans
                    .datalog
                    .hddlog
                    .query_index(
                        Indexes::Index_VariablesForScope as IdxId,
                        scope.into_ddvalue(),
                    )?
                    .into_iter()
                    .map(|name| unsafe { NameInScope::from_ddvalue(name).name }),
            );

            Ok(())
        })?;

        Ok(vars)
    }

    // Note: Ddlog only allows one concurrent transaction, so all calls to this function
    //       will block until the previous completes
    // TODO: We can actually add to the transaction batch concurrently, but transactions
    //       themselves have to be synchronized in some fashion (barrier?)
    pub fn transaction<F>(&self, transaction: F) -> DatalogResult<DerivedFacts>
    where
        F: for<'trans> FnOnce(&mut DatalogTransaction<'trans>) -> DatalogResult<()>,
    {
        let delta = {
            let datalog = self
                .datalog
                .lock()
                .expect("failed to lock datalog for transaction");

            let mut trans = DatalogTransaction::new(&*datalog)?;
            transaction(&mut trans)?;
            trans.commit()?
        };

        Ok(self.update(delta))
    }

    fn update(&self, mut delta: DeltaMap<DDValue>) -> DerivedFacts {
        macro_rules! drain_relations {
            ($($relation:ident->$field:ident),* $(,)?) => {
                $(
                    let relation = delta.clear_rel(Relations::$relation as RelId);
                    let mut $field = Vec::with_capacity(relation.len());
                    for (usage, weight) in relation.into_iter() {
                        match Weight::from(weight) {
                            Weight::Insert => {
                                // Safety: This is the correct type since we pulled it from
                                //         the correct relation
                                $field.push(unsafe { $relation::from_ddvalue(usage) });
                            }
                            Weight::Delete => {}
                        }
                    }
                )*

                DerivedFacts { $( $field, )* }
            };
        }

        drain_relations! {
            InvalidNameUse->invalid_name_uses,
            VarUseBeforeDeclaration->var_use_before_decl,
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
                Relations::EveryScope as RelId,
                InputScope {
                    parent: scope_id,
                    child: scope_id,
                },
            )
            .insert(Relations::InputScope as RelId, scope_id);

        DatalogScope {
            datalog: self.datalog,
            scope_id,
        }
    }

    // TODO: Fully integrate global info into ddlog
    fn implicit_global(&self, global: &JsGlobal) -> GlobalId {
        let id = self.datalog.inc_global();
        self.datalog.insert(
            Relations::ImplicitGlobal as RelId,
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

        #[cfg(debug_assertions)]
        {
            println!("== start transaction ==");
            dump_delta(&delta);
            println!("==  end transaction  ==\n\n");
        }

        Ok(delta)
    }
}

pub trait DatalogBuilder<'ddlog> {
    fn scope_id(&self) -> Scope;

    fn datalog(&self) -> &'ddlog DatalogInner;

    fn scope(&self) -> DatalogScope<'ddlog> {
        let parent = self.datalog().scope_id.get();
        let scope_id = self.datalog().inc_scope();
        self.datalog()
            .insert(
                Relations::EveryScope as RelId,
                InputScope {
                    parent,
                    child: scope_id,
                },
            )
            .insert(Relations::InputScope as RelId, scope_id);

        DatalogScope {
            datalog: self.datalog(),
            scope_id,
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
            Relations::ImplicitGlobal as RelId,
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
        name: Option<Name>,
    ) -> (DatalogFunction<'ddlog>, DatalogScope<'ddlog>) {
        let body = self.scope();
        self.datalog().insert(
            Relations::Function as RelId,
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
                    Relations::LetDecl as RelId,
                    LetDecl {
                        stmt_id,
                        pattern: pattern.into(),
                        value: value.into(),
                    },
                )
                .insert(
                    Relations::Statement as RelId,
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
                    Relations::ConstDecl as RelId,
                    ConstDecl {
                        stmt_id,
                        pattern: pattern.into(),
                        value: value.into(),
                    },
                )
                .insert(
                    Relations::Statement as RelId,
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
                    Relations::VarDecl as RelId,
                    VarDecl {
                        stmt_id,
                        pattern: pattern.into(),
                        value: value.into(),
                    },
                )
                .insert(
                    Relations::Statement as RelId,
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
                Relations::Return as RelId,
                Return {
                    stmt_id,
                    value: value.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::If as RelId,
                If {
                    stmt_id,
                    cond: cond.into(),
                    if_body: if_body.into(),
                    else_body: else_body.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtIf,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn brk(&self, label: Option<String>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::Break as RelId,
                Break {
                    stmt_id,
                    label: label.as_ref().map(internment::intern).into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::DoWhile as RelId,
                DoWhile {
                    stmt_id,
                    body: body.into(),
                    cond: cond.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::While as RelId,
                DoWhile {
                    stmt_id,
                    cond: cond.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::For as RelId,
                For {
                    stmt_id,
                    init: init.into(),
                    test: test.into(),
                    update: update.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::ForIn as RelId,
                ForIn {
                    stmt_id,
                    elem: elem.into(),
                    collection: collection.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtForIn,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn cont(&self, label: Option<String>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::Continue as RelId,
                Continue {
                    stmt_id,
                    label: label.as_ref().map(internment::intern).into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::With as RelId,
                With {
                    stmt_id,
                    cond: cond.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtWith,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn label(&self, name: Option<String>, body: Option<StmtId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::Label as RelId,
                Label {
                    stmt_id,
                    name: name.as_ref().map(internment::intern).into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::Switch as RelId,
                Switch {
                    stmt_id,
                    test: test.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
                Statement {
                    id: stmt_id,
                    kind: StmtKind::StmtSwitch,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for (case, body) in cases {
            datalog.insert(
                Relations::SwitchCase as RelId,
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
                Relations::Throw as RelId,
                Throw {
                    stmt_id,
                    exception: exception.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
                Relations::Try as RelId,
                Try {
                    stmt_id,
                    body: body.into(),
                    handler,
                    finalizer: finalizer.into(),
                },
            )
            .insert(
                Relations::Statement as RelId,
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
            Relations::Statement as RelId,
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
            Relations::Statement as RelId,
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
            Relations::Statement as RelId,
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
                Relations::ExprNumber as RelId,
                ExprNumber {
                    expr_id,
                    value: number.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::ExprNumber as RelId,
                ExprBigInt {
                    expr_id,
                    value: Int::from_bigint(bigint),
                },
            )
            .insert(
                Relations::Expression as RelId,
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

    fn string(&self, string: String, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::ExprString as RelId,
                ExprString {
                    expr_id,
                    value: internment::intern(&string),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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
                Relations::ExprBool as RelId,
                ExprBool {
                    expr_id,
                    value: boolean,
                },
            )
            .insert(
                Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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

    fn name_ref(&self, name: String, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::NameRef as RelId,
                NameRef {
                    expr_id,
                    value: internment::intern(&name),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Yield as RelId,
                Yield {
                    expr_id,
                    value: value.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Await as RelId,
                Await {
                    expr_id,
                    value: value.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Arrow as RelId,
                Arrow {
                    expr_id,
                    body: body.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprArrow,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for param in params {
            datalog.insert(
                Relations::ArrowParam as RelId,
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
                Relations::UnaryOp as RelId,
                UnaryOp {
                    expr_id,
                    op: op.into(),
                    expr: expr.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::BinOp as RelId,
                BinOp {
                    expr_id,
                    op: op.into(),
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Ternary as RelId,
                Ternary {
                    expr_id,
                    test: test.into(),
                    true_val: true_val.into(),
                    false_val: false_val.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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
                Relations::Template as RelId,
                Template {
                    expr_id,
                    tag: tag.into(),
                    elements: elements.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Array as RelId,
                Array {
                    expr_id,
                    elements: elements.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Property as RelId,
                Property {
                    expr_id,
                    key: key.into(),
                    val: Some(val).into(),
                },
            );
        }

        datalog.insert(
            Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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
                Relations::BracketAccess as RelId,
                BracketAccess {
                    expr_id,
                    object: object.into(),
                    prop: prop.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprBracket,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn dot(&self, object: Option<ExprId>, prop: Option<Name>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::DotAccess as RelId,
                DotAccess {
                    expr_id,
                    object: object.into(),
                    prop: prop.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::New as RelId,
                New {
                    expr_id,
                    object: object.into(),
                    args: args.map(Into::into).into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Call as RelId,
                Call {
                    expr_id,
                    callee: callee.into(),
                    args: args.map(Into::into).into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
                Relations::Assign as RelId,
                Assign {
                    expr_id,
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                    op: op.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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
        name: Option<Name>,
        params: Vec<IPattern>,
        body: Option<StmtId>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::InlineFunc as RelId,
                InlineFunc {
                    expr_id,
                    name: name.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
                Expression {
                    id: expr_id,
                    kind: ExprKind::ExprInlineFunc,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for param in params {
            datalog.insert(
                Relations::InlineFuncParam as RelId,
                InlineFuncParam { expr_id, param },
            );
        }

        expr_id
    }

    fn super_call(&self, args: Option<Vec<ExprId>>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::Expression as RelId,
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
            Relations::Expression as RelId,
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
                Relations::ClassExpr as RelId,
                ClassExpr {
                    expr_id,
                    elements: elements.map(Into::into).into(),
                },
            )
            .insert(
                Relations::Expression as RelId,
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
        name: Option<Name>,
        parent: Option<ExprId>,
        elements: Option<Vec<IClassElement>>,
    ) -> (ClassId, DatalogScope<'ddlog>) {
        let scope = self.scope();
        let id = {
            let datalog = self.datalog();
            let id = datalog.inc_class();

            datalog.insert(
                Relations::Class as RelId,
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
}

#[derive(Clone)]
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
            Relations::FunctionArg as RelId,
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
            println!(">> {} {:+}", val, weight);
        }

        if !changes.is_empty() {
            println!();
        }
    }
}
