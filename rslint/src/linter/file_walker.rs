//! A structure for concurrently loading files from a glob pattern

use codespan_reporting::files::Files;
use codespan_reporting::files::SimpleFile;
use glob::{glob, PatternError};
use std::collections::HashMap;
use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::ops::Range;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A struct for loading and managing the files to be linted
#[derive(Debug, Clone)]
pub struct FileWalker {
    pub paths: Vec<PathBuf>,
    pub files: HashMap<usize, SimpleFile<String, String>>,
}

impl FileWalker {
    pub fn with_glob(pattern: String) -> Result<Self, PatternError> {
        Ok(Self {
            paths: Self::get_paths(pattern)?,
            files: HashMap::new(),
        })
    }

    pub fn with_paths<T: AsRef<Path>>(paths: Vec<T>) -> Self {
        Self {
            paths: paths
                .into_iter()
                .map(|x| x.as_ref().to_path_buf())
                .collect(),
            files: HashMap::new(),
        }
    }

    pub fn with_files(files: Vec<(String, String)>) -> Self {
        let mut walker = Self {
            paths: vec![],
            files: HashMap::new(),
        };

        for file in files.into_iter() {
            walker.add(file.0, file.1);
        }
        walker
    }

    /// Check if a glob pattern is valid
    pub fn validate_glob(pattern: String) -> Option<PatternError> {
        glob(&pattern).err()
    }

    /// Get the paths represented by the walker's glob pattern
    pub fn get_paths(pattern: String) -> Result<Vec<PathBuf>, PatternError> {
        let glob = glob(&pattern)?;
        let paths: Vec<PathBuf> = glob.into_iter().filter_map(|path| path.ok()).collect();
        let walked_paths: Vec<PathBuf> = paths
            .iter()
            .flat_map(|path| WalkDir::new(path).into_iter().filter_map(|x| x.ok()))
            .map(|entry| entry.into_path())
            .filter(|path| path.extension() == Some(OsStr::new("js")))
            .collect::<Vec<_>>();

        println!("{:#?}", walked_paths);
        Ok(walked_paths)
    }

    /// Load the files from the glob pattern in parallel and return a vector of warnings for skipped or erroneous files
    /// Or return an error if the glob pattern is invalid
    pub fn load(&mut self) -> Result<Vec<String>, PatternError> {
        let mut diagnostics = vec![];
        let mut handles = vec![];

        for i in self.paths.iter() {
            for entry in WalkDir::new(i) {
                // TODO: issue error if there was an error walking the dir
                if entry.is_err() {
                    diagnostics.push(format!(
                        "Skipping unreadable file at `{}`",
                        entry
                            .unwrap_err()
                            .path()
                            .unwrap_or(Path::new("{unknown}"))
                            .to_string_lossy()
                    ));
                    continue;
                }
                let walked_entry = entry.unwrap();

                let path = walked_entry.path().to_owned();

                // TODO: in the future the config will allow for other files to be configured to be linted
                // this needs to be made dynamic based on config
                if path.extension() == Some(OsStr::new("js")) {
                    // Split up io bound operations across threads
                    handles.push(std::thread::spawn(move || {
                        let path_str = path.to_string_lossy().to_string();
                        let file = File::open(path);

                        // Skip files that cannot be opened
                        if file.is_err() {
                            Err(format!("Skipping unreadable file at `{}`", path_str))
                        } else {
                            let mut buf: Vec<u8> = vec![];
                            file.unwrap().read_to_end(&mut buf).map_err(|e| {
                                format!(
                                    "Encountered an error trying to read a file at `{}`: {}",
                                    path_str,
                                    e.to_string()
                                )
                            })?;
                            let source = String::from_utf8_lossy(&buf).to_string();
                            Ok((path_str, source))
                        }
                    }));
                } else {
                    continue;
                }
            }
        }

        for handle in handles {
            let res = handle.join().expect("Failed to join a thread handle");
            if res.is_err() {
                diagnostics.push(res.unwrap_err());
                continue;
            } else {
                let file = res.unwrap();
                self.add(file.0, file.1);
            }
        }

        Ok(diagnostics)
    }

    pub fn get_existing_source(&self, key: usize) -> Option<&String> {
        self.files.get(&key).map(|file| file.source())
    }

    pub fn add(&mut self, name: String, source: String) -> usize {
        let file_id = self.files.len();
        self.files.insert(file_id, SimpleFile::new(name, source));
        file_id
    }

    pub fn get(&self, file_id: usize) -> Option<&SimpleFile<String, String>> {
        self.files.get(&file_id)
    }
}

impl<'a> Files<'a> for FileWalker {
    type FileId = usize;
    type Name = String;
    type Source = String;

    fn name(&self, file_id: usize) -> Option<Self::Name> {
        Some(self.get(file_id)?.name().clone())
    }

    fn source(&self, file_id: usize) -> Option<Self::Source> {
        Some(String::from(self.get(file_id)?.source()))
    }

    fn line_index(&self, file_id: usize, byte_index: usize) -> Option<usize> {
        self.get(file_id)?.line_index((), byte_index)
    }

    fn line_range(&self, file_id: usize, line_index: usize) -> Option<Range<usize>> {
        self.get(file_id)?.line_range((), line_index)
    }
}
