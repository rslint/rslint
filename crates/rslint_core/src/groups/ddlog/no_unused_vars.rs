use crate::rule_prelude::*;
use rslint_scope::{FileId, NoUnusedVarsConfig};

declare_lint! {
    /**
    Disallows unused variables

    Variables that are declared, but never used are most likely an error.

    ### Invalid Code Examples
    ```js
    var x = 1;

    // `foo` is unused
    function(foo) {
        return 5;
    }

    function getY([x, y]) {
        return y;
    }
    ```
    */
    #[derive(Default)]
    NoUnusedVars,
    ddlog,
    "no-unused-vars",

    #[serde(flatten)]
    config: NoUnusedVarsConfig,
}

#[typetag::serde]
impl CstRule for NoUnusedVars {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let analyzer = ctx.analyzer.as_ref()?.clone();
        let file = FileId::new(ctx.file_id as u32);

        analyzer
            .no_unused_vars(file, Some(self.config.clone()))
            .unwrap();

        analyzer.outputs().no_unused_vars.iter().for_each(|unused| {
            let unused = unused.key();

            if unused.file == file {
                let err = Diagnostic::warning(
                    file.id as usize,
                    "no-unused-vars",
                    format!("`{}` was defined, but never used", *unused.name),
                )
                .primary(unused.span, "defined here");
                ctx.add_err(err);
            }
        });

        None
    }
}

// TODO
rule_tests! {
    NoUnusedVars::default(),
    err: {},
    ok: {}
}
