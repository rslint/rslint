use crate::rule_prelude::*;
use ast::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow setters to return values.

    Setters cannot return values. To be more precise, a setter that returns a value is not treated as an error, but we
    cannot use the returned value at all. Thus, if you write a setter that will return something, it is most likely
    either completely unnecessary or a possible error.

    Note that `return` without a value is allowed because it is considered a control flow statement.

    This rule checks setters in:

    - Object literals
    - Class declarations and class expressions
    - Property descriptors in `Object.create`, `Object.defineProperty`, `Object.defineProperties`, and `Reflect.defineProperty`

    ## Incorrect code examples

    ```js
    let foo = {
        set a(value) {
            this.val = value;
            // The setter always returns a value
            return value;
        }
    };

    class Foo {
        set a(value) {
            this.val = value;
            // The setter always returns a value
            return this.val;
        }
    }

    const Bar = class {
        static set a(value) {
            if (value < 0) {
                this.val = 0;
                // The setter returns `0` if the value is negative
                return 0;
            }
            this.val = value;
        }
    };

    Object.defineProperty(foo, "bar", {
        set(value) {
            if (value < 0) {
                // The setter returns `false` if the value is negative
                return false;
            }
            this.val = value;
        }
    });
    ```

    ## Correct code examples

    ```js
    let foo = {
        set a(value) {
            this.val = value;
        }
    };

    class Foo {
        set a(value) {
            this.val = value;
        }
    }

    const Bar = class {
        static set a(value) {
            if (value < 0) {
                this.val = 0;
                // Returning without a value is allowed
                return;
            }
            this.val = value;
        }
    };

    Object.defineProperty(foo, "bar", {
        set(value) {
            if (value < 0) {
                // Throwing an error is also allowed
                throw new Error("Negative value is not allowed.");
            }
            this.val = value;
        }
    });
    ```
    */
    #[derive(Default)]
    NoSetterReturn,
    errors,
    tags(Recommended),
    "no-setter-return",
}

/// Check if the expr is either
/// - `Object.defineProperty`
/// - `Object.defineProperties`
/// - `Reflect.defineProperties`
/// and receives three arguments.
fn is_define_property(expr: &CallExpr) -> bool {
    expr.callee().map_or(false, |e| {
        [
            ["Object", ".", "defineProperty"],
            ["Object", ".", "defineProperties"],
            ["Reflect", ".", "defineProperties"],
        ]
        .iter()
        .any(|toks| e.syntax().structural_lossy_token_eq(toks))
    }) && expr.arguments().map_or(false, |a| a.args().count() == 3)
}

/// Check if the expr is `Object.create` and receives two arguments.
fn is_object_create(expr: &CallExpr) -> bool {
    expr.callee().map_or(false, |e| {
        e.syntax()
            .structural_lossy_token_eq(&["Object", ".", "create"])
    }) && expr.arguments().map_or(false, |a| a.args().count() == 2)
}

#[typetag::serde]
impl CstRule for NoSetterReturn {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        match node.kind() {
            CALL_EXPR => {
                let expr = node.to::<CallExpr>();
                if is_define_property(&expr) {
                    let args: Vec<Expr> = expr.arguments().unwrap().args().collect();
                    if let Some(obj) = args
                        .get(2)
                        .and_then(|expr| expr.syntax().try_to::<ObjectExpr>())
                    {
                        for prop in obj.props() {
                            self.check_object_props(args[1].syntax(), &prop, ctx);
                        }
                    }
                } else if is_object_create(&expr) {
                    let args: Vec<Expr> = expr.arguments().unwrap().args().collect();
                    if let Some(obj) = args
                        .get(1)
                        .and_then(|expr| expr.syntax().try_to::<ObjectExpr>())
                    {
                        for prop in obj.props() {
                            if let ObjectProp::LiteralProp(literal_prop) = prop {
                                if let Some(Expr::ObjectExpr(inner_obj)) = literal_prop.value() {
                                    for inner_prop in inner_obj.props() {
                                        self.check_object_props(
                                            literal_prop.syntax(),
                                            &inner_prop,
                                            ctx,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
            SETTER => {
                let setter = node.to::<Setter>();
                if let Some(body) = setter.body() {
                    if let Some(key) = setter.key() {
                        self.check_stmts(key.syntax(), body.syntax(), body.stmts(), ctx);
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl NoSetterReturn {
    fn check_object_props(
        &self,
        key: &SyntaxNode,
        prop: &ObjectProp,
        ctx: &mut RuleCtx,
    ) -> Option<()> {
        match prop {
            ObjectProp::LiteralProp(literal_prop) => {
                if literal_prop.key()?.text() != "set" {
                    return None;
                }
                match literal_prop.value()? {
                    Expr::FnExpr(decl) => {
                        self.check_stmts(key, decl.body()?.syntax(), decl.body()?.stmts(), ctx);
                    }
                    Expr::ArrowExpr(arrow) => {
                        if let ExprOrBlock::Block(block) = arrow.body()? {
                            self.check_stmts(key, block.syntax(), block.stmts(), ctx);
                        }
                    }
                    _ => {}
                }
            }
            ObjectProp::Setter(setter) => {
                self.check_stmts(key, setter.body()?.syntax(), setter.body()?.stmts(), ctx);
            }
            ObjectProp::Method(method) => {
                if method.name()?.text() != "set" {
                    return None;
                }
                self.check_stmts(key, method.body()?.syntax(), method.body()?.stmts(), ctx);
            }
            _ => {}
        }

        None
    }

    fn check_stmts(
        &self,
        key: &SyntaxNode,
        body: &SyntaxNode,
        mut stmts: impl Iterator<Item = Stmt>,
        ctx: &mut RuleCtx,
    ) {
        if stmts.any(|stmt| self.stmt_returns_value(&stmt)) {
            let err = ctx
                .err(
                    self.name(),
                    format!(
                        "setter properties are not allowed to return values, but `{}` does.",
                        key.trimmed_text(),
                    ),
                )
                .primary(body, "this setter somethimes or always returns a value");
            ctx.add_err(err);
        }
    }

    /// Return `true` if a stmt returns a value.
    fn stmt_returns_value(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::IfStmt(if_stmt) => {
                let cons_result = if_stmt
                    .cons()
                    .map_or(false, |cons| self.stmt_returns_value(&cons));
                let alt_result = if_stmt
                    .alt()
                    .map_or(false, |alt| self.stmt_returns_value(&alt));
                cons_result || alt_result
            }
            Stmt::BlockStmt(block) => block.stmts().any(|stmt| self.stmt_returns_value(&stmt)),
            Stmt::ReturnStmt(stmt) => stmt.value().is_some(),
            Stmt::SwitchStmt(switch) => switch.cases().any(|case| match case {
                SwitchCase::CaseClause(clause) => {
                    clause.cons().any(|s| self.stmt_returns_value(&s))
                }
                SwitchCase::DefaultClause(clause) => {
                    clause.cons().any(|s| self.stmt_returns_value(&s))
                }
            }),
            _ => false,
        }
    }
}

rule_tests! {
    NoSetterReturn::default(),
    err: {
        "
        let foo = {
            set bar(val) {
                return 42;
            }
        };
        ",
        "
        let bar = {
            set foo(val) {
                if (bar) {
                    return 42;
                }
            }
        };
        ",
        "
        let bar = {
            set foo(val) {
                switch (bar) {
                    case 5:
                    case 6:
                    if (bar) {
                        return 42;
                    }
                }
            }
        };
        ",
        "
        let bar = {
            set foo(val) {
                if (bar) {

                } else {
                    return 42;
                }
            }
        };
        ",
        "
        class Foo {
            set bar(val) {
                return 42;
            }
        }
        ",
        "
        let Foo = class {
            set bar(val) {
                return 42;
            }
        };
        ",
        "
        Object.create(null, {
            foo: {
                set(val) {
                    return 42;
                }
            }
        });
        ",
        "
        Object.defineProperty(foo, 'bar', {
            set(val) {
                return 42;
            }
        });
        ",
        "
        Object.defineProperties(foo, 'bar', {
            set(val) {
                return 42;
            }
        });
        ",
        "
        Reflect.defineProperties(foo, 'bar', {
            set(val) {
                return 42;
            }
        });
        ",
    },
    ok: {
        "({ set foo(val) { return; } })",
        "({ set foo(val) { if (val) { return; } } })",
        "class A { set foo(val) { return; } }",
        "(class { set foo(val) { if (val) { return; } else { return; } return; } })",
        "class A { set foo(val) { try {} catch(e) { return; } } }",
        "Object.defineProperty(foo, 'bar', { set(val) { return; } })",
    },
}
