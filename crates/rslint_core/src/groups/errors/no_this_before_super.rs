use crate::rule_prelude::*;
use ast::{ClassDecl, ClassElement};

declare_lint! {
    /**
    Prevent the use of `this` / `super` before calling `super()`.

    In the constructor of a derived class (`extends` a class), using `this` / `super` before the
    `super()` call, will throw an error.

    ## Incorrect Code Examples

    ```js
    class A extends B {
        constructor() {
            this.a = 0;
            super();
        }
    }
    ```

    ```js
    class A extends B {
        constructor() {
            this.foo();
            super();
        }
    }
    ```

    ```js

    class A extends B {
        constructor() {
            super.foo();
            super();
        }
    }
    ```

    ```js
    class A extends B {
        constructor() {
            super(this.foo());
        }
    }
    ```

    ## Correct Code Examples


    ```js
    class A {
        constructor() {
            this.a = 0; // OK, this class doesn't have an `extends` clause.
        }
    }
    ```

    ```js
    class A extends B {
        constructor() {
            super();
            this.a = 0; // OK, this is after `super()`.
        }
    }
    ```

    ```js
    class A extends B {
        foo() {
            this.a = 0; // OK. this is not in a constructor.
        }
    }
    ```
    */
    #[derive(Default)]
    NoThisBeforeSuper,
    errors,
    tags(Recommended),
    "no-this-before-super",
}

#[typetag::serde]
impl CstRule for NoThisBeforeSuper {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let class_decl = node.try_to::<ClassDecl>()?;
        let constructor = class_decl.body()?.elements().find_map(|x| match x {
            ClassElement::Constructor(c) => Some(c),
            _ => None,
        })?;

        let mut super_call = None;
        let mut this_expr = None;
        for node in constructor.syntax().descendants_with_tokens() {
            if node.kind() == SyntaxKind::SUPER_CALL {
                super_call = Some(node.text_range());

                if let Some(this) = node.as_node().and_then(|node| {
                    node.descendants_with_tokens().skip(2).find(|n| {
                        n.kind() == SyntaxKind::SUPER_KW || n.kind() == SyntaxKind::THIS_EXPR
                    })
                }) {
                    this_expr = Some(this);
                }

                continue;
            }

            if (node.kind() == SyntaxKind::THIS_EXPR
                || (node.kind() == SyntaxKind::SUPER_KW
                    && node.parent()?.kind() != SyntaxKind::SUPER_CALL))
                && super_call.is_none()
            {
                this_expr = Some(node);
                // we don't `break` here, so we can still find the `super();` call,
                // even if it's after the this expression
            }
        }

        match (super_call, this_expr) {
            (Some(super_call), Some(this)) => {
                let use_name = match this.kind() {
                    SyntaxKind::SUPER_KW => "super",
                    SyntaxKind::THIS_EXPR => "this",
                    _ => unreachable!(),
                };
                let err = ctx
                    .err(
                        self.name(),
                        format!("`{}` is not allowed before calling `super()`", use_name),
                    )
                    .primary(this, format!("`{}` is used here...", use_name))
                    .secondary(super_call, "...but `super` is called here")
                    .footer_note(format!(
                        "using `{}` before calling `super()` will result in a runtime error",
                        use_name
                    ));
                ctx.add_err(err);

                None
            }
            _ => None,
        }
    }
}

rule_tests! {
    NoThisBeforeSuper::default(),
    err: {
        "class A extends B { constructor() { this.a = 0; super(); } }",
        "class A extends B { constructor() { this.foo(); super(); } }",
        "class A extends B { constructor() { super.foo(); super(); } }",
        "class A extends B { constructor() { super(this.foo()); } }",
    },
    ok: {
        "class A { constructor() { this.a = 0; } }",
        "class A extends B { constructor() { super(); this.a = 0; } }",
        "class A extends B { foo() { this.a = 0; } }",
    }
}
