//! A lookup table struct for mapping ascii characters to functions

use super::lexer::LexResult;
use super::lexer::Lexer;

type LexerFunc<'a> = fn(&mut Lexer<'a>, char) -> LexResult<'a>;

fn store<'a>(f: LexerFunc<'a>) -> usize {
  f as usize
}

fn load<'a>(u: usize) -> LexerFunc<'a> {
  // # Safety
  //
  // we know for a fact that each element in the lookup table we access is a valid function, because:
  // - inner is a private field that cant be changed other than replacing the function at a byte index
  // - add_*_entry takes ownership of the function
  // - this function is not public
  unsafe { std::mem::transmute(u) }
}

/// A lookup table which maps ascii characters to a function which lexes them into a token.  
/// # Safety 
/// **Using this with non ascii characters has undesired (but not unsafe) results**
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

  /// Add a character with a function to the lookup table
  pub fn add_char_entry<'a>(&mut self, c: char, f: LexerFunc<'a>) {
      self.inner[c as u8 as usize] = store(f);
  }
  
  /// Add a function at a byte index
  pub fn add_byte_entry<'a>(&mut self, c: u8, f: LexerFunc<'a>) {
      self.inner[c as usize] = store(f);
  }

  /// Obtain a function from the lookup table.  
  /// # Returns  
  /// The function at the index or a function which returns `(None, None)` 
  pub fn lookup<'a>(&self, c: char) -> LexerFunc<'a> {
    load(self.inner[c as u8 as usize])
  }
}