use crate::rule_prelude::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow getter properties which do not always return a value.

    Getters are special properties introduced in ES5 which call a function when a property is accessed.
    The value returned will be the value returned for the property access:

    ```ignore
    let obj = {
        // Using object literal syntax
        get foo() {
            return 5;
        }
    }

    // Using the defineProperty function
    Object.defineProperty(obj, "foo", {
        get: function() {
            return 5;
        }
    })
    ```

    Getters are expected to return a value, it is a bad practice to use getters to run some function
    without a return. This rule makes sure that does not happen and enforces a getter always returns a value.

    ## Incorrect code examples

    ```ignore
    // The getter does not always return a value, it would not return anything
    // if bar is falsey
    let obj = {
        get foo() {
            if (bar) {
                return foo;
            }
        }
    }
    ```

    ## Correct code examples

    ```ignore
    // The getter always returns a value
    let obj = {
        get foo() {
            if (bar) {
                return foo;
            } else {
                return bar;
            }
        }
    }
    ```
    */
    #[derive(Default)]
    GetterReturn,
    errors,
    "getter-return",
    /// Whether to allow implicitly returning undefined with `return;`.
    /// `true` by default.
    pub allow_implicit: bool
}

#[typetag::serde]
impl CstRule for GetterReturn {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        match node.kind() {
            CALL_EXPR => {
                let expr = node.to::<ast::CallExpr>();
                let property_definition = expr.callee().map_or(false, |e| {
                    e.syntax()
                        .structural_lossy_token_eq(&["Object", ".", "defineProperty"])
                });
                if property_definition && expr.arguments()?.args().count() == 3 {
                    let args: Vec<ast::Expr> = expr.arguments().unwrap().args().collect();
                    if let Some(obj) = args
                        .get(2)
                        .and_then(|expr| expr.syntax().try_to::<ast::ObjectExpr>())
                    {
                        for prop in obj.props() {
                            if let ast::ObjectProp::LiteralProp(literal_prop) = prop {
                                if literal_prop.key()?.syntax().text() != "get" {
                                    continue;
                                }
                                match literal_prop.value()? {
                                    ast::Expr::FnExpr(decl) => {
                                        self.check_stmts(
                                            args[1].syntax(),
                                            decl.body()?.syntax(),
                                            decl.body()?.stmts(),
                                            ctx,
                                        );
                                    }
                                    ast::Expr::ArrowExpr(arrow) => {
                                        if let ast::ExprOrBlock::Block(block) = arrow.body()? {
                                            self.check_stmts(
                                                args[1].syntax(),
                                                block.syntax(),
                                                block.stmts(),
                                                ctx,
                                            );
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            GETTER => {
                let getter = node.to::<ast::Getter>();
                if let Some(body) = getter.body() {
                    if let Some(key) = getter.key() {
                        self.check_stmts(key.syntax(), body.syntax(), body.stmts(), ctx);
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl GetterReturn {
    fn check_stmts(
        &self,
        key: &SyntaxNode,
        body: &SyntaxNode,
        mut stmts: impl Iterator<Item = ast::Stmt>,
        ctx: &mut RuleCtx,
    ) {
        if !stmts.any(|stmt| self.check_stmt(&stmt)) {
            let err = ctx
                .err(
                    self.name(),
                    format!(
                        "Getter properties must always return a value, but `{}` does not.",
                        key.trimmed_text()
                    ),
                )
                .secondary(
                    key.trimmed_range(),
                    "this key is sometimes or always undefined...",
                )
                .primary(
                    body.trimmed_range(),
                    "...because this getter does not always return a value",
                );

            ctx.add_err(err);
        }
    }

    fn check_stmt(&self, stmt: &ast::Stmt) -> bool {
        match stmt {
            ast::Stmt::IfStmt(if_stmt) => self.check_if(if_stmt),
            ast::Stmt::BlockStmt(block) => block.stmts().any(|stmt| self.check_stmt(&stmt)),
            ast::Stmt::ReturnStmt(stmt) => stmt.value().is_some() || self.allow_implicit,
            ast::Stmt::SwitchStmt(switch) => switch.cases().any(|case| match case {
                ast::SwitchCase::CaseClause(clause) => clause.cons().any(|s| self.check_stmt(&s)),
                ast::SwitchCase::DefaultClause(clause) => {
                    clause.cons().any(|s| self.check_stmt(&s))
                }
            }),
            _ => false,
        }
    }

    /// Check if an if statement unconditionally returns from the statement.
    fn check_if(&self, stmt: &ast::IfStmt) -> bool {
        if stmt.alt().is_none() {
            return false;
        }

        if let Some(cons) = stmt.cons() {
            if !self.check_stmt(&cons) {
                return false;
            }
            return self.check_stmt(&stmt.alt().unwrap());
        }
        false
    }
}

rule_tests! {
    GetterReturn::default(),
    err: {
        "
        let foo = {
            get bar() {
                
            }
        }
        ",
        "
        let bar = {
            get foo() {
                if (bar) {
                    return bar;
                }
            }
        }
        ",
        "
        let bar = {
            get foo() {
                switch (bar) {
                    case 5:
                    case 6:
                    if (bar) {
                        return 5;
                    }
                }
            }
        }
        ",
        "
        let bar = {
            get foo() {
                if (bar) {

                } else {
                    return foo;
                }
            }
        }
        "
    },
    ok: {
        "
        let bar = {
            get foo() {
                return bar;
            }
        }
        ",
        "
        let bar = {
            get foo() {
                if(bar) {
                    if (bar) {
                        return foo;
                    } else {
                        return 6;
                    }
                } else {
                    return 7;
                }
            }
        }
        "
    }
}
