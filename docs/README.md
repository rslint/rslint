# RSLint

RSLint is an extremely fast JavaScript linter written in Rust focusing on ease of use, customizability, and speed. RSLint
helps you find and fix error-prone productions in your code as well as enforce good practices.

## Docs

You can also find documentation about individual rules [here](./rules). There are also dev docs [here](./dev) if you are interested in contributing to RSLint or knowing more about how it works.

## Installation

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

## Running in VSC

RSLint has a basic LSP and VSC extension, it is however not yet published as it is very early in development. If you would like to use it you
must first install the lsp in the project with `cargo install --path crates/rslint_lsp`. Then, open `editors/vscode` with visual studio code and press
`f5` to start a new vscode instance with the extension. Any `js` or `mjs` files you open will now be actively linted. The linter runs on the fly, not on save.

## Note

⚠️ RSLint is still in early development, so expect bugs ⚠️
