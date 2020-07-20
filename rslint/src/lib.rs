//! The main crate for RSLint, an extremely fast and customizeable linter for JavaScript powered by [rslint-parse](rslint_parse).  
//!
//! Focused on speed, ease of use, and customizability
//! # Speed
//! RSLint employs various tactics for making the linting process as fast as possible, these include:
//!  - Using a custom fast parser which retains whitespace
//!  - Using a lookup table and trie based lexer for parsing
//!  - Using separate distinct threads for splitting up IO bound tasks such as loading files
//!  - Linting each file in parallel
//!  - Running each rule from every group in parallel over the concrete syntax tree
//!  - Caching lint results by default
//!
//! # Distinct rule types (Planned)
//! To avoid placing constraints on the productions which can be checked, distinct types of rules are used.  
//! The main type being a [`CstRule`](rules::CstRule), a CstRule is concerned with the concrete syntax tree of a single file (or chunk of code from another file, e.g. md files).  
//! CstRules have to abide by certain rules:
//!  - They must be [`Send`](std::marker::Send) and [`Sync`](std::marker::Sync) as the linting process is highly parallelized
//!  - They cannot rely on the results from separate rules or files (this is impossible as linting is concurrent over files and rules)
//!  - They may not modify files as this may cause corruption of the data if two rules attempt to do that at the same time  
//!       Rules can apply fixes to files but this has to use a special fixer interface to make sure the fixes are applied serially
//!
//! Some rules like import rules may have to construct a source map and/or rely on the concrete syntax tree of all of the files.  
//! To allow this functionality while keeping a sane implementation, RSLint provides a second rule type, LateRules.  
//! LateRules are concerned with all of the concrete syntax trees that have been produced, as well as all of the files that have been loaded
//! LateRules must also abide by certain rules:
//!  - They must be [`Send`](std::marker::Send) and [`Sync`](std::marker::Sync) just like CstRules
//!  - They cannot rely on the result of other LateRules or any CstRules (this may be changed in the future for CstRules)
//!  - They cannot change the files for the same reason as CstRules, they can however use a fixer interface
//!  
//! # Lazy linting and Caching
//! RSLint is a lazy linter, it will not run rules if there is a guarantee that its result will not change between the previous and current run.
//! This is done using a cache file (`.rslintcache`), this is a binary file for size.  
//! By default, to avoid accidental cache commits the cache runner automatically adds the cache file to gitignore  
//! Cache stores the following info:  
//!  - The cargo version of RSLint  
//!  - The modified timestamp of the RSLint binary (mostly for development) 
//!  - The path of each file linted 
//!  - The modified timestamp for each linted file 
//!  - The ID of every rule run 
//!  - A serialized version of every diagnostic emitted for the file  
//!  - The timestamp of when the cache file was generated  
//! 
//! If you would like to know more about cache you should check out [the cache module](cache)
//!  
//! # Profiling  
//! If you would like to profile the performance of RSLint you should first run benchmarks. RSLint also allows for showing an approximation of the duration
//! of every major linting operation, as well as an average of the top ten slowest rules.  
//! To display these statistics set the env var `TIMING` to `1`


pub mod diagnostic;
pub mod formatters;
pub mod linter;
pub mod rules;
pub mod runner;
pub mod test_util;
pub mod visit;
pub mod cache;
pub mod util;
pub mod tablegen;

pub use linter::Linter;
pub use diagnostic::DiagnosticBuilder;
pub use rules::{RuleResult, CstRule, CstRuleGroup, Outcome};
pub use runner::{LintRunner, LintResult};
