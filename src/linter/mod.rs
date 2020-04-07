use glob::{glob, PatternError, Paths};
use crate::parse::lexer::lexer::Lexer;
use std::fs::File;
use std::error::Error;
use std::io::Read;
use codespan_reporting::files::SimpleFiles;
use codespan_reporting::diagnostic::Diagnostic;

pub struct Linter {
  target: Paths,
  files: SimpleFiles<String, String>,
  file_ids: Vec<usize>
}

impl Linter {
  pub fn new(target: String) -> Result<Self, PatternError> {
    let pattern = glob(&target)?;
    Ok(Self {
      target: pattern,
      files: SimpleFiles::new(),
      file_ids: vec![]
    })
  }

  pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
    self.load_glob_files()?;
    for file in &self.file_ids {
      for token in Lexer::new(self.files.get(*file).unwrap().source(), *file) {
        println!("{}", token.unwrap());
      }
    }
    Ok(())
  }

  fn load_glob_files(&mut self) -> Result<(), Box<dyn Error>> {
    for i in &mut self.target {
      if i.is_err() {
        continue;
      }
      let path = i.unwrap();
      let mut file = File::open(&path)?;
      let mut src = String::new();
      file.read_to_string(&mut src)?;
      self.file_ids.push(self.files.add(path.to_string_lossy().as_ref().to_owned(), src));
    }
    Ok(())
  }

  pub fn throw_diagnostic(&self, diagnostic: &Diagnostic<usize>) {
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
    use codespan_reporting::term;

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    term::emit(&mut writer.lock(), &config, &self.files, diagnostic);
  }
}