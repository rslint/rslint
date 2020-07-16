//! A layered cache implementation to store results from the previous linting result.  
//!
//! By using cache by default we can immensely speed up linting after the first run.  
//! Caching uses several layers to make sure results are consistent with and without cache.  
//! These circumstances will cause a total run (total linter run and cache regen):
//!  - The version of RSLint has changed
//!  - The RSLint binary has been changed (`modified` timestamp has changed)
//!  - There is no cache file
//!  - The cache file is unreadable or the data inside of it is illogical
//!  - The cache has been "poisoned", something wrote to it after it was generated and its data cannot be trusted
//!  
//! These circumstances will cause a partial run (some results from cache are used, linter is maybe run for some things):
//!  - Files are added (results from files in cache are used, and linter is run on new files)
//!  - Files are removed (results from cache for the files which still exist are used)
//!  - Files are modified (results from non modified files in cache are used, linter is run for modified files)
//!  - Files are the same (purely cache results are used)
//!  - The rules being run have been changed (results of the rules which have already been run are used, linter is run for the ones that have not been run previously)
//!    This is safe because no rule is allowed to rely on the result of other rules
//!

use codespan_reporting::diagnostic::Diagnostic;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use bincode::{serialize, deserialize};
use std::collections::HashMap;
use std::env::{current_dir, current_exe};
use std::fs::{metadata, write};
use std::path::{PathBuf, Path};
use std::sync::Mutex;
use std::time::SystemTime;

const VERSION: &'static str = env!(
    "CARGO_PKG_VERSION",
    "Caching relies on the cargo version but the env was not found"
);

/// Info about a linted file which needs to be stored in cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo<'a> {
    #[serde(borrow)]
    pub diagnostics: Vec<Diagnostic<&'a str>>,
    pub timestamp: SystemTime,
    pub rules: Vec<String>,
    pub name: String,
}

/// Cache stored in a file in between linting runs to avoid needlessly running the linter on files which havent changed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cache<'a> {
    #[serde(borrow)]
    pub files: HashMap<String, FileInfo<'a>>,
    /// The last write time for the binary, this allows us to
    pub binary_time: SystemTime,
    pub binary_version: String,
    /// When the cache was written to disk, this is to check if the cache has been "poisoned" and is not reliable to use
    pub write_date: SystemTime,
}

impl<'a> Cache<'a> {
    /// Check if a file has been modified using a cached timestamp
    pub fn has_been_modified(timestamp: SystemTime, file: &Path) -> bool {
        Self::get_file_timestamp(file) == Some(timestamp)
    }

    pub fn get_file_timestamp(file: &Path) -> Option<SystemTime> {
        if let Ok(metadata) = metadata(file) {
            metadata.modified().ok()
        } else {
            None
        }
    }

    /// Get a path to the cache file in the current working directory if there is one
    pub fn get_cwd_cache_file<'b>() -> Option<PathBuf> {
        if let Ok(cwd) = current_dir() {
            for entry in cwd.read_dir().expect("Tried finding cache file but somehow cwd is not a dir") {
                if entry.is_err() {
                    continue;
                } else {
                    let unwrapped = entry.unwrap();
                    if unwrapped.file_name() == ".rslintcache" {
                        return Some(unwrapped.path());
                    }
                }
            }
            return None;
        } else {
            return None;
        }
    }

    /// Deserialize cache from bytes
    pub fn from_bytes<'b>(bytes: &'a [u8]) -> Option<Cache<'a>> {
        deserialize(bytes).ok()
    }

    /// Whether cache should be thrown out because the binary or the version has changed
    pub fn should_discard(&self) -> bool {
        let was_binary_modified = if let Ok(path) = current_exe() {
            if let Ok(metadata) = metadata(path) {
                metadata.modified().ok() == Some(self.binary_time)
            } else {
                false
            }
        } else {
            false
        };

        self.binary_version != VERSION || was_binary_modified
    }

    /// Separate new files into files which dont have to be linted, and files which must be partially or fully linted
    /// This is a simple check to see if each file has not been modified and exists in cache
    pub fn file_intersect(&self, new_files: Vec<PathBuf>) -> (Vec<&FileInfo>, Vec<PathBuf>) {
        let cached = Mutex::new(Vec::with_capacity(new_files.len()));
        let uncached = Mutex::new(Vec::with_capacity(new_files.len()));
        let cached_files = &self.files;

        new_files.par_iter().for_each(|path| {
            let path_str = path.as_os_str().to_string_lossy().into_owned();
            if cached_files.contains_key(&path_str) {
                if let Ok(metadata) = metadata(path) {
                    if let Ok(timestamp) = metadata.modified() {
                        let file = cached_files.get(&path_str).unwrap();
                        if timestamp == file.timestamp {
                            cached.lock().unwrap().push(file);
                            return;
                        }
                    }
                }
            }
            uncached.lock().unwrap().push(path.to_owned());
        });

        (cached.into_inner().unwrap(), uncached.into_inner().unwrap())
    }

    /// Write the cache to a `.rslintcache` file in the current working directory
    #[allow(unused_must_use)]
    pub fn persist(&self) -> Result<(), ()>{
        if let Ok(mut cwd) = current_dir() {
            cwd.push(".rslintcache");
            write(
                cwd,
                serialize(self).unwrap(),
            ).map_err(|_| ())?;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn generate(file_info: HashMap<String, FileInfo<'a>>) -> Cache<'a> {
        Self {
            files: file_info,
            binary_time: current_exe().expect("Failed to load current exe for cache").metadata().expect("Failed to get metadata for current exe").modified().expect("Failed to get modified timestamp for current exe"),
            binary_version: VERSION.to_string(),
            write_date: SystemTime::now(),
        }
    }
}
