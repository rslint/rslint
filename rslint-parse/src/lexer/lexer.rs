use super::{
  token::{TokenType, Token, BinToken, AssignToken},
  util::CharExt,
  state::LexerState,
  error::LexerDiagnosticType::*,
  lookup::LexerLookupTable
};
use crate::diagnostic::*;
use std::str::CharIndices;
use std::iter::Peekable;
use once_cell::sync::Lazy;

pub type LexResult<'a> = (Option<Token>, Option<ParserDiagnostic<'a>>);

pub struct Lexer<'a> {
  pub file_id: &'a str, 
  pub source: &'a str,
  pub source_iter: Peekable<CharIndices<'a>>,
  pub source_len: usize,
  pub state: LexerState,
  pub cur: usize,
  pub cur_char: char,
  pub line: usize,
}

macro_rules! range_lookup {
  ($l:expr, $range:expr, $fn:expr) => {
    for i in $range {
      $l.add_byte_entry(i, $fn);
    }
  };
}

macro_rules! tok {
  ($lexer:expr, $type:expr) => {
    (Some($lexer.token($lexer.cur, $type)), None)
  };
  ($lexer:expr, $type:expr, $start:expr) => {
    (Some($lexer.token($start, $type)), None)
  }
}

macro_rules! lookup {
  ($l:expr, $c:expr, $tok:ident) => {
    $l.add_char_entry($c, |lexer, _| {
      let start = lexer.cur;
      lexer.advance();
      (Some(lexer.token(start, TokenType::$tok)), None)
    });
  };
}

macro_rules! lookup_fn {
  ($l:expr, $c:expr, $fn:expr) => {
    $l.add_char_entry($c, $fn);
  };
}

macro_rules! lookup_mul {
  ($l:expr, ($($entry:expr, $(,)?)*), $fn:expr) => {
    $(
      $l.add_char_entry($entry, $fn);
    )*
  }
}

/// A lookup table for matching ascii charactes to functions to handle their tokens
/// Each function is stored as a usize pointer then transmuted when called
/// Unicode characters are handled after the lookup table
pub static LEXER_LOOKUP: Lazy<LexerLookupTable> = Lazy::new(|| {
  use super::token::TokenType::*;

  let mut l = LexerLookupTable::new();
  lookup!(l, '(', ParenOpen);
  lookup!(l, ')', ParenClose);
  lookup!(l, '{', BraceOpen);
  lookup!(l, '}', BraceClose);
  lookup!(l, '[', BracketOpen);
  lookup!(l, ']', BracketClose);
  lookup!(l, ',', Comma);
  lookup!(l, ';', Semicolon);
  lookup!(l, ':', Colon);
  lookup!(l, '?', QuestionMark);
  lookup!(l, '~', BitwiseNot);
  lookup_fn!(l, '\n', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    lexer.advance();
    let tok = tok!(lexer, Linebreak, start);
    lexer.line += 1;
    tok
  });
  lookup_fn!(l, '\r', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    if lexer.advance() == Some('\n') {
      lexer.advance();
    }
    // Linebreak's line should be on the line it ends, not on the next time
    let tok = tok!(lexer, Linebreak, start);
    lexer.line += 1;
    tok
  });
  lookup!(l, '\u{0009}', Whitespace);
  lookup!(l, '\u{000B}', Whitespace);
  lookup!(l, '\u{000C}', Whitespace);
  lookup!(l, '\u{0020}', Whitespace);
  lookup!(l, '\u{00A0}', Whitespace);
  // A - Z - can only be identifier
  range_lookup!(l, 65..91, |lexer: &mut Lexer, _: char| {
    (Some(lexer.resolve_identifier(lexer.cur)), None)
  });
  // a - z - could be keyword
  range_lookup!(l, 97..123, |lexer: &mut Lexer, c: char| {
    (Some(lexer.resolve_ident_or_keyword(c)), None)
  });

  lookup_fn!(l, '!', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    if lexer.advance() == Some('=') {
      if lexer.advance() == Some('=') {
        lexer.advance();
        let tok = BinOp(BinToken::StrictInequality);
        tok!(lexer, tok, start)
      } else {
        let tok = BinOp(BinToken::Inequality);
        tok!(lexer, tok, start)
      }
    } else {
      tok!(lexer, LogicalNot, start)
    }
  });

  lookup_fn!(l, '=', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    if lexer.advance() == Some('=') {
      if lexer.advance() == Some('=') {
        lexer.advance();
        let tok = BinOp(BinToken::StrictEquality);
        tok!(lexer, tok, start)
      } else {
        let tok = BinOp(BinToken::Equality);
        tok!(lexer, tok, start)
      }
    } else {
      tok!(lexer, BinOp(BinToken::Assign), start)
    }
  });

  lookup_fn!(l, '-', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('-') => {
        lexer.advance();
        tok!(lexer, Decrement, start)
      },
      Some('=') => {
        lexer.advance();
        let tok = AssignOp(AssignToken::SubtractAssign);
        tok!(lexer, tok, start)
      },
      _ => {
        let tok = BinOp(BinToken::Subtract);
        tok!(lexer, tok, start)
      }
    }
  });

  lookup_fn!(l, '+', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('+') => {
        lexer.advance();
        tok!(lexer, Increment, start)
      },
      Some('=') => {
        lexer.advance();
        let tok = AssignOp(AssignToken::AddAssign);
        tok!(lexer, tok, start)
      },
      _ => {
        let tok = BinOp(BinToken::Add);
        tok!(lexer, tok, start)
      }
    }
  });

  lookup_fn!(l, '*', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('=') => {
        lexer.advance();
        let tok = AssignOp(AssignToken::MultiplyAssign);
        tok!(lexer, tok, start)
      },
      _ => {
        let tok = BinOp(BinToken::Multiply);
        tok!(lexer, tok, start)
      }
    }
  });

  lookup_fn!(l, '/', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('/') => {
        lexer.advance_while(|x| !x.is_line_break());
        return tok!(lexer, InlineComment, start);
      },
      Some('*') => {
        loop {
          match lexer.advance() {
            Some('*') if lexer.advance() == Some('/') => {
              lexer.advance();
              return tok!(lexer, MultilineComment, start);
            },
            Some(c) if c.is_line_break() => {
              if c == '\r' && lexer.advance() == Some('\n') {
                lexer.advance();
              }
              lexer.line += 1;
            },
            None => {
              return (None, Some(ParserDiagnostic::new(lexer.file_id, ParserDiagnosticType::Lexer(UnterminatedMultilineComment), "Unterminated multiline comment")
                .secondary(start..start + 2, "Multiline comment starts here")
                .primary(lexer.cur..lexer.cur, "File ends here")
              ));
            },
            _ => {}
          }
        }
      }
      _ if lexer.state.expr_allowed => lexer.read_regex(),

      _ => {
        let tok = BinOp(BinToken::Divide);
        return tok!(lexer, tok, start);
      }
    }
  });

  lookup_fn!(l, '<', |lexer: &mut Lexer, _: char| (Some(lexer.read_lt_gt(true)), None));
  lookup_fn!(l, '>', |lexer: &mut Lexer, _: char| (Some(lexer.read_lt_gt(false)), None));

  lookup_fn!(l, '\'', |lexer: &mut Lexer, _: char| lexer.read_str_literal('\''));
  lookup_fn!(l, '"', |lexer: &mut Lexer, _: char| lexer.read_str_literal('"'));

  lookup_fn!(l, '|', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('|') => {
        lexer.advance();
        let tok = BinOp(BinToken::LogicalOr);
        tok!(lexer, tok, start)
      },
      Some('=') => {
        lexer.advance();
        let tok = AssignOp(AssignToken::BitwiseOrAssign);
        tok!(lexer, tok, start)
      },
      _ => {
        let tok = BinOp(BinToken::BitwiseOr);
        tok!(lexer, tok, start)
      }
    }
  });

  lookup_fn!(l, '&', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('&') => {
        lexer.advance();
        let tok = BinOp(BinToken::LogicalAnd);
        tok!(lexer, tok, start)
      },
      Some('=') => {
        lexer.advance();
        let tok = AssignOp(AssignToken::BitwiseAndAssign);
        tok!(lexer, tok, start)
      },
      _ => {
        let tok = BinOp(BinToken::BitwiseAnd);
        tok!(lexer, tok, start)
      }
    }
  });

  lookup_fn!(l, '%', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('=') => {
        lexer.advance();
        let tok = AssignOp(AssignToken::ModuloAssign);
        tok!(lexer, tok, start)
      },
      _ => {
        let tok = BinOp(BinToken::Modulo);
        tok!(lexer, tok, start)
      }
    }
  });

  lookup_fn!(l, '^', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some('=') => {
        lexer.advance();
        let tok = AssignOp(AssignToken::BitwiseXorAssign);
        tok!(lexer, tok, start)
      },
      _ => {
        let tok = BinOp(BinToken::BitwiseXor);
        tok!(lexer, tok, start)
      }
    }
  });

  lookup_fn!(l, '\\', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    if lexer.advance() != Some('u') {
      let err = ParserDiagnostic::new(lexer.file_id, ParserDiagnosticType::Lexer(InvalidUnicodeEscapeSequence), "Invalid token after backslash, expected `u`")
        .primary(start..lexer.next_idx(), "Expected a unicode escape sequence");
      
      // Recover by parsing the identifier eitherways
      lexer.resolve_identifier(start);
      (Some(lexer.token(start, InvalidToken)), Some(err))
    } else {
      let read_unicode = lexer.read_unicode_escape();
      if read_unicode.is_err() {
        let err = read_unicode.err().unwrap();
        
        // Resolve identifier advances no matter what, if the unicode sequence is incomplete we cannot advance as that will skip the next token
        if err.error_type != ParserDiagnosticType::Lexer(IncompleteUnicodeEscapeSequence) { lexer.resolve_identifier(start); }
        (Some(lexer.token(start, InvalidToken)), Some(err))
      } else {
        let c = read_unicode.unwrap();
        let err = if !c.is_identifier_start() {
          Some(ParserDiagnostic::new(lexer.file_id, ParserDiagnosticType::Lexer(InvalidUnicodeIdentStart), "Unicode escape sequence resolves to an invalid identifier start")
            .primary(start..lexer.next_idx(), &format!("Evaluates to `{}`, which is not a valid start to an identifier", c)))
        } else {
          None
        };

        let tok = if err.is_none() { Identifier } else { InvalidToken };
        lexer.resolve_identifier(start);

        (Some(lexer.token(start, tok)), err)
      }
    }
  });

  lookup_fn!(l, '#', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    let next = lexer.advance();

    // Valid shebangs have to be the first character
    if lexer.cur == 1 && next == Some('!') {
      lexer.advance_while(|x| !x.is_line_break());
      tok!(lexer, Shebang, start)
    } else {
      let mut err = ParserDiagnostic::new(lexer.file_id, ParserDiagnosticType::Lexer(UnexpectedToken), "Unexpected token `#`")
        .primary(start..lexer.cur, "Unexpected");
      if next == Some('!') {
        err = err.help("Help: Shebang sequences must start at the first character in the file.");
      }
      (None, Some(err))
    }
  });

  // 0 - 9
  range_lookup!(l, 48..58, |lexer: &mut Lexer, c: char| lexer.read_number(false, c == '0'));

  lookup_fn!(l, '.', |lexer: &mut Lexer, _: char| {
    let start = lexer.cur;
    match lexer.advance() {
      Some(c) if c.is_ascii_digit() => lexer.read_number(true, false),
      _ => tok!(lexer, Period, start)
    }
  });

  lookup_mul!(l, ('_', '$',), |lexer: &mut Lexer, _: char| (Some(lexer.resolve_identifier(lexer.cur)), None));
  l
});

impl<'a> Lexer<'a> {
  pub fn new(source: &'a str, file_id: &'a str) -> Self {
    let iter = source.char_indices().peekable();
    let mut lex = Self {
      file_id,
      source,
      source_iter: iter,
      source_len: source.len(),
      state: LexerState::new(),
      cur: 0,
      cur_char: ' ',
      line: 1,
    };
    lex.cur_char = lex.source_iter.next().map(|c| c.1).unwrap_or(' ');
    lex
  }

  pub fn advance(&mut self) -> Option<char> {
    let res = self.source_iter.next().map(|(i, c)| {
      self.cur = i;
      self.cur_char = c;
      c
    });
    if res.is_none() {
      self.state.last_tok = true;
    }
    res
  }

  pub fn advance_while<F>(&mut self, func: F) 
    where F: Fn(char) -> bool
  {
    loop {
      match self.advance() {
        Some(c) if !func(c) => break,
        None => break,
        Some(c) if func(c) => {},
        _ => break, //Should be unreachable
      }
    }
  }

  pub fn next_idx(&mut self) -> usize {
    self.source_iter.peek().map(|x| x.0).unwrap_or(self.cur + 1)
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
    let end = if self.state.last_tok { self.cur + 1 } else { self.cur };
    Token::new(token, start, end, self.line)
  }

  fn scan_non_lookup_token(&mut self) -> LexResult<'a> {
    let start = self.cur;
    println!("cur: {}, id: {}", self.cur_char, self.cur_char.is_identifier_start());
    match self.cur_char {
      c if c.is_js_whitespace() => {
        self.advance();
        tok!(self, TokenType::Whitespace, start)
      },

      c if c.is_line_break() => {
        self.advance();
        self.line += 1;
        tok!(self, TokenType::Linebreak, start)
      }

      // Keywords are only ascii lowercase, handled by the lookup table, therefore it must be an identifier
      c if c.is_identifier_start() => {
        (Some(self.resolve_identifier(start)), None)
      },

      _ => {
        (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnexpectedToken), &format!("Unexpected token `{}`", self.cur_char))
          .primary(self.cur..self.next_idx(), "Invalid")))
      }
    }
  }

  /// The lexer may yield a token and a diagnostic, this is to allow the parser to recover from some errors
  /// `(Some, None)` is a successful scan
  /// `(Some, Some)` is an error the lexer could recover from
  /// `(None, Some)` is an error the lexer could not recover from
  /// `(None, None)` means the lexer is done
  pub fn scan_token(&mut self) -> LexResult<'a> {
    if self.state.last_tok || self.source_len == 0 {
      return (None, None);
    }
    let c = self.cur_char;
    if (c as u16) < 255 {
      let func = LEXER_LOOKUP.lookup(c);
      let res = func(self, c);
      if res == (None, None) {
        return self.scan_non_lookup_token();
      }
      res
    } else {
      self.scan_non_lookup_token()
    }
  }

  fn validate_escape_sequence(&mut self, escaped: char) -> Result<(), ParserDiagnostic<'a>> {
    match escaped {
      'u' => drop(self.read_unicode_escape()?),
      'x' => drop(self.read_hex_escape()?),
      // u+2028 and u+2029 behave the same as \n so we dont need a separate function
      '\n' | '\u{2028}' | '\u{2029}' => drop(LEXER_LOOKUP.lookup('\n')(self, ' ')),
      '\r' => drop(LEXER_LOOKUP.lookup('\r')(self, ' ')),
      c if c.is_line_break() => {
        self.line += 1;
        self.advance();
      }
      _ => drop(self.advance())
    }
    Ok(())
  }

  /// Reads a hex escape sequence such as \x47
  /// Expects the current char to be the x after the backslash
  // TODO: possibly refactor this and read_unicode_escape into one method
  fn read_hex_escape(&mut self) -> Result<char, ParserDiagnostic<'a>> {
    let start = self.cur - 1;
    let mut digits = String::with_capacity(2);

    for _ in 0..2 {
      match self.advance() {
        Some(c) if c.is_ascii_hexdigit() => digits.push(c), 

        Some(c) if !c.is_identifier_part() => {
          let diagnostic = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteHexEscapeSequence), "Incomplete unicode escape sequence")
            .primary(start..self.cur, "Expected 2 hex digits");
          return Err(diagnostic);
        },

        Some(c) if !c.is_ascii_hexdigit() => {
          let diagnostic = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidHexEscapeSequence), "Invalid character in unicode escape sequence")
            .primary(self.cur..self.next_idx(), "Invalid hex digit");
          return Err(diagnostic);
        },

        None => {
          let diagnostic = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteHexEscapeSequence), "Incomplete unicode escape sequence")
            .primary(start..self.next_idx(), "Expected 2 hex digits");
          return Err(diagnostic);
        },

        _ => unreachable!(),
      }
    }
    let codepoint = u32::from_str_radix(&digits, 16).unwrap();
    Ok(std::char::from_u32(codepoint).unwrap())
  }

  /// Reads a unicode escape sequence such as \u200b
  /// Expects the current char to be the u after the backslash
  fn read_unicode_escape(&mut self) -> Result<char, ParserDiagnostic<'a>> {
    let start = self.cur - 1;
    let mut digits = String::with_capacity(4);

    for _ in 0..4 {
      match self.advance() {
        Some(c) if c.is_ascii_hexdigit() => digits.push(c), 

        Some(c) if !c.is_identifier_part() => {
          let diagnostic = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteUnicodeEscapeSequence), "Incomplete unicode escape sequence")
            .primary(start..self.cur, "Expected 4 hex digits");
          return Err(diagnostic);
        },

        Some(c) if !c.is_ascii_hexdigit() => {
          let diagnostic = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidUnicodeEscapeSequence), "Invalid character in unicode escape sequence")
            .primary(self.cur..self.next_idx(), "Invalid hex digit");
          return Err(diagnostic);
        },

        None => {
          let diagnostic = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteUnicodeEscapeSequence), "Incomplete unicode escape sequence")
            .primary(start..self.next_idx(), "Expected 4 hex digits");
          return Err(diagnostic);
        },

        _ => unreachable!(),
      }
    }
    let codepoint = u32::from_str_radix(&digits, 16).unwrap();
    Ok(std::char::from_u32(codepoint).unwrap())
  }

  fn read_str_literal(&mut self, quote: char) -> LexResult<'a> {
    let start = self.cur;
    let mut err: Option<ParserDiagnostic<'a>> = None;
    let start_line = self.line;

    loop {
      match self.advance() {
        Some('\\') if self.peek().is_some() => {
          let next = self.advance().unwrap();
          let res = self.validate_escape_sequence(next);
          if res.is_err() { err = Some(res.err().unwrap()); }
        },

        Some(c) if c == quote => {
          self.advance();
          // This is to prevent the token's line being the last escaped line
          // This way the token's line is the line the literal starts on
          let mut tok = self.token(start, TokenType::LiteralString);
          tok.line = start_line;
          return (Some(tok), err);
        },
        Some(c) if c.is_line_break() => {
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedString), "Unterminated string literal")
            .secondary(start..start+1, "Literal starts here")
            .primary(self.cur..self.next_idx(), "Line ends here")
          ));
        }
        Some(_) => {},
        None => {
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedString), "Unterminated string literal")
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

    match self.advance() {
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
        match self.advance() {
          // >>>
          Some(c) if c == target && !less_than => {
            // >>>=
            if self.advance() == Some('=') {
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

  fn read_regex(&mut self) -> LexResult<'a> {
    let start = self.cur - 1;
    let mut in_class = false;
    let mut escaped = false;

    loop {
      if escaped {
        self.advance();
        escaped = false;
        continue;
      }
      match self.advance() {
        Some(c) if c.is_line_break() => return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedRegex), "Unterminated regex literal")
          .secondary(start..start + 1, "Regex starts here")
          .primary(self.cur..self.next_idx(), "Line ends here")
        )),
        Some(c) if c == '/' && !in_class => break,
        Some(c) if c == '[' => in_class = true,
        Some(c) if c == ']' && in_class => in_class = false,
        Some(c) if c == '\\' => escaped = true,

        None => {
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(UnterminatedRegex), "Unterminated regex literal")
            .secondary(start..start + 1, "Regex starts here")
            .primary(self.cur..self.next_idx(), "File ends here")
          ))
        }

        _ => {}
      }
    }

    // /a/gi
    //   ^^regex flags
    let err = self.validate_regex_flags();
    let tok = if err.is_none() { TokenType::LiteralRegEx } else { TokenType::InvalidToken };
    (Some(self.token(start, tok)), err)
  }

  fn validate_regex_flags(&mut self) -> Option<ParserDiagnostic<'a>> {
    let (mut global, mut ignore_case, mut multiline) = (false, false, false);

    let flag_err = |lexer: &mut Lexer<'a>| {
      ParserDiagnostic::new(lexer.file_id, ParserDiagnosticType::Lexer(InvalidRegexFlags), "Invalid regex literal flags")
        .primary(lexer.cur..lexer.next_idx(), "")
    };

    loop {
      match self.advance() {
        Some('g') => {
          if global {
            return Some(flag_err(self))
          } else {
            global = true;
          }
        },
        Some('i') => {
          if ignore_case {
            return Some(flag_err(self))
          } else {
            ignore_case = true;
          }
        },
        Some('m') => {
          if multiline {
            return Some(flag_err(self))
          } else {
            multiline = true;
          }
        },
        Some(c) if c.is_identifier_part() => {
          return Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidRegexFlags), "Invalid regex flag")
            .primary(self.cur..self.next_idx(), &format!("`{}` is not a valid regex flag", c))
          )
        },
        None => break,
        Some(c) if !c.is_identifier_part() => break,

        _ => unreachable!(),
      }
    }
    None
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = LexResult<'a>;
  fn next(&mut self) -> Option<Self::Item> {
    let res = self.scan_token();
    if res == (None, None) {
      return None;
    }
    Some(res)
  }
}