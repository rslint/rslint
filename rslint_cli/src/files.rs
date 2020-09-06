//! The structure responsible for managing IO and the files implementation for codespan.

use codespan_reporting::files::Files;
use glob::Paths;
use std::borrow::Cow;
use std::fs::read_to_string;
use std::ops::Range;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::Builder;
use walkdir::WalkDir;
use hashbrown::HashMap;

// 0 is reserved for "no file id" (virtual files)
static FILE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

/// A list of ignored-by-default directory/file names
const IGNORED: [&str; 1] = ["node_modules"];
/// A list of the extension of files linted
const LINTED_FILES: [&str; 2] = ["js", "mjs"];

/// The structure for managing IO to and from the core runner.
/// The walker uses multithreaded IO, spawning a thread for every file being loaded.
// TODO: use IO_Uring for linux
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileWalker {
    pub files: HashMap<usize, JsFile>,
}

impl<'a> Files<'a> for FileWalker {
    type Name = Cow<'a, str>;
    type Source = Cow<'a, str>;
    type FileId = usize;

    fn name(&'a self, id: Self::FileId) -> Option<Cow<'a, str>> {
        let entry = self.files.get(&id)?;
        Some(
            entry
                .path
                .as_ref()
                .map(|p| p.to_string_lossy())
                .unwrap_or((&entry.name).into()),
        )
    }

    fn source(&'a self, id: Self::FileId) -> Option<Cow<'a, str>> {
        let entry = self.files.get(&id)?;
        Some((&entry.source).into())
    }

    fn line_index(&self, id: Self::FileId, byte_index: usize) -> Option<usize> {
        self.files.get(&id)?.line_start(byte_index)
    }

    fn line_range(&self, id: Self::FileId, line_index: usize) -> Option<Range<usize>> {
        let line_start = self.line_start(id, line_index)?;
        let next_line_start = self.line_start(id, line_index + 1)?;

        Some(line_start..next_line_start)
    }
}

impl FileWalker {
    /// Make a new file walker from a compiled glob pattern. This also
    /// skips any unreadable files/dirs
    pub fn from_glob(paths: Paths) -> Self {
        let mut threads = Vec::new();
        for entry in paths.filter_map(Result::ok) {
            if IGNORED.contains(&entry.file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_default().as_str()) {
                continue;
            }

            for file in WalkDir::new(entry).into_iter().filter_map(Result::ok) {
                if !LINTED_FILES.contains(&file.path().extension().map(|osstr| osstr.to_string_lossy().to_string()).unwrap_or_default().as_str()) {
                    continue;
                }
                if IGNORED.contains(&file.file_name().to_string_lossy().to_string().as_str()) {
                    continue;
                }
                // Give each io thread a name so we can potentially debug any io failures easily
                let thread = Builder::new()
                    .name(format!("io-{}", file.file_name().to_string_lossy()))
                    .spawn(move || {
                        (
                            read_to_string(file.path()).expect("Failed to read file"),
                            file.path().to_owned(),
                        )
                    })
                    .expect("Failed to spawn IO thread");
                threads.push(thread);
            }
        }
        let files = threads
            .into_iter()
            .map(|handle| handle.join())
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        let jsfiles = files
            .into_iter()
            .map(|(src, path)| JsFile::new_concrete(src, path.into()))
            .map(|file| (file.id, file))
            .collect();

        Self { files: jsfiles }
    }

    pub fn line_start(&self, id: usize, line_index: usize) -> Option<usize> {
        self.files.get(&id)?.line_starts.get(line_index).copied()
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
    pub line_starts: Vec<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum JsFileKind {
    Script,
    Module,
}

impl JsFile {
    pub fn new_concrete(source: String, path: PathBuf) -> Self {
        let id = FILE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        let kind = if path.extension().map_or("".into(), |ext| ext.to_string_lossy()) == "mjs" {
            JsFileKind::Module
        } else {
            JsFileKind::Script
        };
        let line_starts = Self::line_starts(&source).collect();

        Self {
            source,
            name: path.file_name().map_or(String::new(), |osstr| osstr.to_string_lossy().to_string()),
            path: Some(path),
            id,
            kind,
            line_starts
        }
    }

    fn line_starts<'a>(source: &'a str) -> impl Iterator<Item = usize> + 'a {
        std::iter::once(0).chain(source.match_indices('\n').map(|(i, _)| i + 1))
    }

    pub fn line_start(&self, byte_index: usize) -> Option<usize> {
        match self.line_starts.binary_search(&byte_index) {
            Ok(line) => Some(line),
            Err(next_line) => Some(next_line - 1),
        }
    }
}
