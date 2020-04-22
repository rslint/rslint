use super::{
  token::{TokenType, Token, BinToken, AssignToken},
  util::CharExt,
  state::LexerState,
  error::LexerDiagnosticType::*,
};
use crate::diagnostic::*;
use std::str::CharIndices;
use std::iter::Peekable;

pub struct Lexer<'a> {
  pub file_id: &'a str, 
  pub source: &'a str,
  pub source_iter: Peekable<CharIndices<'a>>,
  pub source_len: usize,
  pub state: LexerState,
  pub start: usize,
  pub cur: usize,
  pub line: usize,
  pub done: bool,
}

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str, file_id: &'a str) -> Self {
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

  pub fn advance_while<F>(&mut self, func: F) 
    where F: FnOnce(char) -> bool + Copy
  {
    loop {
      match self.peek() {
        Some(c) if !func(c) => break,
        None => break,
        Some(c) if func(c) => drop(self.advance()),
        _ => break, //Should be unreachable
      }
    }
  }

  pub fn next_idx(&mut self) -> usize {
    self.source_iter.peek().map(|x| x.0).unwrap_or(self.cur)
  }

  pub fn peek(&mut self) -> Option<char> {
    self.source_iter.peek().map(|x| x.1)
  }

  pub fn token(&mut self, start: usize, token: TokenType) -> Token {
    log::trace!("consuming token: {:?}", token);
    if token != TokenType::Whitespace {
      if token == TokenType::Linebreak {
        self.state.had_linebreak = true;
      } else {
        self.state.update(Some(token));
      }
    }
    Token::new(token, start, self.cur + 1, self.line)
  }

  /// The lexer may yield a token and a diagnostic, this is to allow the parser to recover from some errors
  /// `(Some, None)` is a successful scan
  /// `(Some, Some)` is an error the lexer could recover from
  /// `(None, Some)` is an error the lexer could not recover from
  /// `(None, None)` means the lexer is done
  pub fn scan_token(&mut self) -> (Option<Token>, Option<ParserDiagnostic<'a>>) {
    //TODO tidy this up
    if self.done { return (None, None); }
    if self.source_len == 0 || self.peek().is_none() {
      self.done = true;
      return (Some(self.end()), None);
    }
    let scanned = match self.advance() {
      Some(scanned) => scanned,
      None => {
        return (Some(self.end()), None);
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
        (Some(self.token(self.cur, r#type)), None)
      },

      '.' => {
        match self.peek() {
          Some(c) if c.is_ascii_digit() => self.read_decimal(self.cur),
          _ => (Some(self.token(self.cur, TokenType::Period)), None)
        }
      },

      '!' => {
        if self.peek() == Some('=') {
          self.advance();
          if self.peek() == Some('=') {
            self.advance();
            (Some(self.token(self.cur - 2, TokenType::BinOp(BinToken::StrictInequality))), None)
          } else {
            (Some(self.token(self.cur - 1, TokenType::BinOp(BinToken::Inequality))), None)
          }
        } else {
          (Some(self.token(self.cur, TokenType::LogicalNot)), None)
        }
      },

      '+' => {
        match self.peek() {
          Some(c) if c == '+' => {
            self.advance();
            (Some(self.token(self.cur - 1, TokenType::Increment)), None)
          },
          Some(c) if c == '=' => {
            self.advance();
            (Some(self.token(self.cur - 1, TokenType::AssignOp(AssignToken::AddAssign))), None)
          },
          _ => (Some(self.token(self.cur, TokenType::BinOp(BinToken::Add))), None)
        }
      },

      '-' => {
        match self.peek() {
          Some(c) if c == '-' => {
            self.advance();
            (Some(self.token(self.cur - 1, TokenType::Decrement)), None)
          },
          Some(c) if c == '=' => {
            self.advance();
            (Some(self.token(self.cur - 1, TokenType::AssignOp(AssignToken::SubtractAssign))), None)
          },
          _ => (Some(self.token(self.cur, TokenType::BinOp(BinToken::Subtract))), None)
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
                (Some(self.token(start, TokenType::BinOp(BinToken::StrictEquality))), None)
              },
              _ => (Some(self.token(start, TokenType::BinOp(BinToken::Equality))), None),
            }
          },
          _ => (Some(self.token(start, TokenType::BinOp(BinToken::Assign))), None),
        }
      },

      '|' => {
        match self.peek() {
          Some(c) if c == '|' => {
            self.advance();
            (Some(self.token(self.cur - 1, TokenType::BinOp(BinToken::LogicalOr))), None)
          },
          _ => (Some(self.token(self.cur, TokenType::BinOp(BinToken::BitwiseOr))), None)
        }
      },

      '>' | '<' => (Some(self.read_lt_gt(scanned == '<')), None),

      scanned if scanned == '\\' || scanned.is_identifier_start() => (Some(self.resolve_ident_or_keyword(scanned)), None),

      '1'..='9' => {
        self.read_number()
      },

      '0' => self.read_hex_literal(),

      '/' => {
        match self.peek() {
          Some(c) if c == '/' || c == '*' => self.read_comment(c),
          Some(_) if self.state.expr_allowed => self.read_regex(),
          Some(c) if c == '=' => {
            self.advance();
            (Some(self.token(self.cur - 1, TokenType::AssignOp(AssignToken::DivideAssign))), None)
          },
          _ => (Some(self.token(self.cur, TokenType::BinOp(BinToken::Divide))), None)
        }
      },

      '\'' | '"' => self.read_str_literal(scanned),

      '`' => {
        if true {
          (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(TemplateLiteralInEs5), false, "Invalid template literal")
            .primary(self.cur..self.next_idx(), "Invalid")
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
              return (Some(token), None);
            }
            _ => {
              let token = (Some(self.token(start, TokenType::Linebreak)), None);
              self.line += 1;
              return token;
            }
          }
        }
        let token = (Some(self.token(start, TokenType::Linebreak)), None);
        self.line += 1;
        token
      },

      scanned if scanned.is_js_whitespace() => {
        let start = self.cur;
        loop {
          match self.peek() {
            Some(c) if c.is_js_whitespace() => { self.advance(); },
            _ => return (Some(self.token(start, TokenType::Whitespace)), None)
          }
        }
      },

      _ => (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnexpectedToken), false, "Unexpected token").primary(self.cur..self.next_idx(), "unexpected")))
    }
  }

  pub fn end(&self) -> Token {
    Token::new(TokenType::EndOfProgram, self.source_len, self.source_len, self.line)
  }

  //Reads an inline comment or multiline comment, expects the current pos to be the slash
  fn read_comment(&mut self, next_char: char) -> (Option<Token>, Option<ParserDiagnostic<'a>>) {
    let multiline = next_char == '*';
    let start = self.cur;

    loop {
      match self.peek() {
        Some(c) if c == '*' && multiline => {
          self.advance();
          if self.peek() == Some('/') {
            self.advance();
            return (Some(self.token(start, TokenType::MultilineComment)), None)
          }
        },

        Some(c) if c.is_line_break() && !multiline => {
          return (Some(self.token(start, TokenType::InlineComment)), None)
        },

        Some(_) => { self.advance(); },

        None => {
          if multiline {
            return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedMultilineComment), false, "Unterminated multiline comment")
              .secondary(start..start + 2, "Multiline comment starts here")
              .primary(self.cur..self.cur, "File ends here")
            ));
          } else {
            return (Some(self.token(start, TokenType::InlineComment)), None);
          }
        }
      }
    }
  }

  // Reads a string literal of single or double quotes
  fn read_str_literal(&mut self, quote: char) -> (Option<Token>, Option<ParserDiagnostic<'a>>) {
    let start = self.cur;
    loop {
      match self.peek() {
        Some(c) if c == quote => {
          self.advance();
          return (Some(self.token(start, TokenType::LiteralString)), None);
        },
        Some(c) if c.is_line_break() => {
          //long lines render ugly in codespan errors, so if the line is too long we render it as short
          let short = self.cur - start > 50;
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedString), short, "Unterminated string literal")
            .secondary(start..start+1, "Literal starts here")
            .primary(self.cur..self.next_idx(), "Line ends here")
          ));
        }
        Some(_) => { self.advance(); },
        None => {
          let short = self.cur - start > 50;
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedString), short, "Unterminated string literal")
            .secondary(start..start+1, "Literal starts here")
            .primary(self.cur..self.next_idx(), "File ends here")
          ));
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
          self.token(start, TokenType::BinOp(BinToken::LessThanOrEqual))
        } else {
          self.token(start, TokenType::BinOp(BinToken::GreaterThanOrEqual))
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
              self.token(start, TokenType::AssignOp(AssignToken::UnsignedRightBitshiftAssign))
            } else {
              self.token(start, TokenType::BinOp(BinToken::UnsignedRightBitshift))
            }
          },
          // >>= or <<=
          Some(c) if c == '=' => {
            self.advance();
            if less_than {
              self.token(start, TokenType::AssignOp(AssignToken::LeftBitshiftAssign))
            } else {
              self.token(start, TokenType::AssignOp(AssignToken::RightBitshiftAssign))
            }
          },
          // >> or <<
          _ => {
            if less_than {
              self.token(start, TokenType::BinOp(BinToken::LeftBitshift))
            } else {
              self.token(start, TokenType::BinOp(BinToken::RightBitshift))
            }
          }
        }
      },
      // < or >
      _ => {
        if less_than {
          self.token(start, TokenType::BinOp(BinToken::LessThan))
        } else {
          self.token(start, TokenType::BinOp(BinToken::GreaterThan))
        }
      }
    }
  }

  /// Reads a regex literal, expects the current char to be the slash
  fn read_regex(&mut self) -> (Option<Token>, Option<ParserDiagnostic<'a>>) {
    let start = self.cur;
    let mut in_class = false;
    let mut escaped = false;

    loop {
      if escaped {
        self.advance();
        escaped = false;
        continue;
      }
      match self.peek() {
        Some(c) if c.is_line_break() => return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedRegex), false, "Unterminated regex literal")
          .secondary(start..start + 1, "Regex starts here")
          .primary(self.cur..self.next_idx(), "Line ends here")
        )),
        Some(c) if c == '/' && !in_class => break,
        Some(c) if c == '[' => in_class = true,
        Some(c) if c == ']' && in_class => in_class = false,
        Some(c) if c == '\\' => escaped = true,

        None => {
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedRegex), false, "Unterminated regex literal")
            .secondary(start..start + 1, "Regex starts here")
            .primary(self.cur..self.next_idx(), "File ends here")
          ))
        }

        _ => {}
      }
      self.advance();
    }

    self.advance();
    // /a/gi
    //   ^^regex flags
    let mut flags = String::new();
    loop {
      match self.peek() {
        Some(c) if c.is_identifier_part() => flags += &self.advance().unwrap().to_string(),
        Some(_) | None => break
      }
    }
    self.validate_regex_flags(flags);
    (Some(self.token(start, TokenType::LiteralRegEx)), None)
  }

  fn validate_regex_flags(&mut self, flags: String) -> Option<ParserDiagnostic<'a>> {
    let (mut global, mut ignore_case, mut multiline) = (false, false, false);

    // TODO: This error can be autofixed, but a fixer needs to be implemented first
    let mut flag_err = || {
      ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidRegexFlags), false, "Invalid regex literal flags")
        .primary(self.cur..self.next_idx(), "")
    };

    for (idx, i) in flags.char_indices() {
      match i {
        'g' => {
          if global {
            return Some(flag_err())
          } else {
            global = true;
          }
        },
        'i' => {
          if ignore_case {
            return Some(flag_err())
          } else {
            ignore_case = true;
          }
        },
        'm' => {
          if multiline {
            return Some(flag_err())
          } else {
            multiline = true;
          }
        },
        c => {
          return Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidRegexFlags), false, "Invalid regex flag")
            .primary(self.cur - (flags.chars().count() - idx) + 1..self.cur - (flags.chars().count() - idx) + 1, &format!("{} is not a valid regex flag", c))
          )
        }
      }
    }
    None
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = (Option<Token>, Option<ParserDiagnostic<'a>>);
  fn next(&mut self) -> Option<Self::Item> {
    let res = self.scan_token();
    if res == (None, None) {
      return None;
    }
    Some(res)
  }
}