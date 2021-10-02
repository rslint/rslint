//! The structure responsible for managing IO and the files implementation for codespan.

use crate::lint_warn;
use ignore::{WalkBuilder, WalkState};
use rslint_core::File;
use rslint_errors::file::{FileId, Files};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::ops::Range;
use std::path::PathBuf;

/// A list of the extension of files linted
const LINTED_FILES: [&str; 3] = ["js", "mjs", "ts"];

/// The filename of the ignore file for RSLint
const RSLINT_IGNORE_FILE: &str = ".rslintignore";

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
    pub fn from_glob_parallel(paths: Vec<PathBuf>, num_threads: usize) -> Self {
        let mut base = Self::default();
        base.load_files_parallel(paths.into_iter(), num_threads, false, None, false);
        base
    }

    pub fn load_files_parallel(
        &mut self,
        paths: impl Iterator<Item = PathBuf>,
        num_threads: usize,
        no_ignore: bool,
        ignore_file: Option<PathBuf>,
        use_gitignore: bool,
    ) {
        let build_walker = |path: &PathBuf| {
            let mut builder = WalkBuilder::new(path);
            builder.standard_filters(false);

            if !no_ignore {
                match ignore_file.as_ref() {
                    Some(file) => match builder.add_ignore(file) {
                        Some(err) => {
                            crate::lint_warn!("invalid gitignore file: {}", err);
                        }
                        None => {}
                    },
                    None => {
                        builder.add_custom_ignore_filename(RSLINT_IGNORE_FILE);
                    }
                }

                builder
                    .parents(true)
                    .hidden(true)
                    .git_global(use_gitignore)
                    .git_ignore(use_gitignore)
                    .git_exclude(use_gitignore);
            }

            builder.threads(num_threads).build_parallel()
        };

        for path in paths {
            let (tx, rx) = std::sync::mpsc::channel();
            build_walker(&path).run(|| {
                let tx = tx.clone();
                Box::new(move |entry| {
                    let path = match entry {
                        Ok(entry) => match entry.file_type() {
                            Some(typ) if !typ.is_dir() => entry.into_path(),
                            _ => return WalkState::Continue,
                        },
                        Err(err) => {
                            crate::lint_warn!("invalid gitignore file: {}", err);
                            return WalkState::Continue;
                        }
                    };

                    // check if this is a js/ts file
                    let ext = path.extension().unwrap_or_default().to_string_lossy();
                    if !LINTED_FILES.contains(&ext.as_ref()) {
                        return WalkState::Continue;
                    }

                    // read the content of the file
                    let content = match std::fs::read_to_string(&path) {
                        Ok(c) => c,
                        Err(err) => {
                            crate::lint_err!("failed to read file {}: {}", path.display(), err);
                            return WalkState::Continue;
                        }
                    };

                    tx.send(File::new_concrete(content, path))
                        .expect("failed to send files to receiver thread");
                    WalkState::Continue
                })
            });

            drop(tx);

            self.files
                .extend(rx.into_iter().map(|file| (file.id, file)));
        }
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
