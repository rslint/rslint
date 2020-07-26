//! A fast, lossless, ECMAScript parser used by RSLint.

pub mod cst;
pub mod error;
pub mod state;
pub mod subparsers;
pub mod util;

use crate::diagnostic::ParserDiagnostic;
use crate::diagnostic::ParserDiagnosticType;
use crate::lexer::lexer::Lexer;
use crate::lexer::token::*;
use crate::parser::cst::expr::*;
use crate::parser::cst::stmt::*;
use crate::parser::cst::*;
use crate::parser::error::ParseDiagnosticType;
use crate::parser::state::ParserState;
use crate::peek;
use crate::span::Span;
use crate::util::multipeek::{multipeek, MultiPeek};

/// An error tolerant, (mostly) lossless, ECMAScript parser
pub struct Parser<'a> {
    pub lexer: MultiPeek<Lexer<'a>>,
    pub cur_tok: Token,

    /// Errors reported by the parser or the lexer which have been recovered from.
    pub errors: Vec<ParserDiagnostic>,
    pub source: &'a str,
    pub file_id: usize,

    /// Whether the parser should attempt secondary recovery by throwing out
    /// tokens until a valid one is found
    /// This recovery is dangerous and can yield secondary confusing errors
    pub discard_recovery: bool,
    pub cst: CST,
    /// The optional start for spans, this is for parsing chunks of code in larger files  
    /// Will be `0` if no offset is specified
    pub offset: usize,
    pub state: ParserState<'a>,
    pub options: ParserOptions,
}

/// Options to control the behavior of the parser
pub struct ParserOptions {
    /// Whether a `return` statement is allowed outside of a function declaration
    pub allow_return_outside_function: bool,
    /// Whether shebang sequences at the top of the file are allowed, e.g. `#!/usr/bin/env node`
    /// If a shebang is encountered, an error will be issued
    pub allow_shebang: bool,
}

impl ParserOptions {
    pub fn new(allow_return_outside_function: bool, allow_shebang: bool) -> Self {
        Self {
            allow_return_outside_function,
            allow_shebang,
        }
    }
}

impl<'a> Parser<'a> {
    /// Makes a parser directly from source code, calling the lexer automatically.
    /// Will return None if the source is empty.
    pub fn with_source(source: &'a str, file_id: usize, discard_recovery: bool) -> Option<Self> {
        if source.len() == 0 {
            return None;
        }
        let mut lexer = multipeek(Lexer::new(source, file_id));
        let next = lexer.next();
        Some(Self {
            lexer,
            cur_tok: next.unwrap().0.unwrap(),
            errors: vec![],
            source,
            file_id,
            discard_recovery,
            cst: CST::new(),
            offset: 0,
            state: ParserState::new(),
            options: ParserOptions::new(false, false),
        })
    }

    /// Create a parser from source code, with an offset added for each span in the CST, useful for parsing subchunks of code in larger files.  
    /// # Returns  
    /// Will return `None` if any of the following are true:  
    /// - The source is an empty string  
    /// - The offset is greater or equal to the source length
    pub fn with_source_and_offset(
        source: &'a str,
        file_id: usize,
        discard_recovery: bool,
        offset: usize,
    ) -> Option<Self> {
        if source.len() == 0 || offset >= source.len() {
            return None;
        }

        let mut lexer = multipeek(Lexer::new(source, file_id));
        let next = lexer.next();
        Some(Self {
            lexer,
            cur_tok: next.unwrap().0.unwrap(),
            errors: vec![],
            source,
            file_id,
            discard_recovery,
            cst: CST::new(),
            offset,
            state: ParserState::new(),
            options: ParserOptions::new(false, false),
        })
    }

    /// Parse an ECMAScript script into a concrete syntax tree  
    /// ** The parser can and will emit recoverable errors, you can access them through `parser.errors` after parsing even if the parsing yields
    /// an unrecoverable error**
    pub fn parse_script(&mut self) -> Result<CST, ParserDiagnostic> {
        self.cst.statements = self.parse_stmt_decl_list(None, Some(&[TokenType::EOF]), true)?;
        Ok(self.cst.to_owned())
    }

    /// Advances the parser's lexer and returns the optional token  
    ///  
    /// # Errors  
    /// Returns an Err if the lexer returns an unrecoverable error  
    #[inline(always)]
    pub fn advance_lexer(
        &mut self,
        skip_linebreak: bool,
    ) -> Result<Option<Token>, ParserDiagnostic> {
        let res = self.lexer.next();
        match res {
            // Unrecoverable lexer error
            r @ Some((None, Some(_))) => Err(r.unwrap().1.unwrap()),
            // Lexer is finished after returning EOF
            None => Ok(None),
            // Successful scan
            Some((Some(_), None)) => {
                let tok = res.unwrap().0.unwrap();
                // if the current token isnt a whitespace we should update the state's last token to be the current
                if !self.cur_tok.token_type.is_whitespace() {
                    self.state.last_token = Some(self.cur_tok.token_type);
                }
                if skip_linebreak && tok.token_type == TokenType::Linebreak {
                    while self.cur_tok.token_type == TokenType::Linebreak {
                        self.advance_lexer(false)?;
                    }
                    return Ok(Some(self.cur_tok.to_owned()));
                }

                self.cur_tok = tok.to_owned();
                Ok(Some(tok))
            }
            // Lexer could recover from error
            // This can never be a linebreak currently so we dont have to account for linebreak skipping
            Some((Some(_), Some(_))) => {
                let tuple = res.unwrap();
                self.errors.push(tuple.1.unwrap());
                let tok = tuple.0.unwrap();
                self.cur_tok = tok.to_owned();
                Ok(Some(tok))
            }
            _ => unreachable!(),
        }
    }

    /// Peek the next token without advancing the lexer
    ///  
    /// # Errors  
    /// Returns an Err if the lexer returns an unrecoverable error  
    #[inline(always)]
    pub fn peek_lexer(&mut self) -> Result<Option<&Token>, ParserDiagnostic> {
        let res = self.lexer.peek();
        match res {
            // Unrecoverable lexer error
            Some((None, Some(_))) => Err(res.unwrap().1.to_owned().unwrap()),
            // Lexer is finished after returning EOF
            None => Ok(None),
            // Successful scan
            Some((Some(_), None)) => Ok(Some(&res.unwrap().0.as_ref().unwrap())),
            // Lexer could recover from error
            Some((Some(_), Some(_))) => {
                let tuple = res.unwrap();
                Ok(Some(tuple.0.as_ref().unwrap()))
            }
            _ => unreachable!(),
        }
    }

    /// Peek the lexer while a token matches a function and return the token that does not match or None if the lexer is finished
    #[inline(always)]
    pub fn peek_while<F>(&mut self, func: F) -> Result<Option<Token>, ParserDiagnostic>
    where
        F: Fn(&Token) -> bool,
    {
        loop {
            match self.peek_lexer()? {
                Some(t) if func(&t) => {}

                t => return Ok(t.map(|t| t.to_owned())),
            }
        }
    }

    /// Advance the lexer while a token matches a function
    #[inline(always)]
    pub fn advance_while<F>(
        &mut self,
        skip_linebreak: bool,
        func: F,
    ) -> Result<(), ParserDiagnostic>
    where
        F: Fn(&Token) -> bool,
    {
        loop {
            match self.advance_lexer(skip_linebreak)? {
                Some(t) if !func(&t) => break,
                Some(t) if func(&t) => {}
                _ => break,
            }
        }
        Ok(())
    }

    /// Throw out tokens until a valid one is found, alternatively throw an error with an optional message if `Self::discard_recovery` is `false`
    /// # Errors  
    /// Returns an Err if `Self::discard_recovery` is `false`  
    pub fn discard_recover<F>(
        &mut self,
        message: Option<&'a str>,
        func: F,
    ) -> Result<(), ParserDiagnostic>
    where
        F: Fn(&TokenType) -> bool,
    {
        if !self.discard_recovery {
            Err(self
                .error(
                    ParseDiagnosticType::UnexpectedToken,
                    message.unwrap_or(&format!(
                        "Unexpected token `{}`",
                        self.cur_tok.lexeme.content(self.source)
                    )),
                )
                .primary(
                    self.cur_tok.lexeme.range().to_owned(),
                    "Unexpected in the current context",
                ))
        } else {
            let origin_span = self.cur_tok.lexeme.to_owned();
            self.advance_while(true, |x| func(&x.token_type))?;
            if self.done() {
                return Err(self
                    .error(
                        ParseDiagnosticType::UnexpectedToken,
                        message.unwrap_or(&format!(
                            "Unexpected token `{}`",
                            self.cur_tok.lexeme.content(self.source)
                        )),
                    )
                    .primary(origin_span, "Unexpected in the current context"));
            }
            Ok(())
        }
    }

    /// Get the span of the current token if it is a whitespace or return a span with length zero
    /// With the start and end set to the current token's start position  
    #[inline(always)]
    pub fn whitespace(&mut self, leading: bool) -> Result<Span, ParserDiagnostic> {
        if self.cur_tok.is_whitespace() {
            // If its trailing whitespace, it will not include linebreaks in it
            if !leading && self.cur_tok.token_type == TokenType::Linebreak {
                return Ok(Span::new(
                    self.cur_tok.lexeme.start,
                    self.cur_tok.lexeme.start,
                ));
            }

            let start = self.cur_tok.lexeme.start;
            self.advance_while(leading, |tok: &Token| {
                tok.token_type == TokenType::Whitespace || tok.is_comment()
            })?;
            Ok(Span::new(start, self.cur_tok.lexeme.start))
        } else {
            Ok(Span::new(
                self.cur_tok.lexeme.start,
                self.cur_tok.lexeme.start,
            ))
        }
    }

    /// This handles ASI (automatic semicolon insertion), a semicolon is explicit if the next token is a semicolon.  
    /// A semicolon is implicit if any of the following conditions are true:
    /// - The next token is EOF  
    /// - The previous token was a `}`  
    /// - There is a linebreak after the current token  
    pub fn semi(&mut self) -> Result<Option<Semicolon>, ParserDiagnostic> {
        const ACCEPTABLE: [TokenType; 3] =
            [TokenType::EOF, TokenType::BraceClose, TokenType::Linebreak];

        if self
            .peek_while(|x| x.is_whitespace())?
            .map(|x| x.token_type)
            == Some(TokenType::Semicolon)
            || self.cur_tok.token_type == TokenType::Semicolon
        {
            let before = self.whitespace(true)?;
            self.advance_lexer(false)?;
            let after = self.whitespace(false)?;

            return Ok(Some(Semicolon::Explicit(LiteralWhitespace {
                before,
                after,
            })));
        }

        self.lexer.reset();

        if ACCEPTABLE.contains(&peek!(self).unwrap_or(TokenType::EOF))
            || ACCEPTABLE.contains(&self.state.last_token.unwrap_or(TokenType::EOF))
            || self.cur_tok.token_type == TokenType::Linebreak
            || self
                .peek_while(|x| x.token_type == TokenType::Whitespace || x.is_comment())?
                .map(|x| x.token_type)
                == Some(TokenType::Linebreak)
        {
            return Ok(Some(Semicolon::Implicit));
        } else {
            return Ok(None);
        }
    }

    pub fn error(&self, kind: ParseDiagnosticType, msg: &str) -> ParserDiagnostic {
        let message = &msg.to_owned();
        ParserDiagnostic::new(self.file_id, ParserDiagnosticType::Parser(kind), message)
    }

    pub fn warning(&self, kind: ParseDiagnosticType, msg: &str) -> ParserDiagnostic {
        let message = &msg.to_owned();
        ParserDiagnostic::warning(self.file_id, ParserDiagnosticType::Parser(kind), message)
    }

    pub fn span(&self, start: usize, end: usize) -> Span {
        Span::new(start + self.offset, end + self.offset)
    }

    pub fn done(&self) -> bool {
        self.cur_tok.token_type == TokenType::EOF
    }
}
