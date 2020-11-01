use crate::{
    datalog::{DatalogBuilder, DatalogScope},
    visit::Visit,
    AnalyzerInner,
};
use rslint_core::rule_prelude::{
    ast::{
        BreakStmt, ContinueStmt, DebuggerStmt, Decl, DoWhileStmt, FnDecl, ForHead, ForInStmt,
        ForStmt, IfStmt, LabelledStmt, ReturnStmt, Stmt, SwitchCase, SwitchStmt, ThrowStmt,
        TryStmt, VarDecl, WhileStmt, WithStmt,
    },
    AstNode, SyntaxNodeExt,
};
use types::{internment, ForInit, StmtId, SwitchClause, TryHandler};

impl<'ddlog> Visit<'ddlog, Stmt> for AnalyzerInner {
    type Output = Option<DatalogScope<'ddlog>>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, stmt: Stmt) -> Self::Output {
        match stmt {
            Stmt::BlockStmt(block) => {
                let mut scope = scope.scope();
                for stmt in block.stmts() {
                    if let Some(new_scope) = self.visit(&scope, stmt) {
                        scope = new_scope;
                    }
                }
                // TODO: How to connect blocks with the downstream nodes?
            }
            Stmt::EmptyStmt(_) => { /* Ignored, it's literally nothing */ }
            Stmt::ExprStmt(expr) => {
                expr.expr().map(|expr| self.visit(scope, expr));
            }
            Stmt::IfStmt(branch) => {
                self.visit(scope, branch);
            }
            Stmt::DoWhileStmt(do_while) => {
                self.visit(scope, do_while);
            }
            Stmt::WhileStmt(while_stmt) => {
                self.visit(scope, while_stmt);
            }
            Stmt::ForStmt(for_stmt) => {
                self.visit(scope, for_stmt);
            }
            Stmt::ForInStmt(for_in) => {
                self.visit(scope, for_in);
            }
            Stmt::ContinueStmt(cont) => {
                self.visit(scope, cont);
            }
            Stmt::BreakStmt(brk) => {
                self.visit(scope, brk);
            }
            Stmt::ReturnStmt(ret) => {
                self.visit(scope, ret);
            }
            Stmt::WithStmt(with) => {
                self.visit(scope, with);
            }
            Stmt::LabelledStmt(label) => {
                self.visit(scope, label);
            }
            Stmt::SwitchStmt(switch) => {
                self.visit(scope, switch);
            }
            Stmt::ThrowStmt(throw) => {
                self.visit(scope, throw);
            }
            Stmt::TryStmt(try_stmt) => {
                self.visit(scope, try_stmt);
            }
            Stmt::DebuggerStmt(debugger) => {
                self.visit(scope, debugger);
            }
            Stmt::Decl(decl) => return self.visit(scope, decl),
        }

        None
    }
}

impl<'ddlog> Visit<'ddlog, Decl> for AnalyzerInner {
    type Output = Option<DatalogScope<'ddlog>>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, decl: Decl) -> Self::Output {
        match decl {
            Decl::FnDecl(func) => {
                self.visit(scope, func);
                None
            }
            Decl::ClassDecl(_) => None,
            Decl::VarDecl(var) => Some(self.visit(scope, var)),
        }
    }
}

impl<'ddlog> Visit<'ddlog, FnDecl> for AnalyzerInner {
    type Output = ();

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, func: FnDecl) -> Self::Output {
        let function_id = scope.next_function_id();
        let name = func.name().map(|name| internment::intern(&name.text()));

        let function = scope.decl_function(function_id, name);

        if let Some(params) = func.parameters() {
            for param in params.parameters() {
                function.argument(self.visit_pattern(param));
            }
        }

        if let Some(body) = func.body() {
            let mut scope: Box<dyn DatalogBuilder<'_>> = Box::new(function);

            for stmt in body.stmts() {
                // Enter a new scope after each statement that requires one
                if let Some(new_scope) = self.visit(&*scope, stmt) {
                    scope = Box::new(new_scope);
                }
            }
        }
    }
}

impl<'ddlog> Visit<'ddlog, VarDecl> for AnalyzerInner {
    type Output = DatalogScope<'ddlog>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, var: VarDecl) -> Self::Output {
        let (mut last_scope, span) = (None, var.syntax().trimmed_range());

        for decl in var.declared() {
            let pattern = decl.pattern().map(|pat| self.visit_pattern(pat));
            let value = self.visit(scope, decl.value());

            last_scope = Some(if var.is_let() {
                scope.decl_let(pattern, value, span)
            } else if var.is_const() {
                scope.decl_const(pattern, value, span)
            } else if var.is_var() {
                scope.decl_var(pattern, value, span)
            } else {
                unreachable!("a variable declaration was neither `let`, `const` or `var`");
            });
        }

        last_scope.expect("at least one variable was declared, right?")
    }
}

impl<'ddlog> Visit<'ddlog, ReturnStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, ret: ReturnStmt) -> Self::Output {
        let value = ret.value().map(|val| self.visit(scope, val));
        scope.ret(value, ret.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, BreakStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, brk: BreakStmt) -> Self::Output {
        let label = brk.ident_token().map(|label| label.to_string());
        scope.brk(label, brk.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, IfStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, branch: IfStmt) -> Self::Output {
        let cond = branch
            .condition()
            .and_then(|cond| self.visit(scope, cond.condition()));

        let if_body = branch
            .cons()
            .and_then(|stmt| self.visit(scope, stmt))
            .map(|stmt| stmt.scope_id());

        let else_body = branch
            .alt()
            .and_then(|stmt| self.visit(scope, stmt))
            .map(|stmt| stmt.scope_id());

        scope.if_stmt(cond, if_body, else_body, branch.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, DoWhileStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, do_while: DoWhileStmt) -> Self::Output {
        let body = do_while
            .cons()
            .and_then(|stmt| self.visit(scope, stmt))
            .map(|stmt| stmt.scope_id());

        let cond = do_while
            .condition()
            .and_then(|cond| self.visit(scope, cond.condition()));

        scope.do_while(body, cond, do_while.syntax().text_range())
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
            .and_then(|stmt| self.visit(scope, stmt))
            .map(|stmt| stmt.scope_id());

        scope.while_stmt(cond, body, while_stmt.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, ForStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, for_stmt: ForStmt) -> Self::Output {
        let (init, init_scope) = for_stmt
            .init()
            .and_then(|init| init.inner())
            .map(|init| match init {
                ForHead::Decl(decl) => {
                    let decl_scope = self.visit(scope, decl);

                    (
                        ForInit::ForDecl {
                            stmt_id: decl_scope.scope_id(),
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
            })
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
            .and_then(|stmt| self.visit(init_scope, stmt))
            .map(|stmt| stmt.scope_id());

        // TODO: Does the scope created by the init segment need to be passed on?
        scope.for_stmt(init, test, update, body, for_stmt.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, ForInStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, for_in: ForInStmt) -> Self::Output {
        let (elem, elem_scope) = for_in
            .left()
            .and_then(|elem| elem.inner())
            .map(|elem| match elem {
                ForHead::Decl(decl) => {
                    let decl_scope = self.visit(scope, decl);

                    (
                        ForInit::ForDecl {
                            stmt_id: decl_scope.scope_id(),
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
            })
            .map_or((None, None), |(elem, scope)| (Some(elem), scope));
        let elem_scope: &dyn DatalogBuilder<'_> = elem_scope
            .as_ref()
            .map_or(scope, |s| s as &dyn DatalogBuilder<'_>);

        let collection = for_in.right().map(|coll| self.visit(elem_scope, coll));

        let body = for_in
            .cons()
            .and_then(|stmt| self.visit(elem_scope, stmt))
            .map(|stmt| stmt.scope_id());

        // TODO: Does the scope created by the elem segment need to be passed on?
        scope.for_in(elem, collection, body, for_in.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, ContinueStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, cont: ContinueStmt) -> Self::Output {
        let label = cont.ident_token().map(|label| label.to_string());
        scope.cont(label, cont.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, WithStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, with: WithStmt) -> Self::Output {
        let cond = with
            .condition()
            .and_then(|cond| self.visit(scope, cond.condition()));

        let body = with
            .cons()
            .and_then(|stmt| self.visit(scope, stmt))
            .map(|stmt| stmt.scope_id());

        scope.with(cond, body, with.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, LabelledStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, label: LabelledStmt) -> Self::Output {
        let name = label.label().map(|name| name.to_string());

        let body = label
            .stmt()
            .and_then(|stmt| self.visit(scope, stmt))
            .map(|stmt| stmt.scope_id());

        scope.label(name, body, label.syntax().text_range())
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
                        {
                            let mut scope: Box<dyn DatalogBuilder<'_>> = Box::new(scope.scope());
                            let body_id = scope.scope_id();

                            for stmt in case.cons() {
                                // Enter a new scope after each statement that requires one
                                if let Some(new_scope) = self.visit(&*scope, stmt) {
                                    scope = Box::new(new_scope);
                                }
                            }

                            body_id
                        },
                    ),

                    SwitchCase::DefaultClause(default) => {
                        (SwitchClause::DefaultClause, {
                            let mut scope: Box<dyn DatalogBuilder<'_>> = Box::new(scope.scope());
                            let body_id = scope.scope_id();

                            for stmt in default.cons() {
                                // Enter a new scope after each statement that requires one
                                if let Some(new_scope) = self.visit(&*scope, stmt) {
                                    scope = Box::new(new_scope);
                                }
                            }

                            body_id
                        })
                    }
                };

                (clause, Some(body))
            })
            .collect();

        scope.switch(test, cases, switch.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, ThrowStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, throw: ThrowStmt) -> Self::Output {
        let exception = throw.exception().map(|except| self.visit(scope, except));
        scope.throw(exception, throw.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, TryStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, try_stmt: TryStmt) -> Self::Output {
        let body = try_stmt.test().map(|stmts| {
            let mut scope: Box<dyn DatalogBuilder<'_>> = Box::new(scope.scope());
            let body_id = scope.scope_id();

            for stmt in stmts.stmts() {
                // Enter a new scope after each statement that requires one
                if let Some(new_scope) = self.visit(&*scope, stmt) {
                    scope = Box::new(new_scope);
                }
            }

            body_id
        });

        let handler = try_stmt
            .handler()
            .map(|handler| {
                let pattern = handler.error().map(|pat| self.visit_pattern(pat));
                let body = handler.cons().map(|stmts| {
                    let mut scope: Box<dyn DatalogBuilder<'_>> = Box::new(scope.scope());
                    let body_id = scope.scope_id();

                    for stmt in stmts.stmts() {
                        // Enter a new scope after each statement that requires one
                        if let Some(new_scope) = self.visit(&*scope, stmt) {
                            scope = Box::new(new_scope);
                        }
                    }

                    body_id
                });

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
            .map(|stmts| {
                let mut scope: Box<dyn DatalogBuilder<'_>> = Box::new(scope.scope());
                let body_id = scope.scope_id();

                for stmt in stmts.stmts() {
                    // Enter a new scope after each statement that requires one
                    if let Some(new_scope) = self.visit(&*scope, stmt) {
                        scope = Box::new(new_scope);
                    }
                }

                body_id
            });

        scope.try_stmt(body, handler, finalizer, try_stmt.syntax().text_range())
    }
}

impl<'ddlog> Visit<'ddlog, DebuggerStmt> for AnalyzerInner {
    type Output = StmtId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, debugger: DebuggerStmt) -> Self::Output {
        scope.debugger(debugger.syntax().text_range())
    }
}
