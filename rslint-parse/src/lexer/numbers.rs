use super::{
  lexer::Lexer,
  token::{TokenType, Token},
  util::CharExt,
  error::LexerDiagnosticType::*
};
use crate::diagnostic::*;

impl<'a> Lexer<'a> {

  /// Reads a potential hex literal, expects the current char to be the 0
  /// see: https://www.ecma-international.org/ecma-262/5.1/#sec-7.8.3
  pub fn read_hex_literal(&mut self) -> (Option<Token>, Option<ParserDiagnostic<'a>>) {
    let start = self.cur;

    match self.peek() {
      // 0x and 0X literal
      Some(c) if c.to_ascii_lowercase() == 'x' => {
        self.advance();

        // 0x3
        //  ^
        match self.peek() {
          Some(c) if c.is_ascii_hexdigit() => drop(self.advance()),
          Some(c) if !c.is_identifier_start() => {
            // Recover by parsing as an invalid token
            let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(MissingHexDigit), false, "Invalid hex literal without digit")
              .primary(start..self.next_idx(), "Invalid with no digits");
            
            return (Some(self.token(start, TokenType::InvalidToken)), Some(err));
          },
          None => {
            let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(MissingHexDigit), false, "Invalid hex literal without digit")
              .primary(self.cur..self.next_idx(), "File ends here")
              .secondary(start..start + 1, "Hex literal starts here");

            return (Some(self.token(start, TokenType::InvalidToken)), Some(err));
          }
          Some(_) => {
            self.advance();
            let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidHexCharacter), false, "Invalid character in hex literal")
              .primary(self.cur..self.next_idx(), "Invalid character");
            self.advance_while(|x| !x.is_identifier_start());
            return (Some(self.token(start, TokenType::InvalidToken)), Some(err))
          }
        };

        // 0x3D
        //   ^
        loop {
          match self.peek() {
            Some(c) if c.is_ascii_hexdigit() => drop(self.advance()),
            Some(c) if !c.is_identifier_start() => return (Some(self.token(start, TokenType::LiteralNumber)), None),
            None => return (Some(self.token(start, TokenType::LiteralNumber)), None),
            Some(c) if !c.is_ascii_hexdigit() => {
              // If there are multiple invalid characters in a row, pretty print it as "invalid digits"
              self.advance();
              let invalid_start = self.cur;
              let mut mul_invalid = "";
              match self.peek() {
                Some(c) if !c.is_ascii_hexdigit() => { 
                  self.advance_while(|x| !x.is_ascii_hexdigit());
                  mul_invalid = "s";
                },
                _ => {}
              }
              
              let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidOctalLiteral), false, "Invalid character in hex literal")
                .primary(invalid_start..self.next_idx(), &format!("Invalid hexadecimal digit{}", mul_invalid));

              // Recover by parsing as invalid token
              self.advance_while(|x| !x.is_identifier_part());
              return (Some(self.token(start, TokenType::InvalidToken)), Some(err))
            },
            _ => panic!("Reached wildcard while parsing hex literal")
          }
        }
      },

      Some(c) if c.is_ascii_digit() => {
        // TODO: only issue this error for es5, parse as octal literal in es6+

        // Try to recover from the error by still parsing but as an invalid token
        let mut valid_octal = true;
        let mut invalid_char_pos = 0;

        loop {
          match self.peek() {
            Some('0'..='7') => drop(self.advance()),
            Some(c) if c.is_identifier_part() => {
              self.advance();
              if invalid_char_pos == 0 { invalid_char_pos = self.cur};
              valid_octal = false;
            },
            _ => break
          }
        }

        let mut err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidOctalLiteral), false, "Invalid octal literal in ES5 file")
          .primary(start..self.next_idx(), "Invalid in ES5");
        
        if !valid_octal {
          err = err.secondary(invalid_char_pos..self.next_idx(), "Side-note: Octal literals may only include digits 0 - 7");
        }

        (Some(self.token(start, TokenType::InvalidToken)), Some(err))
      },

      // Literal 0
      Some(c) if !c.is_identifier_start() => {
        (Some(self.token(self.cur, TokenType::LiteralNumber)), None)
      },

      Some(c) if c.is_identifier_start() => {
        self.advance();
        let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(InvalidHexCharacter), false, "Invalid identifier after hex literal")
          .primary(self.cur..self.next_idx(), "The start to an identifier may not occur after a hex literal");
        self.advance_while(|x| !x.is_identifier_part());
        return (Some(self.token(start, TokenType::InvalidToken)), Some(err));
      },

      None => (Some(self.token(self.cur, TokenType::LiteralNumber)), None),

      c => panic!("Hit wildcard while parsing hex literal, char: {}", c.unwrap())
    }
  }

  /// Reads a numeric literal which does not start with a dot and starts with 1 - 9
  /// Expects the current pos to be the first digit
  pub fn read_number(&mut self) -> (Option<Token>, Option<ParserDiagnostic<'a>>) {
    let start = self.cur;

    loop {
      match self.peek() {
        Some(c) if c.to_ascii_lowercase() == 'e' => {
          self.advance();
          let err = self.read_exponent();
          // Recover from error
          println!("Recovering");
          if err.is_some() {
            self.advance_while(|x| x.is_identifier_part());
          }
          let token = if err.is_none() { TokenType::LiteralNumber } else { TokenType::InvalidToken };
          return (Some(self.token(start, token)), err);
        }
        Some(c) if c.is_ascii_digit() => { self.advance(); },
        Some(c) if c == '.' => {
          self.advance();
          return self.read_decimal(start);
        }
        Some(c) if c.is_identifier_start() => {
          self.advance();
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IdentifierStartAfterNumber), false, "Invalid character after numeric literal")
            .primary(self.cur..self.next_idx(), "The start to an identifier cannot appear directly after a numeric literal")
            .note("Help: Did you forget a space after the number?")
          ));
        }
        _ => {
          return (Some(self.token(start, TokenType::LiteralNumber)), None)
        }
      }
    }
  }

  /// Reads a decimal composed of . DecimalDigits ExponentPart(opt)
  pub fn read_decimal(&mut self, start: usize) -> (Option<Token>, Option<ParserDiagnostic<'a>>) {
    loop {
      match self.peek() {
        Some(c) if c.is_ascii_digit() => { self.advance(); },

        Some(c) if c.to_ascii_lowercase() == 'e' => {
          self.advance();
          let err = self.read_exponent();
          // Recover from a failed exponent by parsing the rest as invalid until there is something which isnt an identifier part
          if err.is_some() {
            self.advance_while(|x| x.is_identifier_part());
          }
          let token = if err.is_none() { TokenType::LiteralNumber } else { TokenType::InvalidToken };
          return (Some(self.token(start, token)), err);
        },

        Some(c) if c == '.' => {
          self.advance();
          let err_start = self.cur;
          loop {
            match self.peek() {
              Some(c) if c.is_ascii_digit() => { self.advance(); },
              _ => break
            }
          }
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(PeriodInFloat), false, "Invalid period in floating point number")
            .primary(err_start..self.next_idx(), "A floating point number may not have more than one period")
          ))
        },

        // The character following a numeric literal cannot be an identifier start or digit
        // see: https://www.ecma-international.org/ecma-262/5.1/#sec-7.8.3
        Some(c) if c.is_identifier_start() => {
          self.advance();
          return (None, Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IdentifierStartAfterNumber), false, "Invalid character after numeric literal")
            .primary(self.cur..self.next_idx(), "The start to an identifier cannot appear directly after a numeric literal")
            .note("Help: Did you forget a space after the number?")
          ));
        },

        Some(c) if !c.is_identifier_start() => {
          return (Some(self.token(start, TokenType::LiteralNumber)), None);
        },

        None => return (Some(self.token(start, TokenType::LiteralNumber)), None),

        _ => panic!("Wildcard triggered while parsing numeric literal")
      }
    }
  }

  /// validates an exponent part in a number to see if it is formed correctly
  /// expects the current pos to be the E / e 
  /// The method will not verify that the character after the numbers is whitespace or an identifier start (invalid)
  fn read_exponent(&mut self) -> Option<ParserDiagnostic<'a>> {
    let next = self.peek();
    
    //exponents may contain an explicit sign
    if next == Some('+') || next == Some('-') {
      self.advance();
    }

    //check if the exponent contains no digits which is disallowed
    match self.peek() {
      Some(c) if !c.is_ascii_digit() => 
        return Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteExponent), false, "Incomplete number exponent")
          .primary(self.cur..self.next_idx(), "Exponents require at least one digit")),

      None => 
        return Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteExponent), false, "Incomplete number exponent")
          .primary(self.cur..self.next_idx(), "Exponents require at least one digit, but the file ends here")),

      Some(_) => { self.advance(); }
    }

    loop {
      match self.peek() {
        Some(c) if c.is_ascii_digit() => { self.advance(); },

        Some(c) if c == 'e' || c == 'E' => {
          self.advance();
          let start = self.cur;
          loop {
            match self.peek() {
              Some(c) if c.is_ascii_digit() => { self.advance(); },
              _ => break
            }
          }
          return Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(MultipleExponentsInNumber), false, "Invalid nested exponents")
            .primary(start..self.next_idx(), "A numeric literal may not contain more than one exponent")
          )
        }

        Some(c) if c == '.' => {
          self.advance();
          return Some(ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(DecimalExponent), false, "Invalid decimal exponent")
            .primary(self.cur..self.next_idx(), "")
            .note("Help: Remove the period"));
        }

        _ => return None
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::lexer::{lexer::Lexer, token::TokenType::{LiteralNumber, InvalidToken}};
  use crate::diagnostic::ParserDiagnosticType;
  use crate::lexer::error::LexerDiagnosticType::*;

  macro_rules! num_literal {
    ($source:expr) => {
      let tok = Lexer::new($source, "test").next().unwrap().0.unwrap();
      assert_eq!(tok.token_type, LiteralNumber);
      assert_eq!(tok.lexeme.content($source), $source);
    };
  }

  macro_rules! invalid_num_literal {
    // An invalid token recovery is expected
    ($source:expr, $expected_err:ident) => {
      let lexer_res = Lexer::new($source, "test").next().unwrap();
      let tok = lexer_res.0.unwrap();
      assert_eq!(tok.token_type, InvalidToken);
      assert_eq!(tok.lexeme.content($source), $source);
      assert_eq!(lexer_res.1.unwrap().error_type, ParserDiagnosticType::Lexer($expected_err));
    }
  }

  #[test]
  fn num_one_len() {
    num_literal!("1");
  }
  #[test]
  fn num_mul_len() {
    num_literal!("271894");
  }
  #[test]
  fn num_with_empty_exponent() {
    invalid_num_literal!("6e", IncompleteExponent);
  }
  #[test]
  fn num_exponent_plus_sign_empty() {
    invalid_num_literal!("6e+", IncompleteExponent);
  }
  #[test]
  fn num_exponent_negative_sign_empty() {
    invalid_num_literal!("6e-", IncompleteExponent);
  }
  #[test]
  fn num_exponent_valid() {
    num_literal!("6e55");
  }
  #[test]
  fn num_exponent_valid_plus_sign() {
    num_literal!("6e+77");
  }
  #[test]
  fn num_exponent_valid_negative_sign() {
    num_literal!("6e-77");
  }
  #[test]
  fn num_exponent_ident_after_start() {
    invalid_num_literal!("6ea", IncompleteExponent);
  }
  #[test]
  fn num_multiple_exponents() {
    invalid_num_literal!("6e65e7", MultipleExponentsInNumber);
  }
}