use std::collections::HashMap;

use crate::rule_prelude::*;
use SyntaxKind::*;

declare_lint! {
    /**
    Disallow Self Assignment

    Because self assignments have no effects, this is mostly an indicator for errors.

    ## Invalid Code Examples

    ```js
    foo = foo;
    ```

    ```js
    [a, b] = [a, b];
    ```

    ```js
    [a, ...b] = [x, ...b];
    ```

    ```js
    ({a, b} = {a, x});
    ```

    ## Valid Code Examples

    ```js
    foo = bar;
    ```

    ```js
    [a, b] = [b, a];
    ```

    ```js
    obj.a = obj.b;
    ```

    */
    #[derive(Default)]
    NoSelfAssign,
    errors,
    tags(Recommended),
    "no-self-assign"
}

#[typetag::serde]
impl CstRule for NoSelfAssign {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if node.kind() == ASSIGN_EXPR {
            let mut children = node.children().collect::<Vec<SyntaxNode>>();
            if children.len() != 2 {
                return None;
            }
            let right = children.pop().unwrap();
            let left = children.pop().unwrap();

            match (left.kind(), right.kind()) {
                (NAME_REF, NAME_REF) => {
                    if left.lexical_eq(&right) {
                        self.self_assign_error(&left, &right, ctx);
                    }
                }
                (ARRAY_PATTERN, ARRAY_EXPR) => {
                    for (left, right) in left.children().zip(right.children()) {
                        if (left.kind() == SINGLE_PATTERN && right.kind() == NAME_REF
                            || left.kind() == REST_PATTERN && right.kind() == SPREAD_ELEMENT)
                            && left.lexical_eq(&right)
                        {
                            self.self_assign_error(&left, &right, ctx);
                        }
                    }
                }
                (OBJECT_PATTERN, OBJECT_EXPR) => {
                    let mut seen: HashMap<String, SyntaxNode> =
                        HashMap::with_capacity(left.children().count());
                    left.children().for_each(|child| {
                        seen.insert(child.text().to_string(), child);
                    });

                    right.children().for_each(|child| {
                        let text = child.text().to_string();
                        if let Some(other) = seen.get(&text) {
                            if other.kind() == SINGLE_PATTERN && child.kind() == IDENT_PROP {
                                self.self_assign_error(other, &child, ctx);
                            }
                        }
                    });
                }
                _ => {}
            };
        }

        None
    }
}

impl NoSelfAssign {
    fn self_assign_error(&self, left: &SyntaxNode, right: &SyntaxNode, ctx: &mut RuleCtx) {
        let err = ctx
            .err(
                self.name(),
                format!("`{}` is assigned to itself", left.text()),
            )
            .secondary(right, format!("`{}` is used here", left.text()))
            .primary(
                &left,
                format!("`{}` is then self-assigned here", left.text()),
            );
        ctx.add_err(err)
    }
}

rule_tests! {
  NoSelfAssign::default(),
  err: {
      /// ignore
    "foo = foo",
    /// ignore
    "[a, b] = [a, b]",
    /// ignore
    "[a, ...b] = [x, ...b]",
    "[a, b, c] = [c, b, a]",
    /// ignore
    "({a, b} = {a, x})",
    "({b, a} = {a, b})"
  },
  ok: {
    /// ignore
    "foo = bar",
    /// ignore
    "[a, b] = [b, a]",
    "let foo = foo",
    "[foo = 1] = [foo]",
  }
}
