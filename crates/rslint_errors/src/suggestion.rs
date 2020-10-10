use crate::{file::FileSpan, Applicability};
use smol_str::SmolStr;

/// A Suggestion that is provided by rslint, and
/// can be reported to the user, and can be automatically
/// applied if it has the right [`Applicability`].
#[derive(Debug, Clone)]
pub struct CodeSuggestion {
    substitutions: Vec<Substitution>,
    msg: SmolStr,
    applicability: Applicability,
}

/// A `Substitution` can be used to replace multiple ranges in a file,
/// with other strings.
#[derive(Debug, Clone)]
pub struct Substitution {
    parts: Vec<(FileSpan, String)>,
}
