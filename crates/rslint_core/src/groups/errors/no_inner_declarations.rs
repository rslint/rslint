use crate::rule_prelude::*;
use ast::VarDecl;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow variable and function declarations in nested blocks.

    Prior to ECMAScript 6, function declarations were only allowed in the first level of a program
    or the body of another function, although parsers sometimes incorrectly accept it. This rule only applies to
    function declarations, not function expressions.

    ## Invalid Code Examples

    ```js
    function foo() {
        if (bar) {
            // Move this to foo's body, outside the if statement
            function bar() {}
        }
    }
    ```

    ```js
    if (bar) {
        var foo = 5;
    }
    ```

    ## Correct Code Examples

    ```js
    function foo() {}

    var bar = 5;
    ```
    */
    #[serde(default)]
    NoInnerDeclarations,
    errors,
    tags(Recommended),
    "no-inner-declarations",
    /// What declarations to disallow in nested blocks, it can include two possible options:
    /// "functions" and "variables", you can include either or, or both. Disallows only functions
    /// by default.
    pub disallowed: Vec<String>
}

impl Default for NoInnerDeclarations {
    fn default() -> Self {
        Self {
            disallowed: vec!["functions".to_string()],
        }
    }
}

impl NoInnerDeclarations {
    pub fn disallow_all() -> Self {
        Self {
            disallowed: vec!["functions".to_string(), "variables".to_string()],
        }
    }
}

const VALID_BLOCK_PARENT: [SyntaxKind; 3] = [FN_DECL, FN_EXPR, ARROW_EXPR];
const VALID_PARENT: [SyntaxKind; 5] = [
    SCRIPT,
    MODULE,
    EXPORT_NAMED,
    EXPORT_DEFAULT_DECL,
    EXPORT_DECL,
];

#[typetag::serde]
impl CstRule for NoInnerDeclarations {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == FN_DECL && self.disallowed.contains(&"functions".to_string())
            || node.kind() == VAR_DECL && self.disallowed.contains(&"variables".to_string())
        {
            let parent = node.parent()?;
            if node.kind() == VAR_DECL && parent.kind() == FOR_STMT_INIT {
                return None;
            }

            if let Some(decl) = node.try_to::<VarDecl>() {
                if decl.is_const() || decl.is_let() {
                    return None;
                }
            }

            if parent.kind() == BLOCK_STMT
                && parent
                    .parent()
                    .map_or(false, |parent| VALID_BLOCK_PARENT.contains(&parent.kind()))
            {
                return None;
            }

            if VALID_PARENT.contains(&parent.kind()) {
                return None;
            }

            let enclosing = util::outer_function(node);
            let second_part = if let Some(ref function) = enclosing {
                function
                    .child_with_kind(NAME)
                    .map_or("enclosing function's body".to_string(), |name| {
                        format!("{}'s body", name.text().to_string())
                    })
            } else {
                "program's root".to_string()
            };

            let message = if node.kind() == VAR_DECL {
                format!("move this variable declaration to {}", second_part)
            } else {
                format!("move this function declaration to {}", second_part)
            };

            let mut err = ctx.err(self.name(), message).primary(node, "");

            if let Some(function) = enclosing {
                err = err.secondary(
                    function,
                    "move the declaration to the body of this function",
                );
            } else {
                err = err.footer_help("move the declaration to the program root");
            }

            ctx.add_err(err);
        }
        None
    }
}

rule_tests! {
    NoInnerDeclarations::default(),
    err: {
        "if (test) { function doSomething() { } }",
        "if (foo)  function f(){} ",
        "function bar() { if (foo) function f(){}; }",
        "function doSomething() { do { function somethingElse() { } } while (test); }",
        "(function() { if (test) { function doSomething() { } } }());",
        "if (foo){ function f(){ if(bar){ var a; } } }",
        "if (foo) function f(){ if(bar) var a; } "
    },
    ok: {
        "function doSomething() { }",
        "if (test) { let x = 1; }",
        "if (test) { const x = 1; }",
        "export const foo = [];
        export function bar() {}"
    }
}
