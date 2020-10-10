pub mod file;

mod diagnostic;
mod suggestion;

pub use diagnostic::{Diagnostic, SubDiagnostic};
pub use suggestion::CodeSuggestion;

/// Indicicates how a tool should manage this suggestion.
#[derive(Clone, Copy, Debug)]
pub enum Applicability {
    /// The suggestion is definitely what the user intended.
    /// This suggestion should be automatically applied.
    Always,
    /// The suggestion may be what the user intended, but it is uncertain.
    /// The suggestion should result in valid Rust code if it is applied.
    MaybeIncorrect,
    /// The suggestion contains placeholders like `(...)` or `{ /* fields */ }`.
    /// The suggestion cannot be applied automatically because it will not result in valid Rust code.
    /// The user will need to fill in the placeholders.
    HasPlaceholders,
    /// The applicability of the suggestion is unknown.
    Unspecified,
}

/// Types of severity.
#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Error,
    Warning,
    Help,
    Note,
    Info,
}
