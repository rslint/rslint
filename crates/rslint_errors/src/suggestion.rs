use crate::{file::FileId, Applicability};
use std::ops::Range;

/// A Suggestion that is provided by rslint, and
/// can be reported to the user, and can be automatically
/// applied if it has the right [`Applicability`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct CodeSuggestion {
    /// If the `FileId` is `None`, it's in the same file as
    /// his parent.
    pub substitution: (Option<FileId>, Range<usize>, String),
    pub applicability: Applicability,
    pub msg: String,
}
