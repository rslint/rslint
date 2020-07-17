//! A formatter which outputs colored diagnostics to the terminal

use codespan_reporting::diagnostic::{Severity, Diagnostic};
use codespan_reporting::term;
use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
use crate::linter::file_walker::FileWalker;
use super::Formatter;

/// A structure to output colored diagnostics to the terminal using codespan_reporting's renderer
#[derive(Debug, Clone)]
pub struct CodespanFormatter;

impl CodespanFormatter {
    pub fn new() -> Box<Self> {
        Box::new(Self)
    }
}

impl Formatter for CodespanFormatter {
    fn format(&self, diagnostics: &Vec<Diagnostic<usize>>, walker: &FileWalker) {
        for diagnostic in diagnostics {
            let writer = if diagnostic.severity == Severity::Error {
                StandardStream::stderr(ColorChoice::Always)
            } else {
                StandardStream::stdout(ColorChoice::Always)
            };
    
            let mut config = term::Config::default();
            // currently codespan uses curvy corners, this renders weird on a lot of terminals
            config.chars.multi_top_left = '┌';
            config.chars.multi_bottom_left = '└';
            config.chars.source_border_left_break = '┼';
    
            term::emit(
                &mut writer.lock(),
                &config,
                walker,
                &diagnostic,
            )
            .expect("Failed to throw diagnostic");
        }
    }
}