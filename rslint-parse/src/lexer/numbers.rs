use super::{
  lexer::{Lexer, LexResult},
  token::TokenType,
  util::CharExt,
  error::LexerDiagnosticType::*
};
use crate::diagnostic::*;

impl<'a> Lexer<'a> {

  pub fn read_number(&mut self, dot_start: bool, zero_start: bool) -> LexResult<'a> {
    let start = if dot_start { self.cur - 1 } else { self.cur };

    if dot_start {
      // Lexer makes sure the next character is a number beforehand
      self.read_num_with_possible_expnt(start, false, false)

    } else {
      if zero_start {
        match self.advance() {
          // TODO: maybe clean this up
          Some(c) if c.to_ascii_lowercase() == 'x' && self.peek().filter(|x| x.is_ascii_hexdigit()).is_some() => {
            return self.read_hex_literal(start);
          },

          Some(c) if c.to_ascii_lowercase() == 'e' => {
            let exp_start = self.cur;
            let mut res = self.read_exponent(start);
            if res.1.is_none() {
              res.1 = Some(ParserDiagnostic::note(self.file_id, ParserDiagnosticType::Lexer(RedundantExponent), "Redundant exponent after zero literal")
                .primary(exp_start..self.cur, "Redundant, will evaluate to `0`"));
            }
            return res;
          },

          Some(c) if c == '.' => {
            return self.read_num_with_possible_expnt(start, false, true);
          },

          Some(c) if c.is_identifier_start() => return self.recover_from_ident(start),

          // Literal 0
          Some(c) if !c.is_identifier_start() => return (Some(self.token(start, TokenType::LiteralNumber)), None),
          None => return (Some(self.token(start, TokenType::LiteralNumber)), None),

          _ => unimplemented!()
        }
      }

      return self.read_num_with_possible_expnt(start, true, false);
    }
  }

  fn read_num_with_possible_expnt(&mut self, start: usize, mut dot_possible: bool, mut zero: bool) -> LexResult<'a> {
    let mut trailing_zeroes: Option<usize> = if zero { Some(self.cur) } else { None };
    // Add a note about redundant zeroes such as 50.000
    //                                              ^^^
    loop {
      match self.advance() {
        Some(c) if c.to_ascii_lowercase() == 'e' => {
          let exponent_start = self.cur;
          let mut res = self.read_exponent(start);
          if res.1.is_none() && trailing_zeroes.is_some() {
            res.1 = Some(ParserDiagnostic::note(self.file_id, ParserDiagnosticType::Lexer(RedundantExponent), "Redundant exponent after zero literal")
              .primary(exponent_start..self.cur, "Redundant, will evaluate to `0`"));
          }
          return res;
        },

        Some('.') if dot_possible => {
          dot_possible = false;
          trailing_zeroes = None;
        },

        Some(c) if c.is_ascii_digit() => {
          zero = false;
          if c == '0' && trailing_zeroes.is_none() {
            trailing_zeroes = Some(self.cur);
          } else if c != '0' { 
            trailing_zeroes = None;
          }
        },
        _ => break
      }
    }

    let err = if trailing_zeroes.is_some() && !dot_possible {
      Some(ParserDiagnostic::note(self.file_id, ParserDiagnosticType::Lexer(RedundantZeroesAfterNumber), "Redundant zeroes after number literal")
        .primary(trailing_zeroes.unwrap()..self.cur, "Redundant, will be truncated")
      )
    } else { None };

    match self.cur_char {
      c if c.to_ascii_lowercase() == 'e' => {
        let err_start = self.cur;

        let mut res = self.read_exponent(start);
        if res.1.is_none() { res.1 = err };
        // Warn about redundant exponents like 0.e+5 / 0.e-5
        if res.1.is_none() && zero && trailing_zeroes.is_some() {
          res.1 = Some(ParserDiagnostic::note(self.file_id, ParserDiagnosticType::Lexer(RedundantExponent), "Redundant exponent after zero literal")
            .primary(err_start..self.cur, "Redundant, will evaluate to `0`"));
        }
        res
      },

      c if c.is_identifier_start() => {
        let mut res = self.recover_from_ident(start);
        if res.1.is_none() { res.1 = err };
        res
      },

      c if !c.is_identifier_start() =>{
        return (Some(self.token(start, TokenType::LiteralNumber)), err);
      },

      _ if self.peek() == None => return (Some(self.token(start, TokenType::LiteralNumber)), err),

      _ => unreachable!()
    }
  }

  fn recover_from_ident(&mut self, start: usize) -> LexResult<'a> {
    let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IdentifierStartAfterNumber), "Invalid identifier after number")
      .primary(self.cur..self.next_idx(), "Not allowed after a numeric literal");
    // Recover from the error by parsing as an invalid character
    self.advance_while(|x| x.is_identifier_part());

    return (Some(self.token(start, TokenType::InvalidToken)), Some(err));
  }

  // TODO: Add a note about useless exponents such as 3.3e00
  fn read_exponent(&mut self, start: usize) -> LexResult<'a> {
    match self.peek() {
      Some(next) if next == '+' || next == '-' => {
        self.advance();
      }
      _ => {}
    }

    match self.peek() {
      Some(c) if !c.is_ascii_digit() => {
        let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteExponent), "Invalid exponent without a number")
          .primary(self.cur..self.next_idx(), "Expected a digit here");
        self.advance_while(|x| x.is_identifier_part());

        return (Some(self.token(start, TokenType::InvalidToken)), Some(err));
      },
      None => {
        let next_idx = self.next_idx();
        let err = ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Lexer(IncompleteExponent), "Invalid exponent without a number")
          .primary(self.cur..next_idx, "Expected a digit");

        self.advance_while(|x| x.is_identifier_part());

      return (Some(self.token(start, TokenType::InvalidToken)), Some(err));
      },
      _ => {}
    }

    self.advance_while(|x| x.is_ascii_digit());

    match self.cur_char {
      c if !c.is_identifier_start() => {
        return (Some(self.token(start, TokenType::LiteralNumber)), None);
      }

      c if c.is_identifier_start() => self.recover_from_ident(start),

      _ => {
        return (Some(self.token(start, TokenType::LiteralNumber)), None);
      }
    }
  }

  fn read_hex_literal(&mut self, start: usize) -> LexResult<'a> {
    let (mut leading_zeroes, mut has_non_zero, mut zeroes_end): (Option<usize>, bool, Option<usize>) = (None, false, None);

    // Add a note about redundant leading zeroes: 0x0003
    //                                             ^^^
    // TODO: This should probably be made a rule later on
    loop {
      match self.advance() {
        Some('0') if !has_non_zero && leading_zeroes.is_none() => {
          leading_zeroes = Some(self.cur);
        },

        Some(c) if c.is_ascii_hexdigit() => {
          if zeroes_end.is_none() && c != '0' {
            zeroes_end = Some(self.cur);
          }
          has_non_zero = true;
        },

        _ => break
      }
    }

    let note = if leading_zeroes.is_some() {
      Some(ParserDiagnostic::note(self.file_id, ParserDiagnosticType::Lexer(RedundantHexZeroes), "Redundant leading zeroes inside hex literal")
        .primary(leading_zeroes.unwrap()..zeroes_end.unwrap_or(self.cur), "Will be ignored"))
    } else { None };

    if self.state.last_tok { return (Some(self.token(start, TokenType::LiteralNumber)), note) };

    match self.cur_char {
      c if !c.is_identifier_start() => (Some(self.token(start, TokenType::LiteralNumber)), note),

      c if c.is_identifier_start() => self.recover_from_ident(start),

      _ => unreachable!(),
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
      let tok = Lexer::new($source, 0).next().unwrap().0.unwrap();
      assert_eq!(tok.token_type, LiteralNumber);
      assert_eq!(tok.lexeme.content($source), $source);
    };
  }

  macro_rules! invalid_num_literal {
    // An invalid token recovery is expected
    ($source:expr, $expected_err:ident) => {
      let lexer_res = Lexer::new($source, 0).next().unwrap();
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
    invalid_num_literal!("6e65e7", IdentifierStartAfterNumber);
  }

  #[test]
  fn num_literal_zero() {
    num_literal!("0");
  }

  #[test]
  fn num_ident_after_literal_zero() {
    invalid_num_literal!("0a", IdentifierStartAfterNumber);
  }

  #[test]
  fn num_literal_zero_exponent() {
    num_literal!("0e6");
  }

  #[test]
  fn num_literal_zero_decimal_exponent() {
    num_literal!("0.e+6");
  }

  #[test]
  fn num_hex_literal() {
    num_literal!("0x33ABCDEFabcdef");
  }

  #[test]
  fn num_hex_literal_identifier() {
    invalid_num_literal!("0x0333nnjk", IdentifierStartAfterNumber);
  }

  #[test]
  fn num_hex_literal_all_zeroes() {
    num_literal!("0x00000");
  }

  #[test]
  fn num_hex_literal_no_digits() {
    invalid_num_literal!("0x", IdentifierStartAfterNumber);
  }

  #[test]
  fn num_decimal_redundant_zeroes() {
    let note = Lexer::new("0.500", 0).next().unwrap().1.unwrap();
    assert_eq!(note.error_type, ParserDiagnosticType::Lexer(RedundantZeroesAfterNumber));
    assert_eq!(note.diagnostic.labels[0].range, 3..4);
  }

  #[test]
  fn num_exponent_redundant_zeroes() {
    let note = Lexer::new("0.00e+6", 0).next().unwrap().1.unwrap();
    assert_eq!(note.error_type, ParserDiagnosticType::Lexer(RedundantExponent));
    assert_eq!(note.diagnostic.labels[0].range, 4..6);
  }

  #[test]
  fn num_hex_literal_redundant_zeroes() {
    let note = Lexer::new("0x00300", 0).next().unwrap().1.unwrap();
    assert_eq!(note.error_type, ParserDiagnosticType::Lexer(RedundantHexZeroes));
    assert_eq!(note.diagnostic.labels[0].range, 2..4);
  }
}