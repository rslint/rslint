//! A simple builder for facilitating the creation of diagnostics

use crate::{Diagnostic, RuleResult};
use codespan_reporting::diagnostic::{Label, Severity};
use std::ops::Range;

/// A simple builder for creating codespan diagnostics sequentially
#[derive(Debug, Clone)]
pub struct DiagnosticBuilder(Diagnostic, usize);

impl DiagnosticBuilder {
    /// Create a new builder with a severity of error
    pub fn error(file_id: usize, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(
            Diagnostic {
                code: Some(code.into()),
                message: message.into(),
                severity: Severity::Error,
                labels: vec![],
                notes: vec![],
            },
            file_id,
        )
    }

    /// Create a new builder with a severity of warning
    pub fn warning(file_id: usize, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self(
            Diagnostic {
                code: Some(code.into()),
                message: message.into(),
                severity: Severity::Warning,
                labels: vec![],
                notes: vec![],
            },
            file_id,
        )
    }

    /// Create a new builder with a severity of note
    pub fn note_diagnostic(
        file_id: usize,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self(
            Diagnostic {
                code: Some(code.into()),
                message: message.into(),
                severity: Severity::Note,
                labels: vec![],
                notes: vec![],
            },
            file_id,
        )
    }

    /// Change the severity of this diagnostic
    pub fn severity(mut self, severity: Severity) -> Self {
        self.0.severity = severity;
        self
    }

    /// Add a primary label to the diagnostic
    pub fn primary(mut self, range: impl Into<Range<usize>>, message: impl AsRef<str>) -> Self {
        self.0.labels.append(&mut vec![
            Label::primary(self.1, range.into()).with_message(message.as_ref())
        ]);
        self
    }

    /// Add a secondary label to this diagnostic
    pub fn secondary(mut self, range: impl Into<Range<usize>>, message: impl AsRef<str>) -> Self {
        self.0.labels.append(&mut vec![
            Label::secondary(self.1, range.into()).with_message(message.as_ref())
        ]);
        self
    }

    /// Add a note message to the bottom of the diagnostic (usually a `Help:` or `Note:` message)
    pub fn note(mut self, message: impl AsRef<str>) -> Self {
        self.0.notes.append(&mut vec![message.as_ref().to_owned()]);
        self
    }

    pub fn finish(self) -> Diagnostic {
        self.0
    }
}

impl From<DiagnosticBuilder> for Diagnostic {
    fn from(builder: DiagnosticBuilder) -> Diagnostic {
        builder.0
    }
}

impl From<DiagnosticBuilder> for RuleResult {
    fn from(builder: DiagnosticBuilder) -> RuleResult {
        RuleResult {
            diagnostics: vec![builder.into()],
        }
    }
}

impl From<DiagnosticBuilder> for Option<RuleResult> {
    fn from(builder: DiagnosticBuilder) -> Option<RuleResult> {
        Some(RuleResult {
            diagnostics: vec![builder.into()],
        })
    }
}
