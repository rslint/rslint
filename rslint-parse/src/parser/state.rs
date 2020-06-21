use crate::lexer::token::TokenType;

pub struct ParserState {
    /// The last non-whitespace token 
    pub last_token: Option<TokenType>,
}

impl ParserState {
    pub fn new() -> ParserState {
        ParserState {
            last_token: None
        }
    }

    pub fn update(&mut self, new_token: TokenType) {
        self.last_token = Some(new_token);
    }
}