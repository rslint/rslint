# RSLint

A  (WIP) JavaScript linter written in Rust designed to be as fast as possible, customizable, and easy to use.

# Installation

You must have cargo installed on your machine, then git clone the repository, and either build the binary and run it, or use cargo run directly.

```sh
git clone https://github.com/RDambrosio016/RSLint.git
cd RSLint
cargo run --release -- ./glob/pattern.js
```

# Limitations

The project is in very early development, there will be bugs and weird productions. If you find any bugs feel free to submit an issue 👍.
**RSLint currently only works on ECMAScript 5 due to the parser (rslint-parse) being still in development**, however, support for further ES versions and JSX plus TypeScript is planned in the future.

# Speed

RSLint is designed to be the fastest JavaScript linter ever made, it accomplishes this in various ways: 
  - Using a custom fast parser which retains whitespace
  - Using a lookup table and trie based lexer for parsing
  - Using separate distinct threads for splitting up IO bound tasks such as loading files
  - Linting each file in parallel
  - Running each rule from every group in parallel over the concrete syntax tree
  - Caching lint results by default

# Caching 

RSLint will cache results by default, this is done through a `.rslintcache` binary file. The file is protected in various ways to avoid erroneous runs: 
  - The file is in a binary format which will easily fail serialization on random edits  
  - The file stores the time it was created at, and checks it at runtime, if it does not match then the cache is rejected as "poisoned"

# Implementing new rules

If you would like to implement a new rule there are a few steps you must go through. You can either use the `cst_rule` macro then implement visit for the visitor structure generated (see rules like `no_empty.rs` for examples), or you can make a struct and impl `CstRule` manually. Don't forget to add the rule to the mod file of the group you chose!

# Roadmap

RSLint's goal is to provide extremely fast and user friendly linting for the whole js ecosystem. There are tons of things to do to bring it up to par with existing linters. This is a list of planned features and things to do ranked in order of highest to lowest priority (this is by no definition final, things will change):

[ ] Refine caching system to include rules run and automatically adding to .gitignore  
[ ] More tests for rslint-parse statement subparsers  
[ ] Scope analysis  
[ ] Implementation of ESLint reccomended rules  
[ ] ES6+ Support (mostly just parser work)  
[ ] Benchmarks  
[ ] Markdown support  
[ ] Config files  
[ ] Rule options  
[ ] Prebuilt binary generation  
[ ] Neon bindings to allow for installation via npm with a build script  
[ ] JSX Support  
[ ] TS Support  
[ ] JS Plugins  
[ ] WASM Plugins  
[ ] Documentation website  
