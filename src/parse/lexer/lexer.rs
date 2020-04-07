use super::{
  token::{TokenType, Token},
  util::CharExt,
  error::LexerDiagnostic
};
use std::str::CharIndices;
use std::iter::Peekable;

pub struct Lexer<'a> {
  pub file_id: usize, 
  pub source: &'a String,
  pub source_iter: Peekable<CharIndices<'a>>,
  pub source_len: usize,
  pub start: usize,
  pub cur: usize,
  pub line: usize,
  pub done: bool
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a String, file_id: usize) -> Self {
    Self {
      file_id,
      source,
      source_iter: source.char_indices().peekable(),
      source_len: source.len(),
      start: 0,
      cur: 0,
      line: 1,
      done: false
    }
  }

  pub fn advance(&mut self) -> Option<char> {
    self.source_iter.next().map(|(i, c)| {
      self.cur = i;
      c
    })
  }

  pub fn token(&mut self, start: usize, token: TokenType) -> Token {
    Token::new(token, start, self.cur + 1, self.line)
  }

  pub fn scan_token(&mut self) -> Option<Result<Token, LexerDiagnostic>> {
    if self.done { return None; }
    if self.cur == self.source.len() - 1 {
      self.done = true;
      return Some(Ok(self.end()));
    }
    let scanned = match self.advance() {
      Some(scanned) => scanned,
      None => {
        return Some(Ok(self.end()));
      }
    };
    match scanned {
      scanned if scanned.is_js_whitespace() => Some(Ok(self.token(self.cur, TokenType::Whitespace))),

      scanned if scanned.is_line_break() => {
        let token = self.token(self.cur, TokenType::Linebreak);
        self.line += 1; 
        Some(Ok(token))
      },

      scanned if scanned == '\\' || scanned.is_identifier_start() => Some(Ok(self.resolve_ident_or_keyword(scanned))),

      _ => unimplemented!(),
    }
  }

  pub fn end(&self) -> Token {
    Token::new(TokenType::EndOfProgram, self.source_len, self.source_len, self.line)
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Result<Token, LexerDiagnostic>;
  fn next(&mut self) -> Option<Self::Item> {
    self.scan_token()
  }
}