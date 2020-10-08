# Project Layout

RSLint is a monorepo (workspace), this allows us to distinctly separate logic and redistibute some of that logic as crates.
The actual bulk of linting is done in `rslint_core`, which is where all rules are implemented.

## Linting Flow

Linting consists of distinct steps, which are separated into crates to avoid a single messy crate.

This is a simplified summary of the linting process:

```
CLI parsing --> File walker --> Lexing --> Parsing --> Linting --> Result collection --> Diagnostic emission
    |                                                     |
    v                                                     v
Config parsing                                      Scope analysis
```

## rslint_cli

Linting starts with [`rslint_cli`](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_cli). Its jobs include the following:

- CLI arg parsing
- Configuration parsing
- File loading
- Calling the lint runner
- Emitting results/diagnostics

The crate starts by trying to load the configuration file, this is done by spawning a thread which will try to load
the `rslintrc.toml` file and returning a handle to it. Then it will instantiate the `FileWalker`, this is the structure
which manages loading files from disk, it does so concurrently by spawning one thread per file being loaded.

From then on it will collect the rules it needs to run from the config, and it will call the `lint_file` function from the `rslint_core` crate.

## rslint_core

This is the crate where all the magic happens, it contains every rule plus general utilities. `rslint_core` should never know about CLI logic.
The point of separating the core linter logic and the cli logic is to allow rust users to run the linter on pieces of code without having to worry
about the overhead of CLI/binary logic.

The core structure (well, trait) of `rslint_core` is `CstRule`. `CstRule` is a trait describing a rule which is run on the concrete syntax tree of a single
file. A rule can operate on nodes, tokens, or the root node of a tree. You will notice there is no mention of a visitor anywhere, you can learn why [here](./syntax.md).

It is **very** important that each rule be Send and Sync, because rules are run highly parallel. (the linter will eventually have a type of rule which is run on all of the CSTs of each file). Most rules run on nodes, therefore use `check_node`, however, some need to check the token or the root, which is why `check_token` and `check_root` exist.

As for running rules, the linter starts by taking the source code, and parsing it into a syntax tree using [`rslint_parser`](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_parser). It then uses rayon to run every rule in the `CstRuleStore` in parallel. Syntax nodes are not thread safe because they are backed by an Rc, however we can pass a pointer to a green tree and reconstruct the root syntax node. For each descendant in the root node the linter runs each rule in parallel, each rule gets a new context instance, this instance will be used by the rule to attach diagnostics to it.

## rslint_lexer

`rslint_lexer` is a standard JavaScript lexer, it can also do ANSI syntax highlighting. The lexer is lookup table based, and it contains a decent amount of unsafe to make it stupidly fast, if you are working on it you should be quite careful and include safety comments for any unsafe usage.

## rslint_parser

The parser is the heart of a linter, you can find a complete list of features but the most distinguished ones are:

- Speed
- Complete error recovery
- Simple parsing without explicitly handling AST nodes
- Rich utilities
- (WIP) incremental reparsing

The concepts for the parser and the syntax it produces are taken from [rust analyzer](https://github.com/rust-analyzer/rust-analyzer) and its syntax library, [rowan](https://docs.rs/rowan/0.10.0/rowan/index.html). You can read about the syntax concepts [here](./syntax.md) and in the [rust analyzer docs](https://github.com/rust-analyzer/rust-analyzer/blob/master/docs/dev/syntax.md).

The parser is by no means tied to the linter, it knows nothing about the linter, therefore you are free to reuse it for any projects.

## rslint_scope

`rslint_scope` is the scope analysis library. It produces a scope which contains items such as declared variables, variable refs, and child scopes. The scope analysis library tries to be as detailed as possible, most items are wrapped in Rcs or Weaks which allow it to refer back to declarations or parent scopes.
