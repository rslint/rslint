use crate::{rule_prelude::*, util::find_best_match_for_name};
use rslint_scope::{FileId, NoTypeofUndefConfig, NoUndefConfig};

declare_lint! {
    /**
    Disallow use of undeclared variables

    This rule helps you to locate potential errors that are resulting
    from using a variable that is not defined, which may be caused by misspelling
    names, or implicit globals.

    ## Invalid Code Examples
    ```js
    var foo = someFunction();
    var bar = a + 1;
    ```
    */
    #[derive(Default)]
    NoUndef,
    ddlog,
    "no-undef",

    /**
     * If this option is `true`, any use of undefined values
     * inside a `typeof` expressions will be warned.
     */
    #[serde(rename = "typeof")]
    pub typeof_: Option<NoTypeofUndefConfig>,

    #[serde(flatten)]
    config: NoUndefConfig,
}

#[typetag::serde]
impl CstRule for NoUndef {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let analyzer = ctx.analyzer.as_ref()?.clone();
        let outputs = analyzer.outputs().clone();
        let file = FileId::new(ctx.file_id as u32);

        analyzer.no_undef(file, Some(self.config.clone())).unwrap();

        outputs.no_undef.iter().for_each(|undef| {
            let undef = undef.key();
            if undef.file == file {
                let scope = rslint_scope::ddlog_std::tuple2(file, undef.scope);
                let mut err = ctx
                    .err(
                        "no-undef",
                        format!("`{}` was used, but never defined", undef.name),
                    )
                    .primary(undef.span, "");

                let suggestion = analyzer
                    .variables_for_scope(Some(scope))
                    .ok()
                    .and_then(|vars| {
                        find_best_match_for_name(
                            vars.iter().map(|name| name.name.as_str()),
                            &*undef.name,
                            None,
                        )
                        .map(ToOwned::to_owned)
                    });

                if let Some(suggestion) = suggestion {
                    err = err.suggestion(
                        undef.span,
                        "a variable with a similair name exists",
                        suggestion,
                        Applicability::MaybeIncorrect,
                    );
                }

                ctx.add_err(err);
            }
        });

        if let Some(config) = self.typeof_.clone() {
            analyzer.no_typeof_undef(file, Some(config)).unwrap();

            outputs.no_typeof_undef.iter().try_for_each(|undef| {
                let undef = undef.key();
                let whole_expr = analyzer.get_expr(undef.whole_expr, file)?;
                let undefined_expr = analyzer.get_expr(undef.undefined_expr, file)?;

                if undef.file == file {
                    let d = Diagnostic::warning(
                        file.id as usize,
                        "no-undef",
                        "`typeof` of an undefined value always results in `undefined`",
                    )
                    .primary(whole_expr.span, "this will always return \"undefined\"...")
                    .secondary(
                        undefined_expr.span,
                        "...because this expression is undefined",
                    )
                    .suggestion(
                        whole_expr.span,
                        "try replacing the entire expression with \"undefined\"",
                        "\"undefined\"",
                        Applicability::Always,
                    );

                    ctx.add_err(d);
                }
                Some(())
            });
        }

        None
    }
}

// TODO
rule_tests! {
    NoUndef::default(),
    err: {},
    ok: {}
}
