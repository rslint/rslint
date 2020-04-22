use codespan_reporting::diagnostic::{Diagnostic, Label, Severity};
use std::ops::Range;
use crate::parse::lexer::error::LexerDiagnosticType;
use super::file_walker::FileWalker;

#[derive(Debug)]
pub struct LinterDiagnostic<'a> {
  pub diagnostic: Diagnostic<&'a str>,
  pub simple: bool,
  pub error_type: LinterDiagnosticType,
  pub file_id: &'a str,
}

impl<'a> PartialEq for LinterDiagnostic<'a> {
  fn eq(&self, other: &Self) -> bool {
    self.error_type == other.error_type
  }
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

  pub fn throw(&self, walker: &FileWalker) {
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
    use codespan_reporting::term::DisplayStyle;
    use codespan_reporting::term;

    let writer = if self.diagnostic.severity == Severity::Error {
      StandardStream::stderr(ColorChoice::Always)
    } else {
      StandardStream::stdout(ColorChoice::Always)
    };

    let mut config = term::Config::default();
    if self.simple {
      config.display_style = DisplayStyle::Short;
    }

    term::emit(&mut writer.lock(), &config, walker, &self.diagnostic)
      .expect("Failed to throw diagnostic");
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

#[derive(Debug, PartialEq, Eq)]
pub enum LinterDiagnosticType {
  Lexer(LexerDiagnosticType)
}

