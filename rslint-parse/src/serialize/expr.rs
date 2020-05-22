use crate::lexer::token::*;
use crate::parser::cst::expr::Expr;
use once_cell::sync::Lazy;
use std::string::ToString;

// impl ToString for Expr {
//   fn to_string(&self) -> String {

//   }
// }

impl ToString for TokenType {
    /// Converts constant tokens into their string representation.  
    /// Non-constant token types like `LiteralNumber` should be stringified using `Token::to_string()`, not this.
    fn to_string(&self) -> String {
        match self {
            t if t.is_keyword() => stringify!(self).to_ascii_lowercase(),

            TokenType::BinOp(ref data) => data.to_string(),

            TokenType::AssignOp(ref data) => data.to_string(),

            _ => {
                match self {
                    TokenType::BitwiseNot => "~",
                    TokenType::BraceClose | TokenType::TemplateClosed => "}",
                    TokenType::BraceOpen => "{",
                    TokenType::BracketClose => "]",
                    TokenType::BracketOpen => "[",
                    TokenType::Increment => "++",
                    TokenType::Decrement => "--",
                    TokenType::LogicalNot => "!",
                    TokenType::Spread => "...",
                    TokenType::TemplateOpen => "${",
                    TokenType::QuestionMark => "?",
                    // this shouldnt ever be triggered but id rather have this happen than having the program panic
                    _ => stringify!(self),
                }
                .to_string()
            }
        }
    }
}

// BinTokens and AssignTokens are represented as `u8`s, so we can just use a lookup table
static BIN_TOKEN_LOOKUP: [&'static str; 22] = [
    "==", "!=", "===", "!==", "<", "<=", ">", ">=", "<<", ">>", ">>>", "**", "+", "-", "*", "/",
    "%", "|", "^", "&", "||", "&&",
];

impl ToString for BinToken {
    fn to_string(&self) -> String {
        BIN_TOKEN_LOOKUP[*self as u8 as usize].to_string()
    }
}

static ASSIGN_TOKEN_LOOKUP: [&'static str; 13] = [
    "=", "+=", "-=", "*=", "**=", "%=", "<<=", ">>=", ">>>=", "&=", "|=", "^=", "/=",
];

impl ToString for AssignToken {
    fn to_string(&self) -> String {
        ASSIGN_TOKEN_LOOKUP[*self as u8 as usize].to_string()
    }
}
