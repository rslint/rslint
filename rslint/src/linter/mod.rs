pub mod file_walker;

use rslint_parse::{lexer::lexer::Lexer, diagnostic::ParserDiagnostic};
use std::error::Error;
use crate::linter::file_walker::FileWalker;
use rayon::prelude::*;
use rayon::iter::ParallelIterator;

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
    self.walker.files.par_iter().for_each(|file| {
      let lexer = Lexer::new(file.1.source(), file.0);
      self.lexer_debug(lexer);
    });
    Ok(())
  }

  fn lexer_debug(&self, lexer: Lexer) {
    use rslint_parse::lexer::token::TokenType::Whitespace;

    println!("\n Lexer debug for: {} ------------------- \n", lexer.file_id);
    use ansi_term::Style;
    use ansi_term::Color::*;

    let source = lexer.source;
    let mut cur_ln = 1;
    for token in lexer {
      if token.1.is_some() {
        self.throw_diagnostic(token.1.unwrap());
      }
      if token.0.is_none() { break }
      let tok = token.0.unwrap();
      if tok.line > cur_ln {
        println!();
        cur_ln += 1;
      }
      if tok.token_type == Whitespace {
        print!("{}", Style::new().on(White).paint(" ".repeat(tok.lexeme.size())));
        continue;
      }
      print!(" {}({})", Style::new().on(Cyan).fg(Black).paint(format!("{:?}", tok.token_type)).to_string(), tok.lexeme.content(source));
    }
  }

  fn throw_diagnostic(&self, diagnostic: ParserDiagnostic) {
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
    use codespan_reporting::term::DisplayStyle;
    use codespan_reporting::term;
    use codespan_reporting::diagnostic::Severity;

    let writer = if diagnostic.diagnostic.severity == Severity::Error {
      StandardStream::stderr(ColorChoice::Always)
    } else {
      StandardStream::stdout(ColorChoice::Always)
    };

    let mut config = term::Config::default();
    if diagnostic.simple {
      config.display_style = DisplayStyle::Short;
    }

    term::emit(&mut writer.lock(), &config, &self.walker, &diagnostic.diagnostic)
      .expect("Failed to throw diagnostic");
  }
}