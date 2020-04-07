use crate::parse::span::Span;
use std::fmt;

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
  Instanceof,
  Interface,
  Lesser, // <
  LesserEquals, // <=
  Linebreak,
  LiteralBinary,
  LiteralFalse,
  LiteralNull,
  LiteralNumber,
  LiteralRegEx,
  LiteralString,
  LiteralTrue,
  LiteralUndefined,
  LogicalAnd, // &&
  LogicalNot, // !
  LogicalOr, // ||
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
  Try,
  Typeof,
  UnsignedBitshiftRight, // >>>
  Void,
  While,
  Whitespace,
  With,
  Yield,
}