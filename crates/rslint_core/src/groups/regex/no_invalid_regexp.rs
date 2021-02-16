use super::maybe_parse_and_store_regex;
use crate::rule_prelude::*;

declare_lint! {
    /**
    Disallow invalid regular expressions in literals and `RegExp` constructors.

    Invalid regex patterns in `RegExp` constructors are not caught until runtime. This
    rule checks for calls to `RegExp` and validates the pattern given. This also checks regex literals
    for errors as RSLint's parser currently does not validate regex patterns.

    ## Incorrect Code Examples

    ```js
    RegExp('[');

    RegExp('a', 'h');

    new RegExp('[')
    ```
    */
    #[derive(Default)]
    NoInvalidRegexp,
    regex,
    tags(Recommended),
    "no-invalid-regexp"
}

#[typetag::serde]
impl CstRule for NoInvalidRegexp {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let res = maybe_parse_and_store_regex(node, ctx.file_id)?;
        if let Err((range, string)) = res {
            let err = ctx
                .err(self.name(), "invalid regex pattern")
                .primary(range, string);

            ctx.add_err(err);
        }
        None
    }
}

// no point in adding a lot of explicit tests because
// it just delegates to rslint_regex and there are tests there

rule_tests! {
    NoInvalidRegexp::default(),
    err: {
        "RegExp('[')",
        "new RegExp('[')",
        "RegExp('a', 'h')",
    },
    ok: {}
}
