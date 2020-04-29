use crate::unicode::{is_xid_start, is_xid_continue};
use std::char;

pub trait CharExt: Copy {
  fn is_identifier_start(&self) -> bool;
  fn is_identifier_part(&self) -> bool;
  fn is_line_break(&self) -> bool;
  fn is_js_whitespace(&self) -> bool;
}

impl CharExt for char {
  fn is_identifier_start(&self) -> bool {
    (*self).is_ascii_alphabetic() ||
    *self == '$' ||
    *self == '\u{200c}' ||
    *self == '\u{200d}' ||
    is_xid_start(*self)
  }

  fn is_identifier_part(&self) -> bool {
    (*self).is_ascii_alphanumeric() ||
    *self == '$' ||
    *self == '\u{200c}' ||
    *self == '\u{200d}' ||
    is_xid_continue(*self)
  }

  fn is_line_break(&self) -> bool {
    match *self {
      '\r' | '\n' | '\u{2028}' | '\u{2029}' => true,
      _ => false
    }
  }

  fn is_js_whitespace(&self) -> bool {
    match self {
      '\u{0009}' | '\u{000b}' | '\u{000c}' | '\u{0020}' | '\u{00a0}' | '\u{feff}' => true,
      _ => if self.is_line_break() {
        false
      } else { 
        self.is_whitespace()
      }
    }
  }
}