//! Simple diagnostics builder for building codespan diagnostics

use crate::ParserError;
use codespan_reporting::diagnostic::*;
use std::ops::Range;

/// A builder for generating codespan diagnostics
pub struct ErrorBuilder {
    pub inner: ParserError,
    pub file_id: usize,
}

impl ErrorBuilder {
    /// Make a new builder with error severity
    pub fn error(file_id: usize, message: &str) -> Self {
        Self {
            inner: Diagnostic::error()
                .with_code("ParserError")
                .with_message(message),
            file_id,
        }
    }

    /// Make a new builder with a predefined diagnostic
    pub fn new(file_id: usize, diagnostic: Diagnostic<usize>) -> Self {
        Self {
            inner: diagnostic,
            file_id,
        }
    }

    /// Change the severity of this diagnostic
    pub fn severity(mut self, severity: Severity) -> Self {
        self.inner.severity = severity;
        self
    }

    /// Add a primary label to the diagnostic
    pub fn primary(mut self, range: impl Into<Range<usize>>, message: impl AsRef<str>) -> Self {
        self.inner.labels.append(&mut vec![
            Label::primary(self.file_id, range.into()).with_message(message.as_ref())
        ]);
        self
    }

    /// Add a secondary label to this diagnostic
    pub fn secondary(mut self, range: impl Into<Range<usize>>, message: impl AsRef<str>) -> Self {
        self.inner.labels.append(&mut vec![
            Label::secondary(self.file_id, range.into()).with_message(message.as_ref())
        ]);
        self
    }

    /// Add a help message to the bottom of the diagnostic (usually a `Help:` or `Note:` message)
    pub fn help(mut self, message: &str) -> Self {
        self.inner.notes.append(&mut vec![message.to_string()]);
        self
    }

    /// Consume the builder and return its diagnostic
    pub fn end(self) -> Diagnostic<usize> {
        self.inner
    }
}

impl From<ErrorBuilder> for Diagnostic<usize> {
    fn from(builder: ErrorBuilder) -> Diagnostic<usize> {
        builder.end()
    }
}
