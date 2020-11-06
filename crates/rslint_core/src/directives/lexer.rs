use std::iter::Peekable;

use rslint_lexer::{Lexer as RawLexer, SyntaxKind};
use rslint_parser::{TextRange, TextSize};

/// Any token that is parsed by the `Lexer`, but with
/// a `range` instead of a `len`.
pub struct Token {
    pub kind: SyntaxKind,
    pub range: TextRange,
}

pub struct Lexer<'source> {
    offset: usize,
    cur: usize,
    src: &'source str,
    tokens: Peekable<RawLexer<'source>>,
}

impl<'source> Lexer<'source> {
    pub fn new(src: &'source str, file_id: usize, offset: usize) -> Self {
        Self {
            cur: 0,
            offset,
            src,
            tokens: RawLexer::from_str(src, file_id).peekable(),
        }
    }

    fn abs_range(&self, len: usize) -> TextRange {
        let offset = TextSize::from(self.offset as u32);
        let start = (self.cur - len) as u32;
        let end = self.cur as u32;
        TextRange::new(start.into(), end.into()) + offset
    }

    pub fn source_of(&self, tok: &Token) -> &'source str {
        let range = tok.range - TextSize::from(self.offset as u32);
        &self.src[range]
    }

    pub fn next(&mut self) -> Option<Token> {
        let (tok, _) = self.tokens.next()?;
        self.cur += tok.len;
        if tok.kind == SyntaxKind::WHITESPACE {
            return self.next();
        }

        Some(Token {
            kind: tok.kind,
            range: self.abs_range(tok.len),
        })
    }

    pub fn peek(&mut self) -> Option<Token> {
        let (tok, _) = self.tokens.peek()?.clone();
        self.cur += tok.len;
        if tok.kind == SyntaxKind::WHITESPACE {
            self.tokens.next();
            return self.peek();
        }

        Some(Token {
            kind: tok.kind,
            range: self.abs_range(tok.len),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let src = "abc // rslint-ignore foo";
        let offset = src.rfind('/').unwrap() + 1;
        let mut l = Lexer::new(&src[offset..], 0, offset);

        let t = l.next().unwrap();
        assert_eq!(l.source_of(&t), "rslint");
        let t = l.next().unwrap();
        assert_eq!(l.source_of(&t), "-");
        let t = l.next().unwrap();
        assert_eq!(l.source_of(&t), "ignore");
        let t = l.next().unwrap();
        assert_eq!(l.source_of(&t), "foo");
    }
}
