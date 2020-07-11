pub mod file_walker;

use crate::linter::file_walker::FileWalker;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::diagnostic::Severity;
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use codespan_reporting::term::DisplayStyle;
use rayon::iter::ParallelIterator;
use rayon::prelude::*;
use rslint_parse::parser::Parser;
use rslint_parse::{diagnostic::ParserDiagnostic, lexer::lexer::Lexer};
use std::error::Error;
use std::io;

pub struct Linter {
    walker: FileWalker,
}

impl Linter {
    pub fn new(target: String) -> Self {
        Self {
            walker: FileWalker::new(target),
        }
    }

    pub fn repl() {
        loop {
            let mut source = String::new();

            io::stdin()
                .read_line(&mut source)
                .expect("Failed to read line");

            let linter = Self {
                walker: FileWalker::with_files(vec![(String::from("REPL.js"), source)]),
            };

            let lexer = Lexer::new(
                linter.walker.get_existing_source("REPL.js").unwrap(),
                "REPL.js",
            );
            unimplemented!();

            print!("\n > ");
        }
    }

    pub fn with_source(source: String, filename: String) -> Self {
        Self {
            walker: FileWalker::with_files(vec![(filename, source)]),
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        // Throw warnings for errors while loading files
        self.walker.load()?.iter().for_each(|diagnostic| self.diagnostic(Severity::Warning, diagnostic));
        self.walker.files.par_iter().for_each(|file| {
            unimplemented!();
        });
        Ok(())
    }

    pub fn diagnostic(&self, severity: Severity, msg: &str) {
      let diagnostic = Diagnostic::new(severity).with_message(msg);
      self.throw_diagnostic(diagnostic, false);
    }

    fn throw_diagnostic(&self, diagnostic: Diagnostic<&str>, short: bool) {
        let writer = if diagnostic.severity == Severity::Error {
            StandardStream::stderr(ColorChoice::Always)
        } else {
            StandardStream::stdout(ColorChoice::Always)
        };

        let mut config = term::Config::default();
        config.chars.multi_top_left = '┌';
        config.chars.multi_bottom_left = '└';
        config.chars.source_border_left_break = '┼';
        if short {
            config.display_style = DisplayStyle::Short;
        }

        term::emit(
            &mut writer.lock(),
            &config,
            &self.walker,
            &diagnostic,
        )
        .expect("Failed to throw diagnostic");
    }
}
