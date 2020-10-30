# Core Runner

So we ran through lexing, parsing, and the tree, but where is the actual linter? In this page we will talk about the core runner `rslint_core` and some of the fundamental concepts it has.

## Groups

Something you may have noticed is that RSLint has the concept of rule groups. Rule groups are used to cleanly group rules to allow them to be enabled all at once. Moreover, rule groups allow us to have a better project structure by cleanly separating rule scopes and not keep 200+ rules in a single directory.

## Rules

RSLint separates rules into two traits, [`Rule`](https://docs.rs/rslint_core/0.2.2/rslint_core/trait.Rule.html) which houses metadata such as the rule name, the name of the group it is in, and its documentation, and [`CstRule`](https://docs.rs/rslint_core/0.2.2/rslint_core/trait.CstRule.html) which houses the actual implementation. The reason they are separate traits is to allow us to eventually have multiple types of rules which run on a map of all the files.

You may be expecting a visitor to go through the AST and for the rules to use the visitor, but alas you won't find one. We talked about the untyped tree in a [previous section](../untyped-trees.md), a cool thing which they allow us to do is to simply crawl the descendants of the tree and give each rule a node to check, which is precisely what we do. The rule in question then manipulates the node to get an AST node if it needs one or simply checks it as it is. For example, a rule which wants to check conditions might simply do this:

```rust
let cond = node.try_to::<Condition>()?;
```

Each `CstRule` function returns `Option<()>` to allow us to use `?` since all AST node properties are optional. Rules can also check the raw properties of the node, [`block-spacing`](https://rslint.org/rules/style/block-spacing.html) does this. It first checks if the node's kind matches either a switch statement or a block statement:

```rust
if !matches!(node.kind(), SWITCH_STMT | BLOCK_STMT) {
    return None;
}
```

Then it grabs the `{` and `}` tokens regardless of the node:

```rust
let open_token = node.token_with_kind(L_CURLY)?;
let close_token = node.token_with_kind(R_CURLY)?;
```

This kind of logic allows us to not duplicate code by having to explicitly handle both nodes. Another example of this is being able to check the condition of do_while, while, if, and switch statements without needing a visitor for each statement.

### Configuration

We do rule configuration through the rule structures themselves using typetag, typetag allows us to deserialize trait objects directly. This does however require you to put `#[typetag::serde]` over the `CstRule` implementation of every rule.

## Autofix

There are generally two ways to do autofix:

- AST node transformations (Rome)
- text-change based (ESLint, RSLint)

Each has their pros and cons, we chose text-change based because:

- AST node transformations on an immutable tree are generally harder
- We need finer control over changes to a degree where AST transformations wont work (see: style rules)
- Rust's type system allows us to add fixer methods which seamlessly work on multiple types and are very powerful
- Most fixes are small and AST transformations aren't needed

Adding autofix for a rule is very simple, it involves changing the `fixer` field of `RuleCtx` with a [`Fixer`](https://docs.rs/rslint_core/0.2.2/rslint_core/autofix/struct.Fixer.html) struct. `RuleCtx` has a [`utility method`](https://docs.rs/rslint_core/0.2.2/rslint_core/struct.RuleCtx.html#method.fix) to make a new fixer and give back a mutable reference to it so you can change it. Most of the fixer's methods rely on the [`Span`](https://docs.rs/rslint_core/0.2.2/rslint_core/trait.Span.html) trait, which is a simple trait describing items which can be converted to a range in the source code. These items include:

- `SyntaxNode`
- `SyntaxToken`
- `Range<usize>`
- `TextRange`
- `SyntaxElement`, also called `NodeOrToken`
- A reference to the above

## Indels

Autofix relies on a central structure called an `Indel`, an indel describes a single **atomic** change to the source code which does not overlap another indel, which could be a deletion or an insertion, or both. Fixers are pretty much just wrapper structs which produce indels which are applied to the source code.

But what happens if the indels overlap? It would be catastrophic if we tried applying overlapping indels, which is why we follow a specific procedure in applying indels:

- Tag all indels with the name of the rule they came from
- Go through all the indels
- If any indel overlaps with another then get all of the indels with the same tag as the overlapping indel
- Apply the indels now that they aren't overlapping
- Reparse and relint the changed code
- Repeat up to 10 times

This allows us to apply overlapping fixes by first throwing out overlapping fixes then reparsing and relinting, that way any fixes which were thrown out will hopefully be applied in the next iteration.
