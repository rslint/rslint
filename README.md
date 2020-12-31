<div align="center">
  <h1><code>RSLint</code></h1>

  <p>
    <strong>A fast, customizable, and easy to use 
    <a href="https://www.javascript.com/">JavaScript</a></strong> and
    <a href="https://typescriptlang.org/">TypeScript</a> linter
  </p>

  <p>
    <a href="https://github.com/rslint/rslint/actions?query=workflow%3ARust"><img src="https://github.com/rslint/rslint/workflows/Rust/badge.svg" alt="build status" /></a>
    <a href="https://docs.rs/rslint_core"><img src="https://docs.rs/rslint_core/badge.svg" alt="Documentation Status" /></a>
    <a href="https://crates.io/crates/rslint_core"><img src="https://img.shields.io/crates/v/rslint_core.svg"/></a>
  </p>

  <h3>
    <a href="https://rslint.org/guide/">Guide</a>
    <span> | </span>
    <a href="https://rslint.org/dev/">Contributing</a>
    <span> | </span>
    <a href="https://rslint.org/">Website</a>
    <span> | </span>
    <a href="https://rslint.org/rules/">Linter Rules</a>
  </h3>

<strong>⚠️ RSLint is in early development and should not be used in production, expect bugs! 🐛</strong>

</div>

## Installation

### Through Cargo

```sh
$ cargo install rslint_cli
$ rslint --help
```

### Prebuilt Binaries

We publish prebuilt binaries for Windows, Linux, and MacOS for every release which you can find [here](https://github.com/rslint/rslint/releases).

### Build From Source

```sh
$ git clone https://github.com/rslint/rslint.git
$ cd rslint
$ cargo run --release -- --help
```

## Usage

To use the linter simply pass files to lint to the CLI:

```sh
$ echo "let a = foo.hasOwnProperty('bar');" > foo.js
$ rslint ./foo.js
error[no-prototype-builtins]: do not access the object property `hasOwnProperty` directly from `foo`
  ┌─ ./foo.js:1:9
  │
1 │ let a = foo.hasOwnProperty('bar');
  │         ^^^^^^^^^^^^^^^^^^^^^^^^^
  │
help: get the function from the prototype of `Object` and call it
  │
1 │ let a = Object.prototype.hasOwnProperty.call(foo, 'bar');
  │         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  │
  ╧ note: the method may be shadowed and cause random bugs and denial of service vulnerabilities

Outcome: 1 fail, 0 warn, 0 success

help: for more information about the errors try the explain command: `rslint explain <rules>`
```

The RSLint CLI works without a configuration file and will select reccomended non-stylistic rules to run.

## Features

**Speed**. RSLint uses parallelism to utilize multiple threads to speed up linting on top of being compiled to
native code.

**Low memory footprint**. RSLint's syntax tree utilizes interning and other ways of drastically reducing memory usage
while linting.

**Sensible defaults**. The CLI assumes reccomended non-stylistic rules if no configuration file is specified and ignores directories such as
`node_modules`.

**Error recovery**. RSLint's custom parser can recover from syntax errors and produce a usable syntax tree even when whole parts of
a statement are missing. Allowing accurate on-the-fly linting as you type.

**No confusing options**. ECMAScript version for the parser does not have to be configured, the parser assumes latest syntax and
assumes scripts for `*.js` and modules for `*.mjs`.

**Native TypeScript support**. `*.ts` files are automatically linted, no configuration for different parsers or rules is required.

**Rule groups**. Rules are grouped by scope for ease of configuration, understanding, and a cleaner file structure for the project.

**Understandable errors**. Each error emitted by the linter points out the area in the source code in an understandable and clean manner as well as contains labels, notes, and suggestions to explain how to fix each issue. There is also an alternative formatter similar to ESLint's formatter available using the `-F` flag or the `formatter` key in the config.

**Strongly typed rule configuration**. RSLint ships a JSON schema and links it for `rslintrc.json` to provide autocompletion for the config file in Visual Studio Code. The JSON Schema describes rule config options in full, allowing easy configuration. Moreover, RSLint's language server protocol implementation provides autocompletion for `rslintrc.toml` files too.

**Powerful directives**. Directives (commands through comments) use a parser based around the internal JavaScript lexer with instructions, allowing us to provide:

- Autocompletion for directives such as `// rslint-ignore no-empty` in the language server protocol.
- Hover support for directives to offer information on a command on hover.
- Understandable errors for incorrect directives.

**Standalone**. RSLint is compiled to a single standalone binary, it does not require Node, v8, or any other runtime. RSLint can run on any platform which can be targeted by LLVM.

**Powerful autofix**. Automatic fixes for some errors are provided and can be applied through the `--fix` flag or actions in the IDE. Fixes can even be applied if the file contains syntax errors through the `--dirty` flag.

**Built-in documentation**. RSLint contains rule documentation in its binary, allowing it to show documentation in the terminal through the explain subcommand, e.g. `rslint explain no-empty, for-direction`.

## Internal Features

**Clean and clear project layout**. The RSLint project is laid out in a monorepo and each crate has a distinct job, each crate can be used in other Rust projects and each crate has good documentation and a good API.

**Easy rule declaration**. Rules are declared using a `declare_lint!` macro. The macro accepts doc comments, a struct name, the group name, a rule code, and configuration options. The macro generates a struct definition and a `Rule` implementation and processes the doc comments into the documentation for the struct as well as into a static string used in the `docs()` method on each rule. Everything is concise and kept in one place.

**Full fidelity syntax tree**. Unlike ESTree, RSLint's custom syntax tree retains:

- All whitespace
- All comments
- All tokens

Allowing it to have powerful analysis without having to rely on separate structures such as ESLint's `SourceCode`.

**Untyped Syntax Tree**. RSLint's syntax tree is made of untyped nodes and untyped tokens at the low level, this allows for powerful, efficient traversal through the tree, e.g. `if_stmt.cons()?.child_with_ast::<SwitchStmt>()`.

**Easy APIs**. RSLint uses easy to use builders for its complex errors, as well as builders for autofix. Everything is laid out to minimize the effort required to implement rules.

## License

This project is Licensed under the [MIT license](http://opensource.org/licenses/MIT).
