use crate::rule_prelude::*;
use rslint_scope::FileId;

declare_lint! {
    /**
    Disallows variable declarations to shadow another variable declaration

    Shadowing is the process in which a local variable has the same name as an
    outer variable. In this case the local variable is shadowing the outer variable.
    This leads to confusion while reading.

    ### Invalid Code Examples
    ```js
    var a = 3;
    function b() {
        var a = 10;
    }
    ```
    */
    #[derive(Default)]
    NoShadow,
    errors,
    "no-shadow",
    // TODO: There are also some options for this rule in eslint
    // which we may want to implement too.
}

#[typetag::serde]
impl CstRule for NoShadow {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let outputs = ctx.analyzer.as_ref()?.outputs().clone();
        let file = FileId::new(ctx.file_id as u32);

        outputs.no_shadow.iter().for_each(|shadow| {
            let shadow = shadow.key();

            if shadow.file == file {
                let err = Diagnostic::warning(
                    file.id as usize,
                    "no-shadow",
                    format!("`{}` was shadowed", *shadow.variable),
                )
                .primary(shadow.original.1, "originally defined here")
                .secondary(shadow.shadower.1, "shadowed here");
                ctx.add_err(err);
            }
        });

        None
    }
}

// TODO
rule_tests! {
    NoShadow::default(),
    err: {},
    ok: {}
}
