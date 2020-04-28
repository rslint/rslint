use super::lexer::LexResult;
use super::lexer::Lexer;

type LexerFunc<'a> = fn(&mut Lexer<'a>, char) -> LexResult<'a>;

fn store<'a>(f: LexerFunc<'a>) -> usize {
  f as usize
}

fn load<'a>(u: usize) -> LexerFunc<'a> {
  unsafe { std::mem::transmute(u) }
}

pub struct LexerLookupTable {
  inner: [usize; 256],
}

impl LexerLookupTable {
  pub fn new() -> Self {
    Self {
      inner: [store(|_: &mut Lexer, _: char| {
        (None, None)
      }); 256]
    }
  }

  pub fn add_char_entry<'a>(&mut self, c: char, f: LexerFunc<'a>) {
      self.inner[c as u8 as usize] = store(f);
  }
  
  pub fn add_byte_entry<'a>(&mut self, c: u8, f: LexerFunc<'a>) {
      self.inner[c as usize] = store(f);
  }

  pub fn lookup<'a>(&self, c: char) -> LexerFunc<'a> {
    load(self.inner[c as u8 as usize])
  }
}