# RSLint

A  (WIP) JavaScript linter written in Rust designed to be as fast as possible, customizable, and easy to use.

The project is in early development, there will be bugs and weird productions. If you find any bugs feel free to submit an issue 👍.

# Installation

You must have cargo installed on your machine, then git clone the repository, and either build the binary and run it, or use cargo run directly.

```sh
git clone -b dev https://github.com/RDambrosio016/RSLint.git
cd RSLint
cargo run --release -- ./glob/pattern.js
```

# Configuration 

Please see the [docs](./docs/config.md) for linter configuration details. 

# Rules 

You can find rule documentation [here](./docs/rules).

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
