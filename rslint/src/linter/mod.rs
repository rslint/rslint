//! An extremely fast linter for JavaScript

pub mod file_walker;

use crate::linter::file_walker::FileWalker;
use crate::runner::LintRunner;
use super::formatters::Formatter;
use super::formatters::codespan::CodespanFormatter;
use std::sync::Once;
use clap::App;
use clap::load_yaml;
use std::env::current_dir;

static CONFIGURE_RAYON: Once = Once::new();

/// The entry point for RSLint, it serves as a dispatcher for using the linter from the cli, as a rust crate, etc.
/// The linting process itself is carried out by a distinct [lint runner](crate::runner)
#[derive(Debug)]
pub struct Linter {
    pub walker: FileWalker,
    pub formatter: Box<dyn Formatter>,
}

impl Linter {
    /// Make a new linter with a glob pattern to pass to the file walker
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

    /// Create a new linter from CLI args, this will either use the provided glob pattern or the current working directory
    /// # Panics
    /// This method will panic if there was no glob provided through CLI, and current working directory is unreadable
    pub fn from_cli_args() -> Self {
        let yaml = load_yaml!("../../cli.yml");
        let app = App::from_yaml(yaml);
        let args = app.get_matches();
        let glob = args.value_of("INPUT");

        if let Some(pat) = glob {
            Self::new(pat.to_string())
        } else {
            if let Ok(default) = current_dir() {
                Self::new(default.into_os_string().to_string_lossy().to_string())
            } else {
                panic!("Error: No glob pattern was provided, and the current working directory is unreadable or invalid");
            }
        }
    }

    /// Create a new linter from a single file
    pub fn with_source(source: String, filename: String) -> Self {
        Self {
          walker: FileWalker::with_files(vec![(filename, source)]),
          formatter: CodespanFormatter::new(),
        }
    }

    /// Run the linter, for more details on what this does, check out the [runner documentation](crate::runner)
    pub fn run(&mut self) -> () {
        LintRunner::new().exec(self);
    }
}
