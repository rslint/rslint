# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [Unreleased]

### Added

- Added the `Formatter` trait for describing structs which can emit diagnostics in a certain way
- Added the `ShortFormatter` which emits diagnostics in an eslint-like style

### Changed

- Changed codespan backend to render notes with severity correctly

## [0.1.1]

### Added

- Added multiple utility methods to `Span`
