#![deny(rust_2018_idioms)]

pub mod annotate_snippets;
pub mod file;
#[cfg(feature = "lsp")]
pub mod lsp;

mod diagnostic;
mod emit;
mod suggestion;

pub use diagnostic::{Diagnostic, SubDiagnostic};
pub use emit::Emitter;
pub use file::Span;
pub use suggestion::*;

pub(crate) use annotate_snippets::*;

use annotate_snippets::snippet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum DiagnosticTag {
    Unnecessary,
    Deprecated,
    Both,
}

impl DiagnosticTag {
    pub fn is_unecessary(&self) -> bool {
        matches!(self, DiagnosticTag::Unnecessary | DiagnosticTag::Both)
    }

    pub fn is_deprecated(&self) -> bool {
        matches!(self, DiagnosticTag::Deprecated | DiagnosticTag::Both)
    }
}

/// Indicicates how a tool should manage this suggestion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Applicability {
    /// The suggestion is definitely what the user intended.
    /// This suggestion should be automatically applied.
    Always,
    /// The suggestion may be what the user intended, but it is uncertain.
    /// The suggestion should result in valid Rust code if it is applied.
    MaybeIncorrect,
    /// The suggestion contains placeholders like `(...)` or `{ /* fields */ }`.
    /// The suggestion cannot be applied automatically because it will not result in valid JavaScript/TypeScript code.
    /// The user will need to fill in the placeholders.
    HasPlaceholders,
    /// The applicability of the suggestion is unknown.
    Unspecified,
}

/// Types of severity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Severity {
    Error,
    Warning,
    Help,
    Note,
    Info,
}

impl Into<snippet::AnnotationType> for Severity {
    fn into(self) -> snippet::AnnotationType {
        use snippet::AnnotationType::*;

        match self {
            Severity::Error => Error,
            Severity::Warning => Warning,
            Severity::Help => Help,
            Severity::Note => Note,
            Severity::Info => Info,
        }
    }
}
