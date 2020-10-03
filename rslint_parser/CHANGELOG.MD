# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [Unreleased]

## [0.1.1] - 2020-10-3
### Fixed 
- Fixed proper state handling in generators
- Fixed proper error reporting for function declarations without a name
- Fixed arrow functions with a newline after the arrow being parsed incorrectly
- Fixed potential_arrow_start state not becoming false after parsing an assign_expr was parsed
- Fixed class declaration parents being parsed as names and not lhs expressions
- Fixed identifier_name creating an empty NAME node if there was no identifier
