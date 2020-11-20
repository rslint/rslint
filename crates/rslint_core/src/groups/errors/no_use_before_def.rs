use crate::rule_prelude::*;
use rslint_scope::FileId;

declare_lint! {
    /**
    No undef
    */
    #[derive(Default)]
    NoUseBeforeDef,
    errors,
    "no-use-before-def"
}

#[typetag::serde]
impl CstRule for NoUseBeforeDef {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let outputs = ctx.analyzer.as_ref()?.outputs();
        let file = FileId::new(ctx.file_id as u32);

        ctx.diagnostics
            .extend(outputs.use_before_def.iter().filter_map(|used| {
                let used = used.key();

                if used.file == file {
                    Some(
                        Diagnostic::warning(
                            file.id as usize,
                            "no-use-before-def",
                            format!("`{}` was used before it was defined", *used.name),
                        )
                        .primary(used.used_in, "used here")
                        .secondary(used.declared_in, "defined here"),
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
    NoUseBeforeDef::default(),
    err: {},
    ok: {}
}
