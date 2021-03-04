# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [Unreleased]

### Added

- Added "constructor-super" rule
- Added `util::StyleExt` for stylistic linting
- Added `Fixer::delete_multiple`
- Added more functions to `util::StyleExt`
- Added `RuleCtx::dummy_ctx`
- Added `Inferable` to define rules which can have their options inferred from nodes
- Added benchmarks for linting a file
- Added command descriptors
- Added `File` from rslint_cli
- Added the `regex` group
- Added `no-invalid-regexp`
- Added `util::regex`
- Added `CstRule::tags` and `Tag`
- Added `no-this-before-super`

### Changed

- Moved util from a file to its own directory
- Implemented a new directive parser which allows hover and auto-completion in lsp
- Removed the `module` parameter from `lint_file` and replaced it with `syntax: Syntax`
- Changed the way directive context is handled
- Changed lint_file and others to take a `&File` instead of a file id, source, etc.

### Removed

- Removed the `store` field from `LintResult`
- Removed `rayon` as the threadpool, replaced with `yastl`

### Fixed

- Fixed `no-await-in-loop` rejecting an await expression in the condition of the loop

## [0.2.1] - 2020-10-21

### Added

- Added a `docs` method to `Rule` to get its documentation

## [0.2.0] - 2020-10-20

### Changed

- Switched from codespan-reporting to a custom errors crate
- Added "no-new-symbol" rule
- Added "no-confusing-arrow" rule

### Added

- Added more documentation for some methods and structs
- Added "no-new-symbol" rule
- Added "no-confusing-arrow` rule
- Added autofix for many rules
- Added autofix with the Fixer struct and many utility methods to autofix

## [0.1.1] - 2020-10-3

### Fixed

- Removed a dbg! call from for-direction
