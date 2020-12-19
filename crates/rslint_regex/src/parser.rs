use crate::{
    ir::{self, AssertionKind, Node},
    Diagnostic, Result, Span,
};

/// The actual parser that is responsible for parsing regex.
pub struct Parser<'pat> {
    pattern: &'pat str,
    offset: usize,
    cur: usize,
    file_id: usize,
}

impl<'pat> Parser<'pat> {
    /// Creates a new `Parser` from a given pattern.
    ///
    /// The given offset is used to convert the relative position in the pattern
    /// into an absolute position inside a file. The `pattern` must be the full string
    /// including the flags (e.g. `/a|b|c/im`).
    pub fn new_with_offset(pattern: &'pat str, file_id: usize, offset: usize) -> Self {
        Self {
            pattern,
            offset,
            file_id,
            cur: 0,
        }
    }

    fn error(&mut self, title: impl Into<String>) -> Diagnostic {
        Diagnostic::error(self.file_id, "regex", title)
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.cur += c.len_utf8();
        Some(c)
    }

    fn peek(&mut self) -> Option<char> {
        let slice = &self.pattern.get(self.cur..)?;
        let c = slice.chars().next()?;
        Some(c)
    }

    fn peek_many(&mut self, count: usize) -> Option<&'pat str> {
        self.pattern.get(self.cur..self.cur + count)
    }

    fn take(&mut self, count: usize) -> Option<&'pat str> {
        let slice = self.peek_many(count)?;
        self.cur += slice.len();
        Some(slice)
    }

    fn eat(&mut self, c: char) -> Result<char> {
        let start = self.cur;
        self.next_if(|x| x == c).ok_or_else(|| {
            self.error(format!("expected `{}`", c))
                .primary(self.span(start), "")
        })
    }

    fn try_eat_many(&mut self, eat: &str) -> bool {
        if self
            .peek_many(eat.len())
            .map(|actual| actual == eat)
            .unwrap_or(false)
        {
            self.take(eat.len());
            true
        } else {
            false
        }
    }

    fn next_if<F: FnOnce(char) -> bool>(&mut self, pred: F) -> Option<char> {
        if pred(self.peek()?) {
            Some(self.next().unwrap())
        } else {
            None
        }
    }

    fn span(&self, start: usize) -> Span {
        Span::new(self.offset, start, self.cur - 1)
    }

    fn rewind(&mut self, start: usize) {
        self.cur = start;
    }
}

impl Parser<'_> {
    /// The main entrypoint for parsing a RegEx pattern.
    ///
    /// This will parse the actual pattern and the flags.
    ///
    /// ## Panics
    ///
    /// If the pattern doesn't start with `/` or doesn't have two `/` at the start and end.
    pub fn parse(mut self) -> Result<ir::RegEx> {
        assert_eq!(self.cur, 0, "`pattern` must only be called once.");

        self.next_if(|c| c == '/')
            .expect("Invalid RegEx pattern must be catched by the Lexer/Parser");

        let rest_pattern = &self.pattern[1..];
        let pattern_end = rest_pattern
            .rfind('/')
            .expect("Invalid RegEx pattern must be catched by the Lexer/Parser");

        let (_, flags) = rest_pattern.split_at(pattern_end);

        // `+ 2` because of the first `/` and because `pattern_end` includes the second `/`
        let flags_offset = self.offset + pattern_end + 2;
        let flags = ir::Flags::parse(&flags[1..], flags_offset, self.file_id)?;

        Ok(ir::RegEx {
            node: self.disjunction()?,
            flags,
        })
    }

    /// A Disjunction is a list of nodes separated by `|`
    ///
    /// ```ignore
    /// /a|b|c/
    /// ```
    fn disjunction(&mut self) -> Result<Node> {
        let start = self.cur;

        let first = self.term()?;
        let mut terms = vec![first];
        while self.next_if(|c| c == '|').is_some() {
            terms.push(self.term()?);
        }

        match terms.len() {
            0 => Ok(Node::Empty),
            1 => Ok(terms.remove(0)),
            _ => Ok(Node::Alternation(ir::Alternation {
                span: self.span(start),
                alternatives: terms,
            })),
        }
    }

    /// A term is either a `atom`, `assertion` or an `atom` followed by a `quantifier`.
    fn term(&mut self) -> Result<Node> {
        if let Some(node) = self.assertion()? {
            Ok(node)
        } else {
            todo!()
        }
    }

    /// Tries to parse an assertion, but will rewind to the start if
    /// it failed to find a assertion.
    fn assertion(&mut self) -> Result<Option<Node>> {
        let start = self.cur;

        if let Ok(c) = self.eat('^').or_else(|_| self.eat('$')) {
            return Ok(Some(Node::Assertion(ir::Assertion {
                span: self.span(start),
                kind: if c == '^' {
                    AssertionKind::StartOfLine
                } else {
                    AssertionKind::EndOfLine
                },
            })));
        }

        if self.eat('\\').is_ok() {
            if let Some(c) = self.next_if(|c| c == 'b' || c == 'B') {
                return Ok(Some(Node::Assertion(ir::Assertion {
                    span: self.span(start),
                    kind: if c == 'b' {
                        AssertionKind::WordBoundary
                    } else {
                        AssertionKind::NonWordBoundary
                    },
                })));
            }
            self.rewind(start);
        }

        let l_paren = self.cur;
        if self.try_eat_many("(?") {
            let is_lookbehind = self.eat('<').is_ok();
            let ty = self.eat('=').or_else(|_| self.eat('!'));

            if let Ok(c) = ty {
                let node = self.disjunction()?;
                self.eat(')').map_err(|err| {
                    err.primary(self.cur..self.cur + 1, "expected a parentheses...")
                        .secondary(l_paren..l_paren + 1, "...to close this one")
                })?;
                // FIXME: state
                let kind = match (is_lookbehind, c) {
                    (false, '=') => AssertionKind::Lookahead,
                    (false, '!') => AssertionKind::NegativeLookahead,
                    (true, '=') => AssertionKind::Lookbehind,
                    (true, '!') => AssertionKind::NegativeLookbehind,
                    _ => unreachable!(),
                };

                return Ok(Some(Node::Assertion(ir::Assertion {
                    span: self.span(start),
                    kind: kind(Box::new(node)),
                })));
            }
        }

        // the next token is no assertion, so rewind to the start.
        self.rewind(start);
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ir::{self, Flags, Node},
        Parser, Span,
    };

    fn assert(raw: &str, node: Node) {
        //let file = rslint_errors::file::SimpleFile::new("test".to_string(), raw.to_string());

        let parser = Parser::new_with_offset(raw, 0, 0);
        let regex = parser.parse();

        //if let Err(d) = regex {
        //rslint_errors::Emitter::new(&file)
        //.emit_stderr(&d, true)
        //.unwrap();
        //} else {
        //assert_eq!(regex.unwrap().node, node);
        //}
        assert_eq!(regex.unwrap().node, node);
    }

    #[test]
    fn parse_flags() {
        let raw = "/a|b/imu";

        let parser = Parser::new_with_offset(raw, 0, 0);
        let regex = parser.parse().expect("regex failed to parse");

        assert_eq!(regex.flags, Flags::I | Flags::M | Flags::U);
    }

    #[test]
    fn assertion() {
        assert(
            "/\\b/",
            Node::Assertion(ir::Assertion {
                kind: ir::AssertionKind::WordBoundary,
                span: Span::new(0, 1, 2),
            }),
        )
    }
}
