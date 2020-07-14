//! An extremely fast linter for JavaScript

pub mod file_walker;

use crate::linter::file_walker::FileWalker;
use crate::runner::LintRunner;
use super::formatters::Formatter;
use super::formatters::codespan::CodespanFormatter;
use std::error::Error;
use std::sync::Once;

static CONFIGURE_RAYON: Once = Once::new();

#[derive(Debug)]
pub struct Linter {
    pub walker: FileWalker,
    pub formatter: Box<dyn Formatter>,
}

impl Linter {
    #[allow(unused_must_use)]
    pub fn new(target: String) -> Self {
        CONFIGURE_RAYON.call_once(|| {
            // Initialize the thread pool with a larger stack than the windows default (1 mb) to avoid overflows on very large files
            rayon::ThreadPoolBuilder::new().stack_size(8000000).build_global();
        });

        Self {
            walker: FileWalker::new(target),
            // TODO: Dynamic formatters
            formatter: CodespanFormatter::new()
        }
    }

    pub fn with_source(source: String, filename: String) -> Self {
        Self {
          walker: FileWalker::with_files(vec![(filename, source)]),
          formatter: CodespanFormatter::new(),
        }
      }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        LintRunner::new().exec(self);
        Ok(())
    }
}
