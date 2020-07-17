//! Implementations describing a way of formatting diagnostics into readable text or other formats
//! This includes colored terminal output, JSON, XML, etc.

pub mod codespan;

use codespan_reporting::diagnostic::Diagnostic;
use crate::linter::file_walker::FileWalker;
use std::fmt::Debug;

/// A trait for structures which take in codespan diagnostics and the files structure and format them in some way.  
/// This could include formatting to json, writing to stderr, etc.  
pub trait Formatter: Send + Sync + Debug {
    fn format(&self, diagnostics: &Vec<Diagnostic<usize>>, walker: &FileWalker);
}