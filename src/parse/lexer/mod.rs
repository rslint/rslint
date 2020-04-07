pub mod lexer;
pub mod reserved;
pub mod token;
pub mod util;
pub mod identifier;
pub mod error;

use once_cell::sync::OnceCell;
use std::path::Path;

//houses items each lexer instance uses
// #[derive(Debug)]
// pub struct LexerContext<'ctx> {
  
// }

#[derive(Debug)]
pub struct FileInfo<'a> {
  pub path: &'a Path,
  pub es_version: &'a u8,
}