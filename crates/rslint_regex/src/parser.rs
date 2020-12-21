use crate::{
    ir::{self, AssertionKind, Node},
    Diagnostic, Result, Span,
};

#[derive(Debug, Clone)]
pub struct State {
    /// Number of regex groups that were parsed.
    group_count: u32,
}

impl Default for State {
    fn default() -> Self {
        Self { group_count: 0 }
    }
}

/// The actual parser that is responsible for parsing regex.
pub struct Parser<'pat> {
    pattern: &'pat str,
    offset: usize,
    cur: usize,
    file_id: usize,
    state: State,
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
            state: State::default(),
        }
    }

    fn error(&mut self, title: impl Into<String>) -> Diagnostic {
        Diagnostic::new(self.file_id, rslint_errors::Severity::Error, title)
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
        Span::new(self.offset, start, self.cur)
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
    pub fn parse(mut self) -> Result<ir::Regex> {
        assert_eq!(self.cur, 0, "`pattern` must only be called once.");

        self.next_if(|c| c == '/')
            .expect("Invalid RegEx pattern must be catched by the Lexer/Parser");

        let rest_pattern = &self.pattern[1..];
        let pattern_end = rest_pattern
            .rfind('/')
            .expect("Invalid RegEx pattern must be catched by the Lexer/Parser");

        let (_, flags) = rest_pattern.split_at(pattern_end);
        // strip of the last `/`
        self.pattern = &self.pattern[..pattern_end + 1];

        // `+ 2` because of the first `/` and because `pattern_end` includes the second `/`
        let flags_offset = self.offset + pattern_end + 2;
        let flags = ir::Flags::parse(&flags[1..], flags_offset, self.file_id)?;

        let node = self.disjunction()?;
        assert!(
            self.peek().is_none(),
            "Regex parser must be at the end of the pattern after parsing"
        );
        Ok(ir::Regex { node, flags })
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
            _ => Ok(Node::Alternation(self.span(start), terms)),
        }
    }

    /// A term is either a `atom`, `assertion` or an `atom` followed by a `quantifier`.
    fn term(&mut self) -> Result<Node> {
        if let Some(node) = self.assertion()? {
            Ok(node)
        } else {
            self.atom()
        }
    }

    /// Tries to parse an assertion, but will rewind to the start if
    /// it failed to find a assertion.
    fn assertion(&mut self) -> Result<Option<Node>> {
        let start = self.cur;

        if let Ok(c) = self.eat('^').or_else(|_| self.eat('$')) {
            return Ok(Some(Node::Assertion(
                self.span(start),
                if c == '^' {
                    AssertionKind::StartOfLine
                } else {
                    AssertionKind::EndOfLine
                },
            )));
        }

        if self.eat('\\').is_ok() {
            if let Some(c) = self.next_if(|c| c == 'b' || c == 'B') {
                return Ok(Some(Node::Assertion(
                    self.span(start),
                    if c == 'b' {
                        AssertionKind::WordBoundary
                    } else {
                        AssertionKind::NonWordBoundary
                    },
                )));
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

                return Ok(Some(Node::Assertion(
                    self.span(start),
                    kind(Box::new(node)),
                )));
            }
        }

        // the next token is no assertion, so rewind to the start.
        self.rewind(start);
        Ok(None)
    }

    fn atom(&mut self) -> Result<Node> {
        let start = self.cur;
        let c = match self.next() {
            Some(c) => c,
            None => todo!(),
        };

        let node = match c {
            '.' => Node::Dot(self.span(start)),
            '\\' => self.atom_escape(start)?,
            c => todo!(),
        };
        Ok(node)
    }

    /// Parses anything that comes after a `\`.
    fn atom_escape(&mut self, start: usize) -> Result<Node> {
        let c = match self.next() {
            Some(c) => c,
            None => {
                let err = self
                    .error("unexpected end of esacping sequence")
                    .primary(self.span(start), "something must follow this `\\`");
                return Err(err);
            }
        };

        let span = self.span(start);
        let node = match c {
            't' => Node::Literal(span, '\t'),
            'n' => Node::Literal(span, '\n'),
            'v' => Node::Literal(span, '\x0B'),
            'f' => Node::Literal(span, '\x0C'),
            'r' => Node::Literal(span, '\r'),

            'c' => {
                let c = match self.next() {
                    Some(c) => c,
                    None => {
                        let err = self
                            .error("expected control character")
                            .primary(self.cur..self.cur + 1, "expected a control character...")
                            .secondary(span, "...to follow this escape");
                        return Err(err);
                    }
                };

                if !c.is_ascii_alphabetic() {
                    let err = self
                        .error(format!("invalid control character: `{}`", c))
                        .primary(self.cur - 1..self.cur, "");

                    return Err(err);
                } else {
                    Node::Literal(
                        self.span(start),
                        std::char::from_u32((c as u32) % 32).unwrap(),
                    )
                }
            }

            'd' | 'D' => Node::PerlClass(span, ir::ClassPerlKind::Digit, c == 'D'),
            'w' | 'W' => Node::PerlClass(span, ir::ClassPerlKind::Word, c == 'W'),
            's' | 'S' => Node::PerlClass(span, ir::ClassPerlKind::Space, c == 'S'),
            'p' | 'P' => unimplemented!("Unicode support will be added soon"),

            // a back reference: `/(foo)\1/`
            '1'..='9' => {
                let num = {
                    let mut n = c.to_digit(10).unwrap();
                    while let Some(c) = self.next_if(|c| c.is_digit(10)) {
                        n = 10 * n + c.to_digit(10).unwrap();
                    }
                    n
                };

                // invalid group number
                if num > self.state.group_count {
                    // TODO: Phrase this message in a nicer way
                    let err = self.error("invalid backref to non-existing group").primary(
                        self.span(start),
                        format!("there is no group with number {}", num),
                    );
                    return Err(err);
                }

                Node::BackReference(span, num)
            }
            _ => todo!(),
        };

        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ir::{self, Flags, Node},
        Parser, Span,
    };

    fn assert(raw: &str, node: Node) {
        let file = rslint_errors::file::SimpleFile::new("test".to_string(), raw.to_string());

        let parser = Parser::new_with_offset(raw, 0, 0);
        let regex = parser.parse();

        if let Err(d) = regex {
            rslint_errors::Emitter::new(&file)
                .emit_stderr(&d, true)
                .unwrap();
        } else {
            assert_eq!(regex.unwrap().node, node);
        }
        //assert_eq!(regex.unwrap().node, node);
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
                span: Span::new(0, 1, 3),
            }),
        )
    }

    #[test]
    fn control_char() {
        assert(
            "/\\ca/",
            Node::Assertion(ir::Assertion {
                kind: ir::AssertionKind::WordBoundary,
                span: Span::new(0, 1, 3),
            }),
        )
    }
}
