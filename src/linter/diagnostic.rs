use codespan_reporting::diagnostic::{Diagnostic, Label, Severity, LabelStyle};
use std::ops::Range;
use crate::parse::lexer::error::LexerDiagnosticType;

#[derive(Debug)]
pub struct LinterDiagnostic<'a> {
  pub diagnostic: Diagnostic<&'a str>,
  pub simple: bool,
  pub error_type: LinterDiagnosticType,
  pub file_id: &'a str,
}

impl<'a> LinterDiagnostic<'a> {
  pub fn new(file_id: &'a str, r#type: LinterDiagnosticType, simple: bool, message: &str) -> Self {
    Self {
      diagnostic: Diagnostic::error()
        .with_code("RSLint")
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

#[derive(Debug)]
pub enum LinterDiagnosticType {
  Lexer(LexerDiagnosticType)
}

