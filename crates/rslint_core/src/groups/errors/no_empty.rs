use crate::rule_prelude::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow empty block statements.

    Block statements with nothing in them are very common when refactoring, however
    they can get confusing really quickly. This rule reports empty block statements and empty switch
    case blocks if they do not have a comment.

    ## Invalid Code Examples

    ```js
    {}
    ```

    ```js
    if (foo) {

    }
    ```

    ## Correct Code Examples

    ```js
    if (foo) {
        /* todo */
    }
    ```
    */
    #[derive(Default)]
    #[serde(default)]
    NoEmpty,
    errors,
    tags(Recommended),
    "no-empty",
    /// Whether to disallow empty block statements in function declarations, arrow functions,
    /// getters, setters, and methods.
    pub disallow_empty_functions: bool,
    /// Whether to allow empty `catch` clauses without a comment.
    pub allow_empty_catch: bool
}

const IGNORED: [SyntaxKind; 7] = [
    FN_DECL,
    FN_EXPR,
    ARROW_EXPR,
    GETTER,
    SETTER,
    METHOD,
    CONSTRUCTOR,
];

#[typetag::serde]
impl CstRule for NoEmpty {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == BLOCK_STMT
            && (node
                .parent()
                .map_or(true, |parent| !IGNORED.contains(&parent.kind()))
                || self.disallow_empty_functions)
        {
            if node
                .parent()
                .map_or(false, |parent| parent.kind() == CATCH_CLAUSE)
                && self.allow_empty_catch
            {
                return None;
            }

            if node.first_child().is_none() && !node.contains_comments() {
                let err = ctx
                    .err(self.name(), "empty block statements are not allowed")
                    .primary(node, "");

                ctx.add_err(err);
            }
        }

        if let Some(switch) = node.try_to::<ast::SwitchStmt>() {
            if switch.cases().next().is_none() {
                let start = switch.l_curly_token()?.text_range().end();
                let range =
                    util::token_list_range(&[switch.l_curly_token()?, switch.r_curly_token()?]);

                let is_empty = switch.syntax().tokens().iter().any(|tok| {
                    tok.kind() == SyntaxKind::COMMENT && tok.text_range().start() > start
                });
                if !is_empty {
                    let err = ctx
                        .err(self.name(), "empty switch statements are not allowed")
                        .primary(range, "");

                    ctx.add_err(err);
                }
            }
        }
        None
    }
}

rule_tests! {
    NoEmpty::default(),
    err: {
        "{}",
        /// ignore
        "{  }",
        "if (foo) {}",
        "do { } while (scoot)",
        "for(let i = 5; i < 10; i++) {}",
        "switch (foo) {}",
        "switch (foo /* bar */) {}"
    },
    ok: {
        "{ /* sike you thought it was empty */ }",
        "{
        // foo   
        }",
        "if (foo) { /* */ }",
        "switch (bar) { /* */ }",
        "class Foo { constructor() {} }"
    }
}
