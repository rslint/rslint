use crate::rule_prelude::*;
use rslint_scope::{FileId, NoUnusedLabelsConfig};

declare_lint! {
    /**
    Disallows unused labels

    Labels that are declared, but are never used, are most likely
    an error and should be avoided.

    ### Invalid Code Examples
    ```js
    A: var foo = 0;

    B: {
        foo();
    }

    C:
    for (let i = 0; i < 10; ++i) {
        foo();
    }
    ```
    */
    #[derive(Default)]
    NoUnusedLabels,
    ddlog,
    "no-unused-labels",

    #[serde(flatten)]
    config: NoUnusedLabelsConfig,
}

#[typetag::serde]
impl CstRule for NoUnusedLabels {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let analyzer = ctx.analyzer.as_ref()?.clone();
        let file = FileId::new(ctx.file_id as u32);

        analyzer
            .no_unused_labels(file, Some(self.config.clone()))
            .unwrap();

        analyzer
            .outputs()
            .no_unused_labels
            .iter()
            .for_each(|label| {
                let label = label.key();

                if label.file == file {
                    let err = Diagnostic::warning(
                        file.id as usize,
                        "no-unused-labels",
                        format!("the label `{}` was never used", *label.label_name.data),
                    )
                    .primary(label.label_name.span, "created here");
                    ctx.add_err(err);
                }
            });

        None
    }
}

// TODO
rule_tests! {
    NoUnusedLabels::default(),
    err: {},
    ok: {}
}
