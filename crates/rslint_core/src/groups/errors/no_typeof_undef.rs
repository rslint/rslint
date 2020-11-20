use crate::rule_prelude::*;
use rslint_scope::FileId;

declare_lint! {
    /**
    No typeof undef
    */
    #[derive(Default)]
    NoTypeofUndef,
    errors,
    "no-typeof-undef"
}

#[typetag::serde]
impl CstRule for NoTypeofUndef {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let analyzer = ctx.analyzer.as_ref()?.clone();
        let file = FileId::new(ctx.file_id as u32);

        ctx.diagnostics.extend(
            analyzer
                .outputs()
                .no_typeof_undef
                .iter()
                .filter_map(|undef| {
                    let undef = undef.key();
                    let whole_expr = analyzer.get_expr(undef.whole_expr, file)?;
                    let undefined_expr = analyzer.get_expr(undef.undefined_expr, file)?;

                    if undef.file == file {
                        Some(
                            Diagnostic::warning(
                                file.id as usize,
                                "no-typeof-undef",
                                "`typeof undefined` always results in \"undefined\"",
                            )
                            .primary(whole_expr.span, "this will always return \"undefined\"")
                            .secondary(undefined_expr.span, "because this expression is undefined")
                            .suggestion(
                                whole_expr.span,
                                "try replacing the entire expression with \"undefined\"",
                                "\"undefined\"",
                                Applicability::Always,
                            ),
                        )
                    } else {
                        None
                    }
                }),
        );

        None
    }
}

// TODO
rule_tests! {
    NoTypeofUndef::default(),
    err: {},
    ok: {}
}
