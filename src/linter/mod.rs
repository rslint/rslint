pub mod file_walker;
pub mod diagnostic;

use crate::parse::lexer::lexer::Lexer;
use std::error::Error;
use crate::linter::file_walker::FileWalker;
use crate::linter::diagnostic::LinterDiagnostic;

pub struct Linter {
  walker: FileWalker,
}

impl Linter {
  pub fn new(target: String) -> Self {
    Self {
      walker: FileWalker::new(target),
    }
  }

  pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
    self.walker.load()?;
    for file in &self.walker.files {
      let lexer = Lexer::new(file.1.source(), file.0); 
    }
    Ok(())
  }

  pub fn throw_diagnostic(&self, diagnostic: &LinterDiagnostic) {
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
    use codespan_reporting::diagnostic::Severity;
    use codespan_reporting::term::DisplayStyle;
    use codespan_reporting::term;

    let writer = if diagnostic.diagnostic.severity == Severity::Error {
      StandardStream::stderr(ColorChoice::Always)
    } else {
      StandardStream::stdout(ColorChoice::Always)
    };

    let mut config = term::Config::default();
    if diagnostic.simple {
      config.display_style = DisplayStyle::Short;
    }

    term::emit(&mut writer.lock(), &config, &self.walker, &diagnostic.diagnostic);
  }

  fn lexer_debug(&self, lexer: Lexer) {
    println!("\n Lexer debug for: {} ------------------- \n", lexer.file_id);
    use ansi_term::Style;
    use ansi_term::Color::*;

    let mut cur_ln = 1;
    for token in lexer {
      if token.is_err() {
        self.throw_diagnostic(&token.err().unwrap());
        continue;
      }
      let tok = token.unwrap();
      if tok.line > cur_ln {
        println!();
        cur_ln += 1;
      }
      if tok.token_type == crate::parse::lexer::token::TokenType::Whitespace {
        print!("{}", Style::new().on(White).paint(" ".repeat(tok.lexeme.size())));
        continue;
      }
      print!(" {}", Style::new().on(Cyan).fg(Black).paint(format!("{:?}", tok.token_type)).to_string());
    }
  }
}