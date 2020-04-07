use super::{
  lexer::Lexer,
  token::{TokenType, Token},
  util::CharExt,
};
use crate::keyword_trie;

//unique es5 tokens, only reserved in strict mode
static ES5_RESERVED_TOKENS: [TokenType; 2] = [
  TokenType::DeclarationLet,
  TokenType::Await
];

impl<'a> Lexer<'a> {

  /* 
  * Resolve a sequence which can either be an identifier or a keyword
  * Matching uses a short circuited trie to be as fast as possible
  * The characters have to exactly match sequentially or it will be resolved as an identifier
  */
  pub fn resolve_ident_or_keyword(&mut self, ident_start: char) -> Token {
    let start = self.cur;
    if !ident_start.is_ascii_lowercase() { return self.resolve_identifier(start); }
    keyword_trie!(self, ident_start, start, {
      'a' => {
        'w' => {
          'a' => (Await, "await"),
        },
      },
      'b' => {
        'r' => (Break, "break"),
      },
      'c' => {
        'a' => {
          's' => (Case, "case"),
          't' => (Catch, "catch"),
        },
        'l' => (Class, "class"),
        'o' => (Continue, "continue"),
      },
      'd' => {
        'e' => {
          'b' => (Debugger, "debugger"),
          'f' => (Default, "default"),
          'l' => (Delete, "delete"),
        },
        'o' => (Do, "do"),
      },
      'e' => {
        'l' => (Else, "else"),
        'n' => (Enum, "enum"),
        'x' => {
          'p' => (Export, "export"),
          't' => (Extends, "extends"),
        },
      },
      'f' => {
        'a' => (LiteralFalse, "false"),
        'i' => (Finally, "finally"),
        'o' => (For, "for"),
        'u' => (Function, "function"),
      },
      'i' => {
        'f' => (If, "if"),
        'm' => {
          'p' => {
            'l' => (Implements, "implements"),
            'o' => (Import, "import"),
          },
        },
        'n' => (In, "in"),
        'n' => {
          's' => (Instanceof, "instanceof"),
          't' => (Interface, "interface"),
        },
      },
      'n' => {
        'e' => (New, "new"),
        'u' => (LiteralNull, "null"),
      },
      'p' => {
        'a' => (Package, "package"),
        'r' => {
          'i' => (Private, "private"),
          'o' => (Protected, "protected"),
        },
        'u' => (Public, "public"),
      },
      'r' => (Return, "return"),
      's' => {
        't' => (Static, "static"),
        'u' => (Super, "super"),
        'w' => (Switch, "switch"),
      },
      't' => {
        'h' => {
          'i' => (This, "this"),
          'r' => (Throw, "throw"),
        },
        'r' => (Try, "try"),
        'y' => (Typeof, "typeof"),
      },
      'v' => (Void, "void"),
      'w' => {
        'h' => (While, "while"),
        'i' => (With, "with"),
      },
      'y' => (Yield, "yield"),
      }
    )
  }

  fn resolve_keyword(&mut self, expected: TokenType, start: usize, rest: &str) -> Token {
    for i in rest[self.cur - start + 1..].chars() {
      match self.source_iter.peek() {
        Some(c) if c.1 == i => { self.advance(); },
        Some(c) if c.1 != i && c.1.is_identifier_part() => return self.resolve_identifier(start),
        Some(_) => return self.token(start, TokenType::Identifier),
        None => return self.resolve_identifier(start)
      }
    }

    match self.source_iter.peek() {
      Some(c) if !c.1.is_identifier_part() => self.token(start, expected),
      Some(_) => self.resolve_identifier(start),
      None => self.token(start, expected)
    }
  }

  // Resolves a sequence determined to be an identifier into an identifier token
  fn resolve_identifier(&mut self, start: usize) -> Token {
    loop {
      match self.source_iter.peek() {
        Some(c) if c.1.is_identifier_part() => { self.advance(); },
        Some(c) if !c.1.is_identifier_part() => return self.token(start, TokenType::Identifier),
        Some(_) => return self.token(start, TokenType::Identifier),
        None => return self.token(start, TokenType::Identifier)
      }
    }
  }
}

#[cfg(test)]
mod test {
  use crate::parse::lexer::{
    lexer::Lexer,
    token::*,
    token::TokenType::*,
  };


  macro_rules! compare_tokens {
    ($tokens:expr, $source:expr, $expected:expr) => {
      {
        assert_eq!($tokens.len(), $expected.len());
        for (idx, token) in $tokens.iter().enumerate() {
          assert_eq!(token.token_type, $expected[idx].0);
          assert_eq!(token.lexeme.content($source), $expected[idx].1);
        }
      }
    };
  }

  // #[test]
  // fn a_keywords() {
  //   let source = String::from("await \n\nbreak aaa do await \n a \n\nawait");
  //   let lexer = Lexer::new(&source);
  //   let tokens: Vec<Token> = lexer.map(|x| x.unwrap()).collect();
  //   for i in tokens.iter() {
  //     println!("Token: {}", i);
  //   }
  // }

  // #[test]
  // fn b_keywords() {
  //   let source = "break boolean bonk\n byte ";
  //   let lexer = Lexer::new(source);
  //   let tokens: Vec<Token> = lexer.map(|x| x.unwrap()).collect();
  //   compare_tokens!(tokens, source, vec![
  //     (Break, "break"),
  //     (Whitespace, " "),
  //     (Boolean, "boolean"),
  //     (Whitespace, " "),
  //     (Identifier, "bonk"),
  //     (Linebreak, "\n"),
  //     (Whitespace, " "),
  //     (Byte, "byte"),
  //     (Whitespace, " "),
  //     (EndOfProgram, "")
  //   ])
  // }
}