use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
  pub fn new(start: usize, end: usize) -> Self {
    Self { start, end }
  }

  pub fn content<'a>(&self, source: &'a str) -> &'a str {
    &source[(self.start)..(self.end)]
  }

  pub fn range(&self) -> Range<usize> {
    self.start..self.end
  }

  pub fn size(&self) -> usize {
    self.end - self.start
  }
}