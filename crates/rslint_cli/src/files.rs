use crate::lint_err;
use async_walkdir::{Filtering, WalkDir};
use rslint_core::{
    errors::file::{line_starts, FileId, Files},
    rule_prelude::SyntaxNode,
};
use smol::{fs::read_to_string, prelude::*};
use std::{
    collections::HashMap, ops::Range, path::Path, path::PathBuf, sync::atomic::AtomicUsize,
    sync::atomic::Ordering,
};

// 0 is reserved for "no file id" (virtual files)
static FILE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// A list of ignored-by-default directory/file names
const IGNORED: [&str; 1] = ["node_modules"];
/// A list of the extension of files linted
const LINTED_FILES: [&str; 2] = ["js", "mjs"];

/// The structure for walking directories, reading files
/// and loading files.
// TODO: io_uring on linux
pub struct FileWalker {
    pub files: HashMap<usize, JsFile>,
}

impl FileWalker {
    pub fn empty() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    pub fn new(files: HashMap<usize, JsFile>) -> Self {
        Self { files }
    }

    pub fn into_files(self) -> impl Iterator<Item = JsFile> {
        self.files.into_iter().map(|(_, v)| v)
    }

    pub fn files_stream<'glob>(globs: &'glob [String]) -> impl Stream<Item = JsFile> + 'glob {
        let file_globs = smol::stream::iter(globs)
            .filter(|p| AsRef::<Path>::as_ref(p).is_file())
            .map(|glob| PathBuf::from(glob));

        smol::stream::iter(globs)
            .map(|p| Self::walker_for_glob(p))
            .flatten()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .chain(file_globs)
            .then(|path| async move {
                // TODO: Use `io` module of `smol` so we make use of polling api.
                match smol::spawn(read_to_string(path.clone())).await {
                    Ok(content) => {
                        let file = JsFile::new_concrete(content, path);
                        Some(file)
                    }
                    Err(err) => {
                        crate::lint_err!("failed to read file {}: {}", path.display(), err);
                        None
                    }
                }
            })
            .filter_map(|x| x)
    }

    pub async fn walk_files<F, Fut>(globs: &[String], action: F)
    where
        F: Fn(JsFile) -> Fut,
        Fut: Future<Output = ()>,
    {
        Self::files_stream(globs)
            .then(action)
            .for_each(|_| {})
            .await;
    }

    pub async fn from_globs(globs: Vec<String>) -> Self {
        let files: HashMap<usize, JsFile> = Self::files_stream(&globs)
            .map(|file| (file.id, file))
            .collect()
            .await;
        Self { files }
    }

    fn walker_for_glob(glob: &str) -> WalkDir {
        WalkDir::new(glob).filter(|entry| async move {
            if IGNORED.contains(&entry.file_name().to_string_lossy().as_ref()) {
                Filtering::IgnoreDir
            } else if LINTED_FILES.contains(
                &entry
                    .path()
                    .extension()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref(),
            ) {
                Filtering::Continue
            } else {
                Filtering::Ignore
            }
        })
    }
}

impl Files for FileWalker {
    fn name(&self, id: FileId) -> Option<&str> {
        let entry = self.files.get(&id)?;
        Some(entry.path_or_name())
    }

    fn source(&self, id: FileId) -> Option<&str> {
        let entry = self.files.get(&id)?;
        Some(&entry.source)
    }

    fn line_index(&self, file_id: FileId, byte_index: usize) -> Option<usize> {
        let entry = self.files.get(&file_id)?;
        Some(entry.line_index(byte_index))
    }

    fn line_range(&self, id: FileId, line_index: usize) -> Option<Range<usize>> {
        let entry = self.files.get(&id)?;
        entry.line_range(line_index)
    }
}

/// A structure representing either a concrete (in-disk) or virtual (temporary/non-disk) js or mjs file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct JsFile {
    pub source: String,
    /// The name of the file.
    pub name: String,
    /// The path in disk if this is a concrete file.
    pub path: Option<PathBuf>,
    /// The codespan id assigned to this file used to refer back to it.
    pub id: usize,
    /// Whether this is a js or mjs file (script vs module).
    pub kind: JsFileKind,
    /// The cached line start locations in this file.
    pub line_starts: Box<[usize]>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsFileKind {
    Script,
    Module,
}

impl JsFile {
    pub fn new_concrete(source: String, path: PathBuf) -> Self {
        let id = FILE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        let kind = if path
            .extension()
            .map_or("".into(), |ext| ext.to_string_lossy())
            == "mjs"
        {
            JsFileKind::Module
        } else {
            JsFileKind::Script
        };
        let line_starts = line_starts(&source).collect();

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

    pub fn path_or_name(&self) -> &str {
        if let Some(ref path) = self.path {
            path.to_str().unwrap_or_else(|| self.name.as_ref())
        } else {
            self.name.as_ref()
        }
    }

    pub fn update_src(&mut self, new: String) {
        self.line_starts = line_starts(&new).collect();
        self.source = new;
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

    fn line_range(&self, line_index: usize) -> Option<Range<usize>> {
        let line_start = self.line_start(line_index)?;
        let next_line_start = self.line_start(line_index + 1)?;

        Some(line_start..next_line_start)
    }

    /// Parse this file into a syntax node, ignoring any errors produced. This
    /// will use `parse_module` for `.mjs` and `parse_text` for `.js`
    pub fn parse(&self) -> SyntaxNode {
        if self.kind == JsFileKind::Module {
            rslint_core::parser::parse_module(&self.source, self.id).syntax()
        } else {
            rslint_core::parser::parse_text(&self.source, self.id).syntax()
        }
    }
}

impl Files for JsFile {
    fn name(&self, id: FileId) -> Option<&str> {
        if id == self.id {
            Some(self.path_or_name())
        } else {
            None
        }
    }

    fn source(&self, id: FileId) -> Option<&str> {
        if id == self.id {
            Some(self.source.as_ref())
        } else {
            None
        }
    }

    fn line_index(&self, file_id: FileId, byte_index: usize) -> Option<usize> {
        if file_id == self.id {
            Some(self.line_index(byte_index))
        } else {
            None
        }
    }

    fn line_range(&self, id: FileId, line_index: usize) -> Option<Range<usize>> {
        if id == self.id {
            self.line_range(line_index)
        } else {
            None
        }
    }
}

/// Collects all paths that match the given list of globs.
pub fn collect_globs(globs: &[&str]) -> Vec<PathBuf> {
    globs
        .iter()
        .map(|pat| glob::glob(pat))
        .flat_map(|path| {
            if let Err(err) = path {
                lint_err!("invalid glob pattern: {}", err);
                None
            } else {
                path.ok()
            }
        })
        .flat_map(|paths| paths.filter_map(Result::ok))
        .collect()
}
