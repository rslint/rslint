use crate::{file::FileSpan, Applicability};

/// A Suggestion that is provided by rslint, and
/// can be reported to the user, and can be automatically
/// applied if it has the right [`Applicability`].
#[derive(Debug, Clone)]
pub struct CodeSuggestion {
    pub(crate) substitution: (FileSpan, String),
    pub(crate) msg: String,
    pub(crate) applicability: Applicability,
}
