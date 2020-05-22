use crate::lexer::token::*;
use crate::parser::cst::expr::Expr;
use once_cell::sync::Lazy;
use std::string::ToString;

impl Expr {
  fn to_string(&self, source: &str) -> String {
    match self {
        Expr::This(data)
        | Expr::Number(data)
        | Expr::String(data)
        | Expr::Null(data)
        | Expr::Regex(data)
        | Expr::Identifier(data)
        | Expr::True(data)
        | Expr::False(data)
        => format!(" {} ", data.span.content(source).to_string()),

        Expr::Member(data) => format!(" {}.{}", data.object.to_string(source).trim(), data.property.to_string(source).trim()),

        Expr::Update(data) => {
            if data.prefix {
                format!(" {}{}", data.op.to_string(), data.object.to_string(source).trim_start())
            } else {
                format!("{}{} ", data.object.to_string(source).trim_end(), data.op.to_string())
            }
        }

        Expr::New(data) => {
            format!(" new {}", data.target.to_string(source).trim_start())
        }

        Expr::Unary(data) => {
            format!(" {} {}", data.op.to_string(), data.object.to_string(source).trim_start())
        }

        Expr::Binary(data) => {
            format!("{}{}{}", data.left.to_string(source), data.op.to_string(), data.right.to_string(source))
        }

        Expr::Conditional(data) => {
            format!("{} ? {} : {}",
                data.condition.to_string(source).trim_end(),
                data.if_true.to_string(source).trim(),
                data.if_false.to_string(source).trim_start()
            )
        }
    }
  }
}

impl Token {
    /// Serializes a token into its source code
    fn to_string(&self, source: &str) -> String {
        let kind = self.token_type;
        match kind {
            TokenType::AssignOp(ref data) => data.to_string(),
            TokenType::BinOp(ref data) => data.to_string(),

            TokenType::Identifier
            | TokenType::InlineComment
            | TokenType::LiteralBinary
            | TokenType::LiteralNumber
            | TokenType::LiteralString
            | TokenType::LiteralRegEx
            | TokenType::MultilineComment
            | TokenType::Shebang
            | TokenType::InvalidToken
            | TokenType::Whitespace
            => self.lexeme.content(source).to_string(),
            
            _ => kind.to_string(),
        }
    }
}

impl ToString for TokenType {
    /// Converts constant tokens into their string representation.  
    /// Non-constant token types like `LiteralNumber` should be stringified using `Token::to_string()`, not this.
    fn to_string(&self) -> String {
        match self {
            t if t.is_keyword() => format!("{:?}", *self).to_ascii_lowercase(),

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
                    TokenType::Colon => ":",
                    TokenType::Comma => ",",
                    TokenType::Linebreak => "\n",
                    TokenType::Semicolon => ";",
                    TokenType::Period => ".",

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

#[cfg(test)]
mod tests {
    use crate::expr;

    #[test]
    fn new() {
        let src = "new  new  foo.bar \n\n";
        assert_eq!(expr!(src).to_string(src), " new new foo.bar");
    }

    #[test]
    fn binary() {
        let src = "1 +  7 * 1 / 6";
        assert_eq!(expr!(src).to_string(src), " 1 + 7 * 1 / 6 ");
    }

    #[test]
    fn unary() {
        let src = "delete \n\ntypeof foo  ";
        assert_eq!(expr!(src).to_string(src), " delete typeof foo ");
    }

    #[test]
    fn update() {
        let src = " ++foo + foo.bar++ ";
        assert_eq!(expr!(src).to_string(src), " ++foo + foo.bar++ ");
    }

    #[test]
    fn conditional() {
        let src = "foo && bar ?   87: true";
        assert_eq!(expr!(src).to_string(src), " foo && bar ? 87 : true ");
    }
}