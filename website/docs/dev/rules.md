# Implementing rules

## Implementing a rule

Let's walk through how we would implement a rule, for this example let's implement eslint's `no-extra-semi`.

First, we must decide the correct group for the rule, we will place it in `errors` for this example.
Therefore, lets create a file under `rslint_core/src/groups/errors` called `no_extra_semi.rs`.

Then, go to the `mod.rs` file of the group, and at the end of the `group!` declaration add the rule:

```rust
no_extra_semi::NoExtraSemi
```

Don't worry if you get errors, theyll be fixed soon.

RSLint defines a [rule_prelude](https://github.com/rslint/rslint/blob/master/crates/rslint_core/src/rule_prelude.rs) module, which contains commonly used
items by rules, which saves a ton of painful imports.

the prelude includes a `declare_lint` macro, this macro is a way of easily declaring a new rule, it is also used by
the docgen script to generate user facing documentation. The macro starts with attributes for the struct generated for the rule.
You'll have to either derive default or implement it yourself. These attributes can also include a doc comment which will be used by
docgen for the user facing docs, we will get back to that later.

The next item is just the struct name, which is just the rule name but pascal case, `NoExtraSemi` for this example. Then the name of the group,
`errors` in this case. And finally, the kebab case code for this rule, this must be unique, `no-extra-semi` in this case.

For this rule we won't define any config fields, but you may do so after the code, including any private fields for the struct. Each config field can take attributes including doc comments which will be used by docgen for the user facing docs (to make a config fields table). Don't worry about using camel case for the config fields, the macro will automatically rename all fields to camel case.

The lint declaration would look like this:

```rust
declare_lint! {
  #[derive(Default)]
  NoExtraSemi,
  errors,
  "no-extra-semi"
}
```

### Implementing CstRule

The next step is to implement the `CstRule` trait, youll have to first use the `#[typetag::serde]` attribute on the impl. The reasoning behind this is rslint does configuration by deserializing trait objects themselves, which can only be done with typetag:

```rust
#[typetag::serde]
impl CstRule for NoExtraSemi {
  /* */
}
```

We want to check each `EmptyStatement`, therefore we will want to implement `check_node` in `CstRule`. The function signature is pretty simple, it takes a reference to the node, a mutable context, and returns a `Option<()>`. The context is what we will add diagnostics to, and the return type is simply a hack to be able to return early with `?`, since everything in the AST is optional it can get a little messy without it.

```rust
#[typetag::serde]
impl CstRule for NoExtraSemi {
  fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
    /* */
    None
  }
}
```

We simply want to check any empty statement node, this is very simple, we can just add an if statement checking if the node kind is `SyntaxKind::EMPTY_STMT`:

```rust
#[typetag::serde]
impl CstRule for NoExtraSemi {
  fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
    if node.kind() == SyntaxKind::EMPTY_STMT {
      /* */
    }
    None
  }
}
```

This is where untyped nodes shine, we want to allow empty statements if the parent is a loop, labelled statement, or with statement. We can very easily do this by making a const of syntax kinds we will check. `SyntaxKind` is an enum which lists every possible kind of node or token. For convenience we will add a `use SyntaxKind::*`, all syntax kinds are screaming snake case, so there should not be any conflicts.

```rust
const ALLOWED: [SyntaxKind; 8] = [
  FOR_STMT,
  FOR_IN_STMT,
  FOR_OF_STMT,
  WHILE_STMT,
  DO_WHILE_STMT,
  IF_STMT,
  LABELLED_STMT,
  WITH_STMT
];
```

We can then simply check if the parent is allowed using `map_or`:

```rust
#[typetag::serde]
impl CstRule for NoExtraSemi {
  fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
    if node.kind() == SyntaxKind::EMPTY_STMT && node.parent().map_or(true, |parent| !ALLOWED.contains(&parent.kind())) {
      /* */
    }
    None
  }
}
```

For reporting diagnostics we can use the `DiagnosticBuilder`. `ctx` has a util method for making a new builder called `ctx.err()`. The method
takes the code of the diagnostic (the rule code or `self.name()` in our case), and the primary message. For the primary message we will use `Unnecessary semicolon`. The primary message should say what is wrong in full.

```rust
#[typetag::serde]
impl CstRule for NoExtraSemi {
  fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
    if node.kind() == SyntaxKind::EMPTY_STMT && node.parent().map_or(true, |parent| !ALLOWED.contains(&parent.kind())) {
      let err = ctx.err(self.name(), "Unnecessary semicolon");
      /* */
    }
    None
  }
}
```

Simple errors with only a message are boring and unhelpful, we want to point to the location of the error, and add notes and labels saying what is wrong. We can do this using the `primary`, `secondary`, and `note` methods on the builder. `primary` and `secondary` take a range for the label and a message. `primary` is the primary (red) label and location of the error, there should only be one of these. `secondary` labels are blue labels which provide more context, these are used for explaining more complex errors or providing context, if you want to see a practical use of them look at `for-direction`.

For this example let's add a primary label which tells the user to delete the semicolon:

```rust
#[typetag::serde]
impl CstRule for NoExtraSemi {
  fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
    if node.kind() == SyntaxKind::EMPTY_STMT && node.parent().map_or(true, |parent| !ALLOWED.contains(&parent.kind())) {
      let err = ctx.err(self.name(), "Unnecessary semicolon")
        .primary(node.trimmed_range(), "help: delete this semicolon");

      ctx.add_err(err);
    }
    None
  }
}
```

That's it for the implementation!

## Testing

For testing you can use the `rule_tests!` macro, which uses straight forward syntax. It starts with the rule to check, then an `err: {}` block, and an `ok: {}` block. Each block consists of comma separated string literals which will either be checked for linting failure or for linting success.

Each test will be used in `more incorrect examples` and `more correct examples` section in user facing docs by docgen. You can put `/// ignore` above the literal
to have docgen not show it. Don't worry about indentation or trailing or leading whitespace, docgen will fix both of those issues when generating.

```rust
rule_tests! {
  NoExtraSemi::default(),
  err: {
    /// ignore
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
```

## Documentation

For documentation, it is done through the lint_declaration macro. All you need to do is add a doc comment before the struct name. Documentation is decently large, so you should generally use `/** */` comments over `///` comments. You must include a small description of the rule, then a newline for docgen to use for the top level rules table for each group. Each rule should also generally include an `## Invalid Code Examples` header.

let's add docs for our rule:

````rs
declare_lint! {
  /**
  Disallow unneeded semicolons.

  Unneeded semicolons are often caused by typing mistakes, while this is not an error, it
  can cause confusion when reading the code. This rule disallows empty statements (extra semicolons).

  ## Invalid Code Examples

  ```ignore
  if (foo) {
    ;
  }
  ```

  ```ignore
  class Foo {
    constructor() {};
  }
  ```
  */
  #[derive(Default)]
  NoExtraSemi,
  errors,
  "no-extra-semi"
}
````

And finally, run the docgen with `cargo docgen` or `cargo xtask docgen`. This will create the appropriate file in the rules docs and update readmes.
