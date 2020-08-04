use rslint_parser::TokenSource;
use rslint_lexer::{is_linebreak, Token};
use std::collections::HashSet;
use crate::{SyntaxKind::EOF, TextRange, TextSize};

/// Implementation of `rslint_parser::TokenSource` that takes tokens from source code text.
pub struct TextTokenSource<'t> {
    source: &'t str,
    /// Hashset of offsets for tokens which occur after a linebreak. 
    /// This is required for things such as ASI and postfix expressions
    tokens_after_linebreaks: HashSet<TextSize>,

    /// A vector of tokens and their offset from the start
    token_offset_pairs: Vec<(Token, TextSize)>,

    /// Current token and position
    cur: (rslint_parser::Token, usize),
}

impl<'t> TokenSource for TextTokenSource<'t> {
    fn current(&self) -> rslint_parser::Token {
        self.cur.0.to_owned()
    }

    fn source(&self) -> &str {
        self.source
    }

    fn lookahead_nth(&self, n: usize) -> rslint_parser::Token {
        mk_token(self.cur.1 + n, &self.token_offset_pairs)
    }

    fn bump(&mut self) {
        if self.cur.0.kind == EOF {
            return;
        }

        let pos = self.cur.1 + 1;
        self.cur = (mk_token(pos, &self.token_offset_pairs), pos);
    }

    fn is_keyword(&self, kw: &str) -> bool {
        self.token_offset_pairs
            .get(self.cur.1)
            .map(|(token, offset)| &self.source[TextRange::at(*offset, TextSize::from(token.len as u32))] == kw)
            .unwrap_or(false)
    }

    fn had_linebreak_before_cur(&self) -> bool {
        self.tokens_after_linebreaks.contains(&self.token_offset_pairs[self.cur.1].1)
    }
}

fn mk_token(pos: usize, token_offset_pairs: &[(Token, TextSize)]) -> rslint_parser::Token {
    let (kind, is_jointed_to_next) = match token_offset_pairs.get(pos) {
        Some((token, offset)) => (
            token.kind,
            token_offset_pairs
                .get(pos + 1)
                .map(|(_, next_offset)| offset + TextSize::from(token.len as u32) == *next_offset)
                .unwrap_or(false),
        ),
        None => (EOF, false),
    };
    let range = token_offset_pairs.get(pos).map(|x| {
        let start: usize = x.1.into();
        let end = start + x.0.len;
        start..end
    }).unwrap_or(token_offset_pairs.last().map(|x| {
        let start: usize = x.1.into();
        let end = start + x.0.len;
        start..end
    }).unwrap_or(0..0));

    rslint_parser::Token { kind, is_jointed_to_next, range }
}

impl<'t> TextTokenSource<'t> {
    /// Generate input from tokens(except comments and whitespace).
    /// 
    /// # Panics 
    /// This method will panic in case the source and raw tokens do not match
    /// as it relies on the source code for checking if trivia contains linebreaks
    pub fn new(source: &'t str, raw_tokens: &'t [Token]) -> TextTokenSource<'t> {
        let mut tokens_after_linebreaks = HashSet::new();
        let mut token_offset_pairs = Vec::with_capacity(raw_tokens.len() / 2);

        let mut len: TextSize = 0.into();
        let mut has_linebreak = false;

        for token in raw_tokens {
            if token.kind.is_trivia() {
                let src = source.get(len.into()..(usize::from(len) + token.len)).expect("src and tokens do not match");
                if !has_linebreak && src.chars().any(|c| is_linebreak(c)) {
                    has_linebreak = true;
                }
            } else {
                if has_linebreak {
                    tokens_after_linebreaks.insert(len);
                    has_linebreak = false;
                }
                token_offset_pairs.push((*token, len));
            };

            len += TextSize::from(token.len as u32);
        }

        let first = mk_token(0, token_offset_pairs.as_slice());
        TextTokenSource { source, token_offset_pairs, cur: (first, 0), tokens_after_linebreaks }
    }
}