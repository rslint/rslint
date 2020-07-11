//! An extremely fast and (mostly) lossless parser for JavaScript.  
//! Also includes a RegEx parser  
//! Serves as the main parser for RSLint.  
//! The parser returns a CST (Concrete Syntax Tree) which preserves all comments and whitespace.  

// TODO: check the stability of this and if it can be used
#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

pub mod diagnostic;
pub mod lexer;
pub mod macros;
pub mod parser;
pub mod serialize;
pub mod span;
pub mod unicode;
pub mod util;
pub mod regex;
