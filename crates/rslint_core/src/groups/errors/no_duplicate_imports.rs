use std::collections::HashMap;

use crate::rule_prelude::*;
use ast::{ExportDecl, ImportDecl, Literal};

declare_lint! {
    /**
    Disallow duplicate imports.

    Multiple import statements with the same source can be combined to one statement. This improves readability.

    ## Incorrect Code Examples

    ```js
    import { foo } from "bla";
    import { bar } from "bla";

    // including exports
    export { foo } from "bla";
    ```

    ## Correct Code Examples

    ```js
    import { foo, bar } from "bla";
    export { foo };
    ```
    */
    #[derive(Default)]
    #[serde(default)]
    NoDuplicateImports,
    errors,
    tags(Recommended),
    "no-duplicate-imports",
    /// Whether to check if re-exported
    pub include_exports: bool
}

#[typetag::serde]
impl CstRule for NoDuplicateImports {
    fn check_root(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        // the key of the hashmap is the name of the import source, and
        // a bool that is `true`, if the import/export has a `type` token.
        let mut seen: HashMap<(String, bool), SyntaxNode> = HashMap::default();

        for child in node.children() {
            if let Some(import) = child.try_to::<ImportDecl>() {
                self.process_node(&import.source(), import.type_token(), false, &mut seen, ctx);
            } else if self.include_exports {
                if let Some(export) = child.try_to::<ExportDecl>() {
                    self.process_node(&export.source(), export.type_token(), true, &mut seen, ctx);
                }
            }
        }

        None
    }
}

impl NoDuplicateImports {
    fn process_node(
        &self,
        source: &Option<Literal>,
        type_token: Option<SyntaxToken>,
        is_export: bool,
        seen: &mut HashMap<(String, bool), SyntaxNode>,
        ctx: &mut RuleCtx,
    ) -> Option<()> {
        let source = source.as_ref()?;
        let text = source.inner_string_text()?;
        let text_as_str = text.to_string();

        if let Some(old) = seen.get(&(text_as_str, type_token.is_some())) {
            let err = ctx
                .err(
                    self.name(),
                    if is_export {
                        format!("`{}` import is duplicated as export", text)
                    } else {
                        format!("`{}` import is duplicated", text)
                    },
                )
                .secondary(old, format!("`{}` is first used here", text))
                .primary(
                    source.syntax(),
                    format!("`{}` is then used again here", text),
                );
            ctx.add_err(err);
        } else {
            seen.insert(
                (text.to_string(), type_token.is_some()),
                source.syntax().clone(),
            );
        }

        None
    }
}

ts_rule_tests! {
    NoDuplicateImports::default(),
    err: {
        r#"
        import type { TypeA } from 'bla';
        import { a } from 'bla';
        import type { TypeA } from 'bla';
        "#,
    },
    ok: {
        r#"
        import type { TypeA } from 'bla';
        import { a } from 'bla';
        "#,
    }
}

rule_tests! {
    NoDuplicateImports::default(),
    err: {
        r#"
        import foo from "bla";
        import * as bar from "bla";
        "#,
        /// ignore
        r#"
        import { foo } from "bla";
        import { bar } from 'bla';
        "#,
    },
    ok: {
        /// ignore
        r#"
        import { foo } from "bla";
        export { foo } from "bla";
        "#,
    }
}

rule_tests! {
    include_exports_valid,
    include_exports_invalid,
    NoDuplicateImports { include_exports: true },
    err: {
        /// ignore
        r#"
        import { foo } from "bla";
        export { foo } from "bla";
        "#,
    },
    ok: {
        /// ignore
        r#"
        import { foo } from "bla";
        export { foo };
        "#,
    }
}
