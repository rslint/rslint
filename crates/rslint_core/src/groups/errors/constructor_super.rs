use crate::rule_prelude::*;
use ast::{ClassDecl, ClassElement, Expr, Stmt};

declare_lint! {
    /**
    Verify calls of `super()` in constructors

    The `"extends": "rslint:recommended"` property in a configuration file enables this rule.

    Constructors of derived classes must call `super()`. Constructors of non derived classes must not call `super()`. If this is not observed, the JavaScript engine will raise a runtime error.

    This rule checks whether or not there is a valid `super()` call.
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
        let super_call = class_decl
            .body()?
            // work-around for bug where `.body()?.elements()` returns only one
            // element for whatever reason
            .syntax()
            .children()
            .filter_map(|x| x.try_to::<ClassElement>())
            // get the first constructor we can find. there should only be one.
            .find_map(|x| match x {
                ClassElement::Constructor(c) => Some(c),
                _ => None,
            })?
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
                        class.syntax(),
                        "superclass specified here, but super was not called",
                    );

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
        "class A extends null { constructor() { super(); } }",
        "class A extends null { constructor() { } }",
    },
    ok: {
        "class A { constructor() { } }",
        "class A extends B { constructor() { super(); } }",
    }
}
