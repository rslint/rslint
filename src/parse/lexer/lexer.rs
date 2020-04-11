use super::{
  token::{TokenType, Token},
  util::CharExt,
  error::{LexerDiagnostic, LexerErrors},
  state::LexerState,
};
use std::str::CharIndices;
use std::iter::Peekable;

pub struct Lexer<'a> {
  pub file_id: usize, 
  pub source: &'a str,
  pub source_iter: Peekable<CharIndices<'a>>,
  pub source_len: usize,
  pub state: LexerState,
  pub start: usize,
  pub cur: usize,
  pub line: usize,
  pub done: bool
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str, file_id: usize) -> Self {
    Self {
      file_id,
      source,
      source_iter: source.char_indices().peekable(),
      source_len: source.len(),
      state: LexerState::new(),
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

  pub fn peek(&mut self) -> Option<char> {
    self.source_iter.peek().map(|x| x.1)
  }

  pub fn token(&mut self, start: usize, token: TokenType) -> Token {
    Token::new(token, start, self.cur + 1, self.line)
  }

  pub fn scan_token(&mut self) -> Option<Result<Token, LexerDiagnostic>> {
    //TODO tidy this up
    if self.done { return None; }
    if self.source_len == 0 || self.peek().is_none() {
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

      '(' | ')' | ';' | ',' | '[' | ']' | '{' | '}' => {
        let r#type = match scanned {
          '(' => TokenType::ParenOpen,
          ')' => TokenType::ParenClose,
          '{' => TokenType::BraceOpen,
          ';' => TokenType::Semicolon,
          ',' => TokenType::Comma,
          '}' => TokenType::BraceClose,
          '[' => TokenType::BracketOpen,
          ']' => TokenType::BracketClose,
          _ => unreachable!()
        };
        Some(Ok(self.token(self.cur, r#type)))
      },

      '>' | '<' => Some(Ok(self.read_lt_gt(scanned == '<'))),

      '/' => {
        match self.peek() {
          Some(c) if c == '/' || c == '*' => Some(self.read_comment(c)),
          Some(c) if c == '=' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::DivideAssign)))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::Division)))
        }
      },

      '+' => {
        match self.peek() {
          Some(c) if c == '+' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::Increment)))
          },
          Some(c) if c == '=' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::AddAssign)))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::Addition)))
        }
      },

      '-' => {
        match self.peek() {
          Some(c) if c == '-' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::Decrement)))
          },
          Some(c) if c == '=' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::SubtractAssign)))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::Subtraction)))
        }
      },

      '=' => {
        let start = self.cur;
        // = or ==
        match self.peek() {
          Some(c) if c == '=' => {
            self.advance();

            // == or ===
            match self.peek() {
              Some(c) if c == '=' => {
                self.advance();
                Some(Ok(self.token(start, TokenType::StrictEquality)))
              },
              _ => Some(Ok(self.token(start, TokenType::Equality))),
            }
          },
          _ => Some(Ok(self.token(start, TokenType::Assign))),
        }
      },

      '|' => {
        match self.peek() {
          Some(c) if c == '|' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::LogicalOr)))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::BitwiseOr)))
        }
      },

      '\'' | '"' => Some(self.read_str_literal(scanned)),

      '`' => {
        if self.state.EsVersion < 6 {
          Some(Err(LexerDiagnostic::new(self.file_id, LexerErrors::TemplateLiteralInEs5, false, "Invalid template literal")
            .primary(self.cur..self.cur+1, "Invalid")
            .note("Help: Template literals are allowed in ES6+ but the file is being processed as ES5")
        ))
        } else {
          unimplemented!() //TODO template literals
        }
      },

      scanned if scanned.is_line_break() => {
        let start = self.cur;
        // CRLF linebreaks
        if scanned == '\r' {
          match self.peek() {
            Some(c) if c == '\n' => {
              let token = self.token(start, TokenType::Linebreak);
              self.advance();
              self.line += 1;
              return Some(Ok(token));
            }
            _ => {
              let token = Some(Ok(self.token(start, TokenType::Linebreak)));
              self.line += 1;
              return token;
            }
          }
        }
        let token = Some(Ok(self.token(start, TokenType::Linebreak)));
        self.line += 1;
        token
      },

      scanned if scanned.is_js_whitespace() => {
        let start = self.cur;
        loop {
          match self.peek() {
            Some(c) if c.is_js_whitespace() => { self.advance(); },
            _ => return Some(Ok(self.token(start, TokenType::Whitespace)))
          }
        }
      },

      scanned if scanned == '\\' || scanned.is_identifier_start() => Some(Ok(self.resolve_ident_or_keyword(scanned))),

      _ => Some(Err(LexerDiagnostic::new(self.file_id, LexerErrors::UnexpectedToken, false, "Unexpected token").primary(self.cur..self.cur+1, "unexpected")))
    }
  }

  pub fn end(&self) -> Token {
    Token::new(TokenType::EndOfProgram, self.source_len, self.source_len, self.line)
  }

  //Reads an inline comment or multiline comment, expects the current pos to be the slash
  fn read_comment(&mut self, next_char: char) -> Result<Token, LexerDiagnostic> {
    let multiline = next_char == '*';
    let start = self.cur;

    loop {
      match self.peek() {
        Some(c) if c == '*' && multiline => {
          self.advance();
          if self.peek() == Some('/') {
            self.advance();
            return Ok(self.token(start, TokenType::MultilineComment))
          }
        },

        Some(c) if c.is_line_break() && !multiline => {
          return Ok(self.token(start, TokenType::InlineComment))
        },

        Some(_) => { self.advance(); },

        None => {
          if multiline {
            return Err(LexerDiagnostic::new(self.file_id, LexerErrors::UnterminatedMultilineComment, false, "Unterminated multiline comment")
              .secondary(start..start + 1, "Multiline comment starts here")
              .primary(self.cur..self.cur + 1, "File ends here")
            );
          } else {
            return Ok(self.token(start, TokenType::InlineComment));
          }
        }
      }
    }
  }

  // Reads a string literal of single or double quotes
  fn read_str_literal(&mut self, quote: char) -> Result<Token, LexerDiagnostic> {
    let start = self.cur;
    loop {
      match self.peek() {
        Some(c) if c == quote => {
          self.advance();
          return Ok(self.token(start, TokenType::LiteralString));
        },
        Some(c) if c.is_line_break() => {
          //long lines render ugly in codespan errors, so if the line is too long we render it as short
          let short = self.cur - start > 50;
          return Err(LexerDiagnostic::new(self.file_id, LexerErrors::UnterminatedString, short, "Unterminated string literal")
            .secondary(start..start+1, "Literal starts here")
            .primary(self.cur..self.cur+1, "Line ends here")
          );
        }
        Some(_) => { self.advance(); },
        None => {
          let short = self.cur - start > 50;
          return Err(LexerDiagnostic::new(self.file_id, LexerErrors::UnterminatedString, short, "Unterminated string literal")
            .secondary(start..start+1, "Literal starts here")
            .primary(self.cur..self.cur+1, "File ends here")
          );
        }
      }
    }
  }

  // resolves < or > to < <= << <= <<= or > >= >> >>> >>= >>>=
  fn read_lt_gt(&mut self, less_than: bool) -> Token {
    let target = if less_than { '<' } else { '>' };
    let start = self.cur;

    match self.peek() {
      // >= or <=
      Some(c) if c == '=' => {
        self.advance();
        if less_than {
          self.token(start, TokenType::LesserEquals)
        } else {
          self.token(start, TokenType::GreaterEquals)
        }
      },
      // >>* or <<*
      Some(c) if c == target => {
        self.advance();
        match self.peek() {
          // >>>
          Some(c) if c == target && !less_than => {
            self.advance();
            // >>>=
            if self.peek() == Some('=') {
              self.advance();
              self.token(start, TokenType::BitwiseUnsignedRightAssign)
            } else {
              self.token(start, TokenType::UnsignedBitshiftRight)
            }
          },
          // >>= or <<=
          Some(c) if c == '=' => {
            self.advance();
            if less_than {
              self.token(start, TokenType::BitwiseLeftAssign)
            } else {
              self.token(start, TokenType::BitwiseRightAssign)
            }
          },
          // >> or <<
          _ => {
            if less_than {
              self.token(start, TokenType::BitwiseLeftShift)
            } else {
              self.token(start, TokenType::BitwiseRightShift)
            }
          }
        }
      },
      // < or >
      _ => {
        if less_than {
          self.token(start, TokenType::Lesser)
        } else {
          self.token(start, TokenType::Greater)
        }
      }
    }
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Result<Token, LexerDiagnostic>;
  fn next(&mut self) -> Option<Self::Item> {
    self.scan_token()
  }
}