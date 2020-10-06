# RSLint

A  (WIP) JavaScript linter written in Rust designed to be as fast as possible, customizable, and easy to use.

The project is in early development, there will be bugs and weird productions. If you find any bugs feel free to submit an issue 👍.

# Installation

You must have cargo installed on your machine, then git clone the repository, and either build the binary and run it, or use cargo run directly.

```sh
git clone https://github.com/RDambrosio016/RSLint.git
cd RSLint
cargo run --release -- ./glob/pattern.js
```

You can also directly install rslint_cli:

```sh
cargo install rslint_cli
rslint_cli ./glob/pattern.js
```

# Running in VSC 

RSLint has a basic LSP and VSC extension, it is however not yet published as it is very early in development. If you would like to use it you 
must first install the lsp in the project with `cargo install --path rslint_lsp`. Then, open `editors/vscode` with visual studio code and press
`f5` to start a new vscode instance with the extension. Any `js` or `mjs` files you open will now be actively linted. The linter runs on the fly, not on save,
if you find any panics/errors please report them as they are a bug.

# Configuration 

Please see the [docs](./docs/config.md) for linter configuration details. 

# Rules 

You can find rule documentation [here](./docs/rules).

# Contributing

RSLint's syntax is unlike any other linter, therefore it may be very foreign to people coming from ESTree-like parsers/linters. If you want to learn more about RSLint's syntax tree implementation and how to implement rules you should read the [dev docs](https://github.com/RDambrosio016/RSLint/tree/master/docs/dev). You can further read both the [rslint_parser](https://docs.rs/rslint_parser/0.1.0/rslint_parser/) docs, and the [rslint_core](https://docs.rs/rslint_core/0.1.0/rslint_core/) docs, specifically the `ast` module and `SyntaxNodeExt` methods.

# Differences from other linters 

## Implemented 

- Unbeatably fast 
- Highly parallelized (files linted in parallel, rules run in parallel, nodes could be traversed in parallel in the future) 
- Rich, cross-platform, colored diagnostics with secondary labels, primary labels, and notes 
- Lossless untyped node and token driven linting allowing easy traversal of the syntax tree from any node 
- Automatic docgen for rule documentation removing the need for writing rustdoc docs and user facing docs 
- Distinctly grouped rules 
- Rule examples generated from tests 
- Easy macros for generating rule declarations and config fields 
- No need for dealing with script/module or ecma versions, linter deduces source type and assumes latest syntax 
- No need for a configuration file 
- Completely error tolerant and fast parser 
- Lossless tree used for stylistic linting 
- TOML config (json will be allowed too), (TOML implemented, json not yet)
- Incremental reparsing and native file watching support (WIP, see #16)

## Planned 

- Global config 
- SSR-like templates for node matching and autofix  
- Autofix without requiring reruns of all rules 
- WASM builds 

# Speed

RSLint is designed to be the fastest JavaScript linter ever made, it accomplishes this in various ways: 
  - Using a custom fast parser which retains whitespace
  - Using a lookup table and trie based lexer for parsing
  - Using separate distinct threads for splitting up IO bound tasks such as loading files
  - Linting each file in parallel
  - Running each rule from every group in parallel over the concrete syntax tree
  - (WIP) linting each untyped node in parallel
  - (WIP) Incrementaly reparsing and relinting files
  - (WIP) Having native file watching support using incremental parsing

# Roadmap

RSLint's goal is to provide extremely fast and user friendly linting for the whole js ecosystem. There are tons of things to do to bring it up to par with existing linters. This is a list of planned features and things to do ranked in order of highest to lowest priority (this is by no definition final, things will change):

- [ ] Scope analysis (WIP)  
- [ ] Tests for parser, including test262
- [ ] Implementation of ESLint reccomended rules  
- [ ] Benchmarks  
- [ ] Markdown support  
- [x] Config files (partially done)
- [x] Rule options  
- [ ] Prebuilt binary generation  
- [ ] ~~Neon bindings to allow for installation via npm with a build script~~ we can do this with prebuilt binaries only
- [ ] JSX Support  
- [ ] TS Support  
- [ ] Autofix
- [ ] JS Plugins  
- [ ] WASM Plugins  
- [ ] Documentation website  
