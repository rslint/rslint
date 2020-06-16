#[macro_export]
macro_rules! peek_or {
    ($parser:expr, $expected:expr) => {{
        if $expected.contains(&$parser.cur_tok.token_type) {
            Some($parser.cur_tok.token_type)
        // Peeking should not peek if the current token is not whitespace
        } else if $parser.cur_tok.is_whitespace() {
            let res = $parser
                .peek_while(|x| x.is_whitespace())?
                .map(|x| x.token_type);
            $parser.lexer.reset();
            res
        } else {
            Some($parser.cur_tok.token_type)
        }
    }};
    ($parser:expr) => {{
        if $parser.cur_tok.is_whitespace() {
            let res = $parser
                .peek_while(|x| x.is_whitespace())?
                .map(|x| x.token_type);
            $parser.lexer.reset();
            res
        } else {
            Some($parser.cur_tok.token_type)
        }
    }}
}