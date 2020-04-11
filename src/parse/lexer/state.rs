
/*
* Houses metadata about the file being processed like ES version
* Also contains contexts used while lexing such as if the lexer is in a template literal
*/
#[derive(Debug)]
pub struct LexerState {
  pub EsVersion: u8 //TODO change this to a dedicated enum
}

impl LexerState {
  pub fn new() -> Self {
    Self {
      EsVersion: 5
    }
  }
}