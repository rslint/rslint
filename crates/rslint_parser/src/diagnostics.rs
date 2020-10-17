//! Simple diagnostics builder for building codespan diagnostics

use crate::ParserError;
use rslint_errors::{Diagnostic, Severity};
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
            inner: Diagnostic::error(file_id, "SyntaxError", message),
            file_id,
        }
    }

    pub fn warning(file_id: usize, message: &str) -> Self {
        Self {
            inner: Diagnostic::warning(file_id, "ParserWarning", message),
            file_id,
        }
    }

    /// Make a new builder with a predefined diagnostic
    pub fn new(file_id: usize, diagnostic: Diagnostic) -> Self {
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
    pub fn primary(mut self, range: impl Into<Range<usize>>, message: impl Into<String>) -> Self {
        self.inner.primary(range.into(), message);
        self
    }

    /// Add a secondary label to this diagnostic
    pub fn secondary(mut self, range: impl Into<Range<usize>>, message: impl Into<String>) -> Self {
        self.inner.secondary(range.into(), message);
        self
    }

    /// Add a help message to the bottom of the diagnostic, that is prefixed by a "help:".
    pub fn help(mut self, message: &str) -> Self {
        self.inner.footer_help(message);
        self
    }

    /// Add a help message to the bottom of the diagnostic, that is prefixed by a "note:".
    pub fn note(mut self, message: &str) -> Self {
        self.inner.footer_note(message);
        self
    }

    /// Consume the builder and return its diagnostic
    pub fn end(self) -> Diagnostic {
        self.inner
    }
}

impl From<ErrorBuilder> for Diagnostic {
    fn from(builder: ErrorBuilder) -> Diagnostic {
        builder.end()
    }
}
