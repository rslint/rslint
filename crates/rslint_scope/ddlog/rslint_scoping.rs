use rslint_core::rule_prelude::TextRange;
use std::ops::Range;

impl rslint_core::rule_prelude::Span for Span {
    fn as_range(&self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}

impl From<TextRange> for Span {
    fn from(range: TextRange) -> Self {
        Self {
            start: range.start().into(),
            end: range.end().into(),
        }
    }
}
