//! An extremely fast, lookup table based, ECMAScript lexer which yields SyntaxKind tokens used by the rslint_parse parser.  
//! The tokens yielded by the lexer are "raw", punctuators such as `>>=` will yield `>` + `>` + `=`.  
//! For the purposes of error recovery, tokens may have an error attached to them, which is reflected in the Iterator Item.  
//! The lexer will also yield `COMMENT` and `WHITESPACE` tokens.
//!
//! The lexer operates on raw bytes to take full advantage of lookup table optimizations, these bytes **must** be valid utf8,
//! therefore making a lexer from a `&[u8]` is unsafe since you must make sure the bytes are valid utf8.
//! Do not use this to learn how to lex JavaScript, this is just needlessly fast and demonic because i can't control myself :)

#[macro_use]
mod token;
mod labels;
mod tests;

pub use token::Token;

use codespan_reporting::diagnostic::{Diagnostic, Label};
// There is a way of making these functions 7x faster, but it involves 100kb+ static bitmaps
// Although i am reluctant of using that currently as it does not seem needed, but this will have to be considered
use unicode_xid::UnicodeXID;

pub use rslint_syntax::{SyntaxKind, T};
pub type LexerReturn = (Token, Option<Diagnostic<usize>>);

// Simple macro for unwinding a loop
macro_rules! unwind_loop {
    ($($iter:tt)*) => {
        $($iter)*
        $($iter)*
        $($iter)*
        $($iter)*
        $($iter)*

        loop {
            $($iter)*
            $($iter)*
            $($iter)*
            $($iter)*
            $($iter)*
        }
    };
}

// The first utf8 byte of every valid unicode whitespace char, used for short circuiting whitespace checks
const UNICODE_WHITESPACE_STARTS: [u8; 5] = [
    // NBSP
    0xC2, // BOM
    0xEF, // Ogham space mark
    0xE1, // En quad .. Hair space, narrow no break space, mathematical space
    0xE2, // Ideographic space
    0xE3,
];

// Unicode spaces, designated by the `Zs` unicode property
const UNICODE_SPACES: [char; 16] = [
    '\u{00A0}', '\u{1680}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}', '\u{2004}', '\u{2005}',
    '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}', '\u{200A}', '\u{202F}', '\u{205F}', '\u{3000}',
];

/// An extremely fast, lookup table based, lossless ECMAScript lexer
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Lexer<'src> {
    bytes: &'src [u8],
    cur: usize,
    pub file_id: usize,
}

impl<'src> Lexer<'src> {
    /// Make a new lexer from raw bytes, this is unsafe since you **must** make sure the bytes are valid utf8.
    /// Failure to do so is undefined behavior.
    pub unsafe fn from_bytes(bytes: &'src [u8], file_id: usize) -> Self {
        Self {
            bytes,
            cur: 0,
            file_id,
        }
    }

    /// Make a new lexer from a str, this is safe because strs are valid utf8
    pub fn from_str(string: &'src str, file_id: usize) -> Self {
        Self {
            bytes: string.as_bytes(),
            cur: 0,
            file_id,
        }
    }

    // Bump the lexer and return the token given in
    fn eat(&mut self, tok: LexerReturn) -> LexerReturn {
        self.next();
        tok
    }

    // Consume all whitespace starting from the current byte
    fn consume_whitespace(&mut self) {
        unwind_loop! {
            if let Some(byte) = self.next() {
                // This is the most likely scenario, unicode spaces are very uncommon
                if DISPATCHER[*byte as usize] != Dispatch::WHS {
                    // try to short circuit the branch by checking the first byte of the potential unicode space
                    if *byte > 0xC1 && UNICODE_WHITESPACE_STARTS.contains(&byte) {
                        let chr = self.get_unicode_char();
                        if !UNICODE_SPACES.contains(&chr) {
                            return;
                        }
                        self.cur += chr.len_utf8() - 1;
                    } else {
                        return;
                    }
                }
            } else {
                return;
            }
        }
    }

    // Get the unicode char which starts at the current byte and advance the lexer's cursor
    fn get_unicode_char(&self) -> char {
        // This is unreachable for all intents and purposes, but this is just a precautionary measure
        debug_assert!(self.cur < self.bytes.len());

        // Safety: We know this is safe because we require the input to the lexer to be valid utf8 and we always call this when we are at a char
        let string =
            unsafe { std::str::from_utf8_unchecked(&self.bytes.get_unchecked(self.cur..)) };
        let chr = if let Some(chr) = string.chars().next() {
            chr
        } else {
            // Safety: we always call this when we are at a valid char, so this branch is completely unreachable
            unsafe {
                core::hint::unreachable_unchecked();
            }
        };

        chr
    }

    // Get the next byte and advance the index
    #[inline]
    fn next(&mut self) -> Option<&u8> {
        self.cur += 1;
        self.bytes.get(self.cur)
    }

    // Get the next byte but only advance the index if there is a next byte
    // This is really just a hack for certain methods like escapes
    #[inline]
    fn next_bounded(&mut self) -> Option<&u8> {
        if let Some(b) = self.bytes.get(self.cur + 1) {
            self.cur += 1;
            Some(b)
        } else {
            if self.cur != self.bytes.len() {
                self.cur += 1;
            }
            None
        }
    }

    fn advance(&mut self, amount: usize) {
        self.cur += amount;
    }

    fn lookup(byte: u8) -> Dispatch {
        // Safety: our lookup table maps all values of u8, so its impossible for a u8 to be out of bounds
        unsafe { *DISPATCHER.get_unchecked(byte as usize) }
    }

    // Read a `\u0000` escape sequence, this expects the current char to be the `u`, it also does not skip over the escape sequence
    // The pos after this method is the last hex digit
    fn read_unicode_escape(&mut self, advance: bool) -> Result<char, Diagnostic<usize>> {
        debug_assert_eq!(self.bytes[self.cur], b'u');

        let diagnostic = Diagnostic::error()
            .with_message("Invalid digits after unicode escape sequence")
            .with_labels(vec![Label::primary(
                self.file_id,
                (self.cur - 1)..(self.cur + 1),
            )
            .with_message("Expected 4 hex digits following this")]);

        for idx in 0..4 {
            match self.next_bounded() {
                None => {
                    if !advance {
                        self.cur -= idx + 1;
                    }
                    return Err(diagnostic);
                }
                Some(b) if !(*b as u8).is_ascii_hexdigit() => {
                    if !advance {
                        self.cur -= idx + 1;
                    }
                    return Err(diagnostic);
                }
                _ => {}
            }
        }

        unsafe {
            // Safety: input to the lexer is guaranteed to be valid utf8 and so is the range since we return if there is a wrong amount of digits beforehand
            let digits_str = std::str::from_utf8_unchecked(
                self.bytes.get_unchecked((self.cur - 3)..(self.cur + 1)),
            );
            if let Ok(digits) = u32::from_str_radix(digits_str, 16) {
                if !advance {
                    self.cur -= 4;
                }
                // Safety: we make sure the 4 chars are hex digits beforehand, and 4 hex digits cannot make an invalid char
                return Ok(std::char::from_u32_unchecked(digits));
            } else {
                // Safety: we know this is unreachable because 4 hexdigits cannot make an out of bounds char,
                // and we make sure that the chars are actually hex digits
                core::hint::unreachable_unchecked();
            }
        }
    }

    // Validate a `\x00 escape sequence, this expects the current char to be the `x`, it also does not skip over the escape sequence
    // The pos after this method is the last hex digit
    fn validate_hex_escape(&mut self) -> Option<Diagnostic<usize>> {
        debug_assert_eq!(self.bytes[self.cur], b'x');

        let diagnostic = Diagnostic::error()
            .with_message("Invalid digits after hex escape sequence")
            .with_labels(vec![Label::primary(
                self.file_id,
                (self.cur - 1)..(self.cur + 1),
            )
            .with_message("Expected 2 hex digits following this")]);

        for _ in 0..2 {
            match self.next_bounded() {
                None => return Some(diagnostic),
                Some(b) if !(*b as u8).is_ascii_hexdigit() => return Some(diagnostic),
                _ => {}
            }
        }
        None
    }

    // Validate a `\..` escape sequence and advance the lexer based on it
    fn validate_escape_sequence(&mut self) -> Option<Diagnostic<usize>> {
        let cur = self.cur;
        let next = self.next_bounded();
        if let Some(escape) = next {
            match escape {
                b'u' => self.read_unicode_escape(true).err(),
                b'x' => self.validate_hex_escape(),
                _ => {
                    // We use get_unicode_char to account for escaped source characters which are unicode
                    let chr = self.get_unicode_char();
                    self.cur += chr.len_utf8();
                    None
                }
            }
        } else {
            Some(Diagnostic::error().with_labels(vec![
                Label::primary(self.file_id, cur..(cur + 1)).with_message(
                    "Expected an escape sequence following a backslash, but found none",
                ),
            ]))
        }
    }

    // Consume an identifier by recursively consuming IDENTIFIER_PART kind chars
    #[inline]
    fn consume_ident(&mut self) {
        unwind_loop! {
            match self.next_bounded() {
                // This is the most likely branch, unicode inside identifiers is very rare
                Some(b) => {
                    match Self::lookup(*b) {
                        UNI => {
                            // FIXME: This is technically wrong, since es5 states UnicodeCombiningMark, UnicodeDigit, and UnicodeConnectorPunctuation
                            // and es6+ uses ID not XID
                            let chr = self.get_unicode_char();
                            if !UnicodeXID::is_xid_continue(self.get_unicode_char()) {
                                return;
                            }
                            self.cur += chr.len_utf8() - 1;
                        },
                        IDT | L_B | L_C | L_D | L_E | L_F | L_I | L_N | L_R | L_S | L_T | L_V | L_W => {},
                        _ => return,
                    }
                },
                _ => return,
            }
        }
    }

    // Consume a string literal and advance the lexer, and returning a list of errors that occurred when reading the string
    // This could include unterminated string and invalid escape sequences
    fn read_str_literal(&mut self) -> Option<Diagnostic<usize>> {
        // Safety: this is only ever called from lex_token, which is guaranteed to be called on a char position
        let quote = unsafe { *self.bytes.get_unchecked(self.cur) };
        let start = self.cur;
        let mut diagnostic = None;

        while let Some(byte) = self.next_bounded() {
            match *byte {
                b'\\' => {
                    diagnostic = self.validate_escape_sequence();
                }
                b if b == quote => {
                    self.next();
                    return diagnostic;
                }
                _ => {}
            }
        }

        let unterminated = Diagnostic::error()
            .with_message("Unterminated string literal")
            .with_labels(vec![
                Label::primary(self.file_id, self.cur..self.cur).with_message("Input ends here"),
                Label::secondary(self.file_id, start..start + 1)
                    .with_message("String literal starts here"),
            ]);

        Some(unterminated)
    }

    #[inline]
    fn cur_is_ident_part(&self) -> bool {
        debug_assert!(self.cur < self.bytes.len());

        // Safety: we always call this method on a char
        let b = unsafe { self.bytes.get_unchecked(self.cur) };

        match Self::lookup(*b) {
            IDT | DIG | ZER | L_B | L_C | L_D | L_E | L_F | L_I | L_N | L_R | L_S | L_T | L_V
            | L_W => true,
            UNI => self.get_unicode_char().is_xid_continue(),
            _ => false,
        }
    }

    // check if the current char is an identifier start, this implicitly advances if the char being matched
    // is a `\uxxxx` sequence which is an identifier start, or if the char is a unicode char which is an identifier start
    #[inline]
    fn cur_is_ident_start(&mut self) -> bool {
        debug_assert!(self.cur < self.bytes.len());

        // Safety: we always call this method on a char
        let b = unsafe { self.bytes.get_unchecked(self.cur) };

        match Self::lookup(*b) {
            BSL if self.bytes.get(self.cur + 1) == Some(&b'u') => {
                self.next();
                if let Ok(chr) = self.read_unicode_escape(false) {
                    if chr.is_xid_start() {
                        self.advance(5);
                        return true;
                    }
                }
                self.cur -= 1;
                false
            }
            UNI => {
                let chr = self.get_unicode_char();
                if chr.is_xid_start() {
                    self.cur += chr.len_utf8() - 1;
                    true
                } else {
                    false
                }
            }
            IDT | L_B | L_C | L_D | L_E | L_F | L_I | L_N | L_R | L_S | L_T | L_V | L_W => true,
            _ => false,
        }
    }

    #[inline]
    fn resolve_label(&mut self, label: Dispatch) -> LexerReturn {
        let start = self.cur;
        let kind = match label {
            L_B => self.resolve_label_b(),
            L_C => self.resolve_label_c(),
            L_D => self.resolve_label_d(),
            L_E => self.resolve_label_e(),
            L_F => self.resolve_label_f(),
            L_I => self.resolve_label_i(),
            L_N => self.resolve_label_n(),
            L_R => self.resolve_label_r(),
            L_S => self.resolve_label_s(),
            L_T => self.resolve_label_t(),
            L_V => self.resolve_label_v(),
            L_W => self.resolve_label_w(),
            // Safety: this method is never called outside of the lex_token match, and it is only called on L_* dispatches
            _ => unsafe { core::hint::unreachable_unchecked() },
        };

        if let Some(syntax_kind) = kind {
            if let Some(_) = self.next_bounded() {
                if self.cur_is_ident_part() {
                    self.consume_ident();
                    (Token::new(T![ident], self.cur - start), None)
                } else {
                    (Token::new(syntax_kind, self.cur - start), None)
                }
            } else {
                (Token::new(syntax_kind, self.cur - start), None)
            }
        } else {
            self.consume_ident();
            (Token::new(T![ident], self.cur - start), None)
        }
    }

    // Read a number which does not start with 0, since that can be more things and is handled
    // by another function
    #[inline]
    fn read_number(&mut self) {
        unwind_loop! {
            match self.next_bounded() {
                Some(b'0'..=b'9') => {},
                Some(b'.') => {
                    return self.read_float();
                },
                // TODO: merge this, and read_float's implementation into one so we dont duplicate exponent code
                Some(b'e') | Some(b'E') => {
                    // At least one digit is required
                    match self.bytes.get(self.cur + 1) {
                        Some(b'-') | Some(b'+') => {
                            if let Some(b'0'..=b'9') = self.bytes.get(self.cur + 2) {
                                self.next();
                                return self.read_exponent();
                            } else {
                                return;
                            }
                        },
                        Some(b'0'..=b'9') => return self.read_exponent(),
                        _ => return,
                    }
                },
                _ => return,
            }
        }
    }

    #[inline]
    fn read_float(&mut self) {
        unwind_loop! {
            match self.next_bounded() {
                // LLVM has a hard time optimizing inclusive patterns, perhaps we should check if it makes llvm sad,
                // and optimize this into a lookup table
                Some(b'0'..=b'9') => {},
                Some(b'e') | Some(b'E') => {
                    // At least one digit is required
                    match self.bytes.get(self.cur + 1) {
                        Some(b'-') | Some(b'+') => {
                            if let Some(b'0'..=b'9') = self.bytes.get(self.cur + 2) {
                                self.next();
                                return self.read_exponent();
                            } else {
                                return;
                            }
                        },
                        Some(b'0'..=b'9') => return self.read_exponent(),
                        _ => return,
                    }
                },
                _ => return,
            }
        }
    }

    #[inline]
    fn read_exponent(&mut self) {
        if let Some(b'-') | Some(b'+') = self.bytes.get(self.cur + 1) {
            self.next();
        }

        unwind_loop! {
            if let Some(b'0'..=b'9') = self.next() {

            } else {
                return;
            }
        }
    }

    #[inline]
    fn verify_number_end(&mut self, start: usize) -> LexerReturn {
        let err_start = self.cur;
        if self.cur < self.bytes.len() && self.cur_is_ident_start() {
            self.consume_ident();
            let err = Diagnostic::error()
                .with_message("Numbers cannot be followed by identifiers directly after")
                .with_labels(vec![Label::primary(self.file_id, err_start..self.cur)
                    .with_message("An identifier cannot appear here")]);

            (Token::new(SyntaxKind::ERROR, self.cur - start), Some(err))
        } else {
            tok!(NUMBER, self.cur - start)
        }
    }

    /// Lex the next token
    fn lex_token(&mut self) -> LexerReturn {
        // Safety: we always call lex_token when we are at a valid char
        let byte = unsafe { *self.bytes.get_unchecked(self.cur) };
        let start = self.cur;

        // A lookup table of `byte -> fn(l: &mut Lexer) -> Token` is exponentially slower than this approach
        // The speed difference comes from the difference in table size, a 2kb table is easily fit into cpu cache
        // While a 16kb table will be ejected from cache very often leading to slowdowns, this also allows LLVM
        // to do more aggressive optimizations on the match regarding how to map it to instructions
        let dispatched = Self::lookup(byte);

        match dispatched {
            WHS => {
                self.consume_whitespace();
                tok!(WHITESPACE, self.cur - start)
            }
            EXL => self.eat(tok![!]),
            PRC => self.eat(tok![%]),
            AMP => self.eat(tok![&]),
            PNO => self.eat(tok!(L_PAREN, 1)),
            PNC => self.eat(tok!(R_PAREN, 1)),
            MUL => self.eat(tok![*]),
            PLS => self.eat(tok![+]),
            COM => self.eat(tok![,]),
            MIN => self.eat(tok![-]),
            PRD => {
                if let Some(b'0'..=b'9') = self.bytes.get(self.cur + 1) {
                    self.read_float();
                    self.verify_number_end(start)
                } else {
                    self.eat(tok![.])
                }
            },
            BSL => {
                if self.bytes.get(self.cur + 1) == Some(&b'u') {
                    self.next();
                    match self.read_unicode_escape(true) {
                        Ok(chr) => {
                            if chr.is_xid_start() {
                                self.consume_ident();

                                tok!(IDENT, self.cur - start)
                            } else {
                                let err = Diagnostic::error().with_message("Unexpected unicode escape")
                                    .with_labels(vec![Label::primary(self.file_id, start..self.cur)]).with_message("This escape is unexpected, as it does not designate the start of an identifier");

                                self.next();
                                (Token::new(SyntaxKind::ERROR, self.cur - start), Some(err))
                            }
                        }
                        Err(err) => {
                            (Token::new(SyntaxKind::ERROR, self.cur - start), Some(err))
                        }
                    }
                } else {
                    let err = Diagnostic::error()
                    .with_message(&format!("Unexpected token `{}`", byte as char))
                    .with_labels(vec![Label::primary(self.file_id, start..self.cur + 1)]);
                     self.next();

                (Token::new(SyntaxKind::ERROR, 1), Some(err))
                }
            }
            QOT => {
                if let Some(err) = self.read_str_literal() {
                    // TODO: maybe this should be made `STRING` in case of "minor" errors like invalid escape sequences?
                    (Token::new(SyntaxKind::ERROR, self.cur - start), Some(err))
                } else {
                    tok!(STRING, self.cur - start)
                }
            }
            IDT => {
                self.consume_ident();
                tok!(IDENT, self.cur - start)
            }
            DIG => {
                self.read_number();
                self.verify_number_end(start)
            }
            COL => self.eat(tok![:]),
            SEM => self.eat(tok![;]),
            LSS => self.eat(tok![<]),
            EQL => self.eat(tok![=]),
            MOR => self.eat(tok![>]),
            QST => self.eat(tok![?]),
            BTO => self.eat(tok!(L_BRACK, 1)),
            BTC => self.eat(tok![R_BRACK, 1]),
            CRT => self.eat(tok![^]),
            BEO => self.eat(tok![L_CURLY, 1]),
            BEC => self.eat(tok![R_CURLY, 1]),
            PIP => self.eat(tok![|]),
            TLD => self.eat(tok![~]),
            L_B | L_C | L_D | L_E | L_F | L_I | L_N | L_R | L_S | L_T | L_V | L_W => {
                self.resolve_label(dispatched)
            }
            UNI => {
                if UNICODE_WHITESPACE_STARTS.contains(&byte) {
                    self.cur += self.get_unicode_char().len_utf8() - 1;
                    self.consume_whitespace();
                    tok!(WHITESPACE, self.cur - start)
                } else {
                    let chr = self.get_unicode_char();
                    self.cur += chr.len_utf8() - 1;
                    let err = Diagnostic::error()
                        .with_message(&format!("Unexpected token `{}`", chr as char))
                        .with_labels(vec![Label::primary(self.file_id, start..self.cur + 1)]);
                    self.next();

                    (Token::new(SyntaxKind::ERROR, self.cur - start), Some(err))
                }
            }
            _ => {
                let err = Diagnostic::error()
                    .with_message(&format!("Unexpected token `{}`", byte as char))
                    .with_labels(vec![Label::primary(self.file_id, start..self.cur + 1)]);
                self.next();

                (Token::new(SyntaxKind::ERROR, 1), Some(err))
            }
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = LexerReturn;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.bytes.len() {
            return None;
        }

        Some(self.lex_token())
    }
}

// Every handler a byte coming in could be mapped to
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
enum Dispatch {
    ERR,
    WHS,
    EXL,
    QOT,
    IDT,
    PRC,
    AMP,
    PNO,
    PNC,
    MUL,
    PLS,
    COM,
    MIN,
    PRD,
    SLH,
    ZER,
    DIG,
    COL,
    SEM,
    LSS,
    EQL,
    MOR,
    QST,
    BTO,
    BSL,
    BTC,
    CRT,
    TPL,
    L_B,
    L_C,
    L_D,
    L_E,
    L_F,
    L_I,
    L_N,
    L_R,
    L_S,
    L_T,
    L_V,
    L_W,
    BEO,
    PIP,
    BEC,
    TLD,
    UNI,
}
use Dispatch::*;

// A lookup table mapping any incoming byte to a handler function
// This is taken from the ratel project lexer and modified
// FIXME: Should we ignore the first ascii control chars which are nearly never seen instead of returning Err?
static DISPATCHER: [Dispatch; 256] = [
    //   0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F   //
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, WHS, WHS, WHS, WHS, WHS, ERR, ERR, // 0
    ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, ERR, // 1
    WHS, EXL, QOT, ERR, IDT, PRC, AMP, QOT, PNO, PNC, MUL, PLS, COM, MIN, PRD, SLH, // 2
    ZER, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, DIG, COL, SEM, LSS, EQL, MOR, QST, // 3
    ERR, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, // 4
    IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, IDT, BTO, BSL, BTC, CRT, IDT, // 5
    TPL, IDT, L_B, L_C, L_D, L_E, L_F, IDT, IDT, L_I, IDT, IDT, IDT, IDT, L_N, IDT, // 6
    IDT, IDT, L_R, L_S, L_T, IDT, L_V, L_W, IDT, IDT, IDT, BEO, PIP, BEC, TLD, ERR, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];
