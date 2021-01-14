use std::collections::HashMap;

use SyntaxKind::*;

use crate::rule_prelude::*;

declare_lint! {
    /**
        Disallow defining a class method more than once.

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
            let getters = node
                .children()
                .filter_map(|n| {
                    if n.kind() == GETTER {
                        Some(n.to::<ast::Getter>())
                    } else {
                        None
                    }
                })
                .collect();
            if let Some(d) = check(self.name(), &mut ctx, getters) {
                ctx.add_err(d);
                return None;
            }

            let setters = node
                .children()
                .filter_map(|n| {
                    if n.kind() == SETTER {
                        Some(n.to::<ast::Setter>())
                    } else {
                        None
                    }
                })
                .collect();
            if let Some(d) = check(self.name(), &mut ctx, setters) {
                ctx.add_err(d);
                return None;
            }

            let methods = node
                .children()
                .filter_map(|n| {
                    if n.kind() == METHOD {
                        Some(n.to::<ast::Method>())
                    } else {
                        None
                    }
                })
                .collect();
            if let Some(d) = check(self.name(), &mut ctx, methods) {
                ctx.add_err(d);
                return None;
            }
        }

        None
    }
}

fn check<T: Named>(code: &str, ctx: &mut RuleCtx, nodes: Vec<T>) -> Option<Diagnostic> {
    let mut names = HashMap::new();
    for n in &nodes {
        let mut name = Named::name(n);
        if name == "constructor" {
            continue;
        }
        if n.is_static() {
            name = format!("static {}", name);
        }
        if let Some(prev_decl) = names.insert(name.clone(), n.name_node()) {
            let err = ctx
                .err(code, format!("Duplicate name '{}'", name))
                .secondary(
                    prev_decl,
                    format!("Previous declaration of the method '{}' here", &name),
                )
                .primary(n.name_node(), format!("'{}' redefined here", name));
            return Some(err);
        }
    }

    None
}

trait Named {
    fn name(&self) -> String;

    fn name_node(&self) -> SyntaxNode;

    fn is_static(&self) -> bool;
}

impl Named for ast::Getter {
    fn name(&self) -> String {
        self.name_node().to_string()
    }

    fn name_node(&self) -> SyntaxNode {
        let names = self
            .syntax()
            .children()
            .filter_map(|n| {
                if n.kind() == NAME {
                    Some(n.to::<ast::Name>())
                } else {
                    None
                }
            })
            .collect::<Vec<ast::Name>>();
        names[1].syntax().clone()
    }

    fn is_static(&self) -> bool {
        self.syntax().first_token().unwrap().kind() == STATIC_KW
    }
}

impl Named for ast::Setter {
    fn name(&self) -> String {
        self.name_node().to_string()
    }

    fn name_node(&self) -> SyntaxNode {
        let names = self
            .syntax()
            .children()
            .filter_map(|n| {
                if n.kind() == NAME {
                    Some(n.to::<ast::Name>())
                } else {
                    None
                }
            })
            .collect::<Vec<ast::Name>>();
        names[1].syntax().clone()
    }

    fn is_static(&self) -> bool {
        self.syntax().first_token().unwrap().kind() == STATIC_KW
    }
}

impl Named for ast::Method {
    fn name(&self) -> String {
        self.name().unwrap().as_string().unwrap()
    }

    fn name_node(&self) -> SyntaxNode {
        self.name().unwrap().syntax().clone()
    }

    fn is_static(&self) -> bool {
        self.syntax().first_token().unwrap().kind() == STATIC_KW
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
    }
}
