use crate::rule_prelude::*;
use rslint_scope::FileId;

declare_lint! {
    /**
    No undef
    */
    #[derive(Default)]
    NoShadow,
    errors,
    "no-shadow"
}

#[typetag::serde]
impl CstRule for NoShadow {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let outputs = ctx.analyzer.as_ref()?.outputs();
        let file = FileId::new(ctx.file_id as u32);

        ctx.diagnostics
            .extend(outputs.no_shadow.iter().filter_map(|shadow| {
                let shadow = shadow.key();

                if shadow.file == file {
                    Some(
                        Diagnostic::warning(
                            file.id as usize,
                            "no-shadow",
                            format!("`{}` was shadowed", *shadow.variable),
                        )
                        .primary(shadow.original.1, "originally defined here")
                        .secondary(shadow.shadower.1, "shadowed here"),
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
    NoShadow::default(),
    err: {},
    ok: {}
}
