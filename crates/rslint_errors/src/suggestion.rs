use crate::{file::FileSpan, Applicability};
use ouroboros::self_referencing;

/// A Suggestion that is provided by rslint, and
/// can be reported to the user, and can be automatically
/// applied if it has the right [`Applicability`].
#[self_referencing]
#[derive(Debug, Clone)]
pub struct CodeSuggestion {
    /// The whole string to be displayed for a label.
    ///
    /// `<msg>: <suggestion>`
    pub(crate) label: String,
    pub(crate) substitution: (FileSpan, String),
    pub(crate) applicability: Applicability,
    #[borrows(label)]
    pub(crate) msg: &'this str,
}
