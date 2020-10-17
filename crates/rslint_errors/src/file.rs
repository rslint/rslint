use rslint_rowan::{Language, SyntaxElement, SyntaxNode, SyntaxToken, TextRange};
use std::ops::Range;

/// A value which can be used as the range inside of a diagnostic.
///
/// This is essentially a hack to allow us to use SyntaxElement, SyntaxNode, etc directly
pub trait Span {
    fn as_range(&self) -> Range<usize>;
}

impl<T: Span> Span for &T {
    fn as_range(&self) -> Range<usize> {
        (*self).as_range()
    }
}

impl<T: Span> Span for &mut T {
    fn as_range(&self) -> Range<usize> {
        (**self).as_range()
    }
}

impl<T: Clone> Span for Range<T>
where
    T: Into<usize>,
{
    fn as_range(&self) -> Range<usize> {
        self.start.clone().into()..self.end.clone().into()
    }
}

impl<T: Language> Span for SyntaxNode<T> {
    fn as_range(&self) -> Range<usize> {
        self.text_range().into()
    }
}

impl<T: Language> Span for SyntaxToken<T> {
    fn as_range(&self) -> Range<usize> {
        self.text_range().into()
    }
}

impl<T: Language> Span for SyntaxElement<T> {
    fn as_range(&self) -> Range<usize> {
        match self {
            SyntaxElement::Node(n) => n.text_range(),
            SyntaxElement::Token(t) => t.text_range(),
        }
        .into()
    }
}

impl Span for TextRange {
    fn as_range(&self) -> Range<usize> {
        self.clone().into()
    }
}

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
