//! A simple builder for facilitating the creation of diagnostics

use codespan_reporting::diagnostic::{Severity, Label, Diagnostic};
use std::ops::Range;

/// A simple builder for creating codespan diagnostics sequentially
#[derive(Debug, Clone)]
pub struct DiagnosticBuilder<'a>(Diagnostic<&'a str>, &'a str);

impl<'a> DiagnosticBuilder<'a> {
    /// Create a new builder with a severity of error
    pub fn error<T: Into<String>>(file_id: &'a str, code: T, message: T) -> Self {
        Self(Diagnostic {
            code: Some(code.into()),
            message: message.into(),
            severity: Severity::Error,
            labels: vec![],
            notes: vec![],
        }, file_id)
    }

    /// Change the severity of this diagnostic
    pub fn severity(mut self, severity: Severity) -> Self {
        self.0.severity = severity;
        self
    }
    
    /// Add a primary label to the diagnostic
    pub fn primary(mut self, range: impl Into<Range<usize>>, message: impl AsRef<str>) -> Self {
        self.0.labels.append(&mut vec![Label::primary(self.1, range.into()).with_message(message.as_ref())]);
        self
    }

    /// Add a secondary label to this diagnostic
    pub fn secondary(mut self, range: impl Into<Range<usize>>, message: impl AsRef<str>) -> Self {
        self.0.labels.append(&mut vec![Label::secondary(self.1, range.into()).with_message(message.as_ref())]);
        self
    }

    /// Add a help message to the bottom of the diagnostic (usually a `Help:` or `Note:` message)
    pub fn help(mut self, message: &str) -> Self {
        self.0.notes.append(&mut vec![message.to_string()]);
        self
    }
}

impl<'a> From<DiagnosticBuilder<'a>> for Diagnostic<&'a str> {
    fn from(builder: DiagnosticBuilder<'a>) -> Diagnostic<&'a str> {
        builder.0
    }
}