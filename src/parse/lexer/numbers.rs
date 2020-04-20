use super::{
  lexer::Lexer,
  token::{TokenType, Token},
  util::CharExt,
  error::LexerDiagnosticType::*
};
use crate::linter::diagnostic::{LinterDiagnosticType, LinterDiagnostic};

impl<'a> Lexer<'a> {
  // Reads a numeric literal which does not start with a dot and starts with 1 - 9
  // Expects the current pos to be the first digit
  pub fn read_number(&mut self) -> Result<Token, LinterDiagnostic<'a>> {
    let start = self.cur;

    loop {
      match self.peek() {
        Some(c) if c.is_ascii_digit() => { self.advance(); },
        Some(c) if c == '.' => {
          self.advance();
          return self.read_decimal(start);
        }
        Some(c) if c.is_identifier_start() => {
          self.advance();
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(IdentifierStartAfterNumber), false, "Invalid character after numeric literal")
            .primary(self.cur..self.cur + 1, "The start to an identifier cannot appear directly after a numeric literal")
            .note("Help: Did you forget a space after the number?")
          );
        }
        _ => {
          return Ok(self.token(start, TokenType::LiteralNumber))
        }
      }
    }
  }

  // Reads a decimal composed of . DecimalDigits ExponentPart(opt)
  pub fn read_decimal(&mut self, start: usize) -> Result<Token, LinterDiagnostic<'a>> {
    loop {
      match self.peek() {
        Some(c) if c.is_ascii_digit() => { self.advance(); },

        Some(c) if c.to_ascii_lowercase() == 'e' => {
          self.advance();
          self.read_exponent()?;
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
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(PeriodInFloat), false, "Invalid period in floating point number")
            .primary(err_start..self.cur + 1, "A floating point number may not have more than one period")
          )
        },

        // The character following a numeric literal cannot be an identifier start or digit
        // see: https://www.ecma-international.org/ecma-262/5.1/#sec-7.8.3
        Some(c) if c.is_identifier_start() => {
          self.advance();
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(IdentifierStartAfterNumber), false, "Invalid character after numeric literal")
            .primary(self.cur..self.cur + 1, "The start to an identifier cannot appear directly after a numeric literal")
            .note("Help: Did you forget a space after the number?")
          );
        },

        Some(c) if !c.is_identifier_start() => {
          return Ok(self.token(start, TokenType::LiteralNumber));
        },

        None => return Ok(self.token(start, TokenType::LiteralNumber)),

        _ => panic!("Wildcard triggered while parsing numeric literal")
      }
    }
  }

  // validates an exponent part in a number to see if it is formed correctly
  // expects the current pos to be the E / e 
  // The method will not verify that the character after the numbers is whitespace or an identifier start (invalid)
  fn read_exponent(&mut self) -> Result<(), LinterDiagnostic<'a>> {
    let next = self.peek();
    
    //exponents may contain an explicit sign
    if next == Some('+') || next == Some('-') {
      self.advance();
    }

    //check if the exponent contains no digits which is disallowed
    match self.peek() {
      Some(c) if !c.is_ascii_digit() => 
        return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(IncompleteExponent), false, "Incomplete number exponent")
          .primary(self.cur..self.cur + 1, "Exponents require at least one digit")),

      None => 
        return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(IncompleteExponent), false, "Incomplete number exponent")
          .primary(self.cur..self.cur + 1, "Exponents require at least one digit but the file ends here")),

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
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(MultipleExponentsInNumber), false, "Invalid nested exponents")
            .primary(start..self.cur + 1, "A numeric literal may not contain more than one exponent")
          )
        }

        Some(c) if c == '.' => {
          self.advance();
          return Err(LinterDiagnostic::new(self.file_id, LinterDiagnosticType::Lexer(DecimalExponent), false, "Exponents may not be decimals")
            .primary(self.cur..self.cur + 1, "")
            .note("Help: Remove the period"));
        }

        _ => return Ok(())
      }
    }
  }
}
