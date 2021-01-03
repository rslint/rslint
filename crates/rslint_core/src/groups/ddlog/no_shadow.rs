use crate::rule_prelude::*;
use rslint_scope::{FileId, NoShadowConfig};

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
    ddlog,
    "no-shadow",
    // TODO: There are also some options for this rule in eslint
    // which we may want to implement too.

    #[serde(flatten)]
    config: NoShadowConfig,
}

#[typetag::serde]
impl CstRule for NoShadow {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let analyzer = ctx.analyzer.as_ref()?.clone();
        let file = FileId::new(ctx.file_id as u32);

        analyzer.no_shadow(file, Some(self.config.clone())).unwrap();

        analyzer.outputs().no_shadow.iter().for_each(|shadow| {
            let shadow = shadow.key();

            if shadow.original.0.file() == Some(file) {
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
