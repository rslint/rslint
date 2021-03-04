use crate::rule_prelude::*;
use ast::{ClassDecl, ClassElement, Expr, Stmt};

declare_lint! {
    /**
    Verify calls of `super()` in constructors

    Constructors of derived classes must call `super()`. Constructors of non derived classes must not call `super()`.
    If this is not observed, the JavaScript engine will raise a runtime error.

    This rule checks whether or not there is a valid `super()` call.

    ## Incorrect Code Examples

    ```js
    class Foo {
        constructor() {
            super(); // SyntaxError because Foo does not extend any class.
        }
    }
    ```

    ```js
    class Foo extends Bar {
        constructor() {
            // we need to call Bar's constructor through `super()` but we haven't done that
        }
    }
    ```

    Classes extending a non-constructor are always an issue because we are required to call
    the superclass' constructor, but `null` is not a constructor.

    ```js
    class Foo extends null {
        constructor() {
            super(); // throws a TypeError because null is not a constructor
        }
    }
    ```

    ```js
    class Foo extends null {
        constructor() {
            // throws a ReferenceError
        }
    }
    ```

    ## Correct Code Examples

    ```js
    class Foo {
        constructor() {
            // this is fine because we don't extend anything
        }
    }
    ```

    ```js
    class Foo extends Bar {
        constructor() {
            super(); // this is fine because we extend a class and we call Bar's constructor through `super()`
        }
    }
    ```
    */
    #[derive(Default)]
    ConstructorSuper,
    errors,
    tags(Recommended),
    "constructor-super",
}

#[typetag::serde]
impl CstRule for ConstructorSuper {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let class_decl = node.try_to::<ClassDecl>()?;
        let superclass = class_decl.parent();
        let constructor = class_decl.body()?.elements().find_map(|x| match x {
            ClassElement::Constructor(c) => Some(c),
            _ => None,
        })?;
        let super_call = constructor
            // get only the expression statements for the constructor's body
            .body()?
            .stmts()
            .filter_map(|x| match x {
                Stmt::ExprStmt(expr) => expr.expr(),
                _ => None,
            })
            // find a super call expression. there should only be one.
            .find(|x| matches!(x, Expr::SuperCall(_)));

        // if it's a subclass xor it's constructor calls super, show error.
        match (superclass, super_call) {
            (Some(class), None) => {
                let diagnostic = ctx
                    .err(self.name(), "constructor of derived class must call super")
                    .primary(
                        constructor.syntax(),
                        "no call to super found within constructor",
                    )
                    .secondary(class.syntax(), "superclass specified here");

                ctx.add_err(diagnostic);
            }
            (None, Some(call)) => {
                let diagnostic = ctx
                    .err(
                        self.name(),
                        "cannot call super in constructor of base class",
                    )
                    .primary(
                        call.syntax(),
                        "called super here, but no superclass was specified",
                    );

                ctx.add_err(diagnostic);
            }
            _ => {}
        }

        None
    }
}

rule_tests! {
    ConstructorSuper::default(),
    err: {
        "class A { constructor() { super(); } }",
        "class A extends B { constructor() { } }",
    },
    ok: {
        "class A { constructor() { } }",
        "class A extends B { constructor() { super(); } }",
    }
}
