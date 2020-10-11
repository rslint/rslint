use crate::{file::FileSpan, Applicability};

/// A Suggestion that is provided by rslint, and
/// can be reported to the user, and can be automatically
/// applied if it has the right [`Applicability`].
#[derive(Debug, Clone)]
pub struct CodeSuggestion {
    pub(crate) substitutions: Vec<Substitution>,
    pub(crate) msg: String,
    pub(crate) applicability: Applicability,
}

/// A `Substitution` can be used to replace multiple ranges in a file,
/// with other strings.
#[derive(Debug, Clone)]
pub struct Substitution {
    pub(crate) parts: Vec<(FileSpan, String)>,
}
