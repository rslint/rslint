use crate::{
    file::{FileId, FileSpan, Span},
    Severity,
};
use smol_str::SmolStr;

/// A diagnostic message that can give information
/// like errors or warnings.
pub struct Diagnostic {
    file_id: FileId,

    severity: Severity,
    code: Option<SmolStr>,
    title: SmolStr,

    children: Vec<SubDiagnostic>,
}

impl Diagnostic {
    /// Creates a new [`Diagnostic`] that will be used in a builder-like way
    /// to modify labels, and suggestions.
    pub fn new(file_id: FileId, severity: Severity, title: impl Into<SmolStr>) -> Self {
        Self::new_with_code(file_id, severity, title, None)
    }

    /// Creates a new [`Diagnostic`] with an error code that will be used in a builder-like way
    /// to modify labels, and suggestions.
    pub fn new_with_code(
        file_id: FileId,
        severity: Severity,
        title: impl Into<SmolStr>,
        code: impl Into<Option<SmolStr>>,
    ) -> Self {
        Self {
            file_id,
            code: code.into(),
            severity,
            title: title.into(),
            children: vec![],
        }
    }

    /// Attaches a primary label to this [`Diagnostic`].
    ///
    /// A primary is just a label with the [`Error`](Severity::Error) severity.
    pub fn primary(mut self, span: impl Span, msg: impl Into<SmolStr>) -> Self {
        self.children.push(SubDiagnostic {
            severity: Severity::Error,
            msg: msg.into(),
            span: FileSpan::new(self.file_id, span),
        });
        self
    }

    /// Attaches a secondary label to this [`Diagnostic`].
    ///
    /// A secondary is just a label with the [`Info`](Severity::Info) severity.
    pub fn secondary(mut self, span: impl Span, msg: impl Into<SmolStr>) -> Self {
        self.children.push(SubDiagnostic {
            severity: Severity::Info,
            msg: msg.into(),
            span: FileSpan::new(self.file_id, span),
        });
        self
    }
}

/// Everything that can be added to a diagnostic, like
/// a suggestion that will be displayed under the actual error.
#[derive(Debug, Clone)]
pub struct SubDiagnostic {
    severity: Severity,
    msg: SmolStr,
    span: FileSpan,
}
