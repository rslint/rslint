#[cfg(feature = "rowan")]
use rslint_rowan::{Language, SyntaxElement, SyntaxNode, SyntaxToken, TextRange};
use std::{collections::HashMap, ops::Range};

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

#[cfg(feature = "rowan")]
impl<T: Language> Span for SyntaxNode<T> {
    fn as_range(&self) -> Range<usize> {
        self.text_range().into()
    }
}

#[cfg(feature = "rowan")]
impl<T: Language> Span for SyntaxToken<T> {
    fn as_range(&self) -> Range<usize> {
        self.text_range().into()
    }
}

#[cfg(feature = "rowan")]
impl<T: Language> Span for SyntaxElement<T> {
    fn as_range(&self) -> Range<usize> {
        match self {
            SyntaxElement::Node(n) => n.text_range(),
            SyntaxElement::Token(t) => t.text_range(),
        }
        .into()
    }
}

#[cfg(feature = "rowan")]
impl Span for TextRange {
    fn as_range(&self) -> Range<usize> {
        self.clone().into()
    }
}

/// An id that points into a file database.
pub type FileId = usize;

/// A range that is indexed in a specific file.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FileSpan {
    pub file: FileId,
    pub range: Range<usize>,
}

impl FileSpan {
    pub fn new(file: FileId, span: impl Span) -> Self {
        Self {
            file,
            range: span.as_range(),
        }
    }
}

/// Interface for interacting with source files
/// that are identified by a unique identifier.
pub trait Files {
    /// Returns the name of the file identified by the id.
    fn name(&self, id: FileId) -> Option<&str>;

    /// Returns the source of the file identified by the id.
    fn source(&self, id: FileId) -> Option<&str>;

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
    fn line_index(&self, file_id: FileId, byte_index: usize) -> Option<usize>;
}

/// A file database that contains only one file.
#[derive(Clone, Debug)]
pub struct SimpleFile {
    name: String,
    source: String,
    line_starts: Vec<usize>,
}

impl SimpleFile {
    /// Create a new file with the name and source.
    pub fn new(name: String, source: String) -> Self {
        Self {
            line_starts: line_starts(&source).collect(),
            name,
            source,
        }
    }
}

impl Files for SimpleFile {
    fn name(&self, _id: FileId) -> Option<&str> {
        Some(&self.name)
    }

    fn source(&self, _id: FileId) -> Option<&str> {
        Some(&self.source)
    }

    fn line_index(&self, _file_id: FileId, byte_index: usize) -> Option<usize> {
        Some(
            self.line_starts
                .binary_search(&byte_index)
                .unwrap_or_else(|next_line| next_line - 1),
        )
    }
}

/// A file database that stores multiple files.
#[derive(Clone, Debug, Default)]
pub struct SimpleFiles {
    files: HashMap<FileId, SimpleFile>,
    id: usize,
}

impl SimpleFiles {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file to this database and returns the id for the new file.
    pub fn add(&mut self, name: String, source: String) -> FileId {
        let id = self.id;
        self.id += 1;
        self.files.insert(id, SimpleFile::new(name, source));
        id
    }

    pub fn get(&self, id: FileId) -> Option<&SimpleFile> {
        self.files.get(&id)
    }
}

impl Files for SimpleFiles {
    fn name(&self, id: FileId) -> Option<&str> {
        self.files.get(&id)?.name(id)
    }

    fn source(&self, id: FileId) -> Option<&str> {
        self.files.get(&id)?.source(id)
    }

    fn line_index(&self, id: FileId, byte_index: usize) -> Option<usize> {
        self.files.get(&id)?.line_index(id, byte_index)
    }
}

/// Computes the byte indicies of every line start.
pub fn line_starts(source: &str) -> impl '_ + Iterator<Item = usize> {
    std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
}
