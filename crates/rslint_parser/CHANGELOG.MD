# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [Unreleased]

### Changed

- Made "redundant strict mode declaration" have a primary label instead of a single secondary label

## [0.2.1] - 2020-10-20

### Changed

- Switched from codespan-reporting to a custom errors crate

### Added

- Added handling of shebang where the parser consumes shebang if exists at the start

### Fixed

- Fixed handling of `/=` and `>>=`
- Fixed arrow expr parameters sometimes being `NAME_REF` instead of `NAME`

## [0.2.0] - 2020-10-8

### Changed

- Changed `CATCH_CLAUSE`'s error to be a Pattern according to the ECMAScript spec
- Changed `Parse::ok` to yield `Ok(T)` instead of `Err(T)` if there are only warnings

### Added

- Added an explicit error handler for `...foo = bar`

### Fixed

- Fixed infinite recursion with nested curly braces in object literals (`let a = {{}}`)
- Fixed infinite recursion with initializers in spread elements
- Fixed infinite recursion in array and object patterns with invalid tokens
- Fixed grouping expression not being allowed as assignment targets
- Fixed `arguments` being flagged as an invalid ident in non-strict mode

## [0.1.3] - 2020-10-5

### Fixed

- Fixed infinite recursion with block statement recovery
- Fixed infinite recursion with erroneous tokens as object property names (#30)
- Fixed infinite recursion with formal_parameters without a binding_element

## [0.1.2] - 2020-10-3

### Fixed

- Fixed automatic semicolon insertion for statements which have `}` after it
- Fixed keywords not being allowed in key value patterns
- Fixed LosslessTreeSink accessing unicode char boundaries

## [0.1.1] - 2020-10-3

### Fixed

- Fixed proper state handling in generators
- Fixed proper error reporting for function declarations without a name
- Fixed arrow functions with a newline after the arrow being parsed incorrectly
- Fixed potential_arrow_start state not becoming false after parsing an assign_expr was parsed
- Fixed class declaration parents being parsed as names and not lhs expressions
- Fixed identifier_name creating an empty NAME node if there was no identifier
