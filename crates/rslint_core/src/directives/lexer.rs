use std::iter::Peekable;

use rslint_errors::Diagnostic;
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
    file_id: usize,
    tokens: Peekable<RawLexer<'source>>,
}

impl<'source> Lexer<'source> {
    pub fn new(src: &'source str, file_id: usize, offset: usize) -> Self {
        Self {
            cur: 0,
            offset,
            src,
            file_id,
            tokens: RawLexer::from_str(src, file_id).peekable(),
        }
    }

    fn abs_range(&self, len: usize) -> TextRange {
        let offset = TextSize::from((self.offset + 1) as u32);
        let start = self.cur as u32;
        let end = (self.cur + len) as u32;
        TextRange::new(start.into(), end.into()) + offset
    }

    fn err(&self, msg: &str) -> Diagnostic {
        Diagnostic::error(self.file_id, "directives", msg)
    }

    pub fn abs_cur(&self) -> usize {
        self.offset + self.cur
    }

    pub fn source_of(&self, tok: &Token) -> &'source str {
        let range = tok.range - TextSize::from((self.offset + 1) as u32);
        &self.src[range]
    }

    pub fn expect(&mut self, kind: SyntaxKind) -> Result<Token, Diagnostic> {
        fn format_kind(kind: SyntaxKind) -> String {
            kind.to_string()
                .map(|x| x.to_string())
                .unwrap_or_else(|| format!("{:?}", kind))
        }

        match self.next() {
            Some(tok) if tok.kind == kind => Ok(tok),
            Some(tok) if tok.kind == SyntaxKind::EOF => {
                let d = self
                    .err(&format!(
                        "expected `{}`, but comment ends here",
                        format_kind(kind)
                    ))
                    .primary(tok.range, "");
                Err(d)
            }
            Some(tok) => {
                let d = self
                    .err(&format!(
                        "expected `{}`, found `{}`",
                        format_kind(kind),
                        format_kind(tok.kind)
                    ))
                    .primary(tok.range, "");
                Err(d)
            }
            _ => panic!("`expect` should not be called multiple times after EOF was reached"),
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        let (tok, _) = self.tokens.next()?;
        let range = self.abs_range(tok.len);
        self.cur += tok.len;
        if tok.kind == SyntaxKind::WHITESPACE {
            return self.next();
        }

        Some(Token {
            kind: tok.kind,
            range,
        })
    }

    pub fn peek(&mut self) -> Option<Token> {
        let (tok, _) = self.tokens.peek()?.clone();
        let range = self.abs_range(tok.len);
        if tok.kind == SyntaxKind::WHITESPACE {
            self.tokens.next();
            return self.peek();
        }

        Some(Token {
            kind: tok.kind,
            range,
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
