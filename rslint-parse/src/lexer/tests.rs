#[cfg(test)]
mod test {
  use crate::lexer::*;
  use crate::lexer::token::{TokenType::*, BinToken::*, AssignToken::*};

  macro_rules! tokens {
    ($src:expr) => {
      lexer::Lexer::new(&String::from($src), "").map(|x| x.0.unwrap()).collect::<Vec<token::Token>>();
    };
  }
  
  //TODO make this not look like garbage
  macro_rules! expect_tokens {
    ($tokens:expr, $expected:expr) => {
      assert_eq!($tokens.len(), $expected.len());
      for (idx, token) in $tokens.iter().enumerate() {
        assert_eq!(token.token_type, $expected[idx]);
      };
    };
    ($tokens:expr, $expected:expr, $no_whitespace:expr) => {
      assert_eq!($tokens.iter().filter(|x| !x.is_whitespace()).collect::<Vec<&token::Token>>().len(), $expected.len());
      for (idx, token) in $tokens.iter().filter(|x| !x.is_whitespace()).enumerate() {
        assert_eq!(token.token_type, $expected[idx]);
      }
    };
  }

  #[test]
  fn newlines() {
    let tokens = tokens!("\n\r\r\n\u{2028}\u{2029}");
    println!("tokens: {:?}", tokens);
    expect_tokens!(tokens, [vec![Linebreak; 5], vec![EndOfProgram]].concat());
  }

  #[test]
  fn empty_program() {
    let tokens = tokens!("");
    expect_tokens!(tokens, vec![EndOfProgram]);
  }

  #[test]
  fn whitespace() {
    let tokens = tokens!("\u{0009}\u{000b}\u{000c}\u{0020}\u{00a0}\u{feff}");
    expect_tokens!(tokens, vec![Whitespace, EndOfProgram]);
  }

  #[should_panic]
  #[test]
  fn invalid_templ_literals() {
    let tokens = tokens!("`");
    println!("{:?}", tokens);
  }

  #[test]
  fn single_len_tokens() {
    let tokens = tokens!("( ) { } [ ] ; ,");
    expect_tokens!(tokens, vec![ParenOpen, ParenClose, BraceOpen, BraceClose, BracketOpen, BracketClose, Semicolon, Comma, EndOfProgram], true);
  }

  #[test]
  fn connected_single_len_tokens() {
    let tokens = tokens!("{{}}");
    println!("tokens: {:?}", tokens);
    expect_tokens!(tokens, vec![BraceOpen, BraceOpen, BraceClose, BraceClose, EndOfProgram]);
  }

  #[test]
  fn assignment() {
    let tokens = tokens!("=");
    expect_tokens!(tokens, vec![BinOp(Assign), EndOfProgram]);
  }

  #[test]
  fn equality() {
    let tokens = tokens!("== === ====");
    expect_tokens!(tokens, vec![BinOp(Equality), BinOp(StrictEquality), BinOp(StrictEquality), BinOp(Assign), EndOfProgram], true);
  }

  #[test]
  fn multiple_whitespace() {
    let tokens = tokens!(" a   ");
    expect_tokens!(tokens, vec![Whitespace, Identifier, Whitespace, EndOfProgram]);
  }

  #[test]
  fn plus_sign() {
    let tokens = tokens!("+ ++ += +++");
    expect_tokens!(tokens, vec![BinOp(Add), Increment, AssignOp(AddAssign), Increment, BinOp(Add), EndOfProgram], true);
  }

  #[test]
  fn minus_sign() {
    let tokens = tokens!("- -- -= ---");
    expect_tokens!(tokens, vec![BinOp(Subtract), Decrement, AssignOp(SubtractAssign), Decrement, BinOp(Subtract), EndOfProgram], true);
  }

  #[test]
  fn less_than_sign() {
    let tokens = tokens!("< << <<= <= <<<<==<<=");
    expect_tokens!(tokens, vec![
      BinOp(LessThan), BinOp(LeftBitshift), AssignOp(LeftBitshiftAssign), BinOp(LessThanOrEqual), BinOp(LeftBitshift), AssignOp(LeftBitshiftAssign), BinOp(Assign), AssignOp(LeftBitshiftAssign), EndOfProgram
    ], true);
  }

  #[test]
  fn greater_than_sign() {
    let tokens = tokens!("> >> >>> >>= >>>= >>>>==>>=");
    for i in tokens.clone() {
      println!("tok: {}", i);
    }
    expect_tokens!(tokens, vec![
      BinOp(GreaterThan), BinOp(RightBitshift), BinOp(UnsignedRightBitshift), AssignOp(RightBitshiftAssign), AssignOp(UnsignedRightBitshiftAssign), BinOp(UnsignedRightBitshift),
      BinOp(GreaterThanOrEqual), BinOp(Assign), AssignOp(RightBitshiftAssign), EndOfProgram
    ], true);
  }

  #[test]
  fn inline_comment() {
    let tokens = tokens!("// this is an inline comment");
    expect_tokens!(tokens, vec![InlineComment, EndOfProgram]);
  }

  #[test]
  fn multiline_comment() {
    let tokens = tokens!("/* this
      is a multiline comment
      */");
    println!("toks: {:?}", tokens);
    expect_tokens!(tokens, vec![MultilineComment, EndOfProgram]);
  }

  #[should_panic]
  #[allow(unused_must_use)]
  #[test]
  fn multiline_unterminated_comment() {
    tokens!("/* this
    is a multiline unterminated comment
    ");
  }

  #[test]
  fn dot_start_decimal_literal() {
    let tokens = tokens!(".642 .643e5 .6433e+6 .653e-77 .6E-6");
    expect_tokens!(tokens, vec![LiteralNumber, LiteralNumber, LiteralNumber, LiteralNumber, LiteralNumber, EndOfProgram], true);
  }

  #[test]
  fn brace_stmt_regex() {
    let tokens = tokens!("/a[gg]/gim");
    expect_tokens!(tokens, vec![LiteralRegEx, EndOfProgram], true);
  }

  #[test]
  fn division() {
    let tokens = tokens!("let a = 6 / 3;");
    expect_tokens!(tokens, vec![Let, Identifier, BinOp(Assign), LiteralNumber, BinOp(Divide), LiteralNumber, Semicolon, EndOfProgram], true);
  }

  #[test]
  fn returned_regex_in_func() {
    let tokens = tokens!("
      function a() {
        return /aaa/g
      }
    ");
    expect_tokens!(tokens, vec![Function, Identifier, ParenOpen, ParenClose, BraceOpen, Return, LiteralRegEx, BraceClose, EndOfProgram], true);
  }
}