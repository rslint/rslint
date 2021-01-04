//! Representation of a file for the linter

use rslint_parser::{parse_with_syntax, FileKind, ParserError, SyntaxNode};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

// 0 is reserved for "no file id" (virtual files)
static FILE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// A structure representing either a concrete (in-disk) or virtual (temporary/non-disk) js, ts, or mjs file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct File {
    pub source: String,
    /// The name of the file.
    pub name: String,
    /// The path in disk if this is a concrete file.
    pub path: Option<PathBuf>,
    /// The rslint_errors id assigned to this file used to refer back to it.
    pub id: usize,
    /// The kind of file this is.
    pub kind: FileKind,
    /// The cached line start locations in this file.
    pub line_starts: Vec<usize>,
}

impl File {
    pub fn new_concrete(source: String, path: PathBuf) -> Self {
        let id = FILE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let ext = path
            .extension()
            .map_or("".into(), |ext| ext.to_string_lossy());

        let kind = match ext.as_ref() {
            "mjs" => FileKind::Module,
            "js" => FileKind::Script,
            "ts" => FileKind::TypeScript,
            _ => panic!("tried to make a file with extensions outside of `mjs`, `js`, or `ts`"),
        };
        let line_starts = Self::line_starts(&source).collect();

        Self {
            source,
            name: path
                .file_name()
                .map_or(String::new(), |osstr| osstr.to_string_lossy().to_string()),
            path: Some(path),
            id,
            kind,
            line_starts,
        }
    }

    pub fn from_string(source: impl ToString, kind: FileKind, name: impl ToString) -> Self {
        let id = FILE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let source = source.to_string();
        let line_starts = Self::line_starts(&source).collect();

        Self {
            source,
            name: name.to_string(),
            path: None,
            id,
            kind,
            line_starts,
        }
    }

    pub fn update_src(&mut self, new: String) {
        self.line_starts = Self::line_starts(&new).collect();
        self.source = new;
    }

    // TODO: Needs to work correctly for \u2028, \u2029, and \r line endings
    pub fn line_starts(source: &str) -> impl Iterator<Item = usize> + '_ {
        std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
    }

    pub fn line_start(&self, line_index: usize) -> Option<usize> {
        use std::cmp::Ordering;

        match line_index.cmp(&self.line_starts.len()) {
            Ordering::Less => self.line_starts.get(line_index).cloned(),
            Ordering::Equal => Some(self.source.len()),
            Ordering::Greater => None,
        }
    }

    pub fn line_index(&self, byte_index: usize) -> usize {
        match self.line_starts.binary_search(&byte_index) {
            Ok(line) => line,
            Err(next_line) => next_line - 1,
        }
    }

    pub fn line_col_to_index(&self, line: usize, column: usize) -> Option<usize> {
        let start = self.line_start(line)?;
        Some(start + column)
    }

    pub fn line_range(&self, line_index: usize) -> Option<Range<usize>> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Some(line_start..next_line_start)
    }

    /// Parse this file into a syntax node, ignoring any errors produced. This
    pub fn parse(&self) -> SyntaxNode {
        parse_with_syntax(&self.source, self.id, self.kind.into()).syntax()
    }

    pub fn parse_with_errors(&self) -> (Vec<ParserError>, SyntaxNode) {
        let parse = parse_with_syntax(&self.source, self.id, self.kind.into());
        (parse.errors().to_vec(), parse.syntax())
    }
}
