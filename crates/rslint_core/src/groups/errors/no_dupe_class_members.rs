use std::collections::HashMap;

use SyntaxKind::*;

use crate::rule_prelude::*;

declare_lint! {
    /**
        Disallows defining a class method more than once, unless that method is overload in TypeScript.

        If there are declarations of the same name in class members, the last declaration overwrites other declarations silently.
        It can cause unexpected behaviors.

        ## Incorrect code examples

        ```js
        class Foo {
            bar() { }
            bar() { }
        }

        class Foo {
            bar() { }
            get bar() { }
        }

        class Foo {
            static bar() { }
            static bar() { }
        }

        ```

        ## Correct code examples

        ```js
        class Foo {
            bar() { }
            qux() { }
        }

        class Foo {
            get bar() { }
            set bar(value) { }
        }

        class Foo {
            static bar() { }
            bar() { }
        }

        ```

        ```ts
        // note: this is valid because of method overloading in TypeScript
        class Foo {
            foo(a: string): string;
            foo(a: number): number;
            foo(a: any): any {}
        }
        ```
    */
    #[derive(Default)]
    NoDupeClassMembers,
    errors,
    "no-dupe-class-members"
}

#[typetag::serde]
impl CstRule for NoDupeClassMembers {
    fn check_node(&self, node: &SyntaxNode, mut ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == CLASS_BODY {
            let getters = children_of_kind::<ast::Getter>(node, GETTER);
            check_class_members(self.name(), &mut ctx, getters).ok()?;

            let setters = children_of_kind::<ast::Setter>(node, SETTER);
            check_class_members(self.name(), &mut ctx, setters).ok()?;

            let methods = children_of_kind::<ast::Method>(node, METHOD);
            check_class_members(self.name(), &mut ctx, methods).ok()?;
        }

        None
    }
}

/// Checks if there is a duplicate in the given class members.
/// Adds a `no-dupe-class-member` error to the given `RuleCtx` and returns
/// the diagnostic as Err if it finds an error. Otherwise, it returns Ok.
fn check_class_members(
    code: &str,
    ctx: &mut RuleCtx,
    members: Vec<impl ClassMember>,
) -> Result<(), Diagnostic> {
    let mut identities = HashMap::new();
    for m in &members {
        if m.is_empty_decl() {
            continue;
        }

        let ident = {
            if m.is_static() {
                format!("static {}", m.identity())
            } else {
                m.identity()
            }
        };

        if let Some(prev_decl) = identities.insert(ident.clone(), m.name()) {
            let err = ctx
                .err(code, format!("Duplicate name `{}`", m.name().to_string()))
                .secondary(
                    prev_decl,
                    format!(
                        "Previous declaration of the method `{}` here",
                        &m.name().to_string()
                    ),
                )
                .primary(
                    m.name(),
                    format!("`{}` redefined here", m.name().to_string()),
                );
            ctx.add_err(err.clone());
            return Err(err);
        }
    }

    Ok(())
}

fn children_of_kind<T: AstNode>(node: &SyntaxNode, syn_kind: SyntaxKind) -> Vec<T> {
    node.children()
        .filter_map(|n| {
            if n.kind() == syn_kind {
                Some(n.to::<T>())
            } else {
                None
            }
        })
        .collect()
}

trait ClassMember {
    /// Returns a unique representation of the class member. If two class members
    /// have the same identity, they are considered duplicated.
    fn identity(&self) -> String;

    /// Returns the syntax node corresponding to the name of the member.
    fn name(&self) -> SyntaxNode;

    fn is_static(&self) -> bool;

    /// Returns true if the it is a method declaration without an implementation body.
    /// For example:
    /// ```js
    /// class Foo {
    ///     bar(): any;
    /// }    
    /// ```
    fn is_empty_decl(&self) -> bool;
}

impl ClassMember for ast::Getter {
    fn identity(&self) -> String {
        self.name().to_string()
    }

    fn name(&self) -> SyntaxNode {
        self.key().unwrap().syntax().clone()
    }

    fn is_static(&self) -> bool {
        self.syntax().first_token().unwrap().kind() == STATIC_KW
    }

    fn is_empty_decl(&self) -> bool {
        self.body().is_none()
    }
}

impl ClassMember for ast::Setter {
    fn identity(&self) -> String {
        self.name().to_string()
    }

    fn name(&self) -> SyntaxNode {
        self.key().unwrap().syntax().clone()
    }

    fn is_static(&self) -> bool {
        self.syntax().first_token().unwrap().kind() == STATIC_KW
    }

    fn is_empty_decl(&self) -> bool {
        self.body().is_none()
    }
}

impl ClassMember for ast::Method {
    fn identity(&self) -> String {
        self.name().unwrap().as_string().unwrap()
    }

    fn name(&self) -> SyntaxNode {
        self.name().unwrap().syntax().clone()
    }

    fn is_static(&self) -> bool {
        self.syntax().first_token().unwrap().kind() == STATIC_KW
    }

    fn is_empty_decl(&self) -> bool {
        self.body().is_none()
    }
}

rule_tests! {
    NoDupeClassMembers::default(),
    err: {
        "class A { get foo() {} get foo() {} }",
        "class A { foo() {} foo() {} }",
        "!class A { foo() {} foo() {} };",
        "class A { 'foo'() {} 'foo'() {} }",
    },
    ok: {
        "class A { constructor() {} constructor() {} }",
        "class A { foo() {} bar() {} }",
        "class A { get foo() {} set foo(value) {} }",
        "class A { static foo() {} foo() {} }",
        "class A { static foo() {} get foo() {} set foo(value) {} }",
        "class A { foo() { } } class B { foo() { } }",
        "class A { 1() {} 2() {} }",
        "class A { [12]() {} [123]() {} }",
        "class A { [0x1]() {} [`0x1`]() {} }",
        "class A { [null]() {} ['']() {} }",
        "class Foo {
            foo(a: string): string;
            foo(a: number): number;
            foo(a: any): any {}
          }"
    }
}
