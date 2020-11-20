use super::{DatalogFunction, DatalogInner, DatalogScope};
use crate::globals::JsGlobal;
use differential_datalog::int::Int;
use rslint_parser::{BigInt, TextRange};
use rslint_scoping_ddlog::Relations;
use types::{
    ast::{
        ArrayElement, AssignOperand, BinOperand, ClassId, ExportKind, ExprId, ExprKind, FileId,
        ForInit, FuncId, GlobalId, GlobalPriv, IClassElement, IPattern, ImportClause, LitKind,
        Name, Pattern, PropertyKey, PropertyVal, ScopeId, Spanned, StmtId, StmtKind, SwitchClause,
        TryHandler, UnaryOperand,
    },
    ddlog_std::{tuple2, Either},
    inputs::{
        Array, Arrow, ArrowParam, Assign, Await, BinOp, BracketAccess, Break, Call, Class,
        ClassExpr, ConstDecl, Continue, DoWhile, DotAccess, EveryScope, ExprBigInt, ExprBool,
        ExprNumber, ExprString, Expression, FileExport, For, ForIn, Function, If, ImplicitGlobal,
        ImportDecl, InlineFunc, InlineFuncParam, InputScope, Label, LetDecl, NameRef, New,
        Property, Return, Statement, Switch, SwitchCase, Template, Ternary, Throw, Try, UnaryOp,
        VarDecl, While, With, Yield,
    },
    internment::Intern,
};

pub trait DatalogBuilder<'ddlog> {
    fn scope_id(&self) -> ScopeId;

    fn datalog(&self) -> &'ddlog DatalogInner;

    fn scope(&self) -> DatalogScope<'ddlog> {
        let parent = self.scope_id();
        let child = self.datalog().inc_scope();
        debug_assert_ne!(parent, child);

        self.datalog()
            .insert(
                Relations::inputs_InputScope,
                InputScope {
                    parent,
                    child,
                    file: self.file_id(),
                },
            )
            .insert(
                Relations::inputs_EveryScope,
                EveryScope {
                    scope: child,
                    file: self.file_id(),
                },
            );

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

    fn file_id(&self) -> FileId {
        self.datalog().file_id()
    }

    // TODO: Fully integrate global info into ddlog
    fn implicit_global(&self, file: FileId, global: &JsGlobal) -> GlobalId {
        let id = self.datalog().inc_global();
        self.datalog().insert(
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

    fn decl_function(
        &self,
        id: FuncId,
        name: Option<Spanned<Name>>,
        exported: bool,
    ) -> (DatalogFunction<'ddlog>, DatalogScope<'ddlog>) {
        let body = self.scope();
        self.datalog().insert(
            Relations::inputs_Function,
            Function {
                id,
                file: self.file_id(),
                name: name.into(),
                scope: self.scope_id(),
                body: body.scope_id(),
                exported,
            },
        );

        (
            DatalogFunction {
                datalog: self.datalog(),
                func_id: id,
                body_scope: body.scope_id(),
            },
            body,
        )
    }

    fn decl_let(
        &self,
        pattern: Option<Intern<Pattern>>,
        value: Option<ExprId>,
        span: TextRange,
        exported: bool,
    ) -> (StmtId, DatalogScope<'ddlog>) {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_LetDecl,
                LetDecl {
                    stmt_id,
                    file: self.file_id(),
                    pattern: pattern.into(),
                    value: value.into(),
                    exported,
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
                    kind: StmtKind::StmtLetDecl,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        (stmt_id, self.scope())
    }

    fn decl_const(
        &self,
        pattern: Option<Intern<Pattern>>,
        value: Option<ExprId>,
        span: TextRange,
        exported: bool,
    ) -> (StmtId, DatalogScope<'ddlog>) {
        let stmt_id = {
            let datalog = self.datalog();
            let stmt_id = datalog.inc_statement();

            datalog
                .insert(
                    Relations::inputs_ConstDecl,
                    ConstDecl {
                        stmt_id,
                        file: self.file_id(),
                        pattern: pattern.into(),
                        value: value.into(),
                        exported,
                    },
                )
                .insert(
                    Relations::inputs_Statement,
                    Statement {
                        id: stmt_id,
                        file: self.file_id(),
                        kind: StmtKind::StmtConstDecl,
                        scope: self.scope_id(),
                        span: span.into(),
                    },
                );

            stmt_id
        };

        (stmt_id, self.scope())
    }

    fn decl_var(
        &self,
        pattern: Option<Intern<Pattern>>,
        value: Option<ExprId>,
        span: TextRange,
        exported: bool,
    ) -> (StmtId, DatalogScope<'ddlog>) {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_VarDecl,
                VarDecl {
                    stmt_id,
                    file: self.file_id(),
                    pattern: pattern.into(),
                    value: value.into(),
                    exported,
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
                    kind: StmtKind::StmtVarDecl,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        (stmt_id, self.scope())
    }

    fn ret(&self, value: Option<ExprId>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Return,
                Return {
                    stmt_id,
                    file: self.file_id(),
                    value: value.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_If,
                If {
                    stmt_id,
                    file: self.file_id(),
                    cond: cond.into(),
                    if_body: if_body.into(),
                    else_body: else_body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
                    kind: StmtKind::StmtIf,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn brk(&self, label: Option<Spanned<Name>>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Break,
                Break {
                    stmt_id,
                    file: self.file_id(),
                    label: label.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_DoWhile,
                DoWhile {
                    stmt_id,
                    file: self.file_id(),
                    body: body.into(),
                    cond: cond.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_While,
                While {
                    stmt_id,
                    file: self.file_id(),
                    cond: cond.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_For,
                For {
                    stmt_id,
                    file: self.file_id(),
                    init: init.into(),
                    test: test.into(),
                    update: update.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_ForIn,
                ForIn {
                    stmt_id,
                    file: self.file_id(),
                    elem: elem.into(),
                    collection: collection.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
                    kind: StmtKind::StmtForIn,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn cont(&self, label: Option<Spanned<Name>>, span: TextRange) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Continue,
                Continue {
                    stmt_id,
                    file: self.file_id(),
                    label: label.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_With,
                With {
                    stmt_id,
                    file: self.file_id(),
                    cond: cond.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
                    kind: StmtKind::StmtWith,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        stmt_id
    }

    fn label(
        &self,
        name: Option<Spanned<Name>>,
        body: Option<StmtId>,
        body_scope: ScopeId,
        span: TextRange,
    ) -> StmtId {
        let datalog = self.datalog();
        let stmt_id = datalog.inc_statement();

        datalog
            .insert(
                Relations::inputs_Label,
                Label {
                    stmt_id,
                    file: self.file_id(),
                    name: name.into(),
                    body: body.into(),
                    body_scope,
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_Switch,
                Switch {
                    stmt_id,
                    file: self.file_id(),
                    test: test.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
                    kind: StmtKind::StmtSwitch,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for (case, body) in cases {
            datalog.insert(
                Relations::inputs_SwitchCase,
                SwitchCase {
                    stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_Throw,
                Throw {
                    stmt_id,
                    file: self.file_id(),
                    exception: exception.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
                Relations::inputs_Try,
                Try {
                    stmt_id,
                    file: self.file_id(),
                    body: body.into(),
                    handler,
                    finalizer: finalizer.into(),
                },
            )
            .insert(
                Relations::inputs_Statement,
                Statement {
                    id: stmt_id,
                    file: self.file_id(),
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
            Relations::inputs_Statement,
            Statement {
                id: stmt_id,
                file: self.file_id(),
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
            Relations::inputs_Statement,
            Statement {
                id: stmt_id,
                file: self.file_id(),
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
            Relations::inputs_Statement,
            Statement {
                id: stmt_id,
                file: self.file_id(),
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
                Relations::inputs_ExprNumber,
                ExprNumber {
                    expr_id,
                    file: self.file_id(),
                    value: number.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_ExprNumber,
                ExprBigInt {
                    expr_id,
                    file: self.file_id(),
                    value: Int::from_bigint(bigint),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_ExprString,
                ExprString {
                    expr_id,
                    file: self.file_id(),
                    value,
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
                Relations::inputs_ExprBool,
                ExprBool {
                    expr_id,
                    file: self.file_id(),
                    value: boolean,
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
                Relations::inputs_NameRef,
                NameRef {
                    expr_id,
                    file: self.file_id(),
                    value,
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_Yield,
                Yield {
                    expr_id,
                    file: self.file_id(),
                    value: value.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_Await,
                Await {
                    expr_id,
                    file: self.file_id(),
                    value: value.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
                    kind: ExprKind::ExprAwait,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        expr_id
    }

    fn arrow(
        &self,
        body: Option<tuple2<Either<ExprId, StmtId>, ScopeId>>,
        params: Vec<IPattern>,
        span: TextRange,
    ) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_Arrow,
                Arrow {
                    expr_id,
                    file: self.file_id(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
                    kind: ExprKind::ExprArrow,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for param in params {
            datalog.insert(
                Relations::inputs_ArrowParam,
                ArrowParam {
                    expr_id,
                    file: self.file_id(),
                    param,
                },
            );
        }

        expr_id
    }

    fn unary(&self, op: Option<UnaryOperand>, expr: Option<ExprId>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog
            .insert(
                Relations::inputs_UnaryOp,
                UnaryOp {
                    expr_id,
                    file: self.file_id(),
                    op: op.into(),
                    expr: expr.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_BinOp,
                BinOp {
                    expr_id,
                    file: self.file_id(),
                    op: op.into(),
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_Ternary,
                Ternary {
                    expr_id,
                    file: self.file_id(),
                    test: test.into(),
                    true_val: true_val.into(),
                    false_val: false_val.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
                Relations::inputs_Template,
                Template {
                    expr_id,
                    file: self.file_id(),
                    tag: tag.into(),
                    elements: elements.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_Array,
                Array {
                    expr_id,
                    file: self.file_id(),
                    elements: elements.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_Property,
                Property {
                    expr_id,
                    file: self.file_id(),
                    key: key.into(),
                    val: Some(val).into(),
                },
            );
        }

        datalog.insert(
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
                Relations::inputs_BracketAccess,
                BracketAccess {
                    expr_id,
                    file: self.file_id(),
                    object: object.into(),
                    prop: prop.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_DotAccess,
                DotAccess {
                    expr_id,
                    file: self.file_id(),
                    object: object.into(),
                    prop: prop.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_New,
                New {
                    expr_id,
                    file: self.file_id(),
                    object: object.into(),
                    args: args.map(Into::into).into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_Call,
                Call {
                    expr_id,
                    file: self.file_id(),
                    callee: callee.into(),
                    args: args.map(Into::into).into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
                Relations::inputs_Assign,
                Assign {
                    expr_id,
                    file: self.file_id(),
                    lhs: lhs.into(),
                    rhs: rhs.into(),
                    op: op.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
                Relations::inputs_InlineFunc,
                InlineFunc {
                    expr_id,
                    file: self.file_id(),
                    name: name.into(),
                    body: body.into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
                    kind: ExprKind::ExprInlineFunc,
                    scope: self.scope_id(),
                    span: span.into(),
                },
            );

        for param in params {
            datalog.insert(
                Relations::inputs_InlineFuncParam,
                InlineFuncParam {
                    expr_id,
                    file: self.file_id(),
                    param,
                },
            );
        }

        expr_id
    }

    fn super_call(&self, args: Option<Vec<ExprId>>, span: TextRange) -> ExprId {
        let datalog = self.datalog();
        let expr_id = datalog.inc_expression();

        datalog.insert(
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
            Relations::inputs_Expression,
            Expression {
                id: expr_id,
                file: self.file_id(),
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
                Relations::inputs_ClassExpr,
                ClassExpr {
                    expr_id,
                    file: self.file_id(),
                    elements: elements.map(Into::into).into(),
                },
            )
            .insert(
                Relations::inputs_Expression,
                Expression {
                    id: expr_id,
                    file: self.file_id(),
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
        exported: bool,
    ) -> (ClassId, DatalogScope<'ddlog>) {
        let scope = self.scope();
        let id = {
            let datalog = self.datalog();
            let id = datalog.inc_class();

            datalog.insert(
                Relations::inputs_Class,
                Class {
                    id,
                    file: self.file_id(),
                    name: name.into(),
                    parent: parent.into(),
                    elements: elements.map(Into::into).into(),
                    scope: self.scope_id(),
                    exported,
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
                Relations::inputs_ImportDecl,
                ImportDecl {
                    id,
                    file: self.file_id(),
                    clause,
                },
            );
        }
    }

    fn export_named(&self, name: Option<Spanned<Name>>, alias: Option<Spanned<Name>>) {
        let datalog = self.datalog();

        datalog.insert(
            Relations::inputs_FileExport,
            FileExport {
                file: self.file_id(),
                export: ExportKind::NamedExport {
                    name: name.into(),
                    alias: alias.into(),
                },
                scope: self.scope_id(),
            },
        );
    }
}
