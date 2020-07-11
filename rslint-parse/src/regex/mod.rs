//! A fast parser for ECMAScript defined regex patterns
//! Input can be either given as a raw token (`/a+/gim`) or as a pattern and flags (`a+` and `gim`)

pub mod ast;
pub mod error;

use self::ast::*;
use crate::diagnostic::ParserDiagnostic;
use crate::span::Span;
use std::str::CharIndices;

type RegexParserResult<'a> = Result<PatternItem, ParserDiagnostic<'a>>;

pub struct RegexParser<'a> {
    pub pattern: &'a str,
    /// The flags of the regex as a string
    pub flags: &'a str,
    /// The ID of the file used for diagnostics, usually the file path
    pub file_id: &'a str,
    /// The offset added to the span of each diagnostic
    pub offset: usize,
    iter: CharIndices<'a>,
    cur_char: char,
    cur: usize,
}

impl<'a> RegexParser<'a> {
    /// Create a new parser with the raw string of a pattern and its flags as a string, e.g. `a+` and `gm`  
    /// As well as its file id used for diagnostics, usually this is the file's path as a string
    pub fn with_pattern(pattern: &'a str, flags: &'a str, file_id: &'a str) -> Self {
        // TODO: process flags
        Self {
            pattern,
            flags,
            file_id,
            offset: 0,
            iter: pattern.char_indices(),
            cur: 0,
            cur_char: ' ',
        }
    }

    #[inline]
    fn advance(&mut self) -> Option<char> {
        self.iter.next().map(|x| {
            self.cur = x.0;
            self.cur_char = x.1;
            x.1
        })
    }

    #[inline]
    fn is(&mut self, c: char) -> bool {
        self.advance() == Some(c)
    }

    /// Parse an "either or" pattern, e.g. `a|b`
    pub fn parse_disjunction(&mut self) -> RegexParserResult<'a> {
        let start = self.cur;
        let left = PatternItem::Placeholder;

        if self.is('|') {
            let right = Box::new(self.parse_disjunction()?);

            return Ok(PatternItem::Disjunction(Disjunction {
                span: Span::new(start, self.cur),
                left: Box::new(left),
                right,
            }));
        }
        Ok(left)
    }

    pub fn parse_alternative(&mut self) -> RegexParserResult<'a> {
        unimplemented!();
    }


}
