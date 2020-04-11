use crate::parse::span::Span;
use std::fmt;
use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Token {
  pub token_type: TokenType,
  pub lexeme: Span,
  pub line: usize,
}

impl Token {
  pub fn new(token_type: TokenType, start: usize, stop: usize, line: usize) -> Self {
    Self {
      token_type,
      lexeme: Span::new(start, stop),
      line
    }
  }

  pub fn is_whitespace(&self) -> bool {
    self.token_type == TokenType::Whitespace || self.token_type == TokenType::Linebreak
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Type: {:?} | span: {} - {} | line: {}",
      self.token_type,
      self.lexeme.start,
      self.lexeme.end,
      self.line
    )
  }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
  Accessor,
  AddAssign, // +=
  Addition, // +
  Arrow, // =>
  Assign, // =
  Await,
  BitwiseAnd, // &
  BitwiseAndAssign, // &=
  BitwiseLeftAssign, // <<=
  BitwiseLeftShift, // <<
  BitwiseNot, // ~
  BitwiseNotAssign, // ~=
  BitwiseOr, // |
  BitwiseOrAssign, // |=
  BitwiseRightAssign, // >>=
  BitwiseRightShift, // >>
  BitwiseUnsignedRightAssign, // >>>=
  BitwiseXor, // ^
  BitwiseXorAssign, // ^=
  BraceClose,
  BraceOpen,
  BracketClose,
  BracketOpen,
  Break,
  Case,
  Catch,
  Class,
  Colon,
  Comma,
  Conditional,
  Continue,
  Debugger,
  DeclarationConst,
  DeclarationLet,
  DeclarationVar,
  Decrement,
  Default,
  Delete,
  DivideAssign, // /=
  Division,
  Do,
  Else,
  EndOfProgram,
  Enum,
  Equality,
  Exponent, // ** -- es7
  ExponentAssign, // **= -- es7
  Export,
  Extends,
  False,
  Finally,
  For,
  Function,
  Greater, // >
  GreaterEquals, //>=
  Identifier,
  If,
  Implements,
  Import,
  In,
  Increment, // ++
  Inequality, // !=
  InlineComment,
  Instanceof,
  Interface,
  Lesser, // <
  LesserEquals, // <=
  Linebreak,
  LiteralBinary,
  LiteralNumber,
  LiteralRegEx,
  LiteralString,
  LogicalAnd, // &&
  LogicalNot, // !
  LogicalOr, // ||
  MultilineComment,
  Multiplication, // *
  MultiplyAssign, // *=
  New,
  Package,
  ParenClose,
  ParenOpen,
  Private,
  Protected,
  Public,
  Remainder, // %
  RemainderAssign, // %=
  Return,
  Semicolon,
  Spread, // ... -- es6
  Static,
  StrictEquality, // ===
  StrictInequality, // !==
  StrictMode, // "use strict" or 'use strict'
  SubtractAssign, // -=
  Subtraction, // -
  Super,
  Switch,
  TemplateClosed, // }
  TemplateOpen, // ${
  This,
  Throw,
  True,
  Try,
  Typeof,
  UnsignedBitshiftRight, // >>>
  Void,
  While,
  Whitespace,
  With,
  Null,
  Undefined,
  Yield,
}