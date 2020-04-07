use std::collections::HashMap;
use super::token::TokenType;

macro_rules! insert_tokens {
  ($map:expr, $($literal:expr => $tokentype:ident),* $(,)?) => {
    {
      $($map.insert($literal, TokenType::$tokentype);)*
    }
  };
}

fn get_reserved_tokens(version: u8, strict_mode: bool) -> HashMap<&'static str, TokenType> {
  let mut map = HashMap::new();

  insert_tokens!(map,
    "break"      => Break,
    "case"       => Case,
    "catch"      => Catch,
    "class"      => Class,
    "const"      => DeclarationConst,
    "continue"   => Continue,
    "debugger"   => Debugger,
    "default"    => Default,
    "delete"     => Delete,
    "do"         => Do,
    "else"       => Else,
    "enum"       => Enum,
    "export"     => Export,
    "extends"    => Extends,
    "false"      => LiteralFalse,
    "finally"    => Finally,
    "for"        => For,
    "function"   => Function,
    "if"         => If,
    "implements" => Implements,
    "import"     => Import,
    "in"         => In,
    "instanceof" => Instanceof,
    "interface"  => Interface,
    "new"        => New,
    "null"       => LiteralNull,
    "package"    => Package,
    "private"    => Private,
    "protected"  => Protected,
    "public"     => Public,
    "return"     => Return,
    "static"     => Static,
    "super"      => Super,
    "switch"     => Switch,
    "this"       => This,
    "throw"      => Throw,
    "true"       => LiteralTrue,
    "try"        => Try,
    "typeof"     => Typeof,
    "var"        => DeclarationVar,
    "void"       => Void,
    "while"      => While,
    "with"       => With,
  );

  if version == 5 {
    if strict_mode {
      insert_tokens!(map,
        "let"   => DeclarationLet,
        "yield" => Yield,
      );
      return map;
    }
    return map;
  }

  insert_tokens!(map,
    "await" => Await,
    "let"   => DeclarationLet,
    "yield" => Yield,
  );
  map
}


#[cfg(test)]
mod test {
  use crate::parse::lexer::*;
  use once_cell::sync::Lazy;

  static tokens: Lazy<Vec<&str>> = Lazy::new(|| vec![
    "do", "if", "in", "for", "new", "try", "var", "case", "else", "enum", "null", "this", "true",
    "void", "with", "break", "catch", "class", "const", "false", "super", "throw", "while",
    "delete", "export", "import", "return", "switch", "typeof", "default", "extends", "finally",
    "continue", "debugger", "function",
    "public", "static", "package", "private", "interface", "protected", "implements", "instanceof"
  ]);

  macro_rules! compare_tokens {
    ($vec:expr, $map:expr) => {
      assert_eq!($vec.len(), $map.len());
      for key in $vec.iter() {
        assert!($map.contains_key(key));
      }
    };
  }

  #[test]
  fn es_3_tokens() {
    let es3_token_map = reserved::get_reserved_tokens(3, false);
    let es3_tokens = [tokens.as_slice(), vec!["int","byte","char","goto","long","final","float","short","double", "native","throws","boolean","abstract","volatile","transient","synchronized"].as_slice()].concat();
    compare_tokens!(es3_tokens, es3_token_map);
  }

  #[test]
  fn es_5_tokens_non_strict() {
    let es5_token_map = reserved::get_reserved_tokens(5, false);
    compare_tokens!(tokens, es5_token_map);
  }

  #[test]
  fn es_5_tokens_strict() {
    let es5_token_map = reserved::get_reserved_tokens(5, true);
    let es5_tokens = [tokens.as_slice(), vec!["yield", "let"].as_slice()].concat();
    compare_tokens!(es5_tokens, es5_token_map);
  }
}