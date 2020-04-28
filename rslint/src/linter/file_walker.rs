use codespan_reporting::files::SimpleFile;
use codespan_reporting::files::Files;
use std::ops::Range;
use std::collections::HashMap;
use std::fs::File;
use std::error::Error;
use std::io::Read;
use std::ffi::OsStr;
use glob::glob;
use walkdir::WalkDir;


/// A struct for loading and managing the files to be linted
pub struct FileWalker {
  pub glob: String,
  pub files: HashMap<String, SimpleFile<String, String>>
}

impl FileWalker {
  pub fn new(glob: String) -> Self {
    Self { 
      glob,
      files: HashMap::new(),
    }
  }

  pub fn with_files(files: Vec<(String, String)>) -> Self {
    let mut res = Self {
      glob: String::new(),
      files: HashMap::new(),
    };
    for file in files {
      res.add(file.0, file.1);
    }
    res
  }

  pub fn load(&mut self) -> Result<(), Box<dyn Error>> {
    let paths = glob(&self.glob)?;
    for i in paths {
      // Skip any unreadable files/directories
      if i.is_err() { continue; }

      for entry in WalkDir::new(i.unwrap()) {
        // TODO: issue error if there was an error walking the dir
        if entry.is_err() { continue }
        let walked_entry = entry.unwrap();

        let path = walked_entry.path();

        // TODO: in the future the config will allow for other files to be configured to be linted
        // this needs to be made dynamic based on config
        if path.extension() == Some(OsStr::new("js")) {
          let path_str = path.to_string_lossy().to_string();
          let file = File::open(path);
  
          // Skip files that cannot be opened
          if file.is_err() { continue }
          // TODO: issue warning for unreadable files, this will currently panic 
          let source = self.get_file_source(&mut file.unwrap()).unwrap();
          self.add(path_str, source);
        }
      }
    }
    Ok(())
  }

  fn get_file_source(&self, file: &mut File) -> Result<String, Box<dyn Error>> {
    let mut buf: Vec<u8> = vec![];
    file.read_to_end(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
  }

  pub fn get_existing_source(&self, key: &str) -> Option<&String> {
    self.files.get(key).map(|file| file.source())
  }

  pub fn add(&mut self, name: String, source: String) -> usize {
    let file_id = self.files.len();
    self.files.insert(name.clone(), SimpleFile::new(name, source));
    file_id
  }

  pub fn get(&self, file_id: &str) -> Option<&SimpleFile<String, String>> {
    self.files.get(file_id)
  }
}

impl<'a> Files<'a> for FileWalker {
  type FileId = &'a str;
  type Name = String;
  type Source = String;

  fn name(&self, file_id: &'a str) -> Option<Self::Name> {
    Some(self.get(file_id)?.name().clone())
  }

  fn source(&self, file_id: &'a str) -> Option<Self::Source> {
      Some(String::from(self.get(file_id)?.source()))
  }

  fn line_index(&self, file_id: &'a str, byte_index: usize) -> Option<usize> {
      self.get(file_id)?.line_index((), byte_index)
  }

  fn line_range(&self, file_id: &'a str, line_index: usize) -> Option<Range<usize>> {
      self.get(file_id)?.line_range((), line_index)
  }
}