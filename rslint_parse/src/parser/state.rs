use crate::lexer::token::TokenType;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParserState<'a> {
    /// The last non-whitespace token 
    pub last_token: Option<TokenType>,
    /// We must keep track of labelled statements as we need to throw some early errors because of them
    /// notably duplicate labels and non existant labels for break and continue
    pub labels: Vec<(&'a str, Span)>,
    /// Whether we are in a switch statement where break is valid
    pub in_switch_stmt: bool,
    /// Whether we are in an iteration statement where break and continue are valid
    pub in_iteration_stmt: bool,
    /// Whether we are in a function declaration where return is allowed
    pub in_function: bool,
    /// Whether the expression parser should ignore binary `in` expressions, this is for `for` loops
    /// To avoid ambiguity between `for (foo in bar;;)` and `for (foo in bar)`
    pub no_in: bool,
    /// Whether we are in strict mode code, this is an optional span so we can use the previous declaration's span
    /// To issue a better warning
    pub strict: Option<Span>,
}

impl<'a> ParserState<'a> {
    pub fn new() -> ParserState<'a> {
        ParserState {
            last_token: None,
            labels: vec![],
            in_switch_stmt: false,
            in_iteration_stmt: false,
            in_function: false,
            no_in: false,
            strict: None,
        }
    }

    pub fn update(&mut self, new_token: TokenType) {
        self.last_token = Some(new_token);
    }
}