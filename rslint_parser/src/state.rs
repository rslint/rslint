#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ParserState {
    pub include_in: bool,
}

impl Default for ParserState {
    fn default() -> Self {
        Self {
            include_in: true
        }
    }
}