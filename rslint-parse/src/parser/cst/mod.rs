//! Definitions for the CST (Concrete Syntax Tree) emitted by the RSLint parser

pub mod expr;
pub mod stmt;

use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::span::Span;
use std::marker::PhantomData;
use std::ops::Deref;

/// A concrete representation of a javascript program.
/// The CST is lossless, each stmt/expr has a whitespace property, and comments are in a hashmap
#[derive(Clone, Debug)]
pub struct CST {
    statements: Vec<Stmt>,
}

impl CST {
    pub fn new() -> CST {
        CST {
            statements: Vec::new(),
        }
    }
}
