//! Definitions for the CST (Concrete Syntax Tree) emitted by the RSLint parser

pub mod expr;
pub mod stmt;
pub mod declaration;

use crate::parser::cst::stmt::*;
use crate::span::Span;

/// A concrete representation of a javascript program.
/// The CST is lossless, each stmt/expr has a whitespace property, and comments are in a hashmap
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CST {
    pub statements: Vec<StmtListItem>,
    /// The optional shebang sequence at the top of a file, this will be none if:  
    /// - There is no shebang sequence  
    /// - The allow_shebang option is `false`
    pub shebang: Option<Span>,
    /// The whitespace directly before the end of the file
    pub eof_whitespace: Span,
}

impl CST {
    pub fn new() -> CST {
        CST {
            statements: Vec::new(),
            shebang: None,
            eof_whitespace: Span::new(0, 0),
        }
    }
}
