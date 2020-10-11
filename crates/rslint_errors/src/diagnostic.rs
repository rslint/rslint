use crate::{
    file::{FileId, FileSpan, Span},
    suggestion::Substitution,
    Applicability, CodeSuggestion, Severity,
};

/// A diagnostic message that can give information
/// like errors or warnings.
pub struct Diagnostic {
    pub(crate) file_id: FileId,

    pub(crate) severity: Severity,
    pub(crate) code: Option<String>,
    pub(crate) title: String,

    pub(crate) children: Vec<SubDiagnostic>,
    pub(crate) suggestions: Vec<CodeSuggestion>,
    pub(crate) footer: Vec<Footer>,
}

impl Diagnostic {
    /// Creates a new [`Diagnostic`] with the `Error` severity.
    pub fn error(file_id: FileId, title: String, code: Option<String>) -> Self {
        Self::new_with_code(file_id, Severity::Error, title, code)
    }

    /// Creates a new [`Diagnostic`] with the `Warning` severity.
    pub fn warning(file_id: FileId, title: String, code: Option<String>) -> Self {
        Self::new_with_code(file_id, Severity::Warning, title, code)
    }

    /// Creates a new [`Diagnostic`] with the `Help` severity.
    pub fn help(file_id: FileId, title: String, code: Option<String>) -> Self {
        Self::new_with_code(file_id, Severity::Help, title, code)
    }

    /// Creates a new [`Diagnostic`] with the `Note` severity.
    pub fn note(file_id: FileId, title: String, code: Option<String>) -> Self {
        Self::new_with_code(file_id, Severity::Note, title, code)
    }

    /// Creates a new [`Diagnostic`] with the `Info` severity.
    pub fn info(file_id: FileId, title: String, code: Option<String>) -> Self {
        Self::new_with_code(file_id, Severity::Info, title, code)
    }

    /// Creates a new [`Diagnostic`] that will be used in a builder-like way
    /// to modify labels, and suggestions.
    pub fn new(file_id: FileId, severity: Severity, title: String) -> Self {
        Self::new_with_code(file_id, severity, title, None)
    }

    /// Creates a new [`Diagnostic`] with an error code that will be used in a builder-like way
    /// to modify labels, and suggestions.
    pub fn new_with_code(
        file_id: FileId,
        severity: Severity,
        title: String,
        code: Option<String>,
    ) -> Self {
        Self {
            file_id,
            code,
            severity,
            title,
            children: vec![],
            suggestions: vec![],
            footer: vec![],
        }
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
    pub fn label(mut self, severity: Severity, span: impl Span, msg: String) -> Self {
        self.children.push(SubDiagnostic {
            severity,
            msg,
            span: FileSpan::new(self.file_id, span),
        });
        self
    }

    /// Attaches a primary label to this [`Diagnostic`].
    ///
    /// A primary is just a label with the [`Error`](Severity::Error) severity.
    pub fn primary(self, span: impl Span, msg: String) -> Self {
        self.label(Severity::Error, span, msg)
    }

    /// Attaches a secondary label to this [`Diagnostic`].
    ///
    /// A secondary is just a label with the [`Info`](Severity::Info) severity.
    pub fn secondary(self, span: impl Span, msg: String) -> Self {
        self.label(Severity::Info, span, msg)
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
    ///
    /// The message should not contain the `:` because it's added automatically.
    pub fn suggestion(
        mut self,
        span: impl Span,
        msg: &str,
        suggestion: String,
        applicability: Applicability,
    ) -> Self {
        self.suggestions.push(CodeSuggestion {
            substitutions: vec![Substitution {
                parts: vec![(FileSpan::new(self.file_id, span), suggestion)],
            }],
            msg: msg.into(),
            applicability,
        });
        self
    }

    /// Adds a footer to this `Diagnostic`, which will be displayed under the actual error.
    pub fn footer(mut self, severity: Severity, label: String) -> Self {
        self.footer.push(Footer { label, severity });
        self
    }

    /// Adds a footer to this `Diagnostic`, with the `Help` severity.
    pub fn footer_help(self, label: String) -> Self {
        self.footer(Severity::Help, label)
    }

    /// Adds a footer to this `Diagnostic`, with the `Note` severity.
    pub fn footer_note(self, label: String) -> Self {
        self.footer(Severity::Note, label)
    }
}

/// Everything that can be added to a diagnostic, like
/// a suggestion that will be displayed under the actual error.
#[derive(Debug, Clone)]
pub struct SubDiagnostic {
    pub(crate) severity: Severity,
    pub(crate) msg: String,
    pub(crate) span: FileSpan,
}

/// A note or help that is displayed under the diagnostic.
#[derive(Debug, Clone)]
pub struct Footer {
    pub(crate) label: String,
    pub(crate) severity: Severity,
}
