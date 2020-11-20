use crate::rule_prelude::*;
use rslint_scope::FileId;

declare_lint! {
    /**
    No undef
    */
    #[derive(Default)]
    NoUndef,
    errors,
    "no-undef"
}

#[typetag::serde]
impl CstRule for NoUndef {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let outputs = ctx.analyzer.as_ref()?.outputs();
        let file = FileId::new(ctx.file_id as u32);

        ctx.diagnostics
            .extend(outputs.no_undef.iter().filter_map(|undef| {
                let undef = undef.key();

                if undef.file == file {
                    Some(
                        Diagnostic::error(
                            file.id as usize,
                            "no-undef",
                            format!("`{}` was used, but never defined", *undef.name),
                        )
                        .primary(undef.span, "used here"),
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
    NoUndef::default(),
    err: {},
    ok: {}
}
