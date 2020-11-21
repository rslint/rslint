use crate::{
    datalog::{DatalogBuilder, DatalogScope},
    AnalyzerInner, Visit,
};
use rslint_parser::{
    ast::{
        AstChildren, BlockStmt, BreakStmt, ClassDecl, ContinueStmt, DebuggerStmt, Decl,
        DoWhileStmt, FnDecl, ForHead, ForInStmt, ForOfStmt, ForStmt, IfStmt, LabelledStmt,
        ReturnStmt, Stmt, SwitchCase, SwitchStmt, ThrowStmt, TryStmt, VarDecl, WhileStmt, WithStmt,
    },
    AstNode, SyntaxNodeExt,
};
use types::{
    ast::{ClassId, ForInit, FuncId, Spanned, StmtId, SwitchClause, TryHandler},
    internment::Intern,
    IMPLICIT_ARGUMENTS,
};

// TODO: Make this more fine-grained? What things *don't* require new scopes?
impl<'ddlog> Visit<'ddlog, Stmt> for AnalyzerInner {
    type Output = (Option<StmtId>, DatalogScope<'ddlog>);

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, stmt: Stmt) -> Self::Output {
        let stmt_range = stmt.range();

        match stmt {
            Stmt::BlockStmt(block) => (Some(self.visit(scope, block)), scope.scope()),
            Stmt::EmptyStmt(empty) => (Some(scope.empty(empty.range())), scope.scope()),
            Stmt::ExprStmt(expr) => {
                let expr = expr.expr().map(|expr| self.visit(scope, expr));
                (Some(scope.stmt_expr(expr, stmt_range)), scope.scope())
            }
            Stmt::IfStmt(branch) => (Some(self.visit(scope, branch)), scope.scope()),
            Stmt::DoWhileStmt(do_while) => (Some(self.visit(scope, do_while)), scope.scope()),
            Stmt::WhileStmt(while_stmt) => (Some(self.visit(scope, while_stmt)), scope.scope()),
            Stmt::ForStmt(for_stmt) => (Some(self.visit(scope, for_stmt)), scope.scope()),
            Stmt::ForInStmt(for_in) => (Some(self.visit(scope, for_in)), scope.scope()),
            Stmt::ForOfStmt(for_of) => (Some(self.visit(scope, for_of)), scope.scope()),
            Stmt::ContinueStmt(cont) => (Some(self.visit(scope, cont)), scope.scope()),
            Stmt::BreakStmt(brk) => (Some(self.visit(scope, brk)), scope.scope()),
            Stmt::ReturnStmt(ret) => (Some(self.visit(scope, ret)), scope.scope()),
            Stmt::WithStmt(with) => (Some(self.visit(scope, with)), scope.scope()),
            Stmt::LabelledStmt(label) => (Some(self.visit(scope, label)), scope.scope()),
            Stmt::SwitchStmt(switch) => (Some(self.visit(scope, switch)), scope.scope()),
            Stmt::ThrowStmt(throw) => (Some(self.visit(scope, throw)), scope.scope()),
            Stmt::TryStmt(try_stmt) => (Some(self.visit(scope, try_stmt)), scope.scope()),
            Stmt::DebuggerStmt(debugger) => (Some(self.visit(scope, debugger)), scope.scope()),
            Stmt::Decl(decl) => self.visit(scope, (decl, false)),
        }
    }
}

impl<'ddlog> Visit<'ddlog, (Decl, bool)> for AnalyzerInner {
    type Output = (Option<StmtId>, DatalogScope<'ddlog>);

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        (decl, exported): (Decl, bool),
    ) -> Self::Output {
        match decl {
            Decl::FnDecl(func) => {
                let (_function_id, scope) = self.visit(scope, (func, exported));
                (None, scope)
            }
            Decl::ClassDecl(class) => {
                let (_class_id, scope) = self.visit(scope, (class, exported));
                (None, scope)
            }
            Decl::VarDecl(var) => self.visit(scope, (var, exported)),
        }
    }
}

impl<'ddlog> Visit<'ddlog, (FnDecl, bool)> for AnalyzerInner {
    type Output = (FuncId, DatalogScope<'ddlog>);

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        (func, exported): (FnDecl, bool),
    ) -> Self::Output {
        let s = scope.scope();
        let function_id = s.next_function_id();
        let name = self.visit(&s, func.name());

        let (function, mut body_scope) = s.decl_function(function_id, name, exported);

        // Implicitly introduce `arguments` into the function scope
        function.argument(IMPLICIT_ARGUMENTS.clone(), true);

        if let Some(params) = func.parameters() {
            for param in params.parameters() {
                function.argument(self.visit(&body_scope, param), false);
            }
        }

        if let Some(body) = func.body() {
            for stmt in body.stmts() {
                // Enter a new scope after each statement that requires one
                let (_stmt_id, new_scope) = self.visit(&body_scope, stmt);
                body_scope = new_scope;
            }
        }

        (function_id, scope.scope())
    }
}

impl<'ddlog> Visit<'ddlog, (ClassDecl, bool)> for AnalyzerInner {
    type Output = (ClassId, DatalogScope<'ddlog>);

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        (class, exported): (ClassDecl, bool),
    ) -> Self::Output {
        let name = self.visit(scope, class.name());
        let parent = self.visit(scope, class.parent());
        let elements = self.visit(scope, class.body().map(|body| body.elements()));

        scope.class_decl(name, parent, elements, exported)
    }
}

impl<'ddlog> Visit<'ddlog, (VarDecl, bool)> for AnalyzerInner {
    type Output = (Option<StmtId>, DatalogScope<'ddlog>);

    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        (var, exported): (VarDecl, bool),
    ) -> Self::Output {
        let (mut stmt_id, mut last_scope, span) =
            (None, scope.scope(), var.syntax().trimmed_range());

        for decl in var.declared() {
            let new_scope = last_scope.scope();

            let pattern = decl.pattern().map(|pat| self.visit(&new_scope, pat));
            let value = self.visit(&new_scope, decl.value());

            let (new_id, new_scope) = if var.is_let() {
                new_scope.decl_let(pattern, value, span, exported)
            } else if var.is_const() {
                new_scope.decl_const(pattern, value, span, exported)
            } else if var.is_var() {
                new_scope.decl_var(pattern, value, span, exported)
            } else {
                continue;
            };

            last_scope = new_scope;
            if stmt_id.is_none() {
                stmt_id = Some(new_id);
            }
        }

        (stmt_id, last_scope)
    }
}

impl<'ddlog> Visit<'ddlog, ReturnStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, ret: ReturnStmt) -> Self::Output {
        let value = ret.value().map(|val| self.visit(scope, val));
        scope.ret(value, ret.range())
    }
}

impl<'ddlog> Visit<'ddlog, BreakStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, brk: BreakStmt) -> Self::Output {
        let label = brk
            .name()
            .map(|name| Spanned::new(Intern::new(name.to_string()), name.syntax().trimmed_range()));

        scope.brk(label, brk.range())
    }
}

impl<'ddlog> Visit<'ddlog, IfStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, branch: IfStmt) -> Self::Output {
        let cond = branch
            .condition()
            .and_then(|cond| self.visit(scope, cond.condition()));
        let if_body = branch.cons().and_then(|stmt| self.visit(scope, stmt).0);
        let else_body = branch.alt().and_then(|stmt| self.visit(scope, stmt).0);

        scope.if_stmt(cond, if_body, else_body, branch.range())
    }
}

impl<'ddlog> Visit<'ddlog, DoWhileStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, do_while: DoWhileStmt) -> Self::Output {
        let body = do_while.cons().and_then(|stmt| self.visit(scope, stmt).0);
        let cond = do_while
            .condition()
            .and_then(|cond| self.visit(&scope.scope(), cond.condition()));

        scope.do_while(body, cond, do_while.range())
    }
}

impl<'ddlog> Visit<'ddlog, WhileStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, while_stmt: WhileStmt) -> Self::Output {
        let cond = while_stmt
            .condition()
            .and_then(|cond| self.visit(scope, cond.condition()));
        let body = while_stmt
            .cons()
            .and_then(|stmt| self.visit(&scope.scope(), stmt).0);

        scope.while_stmt(cond, body, while_stmt.range())
    }
}

impl<'ddlog> Visit<'ddlog, ForStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, for_stmt: ForStmt) -> Self::Output {
        let (init, init_scope) = for_stmt
            .init()
            .and_then(|init| init.inner())
            .map(|init| self.visit(scope, init))
            .map_or((None, None), |(init, scope)| (Some(init), scope));
        let init_scope: &dyn DatalogBuilder<'_> = init_scope
            .as_ref()
            .map_or(scope, |s| s as &dyn DatalogBuilder<'_>);

        let test = for_stmt
            .test()
            .and_then(|test| self.visit(init_scope, test.expr()));

        let update = for_stmt
            .update()
            .and_then(|update| self.visit(init_scope, update.expr()));

        let body = for_stmt
            .cons()
            .and_then(|stmt| self.visit(init_scope, stmt).0);

        // TODO: Does the scope created by the init segment need to be passed on?
        scope.for_stmt(init, test, update, body, for_stmt.range())
    }
}

impl<'ddlog> Visit<'ddlog, ForInStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, for_in: ForInStmt) -> Self::Output {
        let (elem, elem_scope) = for_in
            .left()
            .and_then(|elem| elem.inner())
            .map(|elem| self.visit(scope, elem))
            .map_or((None, None), |(elem, scope)| (Some(elem), scope));
        let elem_scope: &dyn DatalogBuilder<'_> = elem_scope
            .as_ref()
            .map_or(scope, |s| s as &dyn DatalogBuilder<'_>);

        let collection = for_in.right().map(|coll| self.visit(elem_scope, coll));
        let body = for_in
            .cons()
            .and_then(|stmt| self.visit(elem_scope, stmt).0);

        // TODO: Does the scope created by the elem segment need to be passed on?
        scope.for_in(elem, collection, body, for_in.range())
    }
}

impl<'ddlog> Visit<'ddlog, ForOfStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, for_of: ForOfStmt) -> Self::Output {
        let awaited = for_of.await_token().is_some();
        let (elem, elem_scope) = for_of
            .left()
            .and_then(|elem| elem.inner())
            .map(|elem| self.visit(scope, elem))
            .map_or((None, None), |(elem, scope)| (Some(elem), scope));
        let elem_scope: &dyn DatalogBuilder<'_> = elem_scope
            .as_ref()
            .map_or(scope, |s| s as &dyn DatalogBuilder<'_>);

        let collection = for_of.right().map(|coll| self.visit(elem_scope, coll));
        let body = for_of
            .cons()
            .and_then(|stmt| self.visit(elem_scope, stmt).0);

        // TODO: Does the scope created by the elem segment need to be passed on?
        scope.for_of(awaited, elem, collection, body, for_of.range())
    }
}

impl<'ddlog> Visit<'ddlog, ContinueStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, cont: ContinueStmt) -> Self::Output {
        let label = cont
            .name()
            .map(|name| Spanned::new(Intern::new(name.to_string()), name.syntax().trimmed_range()));

        scope.cont(label, cont.range())
    }
}

impl<'ddlog> Visit<'ddlog, WithStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, with: WithStmt) -> Self::Output {
        let cond = with
            .condition()
            .and_then(|cond| self.visit(scope, cond.condition()));
        let body = with.cons().and_then(|stmt| self.visit(scope, stmt).0);

        scope.with(cond, body, with.range())
    }
}

impl<'ddlog> Visit<'ddlog, LabelledStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, label: LabelledStmt) -> Self::Output {
        let name = self.visit(scope, label.label());
        let body_scope = scope.scope();
        let body = label.stmt().map(|stmt| {
            let range = stmt.range();
            self.visit(&body_scope, stmt)
                .0
                .unwrap_or_else(|| body_scope.empty(range))
        });

        scope.label(name, body, body_scope.scope_id(), label.range())
    }
}

impl<'ddlog> Visit<'ddlog, SwitchStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, switch: SwitchStmt) -> Self::Output {
        let test = switch
            .test()
            .and_then(|test| self.visit(scope, test.condition()));

        let cases = switch
            .cases()
            .map(|case| {
                let (clause, body) = match case {
                    SwitchCase::CaseClause(case) => (
                        SwitchClause::CaseClause {
                            test: self.visit(scope, case.test()).into(),
                        },
                        self.visit(scope, case.cons()),
                    ),

                    SwitchCase::DefaultClause(default) => (
                        SwitchClause::DefaultClause,
                        self.visit(scope, default.cons()),
                    ),
                };

                (clause, body)
            })
            .collect();

        scope.switch(test, cases, switch.range())
    }
}

impl<'ddlog> Visit<'ddlog, ThrowStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, throw: ThrowStmt) -> Self::Output {
        let exception = throw.exception().map(|except| self.visit(scope, except));
        scope.throw(exception, throw.range())
    }
}

impl<'ddlog> Visit<'ddlog, TryStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, try_stmt: TryStmt) -> Self::Output {
        let body = try_stmt.test().map(|block| self.visit(scope, block));

        let handler = try_stmt
            .handler()
            .map(|handler| {
                let pattern = handler.error().map(|pat| self.visit(scope, pat));
                let body = handler.cons().map(|handler| self.visit(scope, handler));

                (pattern.into(), body.into())
            })
            .map_or(
                TryHandler {
                    error: None.into(),
                    body: None.into(),
                },
                |(error, body)| TryHandler { error, body },
            );

        let finalizer = try_stmt
            .finalizer()
            .and_then(|finalizer| finalizer.cons())
            .map(|finalizer| self.visit(scope, finalizer));

        scope.try_stmt(body, handler, finalizer, try_stmt.range())
    }
}

impl<'ddlog> Visit<'ddlog, DebuggerStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, debugger: DebuggerStmt) -> Self::Output {
        scope.debugger(debugger.range())
    }
}

impl<'ddlog> Visit<'ddlog, BlockStmt> for AnalyzerInner {
    type Output = StmtId;

    // TODO: Should blocks get their own statement type along with the scope's span?
    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, block: BlockStmt) -> Self::Output {
        let scope = scope.scope();
        self.visit(&scope, block.stmts())
            .unwrap_or_else(|| scope.empty(block.range()))
    }
}

impl<'ddlog> Visit<'ddlog, AstChildren<Stmt>> for AnalyzerInner {
    type Output = Option<StmtId>;

    // TODO: Should children get their own statement type along with the scope's span?
    fn visit(
        &self,
        scope: &dyn DatalogBuilder<'ddlog>,
        children: AstChildren<Stmt>,
    ) -> Self::Output {
        let (mut stmt_id, mut scope) = (None, scope.scope());

        for stmt in children {
            let (new_id, new_scope) = self.visit(&scope, stmt);

            // Enter a new scope after any statements that create a new one
            scope = new_scope;

            // Get the id of the first statement so we can return it for the entire block
            if let Some(new_id) = new_id {
                if stmt_id.is_none() {
                    stmt_id = Some(new_id);
                }
            }
        }

        stmt_id
    }
}

impl<'ddlog> Visit<'ddlog, ForHead> for AnalyzerInner {
    type Output = (ForInit, Option<DatalogScope<'ddlog>>);

    // TODO: Should blocks get their own statement type along with the scope's span?
    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, head: ForHead) -> Self::Output {
        match head {
            ForHead::Decl(decl) => {
                let (stmt_id, decl_scope) = self.visit(scope, (decl, false));

                (
                    ForInit::ForDecl {
                        stmt_id: stmt_id.into(),
                    },
                    Some(decl_scope),
                )
            }

            ForHead::Expr(expr) => (
                ForInit::ForExpr {
                    expr_id: self.visit(scope, expr),
                },
                None,
            ),
        }
    }
}
