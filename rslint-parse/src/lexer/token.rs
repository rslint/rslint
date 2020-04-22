use crate::span::Span;
use std::fmt;
use std::collections::HashSet;
use once_cell::sync::Lazy;
use std::iter::FromIterator;
use ansi_term::Color::Red;

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

  pub fn format_with_span_source(&self, source: &str) -> String {
    format!("Type: {:?} | line: {} | span: {} ({} - {})",
      self.token_type,
      self.line,
      Red.paint(self.lexeme.content(source)),
      self.lexeme.start,
      self.lexeme.end
    )
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

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum TokenType {
  AssignOp(AssignToken),
  Await,
  BinOp(BinToken),
  BitwiseNot, // ~
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
  Const,
  Continue,
  Debugger,
  Decrement,
  Default,
  Delete,
  Do,
  Else,
  EndOfProgram,
  Enum,
  Export,
  Extends,
  False,
  Finally,
  For,
  Function,
  Identifier,
  If,
  Implements,
  Import,
  In,
  Increment, // ++
  InlineComment,
  Instanceof,
  Interface,
  Let,
  Linebreak,
  LiteralBinary,
  LiteralNumber,
  LiteralRegEx,
  LiteralString,
  LogicalNot, // !
  MultilineComment,
  New,
  Of,
  Package,
  ParenClose,
  ParenOpen,
  Period,
  Private,
  Protected,
  Public,
  Return,
  Semicolon,
  Spread, // ... -- es6
  Static,
  StrictMode, // "use strict" or 'use strict'
  Super,
  Switch,
  TemplateClosed, // }
  TemplateOpen, // ${
  This,
  Throw,
  True,
  Try,
  Typeof,
  Var,
  Void,
  While,
  Whitespace,
  With,
  Null,
  Undefined,
  Yield,
  QuestionMark,

  InvalidToken
}

/// Binary operation tokens such as <, >, and ~
/// Does not include assign ops
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum BinToken {
  Assign,
  Equality,
  Inequality,
  StrictEquality,
  StrictInequality,
  LessThan,
  LessThanOrEqual,
  GreaterThan,
  GreaterThanOrEqual,
  LeftBitshift,
  RightBitshift,
  UnsignedRightBitshift,
  Exponent,
  Add,
  Subtract,
  Multiply,
  Divide,
  Modulo,
  BitwiseOr,
  BitwiseXor,
  BitwiseAnd,
  LogicalOr,
  LogicalAnd,
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum AssignToken {
  AddAssign,
  SubtractAssign,
  MultiplyAssign,
  ExponentAssign,
  ModuloAssign,
  LeftBitshiftAssign,
  RightBitshiftAssign,
  UnsignedRightBitshiftAssign,
  BitwiseAndAssign,
  BitwiseOrAssign,
  BitwiseXorAssign,
  DivideAssign
}

pub static KEYWORDS: Lazy<HashSet<TokenType>> = Lazy::new(|| {
  use TokenType::*;
  HashSet::from_iter(vec![
    Await,
    Break,
    Case,
    Catch,
    Class,
    Const,
    Continue,
    Debugger,
    Default,
    Delete,
    Do,
    Else,
    Enum,
    Export,
    Extends,
    Finally,
    For,
    Function,
    If,
    Implements,
    Import,
    In,
    Instanceof,
    Interface,
    Let,
    New,
    Private,
    Protected,
    Public,
    Return,
    Static,
    Super,
    Switch,
    This,
    Throw,
    Try,
    Typeof,
    Var,
    Void,
    While,
    With,
    Yield
  ])
});

pub static BEFORE_EXPR: Lazy<HashSet<TokenType>> = Lazy::new(|| {
  use TokenType::*;
  HashSet::from_iter(vec![
    Spread,
    LogicalNot,
    ParenOpen,
    BracketOpen,
    BraceOpen,
    Semicolon,
    Comma,
    Colon,
    TemplateOpen,
    QuestionMark,
    Increment,
    Decrement,
    BitwiseNot,
    Await,
    Case,
    Default,
    Do,
    Else,
    Return,
    Throw,
    New,
    Extends,
    Yield,
    In,
    Typeof,
    Void,
    Delete
  ])
});

impl TokenType {
  pub fn is_keyword(&self) -> bool {
    KEYWORDS.contains(self)
  }

  pub fn is_before_expr(&self) -> bool {
    match self {
      TokenType::BinOp(_) | TokenType::AssignOp(_) => true,
      _ if BEFORE_EXPR.contains(self) => true,
      _ => false
    }
  }
}