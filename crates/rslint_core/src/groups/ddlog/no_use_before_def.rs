use crate::rule_prelude::*;
use rslint_scope::{
    ast::{AnyId, StmtKind},
    FileId,
};

declare_lint! {
    /**
    Disallows use of a variable before it was defined

    In JavaScript it's possible to use variables to use identifiers
    before their declaration in the code. This can be confusing for readers
    and should be avoided.

    ### Invalid Code Examples
    ```js
    alert(a);
    var a = 10;

    f();
    function f() {}
    ```
    */
    NoUseBeforeDef,
    errors,
    "no-use-before-def",

    /**
     * If this is `true`, this rule warns for every function that is used
     * before it is declared. Default `true`.
     */
    pub functions: bool,
    /**
     * If this is `true`, this rule warns for every class that is used
     * before it is declared. Default `true`.
     */
    pub classes: bool,
    /**
     * If this is `true`, this rule warns for every variable that is used
     * before it is declared. Default `true`.
     */
    pub variables: bool,
}

impl Default for NoUseBeforeDef {
    fn default() -> Self {
        Self {
            functions: true,
            classes: true,
            variables: true,
        }
    }
}

#[typetag::serde]
impl CstRule for NoUseBeforeDef {
    fn check_root(&self, _root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        let analyzer = ctx.analyzer.as_ref()?.clone();
        let outputs = analyzer.outputs();
        let file = FileId::new(ctx.file_id as u32);

        outputs.use_before_def.iter().for_each(|used| {
            let used = used.key();
            if used.file != file {
                return;
            }

            match used.declared {
                AnyId::AnyIdFunc { .. } if !self.functions => return,
                AnyId::AnyIdClass { .. } if !self.classes => return,
                AnyId::AnyIdStmt { stmt } if !self.variables => {
                    if matches!(
                        analyzer.get_stmt(stmt, file).map(|stmt| stmt.kind),
                        Some(StmtKind::StmtVarDecl)
                    ) {
                        return;
                    }
                }
                _ => {}
            }

            if !self.functions && matches!(used.declared, AnyId::AnyIdFunc { .. })
                || !self.variables && matches!(used.declared, AnyId::AnyIdStmt { .. })
            {
                return;
            }

            let err = Diagnostic::warning(
                file.id as usize,
                "no-use-before-def",
                format!("`{}` was used before it was defined", *used.name),
            )
            .primary(used.used_in, "used here")
            .secondary(used.declared_in, "defined here");

            ctx.add_err(err);
        });

        None
    }
}

// TODO
rule_tests! {
    NoUseBeforeDef::default(),
    err: {},
    ok: {}
}
