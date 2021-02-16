use crate::rule_prelude::*;

declare_lint! {
    /**
    Disallow sparse arrays.

    Sparse arrays are arrays with empty slots, they are denoted by extra commas, such as:

    ```js
    let foo = [,,];
    let foo = [bar,, baz];
    ```

    Sparse elements will be filled in as undefined elements and count towards array length.
    This is often a typo or is hard to comprehend and an explicit method should be used.

    ## Invalid Code Examples

    ```js
    let foo = [,];
    let bar = [foo,, bar];
    ```
    */
    #[derive(Default)]
    NoSparseArrays,
    errors,
    tags(Recommended),
    "no-sparse-arrays"
}

#[typetag::serde]
impl CstRule for NoSparseArrays {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let elems = node.try_to::<ast::ArrayExpr>()?.sparse_elements();
        if !elems.is_empty() {
            let mut err = ctx.err(self.name(), "sparse arrays are not allowed");
            for elem in elems {
                err = err.primary(elem, "");
            }
            err = err.footer_note(
                "the sparse elements will become elements with a value of `undefined`",
            );
            ctx.add_err(err);
        }
        None
    }
}

rule_tests! {
    NoSparseArrays::default(),
    err: {
        "[,]",
        "[...2,, 3]",
        "[4,,]"
    },
    ok: {
        "[1, 2]",
        "[3,]"
    }
}
