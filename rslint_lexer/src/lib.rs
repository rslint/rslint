//! An extremely fast, lookup table based, ECMAScript lexer which yields SyntaxKind tokens used by the rslint_parse parser.  
//! The tokens yielded by the lexer are "raw", punctuators such as `>>=` will yield `>` + `>` + `=`.  
//! For the purposes of error recovery, tokens may have an error attached to them, which is reflected in the Iterator Item.  
//! The lexer will also yield `COMMENT` and `WHITESPACE` tokens.
//!
//! The lexer operates on raw bytes to take full advantage of lookup table optimizations, these bytes **must** be valid utf8,
//! therefore making a lexer from a `&[u8]` is unsafe since you must make sure the bytes are valid utf8.

#[macro_use]
mod token;
mod tests;

pub use token::Token;

use codespan_reporting::diagnostic::{Diagnostic, Label};
use rslint_syntax::{SyntaxKind, T};
// There is a way of making these functions 7x faster, but it involves 100kb+ static bitmaps
// Although i am reluctant of using that currently as it does not seem needed, but this will have to be considered
use unicode_xid::UnicodeXID;

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
                if DISPATCHER[byte as usize] != Dispatch::WHS {
                    // try to short circuit the branch by checking the first byte of the potential unicode space
                    if byte > 0xC1 && UNICODE_WHITESPACE_STARTS.contains(&byte) {
                        let chr = self.get_unicode_char();
                        if !UNICODE_SPACES.contains(&chr) {
                            return;
                        }
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
    fn get_unicode_char(&mut self) -> char {
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

        self.cur += chr.len_utf8() - 1;
        chr
    }

    // Get the next byte and advance the index
    fn next(&mut self) -> Option<u8> {
        self.cur += 1;
        self.bytes.get(self.cur).map(|x| *x)
    }

    fn lookup(byte: u8) -> Dispatch {
        // Safety: our lookup table maps all values of u8, so its impossible for a u8 to be out of bounds
        unsafe { *DISPATCHER.get_unchecked(byte as usize) }
    }

    // Validate a `\u0000` escape sequence, this expects the current char to be the `u`
    fn read_unicode_escape(&mut self) -> Result<char, Diagnostic<usize>> {
        debug_assert_eq!(self.bytes[self.cur], b'u');

        let diagnostic = Diagnostic::error().with_labels(vec![Label::primary(
            self.file_id,
            (self.cur - 1)..(self.cur + 1),
        )
        .with_message("Expected 4 hex digits following a unicode escape sequence")]);

        for _ in 0..4 {
            match self.next() {
                None => return Err(diagnostic),
                Some(b) if !(b as u8).is_ascii_hexdigit() => return Err(diagnostic),
                _ => {}
            }
        }

        unsafe {
            // Safety: input to the lexer is guaranteed to be valid utf8 and so is the range since we return if there is a wrong amount of digits beforehand
            let digits_str = std::str::from_utf8_unchecked(
                self.bytes.get_unchecked((self.cur - 3)..(self.cur + 1)),
            );
            if let Ok(digits) = u32::from_str_radix(digits_str, 16) {
                // Safety: we make sure the 4 chars are hex digits beforehand, and 4 hex digits cannot make an invalid char
                return Ok(std::char::from_u32_unchecked(digits));
            } else {
                // Safety: we know this is unreachable because 4 hexdigits cannot make an out of bounds char,
                // and we make sure that the chars are actually hex digits
                core::hint::unreachable_unchecked();
            }
        }
    }

    // fn validate_escape_sequence(&mut self) -> Option<Diagnostic<usize>> {
    //     let cur = self.cur;
    //     let next = self.next();
    //     if let Some(escape) = next {
    //         match escape {
    //             b'u' => {
    //                 for _ in
    //             }
    //         }
    //     } else {
    //         Some(Diagnostic::error().with_labels(vec![
    //             Label::primary(self.file_id, cur..(cur + 1)).with_message(
    //                 "Expected an escape sequence following a backslash, but found none",
    //             ),
    //         ]))
    //     }
    // }

    // Consume an identifier by recursively consuming IDENTIFIER_PART kind chars
    fn consume_ident(&mut self) {
        unwind_loop! {
            match self.next() {
                // This is the most likely branch, unicode inside identifiers is very rare
                Some(b) => {
                    match Self::lookup(b) {
                        UNI => {
                            // FIXME: This is technically wrong, since es5 states UnicodeCombiningMark, UnicodeDigit, and UnicodeConnectorPunctuation
                            // and es6+ uses ID not XID
                            if !UnicodeXID::is_xid_continue(self.get_unicode_char()) {
                                return;
                            }
                        },
                        IDT | L_B | L_C | L_D | L_E | L_F | L_I | L_L | L_N | L_P | L_R | L_S | L_T | L_U | L_V | L_W | L_Y => {},
                        _ => return,
                    }
                },
                _ => return,
            }
        }
    }

    // Consume a string literal and advance the lexer, and returning a list of errors that occurred when reading the string
    // This could include unterminated string and invalid escape sequences
    fn read_str_literal(&mut self) -> Vec<Diagnostic<usize>> {
        // Safety: this is only ever called from lex_token, which is guaranteed to be called on a char position
        let quote = unsafe { *self.bytes.get_unchecked(self.cur) };
        let mut diagnostics = vec![];

        while let Some(byte) = self.next() {}

        diagnostics
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
            ERR => tok!(ERROR, 1),
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
            PRD => self.eat(tok![.]),
            BSL => {
                self.next();
                println!("{:#?}", self.read_unicode_escape().unwrap_err());
                self.next();
                tok!(ERROR, 1)
            }
            IDT => {
                let start = self.cur;
                self.consume_ident();
                tok!(IDENT, self.cur - start)
            }
            /* digits */
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
            // UNI => {

            // }
            b => panic!("yeet {:?}", b),
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
    L_L,
    L_N,
    L_P,
    L_R,
    L_S,
    L_T,
    L_U,
    L_V,
    L_W,
    L_Y,
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
    TPL, IDT, L_B, L_C, L_D, L_E, L_F, IDT, IDT, L_I, IDT, IDT, L_L, IDT, L_N, IDT, // 6
    L_P, IDT, L_R, L_S, L_T, L_U, L_V, L_W, IDT, L_Y, IDT, BEO, PIP, BEC, TLD, ERR, // 7
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 8
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // 9
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // A
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // B
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // C
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // D
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // E
    UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, UNI, // F
];
