# Getting Started

::: warning
RSLint is still in early development, so expect bugs
:::

RSLint is a tool for finding and fixing problematic productions in ECMAScript/JavaScript code. RSLint helps you fix potential errors as well as consistently
enforce style and good practices. RSLint is similar to ESLint with some major differences:

- RSLint is written in Rust and therefore very fast.
- RSLint uses a [`custom parser`](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_parser/src) to parse JavaScript.
- RSLint uses a CST (concrete syntax tree) as well as untyped nodes to evaluate patterns in code.
- RSLint can lint any code no matter how wrong it is.
- RSLint groups rules into distinct groups.
- RSLint

# Installation

You must have cargo installed on your machine, then git clone the repository, and either build the binary and run it, or use cargo run directly.

```sh
git clone https://github.com/RDambrosio016/RSLint.git
cd RSLint
cargo run --release -- ./glob/pattern.js
```

You can also directly install the rslint cli:

```sh
cargo install rslint_cli
rslint_cli ./glob/pattern.js
```

If you do not have rust installed you can find prebuilt binaries for every release [here](https://github.com/RDambrosio016/RSLint/releases).

# Running in VSC

RSLint has a basic LSP and VSC extension, it is however not yet published as it is very early in development. If you would like to use it you
must first install the lsp in the project with `cargo install --path crates/rslint_lsp`. Then, open `editors/vscode` with visual studio code and press
`f5` to start a new vscode instance with the extension. Any `js` or `mjs` files you open will now be actively linted. The linter runs on the fly, not on save.

# Sponsoring

Consider supporting RSLint's development through our [Open Collective](https://opencollective.com/rslint). Any help is greatly appreciated.
