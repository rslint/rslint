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
        let mut seen: HashMap<String, SyntaxNode> = HashMap::default();

        for child in node.children() {
            if let Some(import) = child.try_to::<ImportDecl>() {
                self.process_node(&import.source(), false, &mut seen, ctx);
            } else if self.include_exports {
                if let Some(export) = child.try_to::<ExportDecl>() {
                    self.process_node(&export.source(), true, &mut seen, ctx);
                }
            }
        }

        None
    }
}

impl NoDuplicateImports {
    fn process_node(&self, source: &Option<Literal>, is_export: bool, seen: &mut HashMap<String, SyntaxNode>, ctx: &mut RuleCtx) {
        if let Some(source) = source {
            if let Some(text) = source.inner_string_text() {
                let text_as_str = text.to_string();
                if let Some(old) = seen.get(&text_as_str) {
                    let err = ctx
                        .err(
                            self.name(),
                            if is_export {
                                format!("`{}` import is duplicated as export", text_as_str)
                            } else {
                                format!("`{}` import is duplicated", text_as_str)
                            }
                        )
                        .secondary(old, format!("`{}` is first used here", text_as_str))
                        .primary(
                            source.syntax(),
                            format!("`{}` is then used again here", text_as_str),
                        );
                    ctx.add_err(err);
                } else {
                    seen.insert(text.to_string(), source.syntax().clone());
                }
            }
        }
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