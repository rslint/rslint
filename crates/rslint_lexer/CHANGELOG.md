# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [Unreleased]

## [0.1.2] - 2020-10-5

### Fixed

- Fixed proper handling of identifiers starting with unicode letters
- Fixed the lexer to use ID_Start and ID_Continue instead of XID_Start and XID_Continue

## [0.1.1] - 2020-10-3

### Fixed

- Fixed incorrect lexing of single line comments with unicode characters inside of them
