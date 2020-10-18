use crate::{
    file::{FileId, FileSpan, Span},
    Applicability, CodeSuggestion, DiagnosticTag, Severity,
};

/// A diagnostic message that can give information
/// like errors or warnings.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Diagnostic {
    pub file_id: FileId,

    pub severity: Severity,
    pub code: Option<String>,
    pub title: String,
    pub tag: Option<DiagnosticTag>,

    pub primary: Option<SubDiagnostic>,
    pub children: Vec<SubDiagnostic>,
    pub suggestions: Vec<CodeSuggestion>,
    pub footers: Vec<Footer>,
}

impl Diagnostic {
    /// Creates a new [`Diagnostic`] with the `Error` severity.
    pub fn error(file_id: FileId, code: impl Into<String>, title: impl Into<String>) -> Self {
        Self::new_with_code(file_id, Severity::Error, title, Some(code.into()))
    }

    /// Creates a new [`Diagnostic`] with the `Warning` severity.
    pub fn warning(file_id: FileId, code: impl Into<String>, title: impl Into<String>) -> Self {
        Self::new_with_code(file_id, Severity::Warning, title, Some(code.into()))
    }

    /// Creates a new [`Diagnostic`] with the `Help` severity.
    pub fn help(file_id: FileId, code: impl Into<String>, title: impl Into<String>) -> Self {
        Self::new_with_code(file_id, Severity::Help, title, Some(code.into()))
    }

    /// Creates a new [`Diagnostic`] with the `Note` severity.
    pub fn note(file_id: FileId, code: impl Into<String>, title: impl Into<String>) -> Self {
        Self::new_with_code(file_id, Severity::Note, title, Some(code.into()))
    }

    /// Creates a new [`Diagnostic`] with the `Info` severity.
    pub fn info(file_id: FileId, code: impl Into<String>, title: impl Into<String>) -> Self {
        Self::new_with_code(file_id, Severity::Info, title, Some(code.into()))
    }

    /// Creates a new [`Diagnostic`] that will be used in a builder-like way
    /// to modify labels, and suggestions.
    pub fn new(file_id: FileId, severity: Severity, title: impl Into<String>) -> Self {
        Self::new_with_code(file_id, severity, title, None)
    }

    /// Creates a new [`Diagnostic`] with an error code that will be used in a builder-like way
    /// to modify labels, and suggestions.
    pub fn new_with_code(
        file_id: FileId,
        severity: Severity,
        title: impl Into<String>,
        code: Option<String>,
    ) -> Self {
        Self {
            file_id,
            code,
            severity,
            title: title.into(),
            primary: None,
            tag: None,
            children: vec![],
            suggestions: vec![],
            footers: vec![],
        }
    }

    /// Overwrites the severity of this diagnostic.
    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    /// Marks this diagnostic as deprecated code, which will
    /// be displayed in the language server.
    ///
    /// This does not have any influence on the diagnostic rendering.
    pub fn deprecated(mut self) -> Self {
        self.tag = if matches!(self.tag, Some(DiagnosticTag::Unnecessary)) {
            Some(DiagnosticTag::Both)
        } else {
            Some(DiagnosticTag::Deprecated)
        };
        self
    }

    /// Marks this diagnostic as unnecessary code, which will
    /// be displayed in the language server.
    ///
    /// This does not have any influence on the diagnostic rendering.
    pub fn unnecessary(mut self) -> Self {
        self.tag = if matches!(self.tag, Some(DiagnosticTag::Deprecated)) {
            Some(DiagnosticTag::Both)
        } else {
            Some(DiagnosticTag::Unnecessary)
        };
        self
    }

    /// Attaches a label to this [`Diagnostic`], that will point to another file
    /// that is provided.
    pub fn label_in_file(mut self, severity: Severity, span: FileSpan, msg: String) -> Self {
        self.children.push(SubDiagnostic {
            severity,
            msg,
            span,
        });
        self
    }

    /// Attaches a label to this [`Diagnostic`].
    ///
    /// The given span has to be in the file that was provided while creating this [`Diagnostic`].
    pub fn label(mut self, severity: Severity, span: impl Span, msg: impl Into<String>) -> Self {
        self.children.push(SubDiagnostic {
            severity,
            msg: msg.into(),
            span: FileSpan::new(self.file_id, span),
        });
        self
    }

    /// Attaches a primary label to this [`Diagnostic`].
    ///
    /// A primary is just a label with the [`Error`](Severity::Error) severity.
    pub fn primary(mut self, span: impl Span, msg: impl Into<String>) -> Self {
        self.primary = Some(SubDiagnostic {
            severity: self.severity,
            msg: msg.into(),
            span: FileSpan::new(self.file_id, span),
        });
        self
    }

    /// Attaches a secondary label to this [`Diagnostic`].
    ///
    /// A secondary is just a label with the [`Info`](Severity::Info) severity.
    pub fn secondary(self, span: impl Span, msg: impl Into<String>) -> Self {
        self.label(Severity::Info, span, msg)
    }

    /// Prints out a message that suggests a possible solution, that is in another
    /// file as this `Diagnostic`, to the error.
    ///
    /// If the message plus the suggestion is longer than 25 chars,
    /// the suggestion is displayed as a new children of this `Diagnostic`,
    /// otherwise it will be inlined with the other labels.
    ///
    /// A suggestion is displayed like:
    /// ```no_rust
    /// try adding a `;`: console.log();
    /// ```
    /// or in a separate multiline suggestion
    ///
    /// The message should not contain the `:` because it's added automatically.
    /// The suggestion will automatically be wrapped inside two backticks.
    pub fn suggestion_in_file(
        mut self,
        span: impl Span,
        msg: &str,
        suggestion: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        let suggestion = CodeSuggestion {
            substitution: (None, span.as_range(), suggestion.into()),
            applicability,
            msg: msg.to_string(),
        };
        self.suggestions.push(suggestion);
        self
    }

    /// Prints out a message that suggests a possible solution to the error.
    ///
    /// If the message plus the suggestion is longer than 25 chars,
    /// the suggestion is displayed as a new children of this `Diagnostic`,
    /// otherwise it will be inlined with the other labels.
    ///
    /// A suggestion is displayed like:
    /// ```no_rust
    /// try adding a `;`: console.log();
    /// ```
    /// or in a separate multiline suggestion
    ///
    /// The message should not contain the `:` because it's added automatically.
    /// The suggestion will automatically be wrapped inside two backticks.
    pub fn suggestion(
        mut self,
        span: impl Span,
        msg: &str,
        suggestion: impl Into<String>,
        applicability: Applicability,
    ) -> Self {
        let suggestion = CodeSuggestion {
            substitution: (None, span.as_range(), suggestion.into()),
            applicability,
            msg: msg.to_string(),
        };
        self.suggestions.push(suggestion);
        self
    }

    /// Adds a footer to this `Diagnostic`, which will be displayed under the actual error.
    pub fn footer(mut self, severity: Severity, msg: impl Into<String>) -> Self {
        self.footers.push(Footer {
            msg: msg.into(),
            severity,
        });
        self
    }

    /// Adds a footer to this `Diagnostic`, with the `Help` severity.
    pub fn footer_help(self, msg: impl Into<String>) -> Self {
        self.footer(Severity::Help, msg)
    }

    /// Adds a footer to this `Diagnostic`, with the `Note` severity.
    pub fn footer_note(self, msg: impl Into<String>) -> Self {
        self.footer(Severity::Note, msg)
    }
}

/// Everything that can be added to a diagnostic, like
/// a suggestion that will be displayed under the actual error.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SubDiagnostic {
    pub severity: Severity,
    pub msg: String,
    pub span: FileSpan,
}

/// A note or help that is displayed under the diagnostic.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Footer {
    pub msg: String,
    pub severity: Severity,
}
