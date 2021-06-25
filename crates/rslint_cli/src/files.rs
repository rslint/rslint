//! The structure responsible for managing IO and the files implementation for codespan.

use crate::lint_warn;
use rslint_core::File;
use rslint_errors::file::{FileId, Files};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::ops::Range;
use std::path::PathBuf;
use walkdir::WalkDir;

/// A list of ignored-by-default directory/file names
const IGNORED: [&str; 1] = ["node_modules"];
/// A list of the extension of files linted
const LINTED_FILES: [&str; 3] = ["js", "mjs", "ts"];

/// The structure for managing IO to and from the core runner.
/// The walker uses multithreaded IO, spawning a thread for every file being loaded.
// TODO: use IO_Uring for linux
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileWalker {
    pub files: HashMap<usize, File>,
}

impl Files for FileWalker {
    fn name(&self, id: FileId) -> Option<&str> {
        let entry = self.files.get(&id)?;
        let name = entry
            .path
            .as_ref()
            .and_then(|path| path.to_str())
            .unwrap_or_else(|| entry.name.as_str());
        Some(name)
    }

    fn source(&self, id: FileId) -> Option<&str> {
        let entry = self.files.get(&id)?;
        Some(&entry.source)
    }

    fn line_index(&self, id: FileId, byte_index: usize) -> Option<usize> {
        Some(self.files.get(&id)?.line_index(byte_index))
    }

    fn line_range(&self, file_id: FileId, line_index: usize) -> Option<Range<usize>> {
        self.files.get(&file_id)?.line_range(line_index)
    }
}

impl FileWalker {
    pub fn empty() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Make a new file walker from a compiled glob pattern. This also
    /// skips any unreadable files/dirs
    pub fn from_glob(paths: Vec<PathBuf>) -> Self {
        let mut base = Self::default();
        base.load_files(paths.into_iter());
        base
    }

    pub fn load_files(&mut self, paths: impl Iterator<Item = PathBuf>) {
        let jsfiles: HashMap<usize, File> = paths
            .filter(|p| {
                !IGNORED.contains(&p.file_name().unwrap_or_default().to_string_lossy().as_ref())
            })
            .flat_map(|path| {
                WalkDir::new(path)
                    .into_iter()
                    .filter_entry(|p| !IGNORED.contains(&p.file_name().to_string_lossy().as_ref()))
                    .filter_map(Result::ok)
            })
            .filter(|p| {
                LINTED_FILES.contains(
                    &p.path()
                        .extension()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .as_ref(),
                ) && !p.file_type().is_dir()
            })
            .filter_map(|entry| {
                let path = entry.path();
                let content = match read_to_string(path) {
                    Ok(v) => v,
                    Err(err) => {
                        crate::lint_err!("failed to read file {}: {}", path.display(), err);
                        return None;
                    }
                };
                Some((content, path.to_owned()))
            })
            .map(|(src, path)| File::new_concrete(src, path))
            .map(|file| (file.id, file))
            .collect();
        self.files.extend(jsfiles);
    }

    pub fn line_start(&self, id: usize, line_index: usize) -> Option<usize> {
        self.files.get(&id)?.line_start(line_index)
    }

    /// try loading a file's source code and updating the correspoding file in the walker
    pub fn maybe_update_file_src(&mut self, path: PathBuf) {
        if let Some(file) = self.files.values_mut().find(|f| {
            f.path
                .clone()
                .map_or(false, |x| x.file_name() == path.file_name())
        }) {
            let src = if let Ok(src) = read_to_string(&path) {
                src
            } else {
                return lint_warn!(
                    "failed to reload the source code at `{}`",
                    path.to_string_lossy()
                );
            };
            file.source = src;
            file.line_starts = File::line_starts(&file.source).collect();
        }
    }
}
