macro_rules! tokens {
  ($src:expr) => {
    lexer::Lexer::new(&String::from($src), 0).map(|x| x.unwrap()).collect::<Vec<token::Token>>();
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

#[cfg(test)]
mod test {
  use crate::parse::lexer::*;
  use crate::parse::lexer::token::TokenType::*;

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
    expect_tokens!(tokens, vec![Assign, EndOfProgram]);
  }

  #[test]
  fn equality() {
    let tokens = tokens!("== === ====");
    expect_tokens!(tokens, vec![Equality, StrictEquality, StrictEquality, Assign, EndOfProgram], true);
  }

  #[test]
  fn multiple_whitespace() {
    let tokens = tokens!(" a   ");
    expect_tokens!(tokens, vec![Whitespace, Identifier, Whitespace, EndOfProgram]);
  }

  #[test]
  fn plus_sign() {
    let tokens = tokens!("+ ++ += +++");
    expect_tokens!(tokens, vec![Addition, Increment, AddAssign, Increment, Addition, EndOfProgram], true);
  }

  #[test]
  fn minus_sign() {
    let tokens = tokens!("- -- -= ---");
    expect_tokens!(tokens, vec![Subtraction, Decrement, SubtractAssign, Decrement, Subtraction, EndOfProgram], true);
  }

  #[test]
  fn less_than_sign() {
    let tokens = tokens!("< << <<= <= <<<<==<<=");
    expect_tokens!(tokens, vec![
      Lesser, BitwiseLeftShift, BitwiseLeftAssign, LesserEquals, BitwiseLeftShift, BitwiseLeftAssign, Assign, BitwiseLeftAssign, EndOfProgram
    ], true);
  }

  #[test]
  fn greater_than_sign() {
    let tokens = tokens!("> >> >>> >>= >>>= >>>>==>>=");
    for i in tokens.clone() {
      println!("tok: {}", i);
    }
    expect_tokens!(tokens, vec![
      Greater, BitwiseRightShift, UnsignedBitshiftRight, BitwiseRightAssign, BitwiseUnsignedRightAssign, UnsignedBitshiftRight,
      GreaterEquals, Assign, BitwiseRightAssign, EndOfProgram
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
  #[test]
  fn multiline_unterminated_comment() {
    let tokens = tokens!("/* this
    is a multiline unterminated comment
    ");
  }
}