# Changelog

All notable changes to this crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),

## [Unreleased]

## [0.2.1] - 2020-10-21

### Changed

- Removed dependency on ureq for rule explanations

## [0.2.0] - 2020-10-20

### Added

- Added `--fix` (`-f`) and `--dirty` (`-D`) for running autofix
- Switched to rslint_errors for errors

### Changed

- Switched from codespan-reporting to a custom errors crate
- Changed panic hook to lock stderr and exit the program immediately after

## [0.1.2] 2020-10-3

### Fixed

- Fixed directory ignoring to work properly

## [0.1.1] 2020-10-3

### Fixed

- Fixed repo links for the explanation runner
