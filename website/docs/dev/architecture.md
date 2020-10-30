# Architecture

At the highest level, all that RSLint does is it parses JavaScript source code then checks the parsed representation for problems. This is the concept most linters rely on. RSLint however employs various concepts which make it stand out both as a user and as a contributor, these concepts may be foreign at first, but you will soon understand why they are used.

## High level flow

RSLint's linting flow can be categoried into several distinct steps as shown below:

```
CLI args -> Load Config -> Load Files -> Call Runner -> Parse File -> Run Rules -> Maybe Autofix -> Report Diagnostics

rslint_cli  rslint_config  rslint_cli    rslint_core   rslint_parser  rslint_core  ----------- rslint_cli ------------
```

Each step is roughly separated to allow us to reuse some of the concepts from each crate to avoid code duplication.

## Project structure

RSLint is laid out as a monorepo, a structure you may be familiar with, Rust monorepos are more specifically refered to as workspaces. RSLint separates distinct steps into separate crates (e.g. CLI, parsing, lexing, and the core), this allows us to reuse and distibute individual parts of the linter for use outside of RSLint. Let's briefly go through what each crate is responsible for.

::: tip

If you come from a JavaScript background, Rust crates are essentially the same as packages.

:::

- `rslint_cli`: This crate contains all of the logic used in the rslint CLI and is one of the only crates which is rslint-specific.

- `rslint_config`: This crate houses all of the logic for `rslintrc.toml`, it is used by both `rslint_cli` and `rslint_lsp`.

- `rslint_core`: This is where all of the linting takes place and where all of the rules are, the core runner contains no `rslint_cli` specific logic, therefore you could use it as a crate and embed it into your rust programs easily.

- `rslint_errors`: This crate contains all of the error (diagnostic) logic shared across every crate, it has a single `Diagnostic` type which is used everywhere in RSLint. It also has multiple formatters which you may have read about [here](../guide/formatters.md).

- `rslint_lexer`: This is an extremely fast ECMAScript lexer which produces tokens from source code. It also contains logic for syntax highlighting code with ANSI coloring which is used for the `explain` subcommand.

- `rslint_lsp`: This is the (unfinished) language server protocol implementation, it will be used for the VSC, Vim, Emacs, and Atom extensions.

- `rslint_macros`: A simple crate for proc macros (procedural macros) used by `rslint_core`.

- `rslint_parser`: This is the heart of the linter, it is an ECMAScript 2021 parser which produces a lossless syntax tree and can produce an AST from any source code using error recovery. It is unconventional in the techniques it uses, but we will cover that later on.

- `rslint_syntax`: This houses the `SyntaxKind` enum which is used by both `rslint_lexer` and `rslint_parser` (and consumed by other crates through `rslint_parser`)

- `rslint_text_edit`: This houses simple structs and methods to represent and apply text edits to strings. It is shared between `rslint_core` (for autofix), `rslint_errors` (for suggestions), `rslint_lsp` (for fix commands), and soon `rslint_parser` (for incremental reparsing).

## Parallelism

RSLint employs a high level of parallelism to achieve high speeds. This includes:

- Loading files in parallel
- Linting files in parallel
- Running rules in parallel

This means each rule must be able to be shared across threads as well as be able to be sent across threads. Or in rust terms,
it must be `Send + Sync`. This is not the end of the line for more "contextual" rules however! RSLint (soon) defines a different rule type which
can act on the linted files as a whole.
