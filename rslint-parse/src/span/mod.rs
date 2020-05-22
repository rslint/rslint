use std::ops::{Add, Range};

/// A Struct representing a span of code inside of source code.
#[derive(Clone, Debug, PartialEq, Eq)]
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

<<<<<<< Updated upstream
  pub fn range(&self) -> Range<usize> {
    self.start..self.end
  }
=======
    #[inline]
    pub fn range(&self) -> Range<usize> {
        self.start..self.end
}

impl Add for Span {
    type Output = Span;

    fn add(self, other: Self) -> Self {
        Self::new(self.start, other.end)
    }
}

impl Into<Range<usize>> for Span {
    fn into(self) -> Range<usize> {
        self.range()
    }
}
>>>>>>> Stashed changes

#[cfg(test)]
mod tests {
    use crate::span::Span;

    #[test]
    fn new_span() {
        assert_eq!(Span::new(0, 10).range(), 0..10);
    }

    #[test]
    fn content() {
        assert_eq!(Span::new(0, 2).content("oh hi mark"), "oh");
    }

    #[test]
    fn size() {
        assert_eq!(Span::new(0, 10).size(), 10);
    }

    #[test]
    fn add_spans() {
        assert_eq!(Span::new(0, 5) + Span::new(0, 10), Span::new(0, 10));
    }
}
