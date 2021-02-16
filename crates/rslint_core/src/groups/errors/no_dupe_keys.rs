use crate::rule_prelude::*;
use ast::*;
use rslint_parser::{NodeOrToken, SyntaxToken};

declare_lint! {
    /**
    Disallow duplicate keys in object literals.

    Object literals allow keys to be declared multiple times, however this causes unwanted
    behavior by shadowing the first declaration.

    ## Invalid Code Examples

    ```js
    let foo = {
        bar: 1,
        baz: 2,
        bar: 3
    }
    ```
    */
    #[derive(Default)]
    NoDupeKeys,
    errors,
    tags(Recommended),
    "no-dupe-keys"
}

// FIXME: this should consider the value of a number key, aka 1 and 0x1
#[typetag::serde]
impl CstRule for NoDupeKeys {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if let Expr::ObjectExpr(obj) = node.try_to()? {
            // String based equality is evil! using tokens is the correct way,
            // because it counts `foo . bar` and `foo.bar` as the same expr, string eq does not.
            let mut declared: Vec<(Vec<SyntaxToken>, std::string::String)> = vec![];

            for prop in obj.props().filter_map(|prop| prop.key_element()) {
                let tokens = match prop.clone() {
                    NodeOrToken::Node(node) => node.lossy_tokens(),
                    NodeOrToken::Token(tok) => vec![tok],
                };

                let text = match prop {
                    NodeOrToken::Node(node) => node.trimmed_text().to_string(),
                    NodeOrToken::Token(tok) => tok.text().to_string(),
                };
                if let Some((found, text)) = declared
                    .iter()
                    .find(|(decl_tokens, _)| util::string_token_eq(decl_tokens, &tokens))
                {
                    let range = util::token_list_range(found);

                    let err = ctx
                        .err(
                            self.name(),
                            format!("duplicate property definition `{}`", text),
                        )
                        .secondary(range, format!("`{}` is first declared here", text))
                        .primary(
                            util::token_list_range(tokens),
                            format!("`{}` is then redeclared here", text),
                        );

                    ctx.add_err(err);
                } else {
                    declared.push((tokens, text));
                }
            }
        }
        None
    }
}

rule_tests! {
    NoDupeKeys::default(),
    err: {
        "
        let foo = {
            bar,
            baz,
            get bar() {

            }
        }
        ",
        "
        let foo = {
            get bar() {

            },
            set bar(foo)  {

            }
        }
        "
    },
    ok: {
        "
        let foo = {
            bar: {
                bar: {},
                baz: 5
            },
            baz: {}
        }
        "
    }
}
