use crate::rule_prelude::*;
use rslint_scope::FileId;

declare_lint! {
    /**
    No undef
    */
    #[derive(Default)]
    NoUnusedLabels,
    errors,
    "no-unused-labels"
}

#[typetag::serde]
impl CstRule for NoUnusedLabels {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let outputs = ctx.analyzer.as_ref()?.outputs();
        let file = FileId::new(ctx.file_id as u32);

        ctx.diagnostics
            .extend(outputs.no_unused_labels.iter().filter_map(|label| {
                let label = label.key();

                if label.file == file {
                    Some(
                        Diagnostic::warning(
                            file.id as usize,
                            "no-unused-labels",
                            format!("the label `{}` was never used", *label.label_name.data),
                        )
                        .primary(label.label_name.span, "created here"),
                    )
                } else {
                    None
                }
            }));

        None
    }
}

// TODO
rule_tests! {
    NoUnusedLabels::default(),
    err: {},
    ok: {}
}
