# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [Unreleased]

### Added

- Added `util::StyleExt` for stylistic linting
- Added `Fixer::delete_multiple`

### Changed

- Moved util from a file to its own directory

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
