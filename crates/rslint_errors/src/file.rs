/// An id that points into a file database.
pub type FileId = usize;

/// Interface for interacting with source files
/// that are identified by a unique identifier.
pub trait Files {
    /// The name for identifying a file.
    type Name: AsRef<str>;
    /// The source code of a file.
    type Source: AsRef<str>;

    /// Returns the name of the file identified by the id.
    fn name(&self, id: FileId) -> Self::Name;

    /// Returns the source of the file identified by the id.
    fn source(&self, id: FileId) -> Self::Source;

    /// The index of the line at the byte index.
    ///
    /// ## Implementation
    /// This can be implemented by caching the results of [`line_starts`]
    /// and then use [`binary_search`](slice::binary_search) to compute the line index.
    ///
    /// ```ignore
    /// match self.line_starts.binary_search(byte_index) {
    ///     Ok(line) => line,
    ///     Err(next_line) => next_line - 1,
    /// }
    /// ```
    fn line_index(&self, byte_index: usize) -> usize;
}

/// Computes the byte indicies of every line start.
pub fn line_starts<'src>(source: &'src str) -> impl 'src + Iterator<Item = usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}
