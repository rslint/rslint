# RSLint

A  (WIP) JavaScript linter written in Rust designed to be as fast as possible, customizable, and easy to use.

# Installation

You must have cargo installed on your machine, then git clone the repository, and either build the binary and run it, or use cargo run directly.

```sh
git clone -b dev https://github.com/RDambrosio016/RSLint.git
cd RSLint
cargo run --release -- ./glob/pattern.js
```

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

## Planned 

- Global config 
- TOML config (json will be allowed too) 
- SSR-like templates for node matching and autofix  
- Autofix without requiring reruns of all rules 
- WASM builds 

# Limitations

The project is in very early development, there will be bugs and weird productions. If you find any bugs feel free to submit an issue 👍.
~~RSLint currently only works on ECMAScript 5 due to the parser (rslint-parse) being still in development~~ Rslint-core and rslint-parser, however, support for further ES versions and JSX plus TypeScript is planned in the future.

# Speed

RSLint is designed to be the fastest JavaScript linter ever made, it accomplishes this in various ways: 
  - Using a custom fast parser which retains whitespace
  - Using a lookup table and trie based lexer for parsing
  - Using separate distinct threads for splitting up IO bound tasks such as loading files
  - Linting each file in parallel
  - Running each rule from every group in parallel over the concrete syntax tree
  - Caching lint results by default

This is evidenced by crude benchmarks (these will be updated with proper benchmarks later) outlining the major operations and top 10 slowest rules
```
╒═══════════════════════╤════════════════════╤═══════════════╕
│ Rule                  │ Avg duration (μs)  │ Percent total │
╞═══════════════════════╪════════════════════╪═══════════════╡
│ no-constant-condition │ 27                 │ 3             │
├───────────────────────┼────────────────────┼───────────────┤
│ no-empty              │ 20                 │ 2             │
├───────────────────────┼────────────────────┼───────────────┤
│ no-duplicate-case     │ 11                 │ 1             │
├───────────────────────┼────────────────────┼───────────────┤
│ no-compare-neg-zero   │ 10                 │ 1             │
├───────────────────────┼────────────────────┼───────────────┤
│ no-cond-assign        │ 6                  │ 1             │
├───────────────────────┼────────────────────┼───────────────┤
│ no-unsafe-finally     │ 5                  │ 1             │
└───────────────────────┴────────────────────┴───────────────┘

╒═══════════════╤════════════════╤═══════════════╕
│ Operation     │ Duration (μs)  │ Percent total │
╞═══════════════╪════════════════╪═══════════════╡
│ Loading cache │ 289            │ 33            │
├───────────────┼────────────────┼───────────────┤
│ Loading files │ 277            │ 31            │
├───────────────┼────────────────┼───────────────┤
│ Linting files │ 314            │ 35            │
├───────────────┼────────────────┼───────────────┤
│ Overall       │ 888            │               │
└───────────────┴────────────────┴───────────────┘
```

If you would like to generate these tables for your run, set the `TIMING` env var to `1`  
**Note that these benchmarks are highly inaccurate, the linting process will end up being a lot faster if benchmarked over thousands of iterations**  
Furthermore, rule times are measured as the average over all files, so the total time is closer to the avg duration than `duration * files`.

# Roadmap

RSLint's goal is to provide extremely fast and user friendly linting for the whole js ecosystem. There are tons of things to do to bring it up to par with existing linters. This is a list of planned features and things to do ranked in order of highest to lowest priority (this is by no definition final, things will change):

- [ ] Refine caching system to include rules run and automatically adding to .gitignore  
- [ ] More tests for rslint-parse statement subparsers  
- [ ] Scope analysis  
- [ ] Implementation of ESLint reccomended rules  
- [ ] ES6+ Support (mostly just parser work)  
- [ ] Benchmarks  
- [ ] Markdown support  
- [ ] Config files  
- [ ] Rule options  
- [ ] Prebuilt binary generation  
- [ ] Neon bindings to allow for installation via npm with a build script  
- [ ] JSX Support  
- [ ] TS Support  
- [ ] JS Plugins  
- [ ] WASM Plugins  
- [ ] Documentation website  
