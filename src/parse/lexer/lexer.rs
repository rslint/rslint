use super::{
  token::{TokenType, Token, BinToken, AssignToken},
  util::CharExt,
  state::LexerState,
  error::LexerDiagnosticType::*,
};
use crate::linter::diagnostic::*;
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
  pub done: bool
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

  pub fn scan_token(&mut self) -> Option<Result<Token, LinterDiagnostic<'a>>> {
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

      '.' => {
        match self.peek() {
          Some(c) if c.is_ascii_digit() => Some(self.read_decimal(self.cur)),
          _ => Some(Ok(self.token(self.cur, TokenType::Period)))
        }
      },

      '!' => {
        if self.peek() == Some('=') {
          self.advance();
          if self.peek() == Some('=') {
            self.advance();
            Some(Ok(self.token(self.cur - 2, TokenType::BinOp(BinToken::StrictInequality))))
          } else {
            Some(Ok(self.token(self.cur - 1, TokenType::BinOp(BinToken::Inequality))))
          }
        } else {
          Some(Ok(self.token(self.cur, TokenType::LogicalNot)))
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
            Some(Ok(self.token(self.cur - 1, TokenType::AssignOp(AssignToken::AddAssign))))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::BinOp(BinToken::Add))))
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
            Some(Ok(self.token(self.cur - 1, TokenType::AssignOp(AssignToken::SubtractAssign))))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::BinOp(BinToken::Subtract))))
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
                Some(Ok(self.token(start, TokenType::BinOp(BinToken::StrictEquality))))
              },
              _ => Some(Ok(self.token(start, TokenType::BinOp(BinToken::Equality)))),
            }
          },
          _ => Some(Ok(self.token(start, TokenType::BinOp(BinToken::Assign)))),
        }
      },

      '|' => {
        match self.peek() {
          Some(c) if c == '|' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::BinOp(BinToken::LogicalOr))))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::BinOp(BinToken::BitwiseOr))))
        }
      },

      '>' | '<' => Some(Ok(self.read_lt_gt(scanned == '<'))),

      scanned if scanned == '\\' || scanned.is_identifier_start() => Some(Ok(self.resolve_ident_or_keyword(scanned))),

      '1'..='9' => {
        Some(self.read_number())
      },

      '/' => {
        match self.peek() {
          Some(c) if c == '/' || c == '*' => Some(self.read_comment(c)),
          Some(_) if self.state.expr_allowed => Some(self.read_regex()),
          Some(c) if c == '=' => {
            self.advance();
            Some(Ok(self.token(self.cur - 1, TokenType::AssignOp(AssignToken::DivideAssign))))
          },
          _ => Some(Ok(self.token(self.cur, TokenType::BinOp(BinToken::Divide))))
        }
      },

      '\'' | '"' => Some(self.read_str_literal(scanned)),

      '`' => {
        if true {
          Some(Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(TemplateLiteralInEs5), false, "Invalid template literal")
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

      _ => Some(Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(UnexpectedToken), false, "Unexpected token").primary(self.cur..self.cur+1, "unexpected")))
    }
  }

  pub fn end(&self) -> Token {
    Token::new(TokenType::EndOfProgram, self.source_len, self.source_len, self.line)
  }

  //Reads an inline comment or multiline comment, expects the current pos to be the slash
  fn read_comment(&mut self, next_char: char) -> Result<Token, LinterDiagnostic<'a>> {
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
            return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(UnterminatedMultilineComment), false, "Unterminated multiline comment")
              .secondary(start..start + 2, "Multiline comment starts here")
              .primary(self.cur..self.cur, "File ends here")
            );
          } else {
            return Ok(self.token(start, TokenType::InlineComment));
          }
        }
      }
    }
  }

  // Reads a string literal of single or double quotes
  fn read_str_literal(&mut self, quote: char) -> Result<Token, LinterDiagnostic<'a>> {
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
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(UnterminatedString), short, "Unterminated string literal")
            .secondary(start..start+1, "Literal starts here")
            .primary(self.cur..self.cur+1, "Line ends here")
          );
        }
        Some(_) => { self.advance(); },
        None => {
          let short = self.cur - start > 50;
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(UnterminatedString), short, "Unterminated string literal")
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
  fn read_regex(&mut self) -> Result<Token, LinterDiagnostic<'a>> {
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
        Some(c) if c.is_line_break() => return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(UnterminatedRegex), false, "Unterminated regex literal")
          .secondary(start..start + 1, "Regex starts here")
          .primary(self.cur..self.cur + 1, "Line ends here")
        ),
        Some(c) if c == '/' && !in_class => break,
        Some(c) if c == '[' => in_class = true,
        Some(c) if c == ']' && in_class => in_class = false,
        Some(c) if c == '\\' => escaped = true,

        None => {
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(UnterminatedRegex), false, "Unterminated regex literal")
            .secondary(start..start + 1, "Regex starts here")
            .primary(self.cur..self.cur + 1, "File ends here")
          )
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
    self.validate_regex_flags(flags)?;
    Ok(self.token(start, TokenType::LiteralRegEx))
  }

  fn validate_regex_flags(&self, flags: String) -> Result<(), LinterDiagnostic<'a>> {
    let (mut global, mut ignore_case, mut multiline) = (false, false, false);

    // TODO: This error can be autofixed, but a fixer needs to be implemented first
    let flag_err = |flag: char| {
      LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(InvalidRegexFlags), false, "Invalid regex literal flags")
        .primary(self.cur..self.cur + 1, &format!("The `{}` flag may not appear multiple times", flag))
    };

    for (idx, i) in flags.char_indices() {
      match i {
        'g' => {
          if global {
            return Err(flag_err('g'))
          } else {
            global = true;
          }
        },
        'i' => {
          if ignore_case {
            return Err(flag_err('i'))
          } else {
            ignore_case = true;
          }
        },
        'm' => {
          if multiline {
            return Err(flag_err('m'))
          } else {
            multiline = true;
          }
        },
        c => {
          // TODO: rework this cursed range
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(InvalidRegexFlags), false, "Invalid regex flag")
            .primary(self.cur - (flags.chars().count() - idx) + 1..self.cur - (flags.chars().count() - idx) + 1, &format!("{} is not a valid regex flag", c))
          )
        }
      }
    }
    Ok(())
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Result<Token, LinterDiagnostic<'a>>;
  fn next(&mut self) -> Option<Self::Item> {
    self.scan_token()
  }
}