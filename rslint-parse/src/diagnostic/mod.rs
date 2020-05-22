use crate::lexer::error::LexerDiagnosticType;
use crate::parser::error::ParseDiagnosticType;
use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use std::ops::Range;

#[derive(Debug)]
pub struct ParserDiagnostic<'a> {
  pub diagnostic: Diagnostic<&'a str>,
  pub simple: bool,
  pub error_type: ParserDiagnosticType,
  pub file_id: &'a str,
}

impl<'a> PartialEq for ParserDiagnostic<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.error_type == other.error_type
  }
}

impl<'a> ParserDiagnostic<'a> {
  pub fn new(file_id: &'a str, r#type: ParserDiagnosticType, message: &str) -> Self {
    Self {
      diagnostic: Diagnostic::error()
        .with_code("ParseError")
        .with_message(message),
      simple: false,
      error_type: r#type,
      file_id
    }
  }

  pub fn warning(file_id: &'a str, r#type: ParserDiagnosticType, message: &str) -> Self {
    Self {
      diagnostic: Diagnostic::warning()
        .with_code("ParserWarning")
        .with_message(message),
      simple: false,
      error_type: r#type,
      file_id
    }
  }

  pub fn note(file_id: &'a str, r#type: ParserDiagnosticType, message: &str) -> Self {
    Self {
      diagnostic: Diagnostic::note()
        .with_code("ParserNote")
        .with_message(message),
      simple: false,
      error_type: r#type,
      file_id
    }
  }

  pub fn severity(mut self, severity: Severity) -> Self {
    self.diagnostic.severity = severity;
    self
  }

  pub fn primary(mut self, range: Range<usize>, message: &str) -> Self {
    if range.len() > 200 {
      self.simple = true;
    }
    self.diagnostic.labels.append(&mut vec![Label::primary(self.file_id, range).with_message(message)]);
    self
  }

  pub fn secondary(mut self, range: Range<usize>, message: &str) -> Self {
    if range.len() > 200 {
      self.simple = true;
    }
    self.diagnostic.labels.append(&mut vec![Label::secondary(self.file_id, range).with_message(message)]);
    self
  }

  pub fn help(mut self, message: &str) -> Self {
    self.diagnostic.notes.append(&mut vec![message.to_string()]);
    self
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParserDiagnosticType {
    Lexer(LexerDiagnosticType),
    Parser(ParseDiagnosticType),
}
