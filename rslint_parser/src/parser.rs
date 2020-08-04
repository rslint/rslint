//! The physical parser structure. 
//! This may not hold your expectations of a traditional parser,
//! the parser yields events like `Start node`, `Error`, etc. 
//! These events are then applied to a `TreeSink`. 

use std::cell::Cell;

use crate::*;

/// An extremely fast, error tolerant, completely lossless JavaScript parser
pub struct Parser<'t> {
    pub file_id: usize,
    tokens: &'t mut dyn TokenSource,
    events: Vec<Event>,
    // This is for tracking if the parser is infinitely recursing. 
    // We use a cell so we dont need &mut self on `nth()`
    steps: Cell<u32>,
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &'t mut dyn TokenSource, file_id: usize) -> Parser<'t> {
        Parser {
            file_id,
            tokens,
            events: vec![],
            steps: Cell::new(0)
        }
    }
    
    /// Consume the parser and return the list of events it produced
    pub fn finish(self) -> Vec<Event> {
        self.events
    }

    /// Get the current token kind of the parser
    pub fn cur(&self) -> SyntaxKind {
        self.nth(0)
    }

    /// Get the current token of the parser
    pub fn cur_tok(&self) -> Token {
        self.nth_tok(0)
    }

    /// Look ahead at a token and get its kind, **The max lookahead is 4**.  
    /// 
    /// # Panics 
    /// This method panics if the lookahead is higher than `4`,
    /// or if the parser has run this method more than 10m times, as it is a sign of infinite recursion
    pub fn nth(&self, n: usize) -> SyntaxKind {
        assert!(n <= 4);

        let steps = self.steps.get();
        assert!(steps <= 10_000_000, "The parser seems to be recursing forever");
        self.steps.set(steps + 1);

        self.tokens.lookahead_nth(n).kind
    }

    /// Look ahead at a token, **The max lookahead is 4**.  
    /// 
    /// # Panics 
    /// This method panics if the lookahead is higher than `4`,
    /// or if the parser has run this method more than 10m times, as it is a sign of infinite recursion
    pub fn nth_tok(&self, n: usize) -> Token {
        assert!(n <= 4);

        let steps = self.steps.get();
        assert!(steps <= 10_000_000, "The parser seems to be recursing forever");
        self.steps.set(steps + 1);

        self.tokens.lookahead_nth(n)
    }

    /// Check if the parser is currently at a specific token
    pub fn at(&self, kind: SyntaxKind) -> bool {
        self.nth_at(0, kind)
    }

    /// Check if a token lookahead is something, `n` must be smaller or equal to `4`
    pub fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
        match kind {
            T![-=] => self.at_composite2(n, T![-], T![=]),
            T![=>] => self.at_composite2(n, T![=], T![>]),
            T![!=] => self.at_composite2(n, T![!], T![=]),
            T![*=] => self.at_composite2(n, T![*], T![=]),
            T![/=] => self.at_composite2(n, T![/], T![=]),
            T![&&] => self.at_composite2(n, T![&], T![&]),
            T![&=] => self.at_composite2(n, T![&], T![=]),
            T![%=] => self.at_composite2(n, T![%], T![=]),
            T![^=] => self.at_composite2(n, T![^], T![=]),
            T![+=] => self.at_composite2(n, T![+], T![=]),
            T![<<] => self.at_composite2(n, T![<], T![<]),
            T![<=] => self.at_composite2(n, T![<], T![=]),
            T![==] => self.at_composite2(n, T![=], T![=]),
            T![>=] => self.at_composite2(n, T![>], T![=]),
            T![>>] => self.at_composite2(n, T![>], T![>]),
            T![|=] => self.at_composite2(n, T![|], T![=]),
            T![||] => self.at_composite2(n, T![|], T![|]),

            T![<<=] => self.at_composite3(n, T![<], T![<], T![=]),
            T![>>=] => self.at_composite3(n, T![>], T![>], T![=]),

            T![>>>=] => self.at_composite4(n, T![>], T![>], T![>], T![=]),

            _ => self.tokens.lookahead_nth(n).kind == kind,
        }
    }

    /// Consume the next token if `kind` matches.
    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        let n_raw_tokens = match kind {
            T![-=]
            | T![=>]
            | T![!=]
            | T![*=]
            | T![/=]
            | T![&&]
            | T![&=]
            | T![%=]
            | T![^=]
            | T![+=]
            | T![<<]
            | T![<=]
            | T![==]
            | T![>=]
            | T![>>]
            | T![|=]
            | T![||] => 2,

            T![<<=] | T![>>=] => 3,

            T![>>>=] => 4,
            _ => 1,
        };
        self.do_bump(kind, n_raw_tokens);
        true
    }

    fn at_composite2(&self, n: usize, k1: SyntaxKind, k2: SyntaxKind) -> bool {
        let t1 = self.tokens.lookahead_nth(n);
        if t1.kind != k1 || !t1.is_jointed_to_next {
            return false;
        }
        let t2 = self.tokens.lookahead_nth(n + 1);
        t2.kind == k2
    }

    fn at_composite3(&self, n: usize, k1: SyntaxKind, k2: SyntaxKind, k3: SyntaxKind) -> bool {
        let t1 = self.tokens.lookahead_nth(n);
        if t1.kind != k1 || !t1.is_jointed_to_next {
            return false;
        }
        let t2 = self.tokens.lookahead_nth(n + 1);
        if t2.kind != k2 || !t2.is_jointed_to_next {
            return false;
        }
        let t3 = self.tokens.lookahead_nth(n + 2);
        t3.kind == k3
    }

    fn at_composite4(&self, n: usize, k1: SyntaxKind, k2: SyntaxKind, k3: SyntaxKind, k4: SyntaxKind) -> bool {
        let t1 = self.tokens.lookahead_nth(n);
        if t1.kind != k1 || !t1.is_jointed_to_next {
            return false;
        }
        let t2 = self.tokens.lookahead_nth(n + 1);
        if t2.kind != k2 || !t2.is_jointed_to_next {
            return false;
        }
        let t3 = self.tokens.lookahead_nth(n + 2);
        if t3.kind != k3 || !t3.is_jointed_to_next {
            return false;
        }
        let t4 = self.tokens.lookahead_nth(n + 3);
        t4.kind == k4
    }

    /// 
    pub fn err_recover(&mut self, error: impl Into<ParserError>, recovery: TokenSet) {
        match self.cur() {
            T!['{'] | T!['}'] => {
                self.error(error);
                return;
            }
            _ => (),
        }

        if self.at_ts(recovery) {
            self.error(error);
            return;
        }

        let m = self.start();
        self.error(error);
        self.bump_any();
        m.complete(self, SyntaxKind::ERROR);
    }


    /// Starts a new node in the syntax tree. All nodes and tokens
    /// consumed between the `start` and the corresponding `Marker::complete`
    /// belong to the same node.
    pub fn start(&mut self) -> Marker {
        let pos = self.events.len() as u32;
        self.push_event(Event::tombstone());
        Marker::new(pos)
    }

    /// Consume the next token if `kind` matches.
    pub fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    /// Advances the parser by one token
    pub fn bump_any(&mut self) {
        let kind = self.nth(0);
        if kind == SyntaxKind::EOF {
            return;
        }
        self.do_bump(kind, 1)
    }

    /// Make a new error builder with `error` severity
    pub fn err_builder(&self, message: &str) -> ErrorBuilder {
        ErrorBuilder::error(self.file_id, message)
    }

    /// Add an error event
    pub fn error(&mut self, err: impl Into<ParserError>) {
        self.push_event(Event::Error { err: err.into() });
    }

    /// Check if the parser's current token is contained in a token set
    pub fn at_ts(&self, kinds: TokenSet) -> bool {
        kinds.contains(self.cur())
    }

    fn do_bump(&mut self, kind: SyntaxKind, n_raw_tokens: u8) {
        for _ in 0..n_raw_tokens {
            self.tokens.bump();
        }

        self.push_event(Event::Token { kind, n_raw_tokens });
    }

    fn push_event(&mut self, event: Event) {
        self.events.push(event)
    }

    /// Get the source code of the parser's current token. 
    /// 
    /// # Panics 
    /// This method panics if the token range and source code range mismatch
    pub fn cur_src(&self) -> &str {
        self.tokens.source().get(self.nth_tok(0).range).expect("Parser source and tokens mismatch")
    }

    /// Try to eat a specific token kind, if the kind is not there then add an error to the events stack. 
    pub fn expect(&mut self, kind: SyntaxKind) -> bool {
        if self.eat(kind) {
            true
        } else {
            let err = self.err_builder(&format!("Expected token `{:?}` but instead found `{:?}`", kind, self.cur()))
                .primary(self.cur_tok().range, "Unexpected");

            self.error(err);
            false
        }
    }
}

/// A structure signifying the start of parsing of a syntax tree node
pub struct Marker {
    pos: u32,
}

impl Marker {
    pub fn new(pos: u32) -> Marker {
        Marker { pos }
    }

    /// Finishes the syntax tree node and assigns `kind` to it,
    /// and mark the create a `CompletedMarker` for possible future
    /// operation like `.precede()` to deal with forward_parent.
    pub fn complete(mut self, p: &mut Parser, kind: SyntaxKind) -> CompletedMarker {
        let idx = self.pos as usize;
        match p.events[idx] {
            Event::Start { kind: ref mut slot, .. } => {
                *slot = kind;
            }
            _ => unreachable!(),
        }
        let finish_pos = p.events.len() as u32;
        p.push_event(Event::Finish);
        CompletedMarker::new(self.pos, finish_pos, kind)
    }

    /// Abandons the syntax tree node. All its children
    /// are attached to its parent instead.
    pub fn abandon(mut self, p: &mut Parser) {
        let idx = self.pos as usize;
        if idx == p.events.len() - 1 {
            match p.events.pop() {
                Some(Event::Start { kind: SyntaxKind::TOMBSTONE, forward_parent: None }) => (),
                _ => unreachable!(),
            }
        }
    }
}

/// A structure signifying a completed marker
pub struct CompletedMarker {
    start_pos: u32,
    finish_pos: u32,
    kind: SyntaxKind,
}

impl CompletedMarker {
    pub fn new(start_pos: u32, finish_pos: u32, kind: SyntaxKind) -> Self {
        CompletedMarker { start_pos, finish_pos, kind }
    }

    /// This method allows to create a new node which starts
    /// *before* the current one. That is, parser could start
    /// node `A`, then complete it, and then after parsing the
    /// whole `A`, decide that it should have started some node
    /// `B` before starting `A`. `precede` allows to do exactly
    /// that. See also docs about `forward_parent` in `Event::Start`.
    ///
    /// Given completed events `[START, FINISH]` and its corresponding
    /// `CompletedMarker(pos: 0, _)`.
    /// Append a new `START` events as `[START, FINISH, NEWSTART]`,
    /// then mark `NEWSTART` as `START`'s parent with saving its relative
    /// distance to `NEWSTART` into forward_parent(=2 in this case);
    pub fn precede(self, p: &mut Parser) -> Marker {
        let new_pos = p.start();
        let idx = self.start_pos as usize;
        match p.events[idx] {
            Event::Start { ref mut forward_parent, .. } => {
                *forward_parent = Some(new_pos.pos - self.start_pos);
            }
            _ => unreachable!(),
        }
        new_pos
    }

    /// Undo this completion and turns into a `Marker`
    pub fn undo_completion(self, p: &mut Parser) -> Marker {
        let start_idx = self.start_pos as usize;
        let finish_idx = self.finish_pos as usize;
        match p.events[start_idx] {
            Event::Start { ref mut kind, forward_parent: None } => *kind = SyntaxKind::TOMBSTONE,
            _ => unreachable!(),
        }
        match p.events[finish_idx] {
            ref mut slot @ Event::Finish => *slot = Event::tombstone(),
            _ => unreachable!(),
        }
        Marker::new(self.start_pos)
    }

    pub fn kind(&self) -> SyntaxKind {
        self.kind
    }
}