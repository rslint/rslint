use std::hash::Hash;
use std::ops::Range;
use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};

#[derive(Eq, PartialEq, Hash, Debug)]
pub enum LexerErrors {
  UnterminatedString,
  UnexpectedToken,
  TemplateLiteralInEs5,
  UnterminatedMultilineComment,
}

#[derive(Debug)]
pub struct LexerDiagnostic {
  pub diagnostic: Diagnostic<usize>,
  pub simple: bool,
  pub error_type: LexerErrors,
  pub file_id: usize
}

impl LexerDiagnostic {
  pub fn new(file_id: usize, r#type: LexerErrors, simple: bool, message: &str) -> Self {
    Self {
      diagnostic: Diagnostic::error()
        .with_code("LexerDiagnostic")
        .with_message(message),
      simple,
      error_type: r#type,
      file_id
    }
  }

  pub fn severity(mut self, severity: Severity) -> Self {
    self.diagnostic.severity = severity;
    self
  }

  pub fn primary(mut self, range: Range<usize>, message: &str) -> Self {
    self.diagnostic.labels.append(&mut vec![Label::primary(self.file_id, range).with_message(message)]);
    self
  }

  pub fn secondary(mut self, range: Range<usize>, message: &str) -> Self {
    self.diagnostic.labels.append(&mut vec![Label::secondary(self.file_id, range).with_message(message)]);
    self
  }

  pub fn note(mut self, message: &str) -> Self {
    self.diagnostic.notes.append(&mut vec![message.to_string()]);
    self
  }
}
