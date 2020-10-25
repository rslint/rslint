use crate::rule_prelude::*;
use crate::util::StyleExt;
use crate::Inferable;
use ast::{BlockStmt, SwitchStmt};
use SyntaxKind::{BLOCK_STMT, L_CURLY, R_CURLY, SWITCH_STMT};

declare_lint! {
    /**
    Enforce or disallow spaces inside of blocks after the opening and closing brackets.

    This rule enforces consistent spacing inside blocks by enforcing the opening token and the next token
    being on the same line. It also enforces consistent spacing with a closing token and the previous token being
    on the same line.

    ## Always

    ### Incorrect code examples

    ```js
    function foo() {return true;}
    if (foo) { bar = 0;}
    function baz() {let i = 0;
        return i;
    }
    ```

    ### Correct code examples

    ```js
    function foo() { return true; }
    if (foo) { bar = 0; }
    ```

    ## Never

    ### Incorrect code examples

    ```js
    function foo() { return true; }
    if (foo) { bar = 0;}
    ```

    ### Correct code examples

    ```js
    function foo() {return true;}
    if (foo) {bar = 0;}
    ```
    */
    #[serde(default)]
    #[derive(rslint_macros::Mergeable)]
    BlockSpacing,
    style,
    "block-spacing",
    /// The style of spacing, either "always" (default) to require one or more spaces, or
    /// "never" to disallow spaces
    pub style: String
}

impl Default for BlockSpacing {
    fn default() -> Self {
        Self {
            style: "always".to_string(),
        }
    }
}

#[typetag::serde]
impl CstRule for BlockSpacing {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if !matches!(node.kind(), SWITCH_STMT | BLOCK_STMT) {
            return None;
        }

        let open_token = node.token_with_kind(L_CURLY)?;
        let close_token = node.token_with_kind(R_CURLY)?;
        if is_empty(node) {
            return None;
        }

        let msg = |loc: &str, tok: &str| {
            if self.style == "always" {
                format!("Expected a space {} `{}`", loc, tok)
            } else {
                format!("Unexpected space(s) {} `{}`", loc, tok)
            }
        };

        if !(open_token.trailing_trivia_has_linebreak(true)
            || (open_token.has_trailing_whitespace(false, true) == (self.style == "always")))
        {
            let err = ctx.err(self.name(), msg("after", "{")).primary(node, "");

            ctx.add_err(err);
            let fix = ctx
                .fix()
                .delete_multiple(open_token.trailing_whitespace(false));
            if self.style == "always" {
                fix.insert_after(open_token, " ");
            }
        }

        if !(close_token.leading_trivia_has_linebreak(true)
            || (close_token.has_leading_whitespace(false, false) == (self.style == "always")))
        {
            let err = ctx.err(self.name(), msg("before", "}")).primary(node, "");

            ctx.add_err(err);
            let fix = ctx
                .fix()
                .delete_multiple(close_token.leading_whitespace(false));
            if self.style == "always" {
                fix.insert_before(close_token, " ");
            }
        }
        None
    }
}

fn is_empty(node: &SyntaxNode) -> bool {
    node.try_to::<SwitchStmt>()
        .map(|x| x.cases().next().is_none())
        .unwrap_or_default()
        || node
            .try_to::<BlockStmt>()
            .map(|x| x.stmts().next().is_none())
            .unwrap_or_default()
}

#[typetag::serde]
impl Inferable for BlockSpacing {
    fn infer(&mut self, nodes: &[SyntaxNode]) {
        let mut inferred_structs = vec![];
        for node in nodes {
            if matches!(node.kind(), SWITCH_STMT | BLOCK_STMT) {
                let mut ctx = RuleCtx::dummy_ctx();
                Self::default().check_node(node, &mut ctx);
                if ctx.diagnostics.is_empty() {
                    inferred_structs.push(Self::default());
                } else {
                    inferred_structs.push(Self {
                        style: "never".to_string(),
                    });
                }
            }
        }
        if let Some(new) = Self::merge(inferred_structs) {
            *self = new;
        }
    }
}

rule_tests! {
    BlockSpacing::default(),
    err: {
        "{foo();}",
        "{foo();}",
        "{ foo();}",
        "{foo(); }",
        "{foo();\n}",
        "if (a) {foo();}",
        "if (a) {} else {foo();}",
        "switch (a) {case 0: foo();}",
        "while (a) {foo();}",
        "do {foo();} while (a);",
        "for (;;) {foo();}",
        "for (var a in b) {foo();}",
        "for (var a of b) {foo();}",
        "try {foo();} catch (e) {foo();} finally {foo();}",
        "function foo() {bar();}",
        "(function() {bar();});",
        "(() => {bar();});",
        "if (a) {//comment\n foo(); }"
    },
    ok: {
        "{ foo(); }",
        "{ foo();\n}",
        "{\nfoo(); }",
        "{\r\nfoo();\r\n}",
        "if (a) { foo(); }",
        "if (a) {} else { foo(); }",
        "switch (a) {}",
        "switch (a) { case 0: foo(); }",
        "while (a) { foo(); }",
        "do { foo(); } while (a);",
        "for (;;) { foo(); }",
        "for (var a in b) { foo(); }",
        "for (var a of b) { foo(); }",
        "try { foo(); } catch (e) { foo(); }",
        "function foo() { bar(); }",
        "(function() { bar(); });",
        "(() => { bar(); });",
        "if (a) { /* comment */ foo(); /* comment */ }",
        "if (a) { //comment\n foo(); }",
    }
}

rule_tests! {
    block_spacing_never_valid,
    block_spacing_never_invalid,
    BlockSpacing { style: "never".to_string() },
    err: {
        "{ foo(); }",
        "{ foo();}",
        "{foo(); }",
        "{\nfoo(); }",
        "{ foo();\n}",
        "if (a) { foo(); }",
        "if (a) {} else { foo(); }",
        "switch (a) { case 0: foo(); }",
        "while (a) { foo(); }",
        "do { foo(); } while (a);",
        "for (;;) { foo(); }",
        "for (var a in b) { foo(); }",
        "for (var a of b) { foo(); }",
        "try { foo(); } catch (e) { foo(); } finally { foo(); }",
        "function foo() { bar(); }",
        "(function() { bar(); });",
        "(() => { bar(); });",
        "if (a) { /* comment */ foo(); /* comment */ }",
        "(() => {   bar();});",
        "(() => {bar();   });",
        "(() => {   bar();   });"
    },
    ok: {
        "{foo();}",
        "{foo();\n}",
        "{\nfoo();}",
        "{\r\nfoo();\r\n}",
        "if (a) {foo();}",
        "if (a) {} else {foo();}",
        "switch (a) {}",
        "switch (a) {case 0: foo();}",
        "while (a) {foo();}",
        "do {foo();} while (a);",
        "for (;;) {foo();}",
        "for (var a in b) {foo();}",
        "for (var a of b) {foo();}",
        "try {foo();} catch (e) {foo();}",
        "function foo() {bar();}",
        "(function() {bar();});",
        "(() => {bar();});",
    }
}
