# Untyped Trees

RSLint deeply relies on the concept of an untyped tree as opposed to a traditional purely typed AST. This concept allows RSLint to easily do node traversal and token-based linting without any extra structures. However, it often confuses contributors as it is very unusual compared to a normal AST.

## The Problem

One of the most important things linter rules need to do is traverse the parents of a node. This is very simple for a dynamically typed, garbage collected language, because you don't need a central representation of "any" AST node, and you do not have to worry about reference cycles. Alas, this is not simple to do with Rust, Rust is statically typed and does not have a garbage collector. No garbage collector means it is harder to represent the parent of a node from the node itself. Dropping the parent node could mean a use-after-free if the child still exists, which is very very bad for a language which aims to be fully sound.

Moreover, ASTs are lossy, as implied by the name (_Abstract_ Syntax Tree), therefore we are out of luck if we want to get to the raw tokens just from the AST, we need a separate token store. ESLint's `SourceCode` is an example of this. But this makes for an ugly interface because it separates the nodes from the tokens, therefore any interactions between the nodes and the tokens get a bit clunky. And it is one more structure to implement and to maintain.

And finally, what if we just want to give a structure an abstract node which could be any node? We could use `Box<dyn Any>`, however, that incurs another performance penalty for downcasting, and it does not solve the issue of us wanting to crawl the parents/siblings/descendants of the node no matter the node kind.

## The solution

_What if we could step down to an untyped interface which retains all tokens?_

At the highest level, RSLint's AST is the same as normal ASTs, except what would normally be struct fields are now struct methods. However, RSLint's AST at a deeper level is much much different.

RSLint's AST is composed of multiple levels of abstraction, the lowest of these being the green tree. The green tree is a DST (Dynamically Sized Type) and is fully immutable. Green trees are a single allocation laid out as such:

```
*-----------+------+----------+------------+--------+--------+-----+--------*
| ref_count | kind | text_len | n_children | child1 | child2 | ... | childn |
*-----------+------+----------+------------+--------+--------+-----+--------*
```

This tree is fully lossless, all of the source code is represented in it, including whitespace and comments. You never mess with the green tree directly, it is simply used behind a pointer by the next abstraction layer, [`SyntaxNodes`](https://docs.rs/rowan/0.10.0/rowan/api/struct.SyntaxNode.html).

Syntax nodes represent some node in the green tree which could be of any kind. They are simply an [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) which points to some node data, which further points to some place in the green tree. What does this mean for us? Well this allows us to do many different things, for one, `Arc`s are cheap to clone, which means we can give nodes to anything which wants them without worrying about the cost. Moreover, it allows us to transfer any ast nodes without having to explicitly say what kind of node they are. One place where this is exploited is root nodes. Root AST nodes could either be a module or a script, but we don't have to worry because we can just pass an untyped node.

Another aspect of RSLint's tree is it contains all of the tokens, including whitespace and comment tokens. We exploit this functionality throughout the whole linter, this includes:

- Lexical equality (token based equality)
- Directives scoped to nodes
- Syntax highlighting
- Stylistic linting
- Easy autofix changes

_But what about a typed interface?_

A linter cannot fully operate on just an untyped interface, we need typed nodes to easily traverse the tree and find what we need. The solution for this is simple yet powerful.

AST Nodes are simply structs with a single field, that field being a syntax node (untyped node). Then, when we want to get some data, we crawl the children of the root node and cast the data. For example, say we have this JavaScript code:

```js
if (true) {
  /* */
} else {
  /* */
}
```

The untyped representation of this can be displayed as such:

```js
IF_STMT@0..38
  IF_KW@0..2 "if"
  WHITESPACE@2..3 " "
  CONDITION@3..9
    L_PAREN@3..4 "("
    LITERAL@4..8
      TRUE_KW@4..8 "true"
    R_PAREN@8..9 ")"
  WHITESPACE@9..10 " "
  BLOCK_STMT@10..21
    L_CURLY@10..11 "{"
    WHITESPACE@11..14 "\n  "
    COMMENT@14..19 "/* */"
    WHITESPACE@19..20 "\n"
    R_CURLY@20..21 "}"
  WHITESPACE@21..22 " "
  ELSE_KW@22..26 "else"
  WHITESPACE@26..27 " "
  BLOCK_STMT@27..38
    L_CURLY@27..28 "{"
    WHITESPACE@28..31 "\n  "
    COMMENT@31..36 "/* */"
    WHITESPACE@36..37 "\n"
    R_CURLY@37..38 "}"
```

If we wanted to get the condition of this If statement then we can simply crawl the children of the node and search for a node which matches the `CONDITION` kind. Then we simply cast the node to the Condition AST node. The conversion from untyped to typed is a simple check to see if the node's kind matches. The conversion from typed to untyped is free and can be done using the `syntax()` method on every AST node.

This Untyped <---> Typed representation allows us to cleanly implement many rules, such as [no-await-in-loop](https://rslint.org/rules/errors/no-await-in-loop.html):

```rust
fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
    if let Some(err_node) = node.children().find(|node| node.kind() == AWAIT_EXPR) {
        for ancestor in node.ancestors() {
            match ancestor.kind() {
                FN_DECL | FN_EXPR | ARROW_EXPR => return None,
                FOR_OF_STMT if ancestor.to::<ast::ForOfStmt>().await_token().is_some() => {
                    return None
                }
                _ => {}
            }

            if ancestor.is_loop() {
                // notice the pretty errors and builder :)
                let err = ctx.err(self.name(), "Unexpected `await` in loop")
                    .primary(err_node, "this expression causes the loop to wait for the promise to resolve before continuing")
                    .footer_note("the promises are resolved one after the other, not at the same time")
                    .footer_help(format!("try adding the promises to an array, then resolving them all outside the loop using `{}`", color("Promise.all(/* promises */)")));

                ctx.add_err(err);
                return None;
            }
        }
    }
    None
}
```

In this example, we must crawl the ancestors of the node to see if we are actually in a loop. This is trivial to do, we can simply use `node.ancestors()` and iterate through the nodes produced from that iterator.
