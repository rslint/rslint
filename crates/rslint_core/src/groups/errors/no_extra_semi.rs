use crate::rule_prelude::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow unneeded semicolons.

    Unneeded semicolons are often caused by typing mistakes, while this is not an error, it
    can cause confusion when reading the code. This rule disallows empty statements (extra semicolons).

    ## Invalid Code Examples

    ```js
    if (foo) {
        ;
    }
    ```

    ```js
    class Foo {
        constructor() {};
    }
    ```
    */
    #[derive(Default)]
    NoExtraSemi,
    errors,
    tags(Recommended),
    "no-extra-semi"
}

const ALLOWED: [SyntaxKind; 8] = [
    FOR_STMT,
    FOR_IN_STMT,
    FOR_OF_STMT,
    WHILE_STMT,
    DO_WHILE_STMT,
    IF_STMT,
    LABELLED_STMT,
    WITH_STMT,
];

#[typetag::serde]
impl CstRule for NoExtraSemi {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == SyntaxKind::EMPTY_STMT
            && node
                .parent()
                .map_or(true, |parent| !ALLOWED.contains(&parent.kind()))
        {
            ctx.fix().delete(node);
            let err = ctx
                .err(self.name(), "unnecessary semicolon")
                .primary(node, "")
                .suggestion(node, "delete the semicolon", "", Applicability::Always);

            ctx.add_err(err);
        }
        None
    }
}

rule_tests! {
  NoExtraSemi::default(),
  err: {
    ";",
    "
      if (foo) {
        ;
      }
      ",
    "
      class Foo {
        ;
      }
      ",
    "class Foo extends Bar {
        constructor() {};
      }
      "
  },
  ok: {
    "
      class Foo {}
      "
  }
}
