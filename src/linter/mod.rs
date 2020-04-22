pub mod file_walker;
pub mod diagnostic;

use crate::parse::lexer::lexer::Lexer;
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
    println!("\n Lexer debug for: {} ------------------- \n", lexer.file_id);
    use ansi_term::Style;
    use ansi_term::Color::*;

    let source = lexer.source;
    let mut cur_ln = 1;
    for token in lexer {
      if token.1.is_some() {
        token.1.unwrap().throw(&self.walker);
      }
      if token.0.is_none() { break }
      let tok = token.0.unwrap();
      if tok.line > cur_ln {
        println!();
        cur_ln += 1;
      }
      if tok.token_type == crate::parse::lexer::token::TokenType::Whitespace {
        print!("{}", Style::new().on(White).paint(" ".repeat(tok.lexeme.size())));
        continue;
      }
      print!(" {}({})", Style::new().on(Cyan).fg(Black).paint(format!("{:?}", tok.token_type)).to_string(), tok.lexeme.content(source));
    }
  }
}