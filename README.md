# RSLint

A (WIP) JavaScript linter written in Rust designed to be as fast as possible, customizable, and easy to use.

[User Documentation](https://rdambrosio016.github.io/RSLint/) | [Dev Documentation](https://rdambrosio016.github.io/RSLint/dev/index.html) | [Rustdoc Documentation](https://docs.rs/rslint_core/0.1.2/rslint_core/) | [Website](http://rslint.org)

## Docs and Installation

Please see the [website](https://rdambrosio016.github.io/RSLint/) for installation instructions and documentation.

## Currently known big issues

- Optional chaining is not parsed correctly
- Empty template literals panic
- A lot of error recoveries do not work and result in infinite recursion

### Financial Contributors

Become a financial contributor and help us sustain our community. [[Contribute](https://opencollective.com/rslint/contribute)]

#### Individuals

<a href="https://opencollective.com/rslint"><img src="https://opencollective.com/rslint/individuals.svg?width=890"></a>

#### Sponsors

Support this project with your organization. Your logo will show up here with a link to your website. [[Become a sponsor](https://opencollective.com/rslint/contribute)]

## Differences from other linters

### Implemented

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

### Planned

- Global config
- SSR-like templates for node matching and autofix
- Autofix without requiring reruns of all rules
- WASM builds

## Speed

RSLint is designed to be the fastest JavaScript linter ever made, it accomplishes this in various ways:

- Using a custom fast parser which retains whitespace
- Using a lookup table and trie based lexer for parsing
- Using separate distinct threads for splitting up IO bound tasks such as loading files
- Linting each file in parallel
- Running each rule from every group in parallel over the concrete syntax tree
- (WIP) linting each untyped node in parallel
- (WIP) Incrementaly reparsing and relinting files
- (WIP) Having native file watching support using incremental parsing

## Roadmap

RSLint's goal is to provide extremely fast and user friendly linting for the whole js ecosystem. There are tons of things to do to bring it up to par with existing linters. This is a list of planned features and things to do ranked in order of highest to lowest priority (this is by no definition final, things will change):

- [ ] Scope analysis (WIP)
- [x] Tests for parser, including test262
- [ ] Implementation of ESLint reccomended rules
- [ ] Benchmarks
- [ ] Markdown support
- [x] Config files (partially done)
- [x] Rule options
- [x] Prebuilt binary generation
- [ ] Npm package (needs a build script to pull a prebuilt binary)
- [ ] JSX Support
- [ ] TS Support
- [ ] Autofix (#45)
- [ ] JS Plugins
- [ ] WASM Plugins
- [x] Documentation website

## License

This project is Licensed under the [MIT license](http://opensource.org/licenses/MIT).
