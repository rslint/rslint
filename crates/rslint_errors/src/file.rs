pub use rslint_core::Span;
use std::ops::Range;

/// An id that points into a file database.
pub type FileId = usize;

/// A range that is indexed in a specific file.
#[derive(Debug, Clone)]
pub struct FileSpan {
    pub file: FileId,
    pub span: Range<usize>,
}

impl FileSpan {
    pub fn new(file: FileId, span: impl Span) -> Self {
        Self {
            file,
            span: span.as_range(),
        }
    }
}

/// Interface for interacting with source files
/// that are identified by a unique identifier.
pub trait Files {
    /// Returns the name of the file identified by the id.
    fn name(&self, id: FileId) -> Option<&str>;

    /// Returns the source of the file identified by the id.
    fn source(&self, id: FileId) -> &str;

    /// The index of the line at the byte index.
    ///
    /// ## Implementation
    /// This can be implemented by caching the results of [`line_starts`]
    /// and then use [`binary_search`](https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search)
    /// to compute the line index.
    ///
    /// ```ignore
    /// match self.line_starts.binary_search(byte_index) {
    ///     Ok(line) => line,
    ///     Err(next_line) => next_line - 1,
    /// }
    /// ```
    fn line_index(&self, file_id: FileId, byte_index: usize) -> usize;
}

/// Computes the byte indicies of every line start.
pub fn line_starts(source: &str) -> impl '_ + Iterator<Item = usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}
