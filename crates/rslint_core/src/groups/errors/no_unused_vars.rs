use crate::rule_prelude::*;
use rslint_scope::FileId;

declare_lint! {
    /**
    No undef
    */
    #[derive(Default)]
    NoUnusedVars,
    errors,
    "no-unused-vars"
}

#[typetag::serde]
impl CstRule for NoUnusedVars {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let outputs = ctx.analyzer.as_ref()?.outputs();
        let file = FileId::new(ctx.file_id as u32);

        ctx.diagnostics
            .extend(outputs.no_unused_vars.iter().filter_map(|unused| {
                let unused = unused.key();

                if unused.file == file {
                    Some(
                        Diagnostic::warning(
                            file.id as usize,
                            "no-unused-vars",
                            format!("`{}` was defined, but never used", *unused.name),
                        )
                        .primary(unused.span, "defined here"),
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
    NoUnusedVars::default(),
    err: {},
    ok: {}
}
