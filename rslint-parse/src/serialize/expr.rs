use crate::lexer::token::*;
use crate::parser::cst::expr::*;
use std::string::ToString;

impl Arguments {
    /// Serialize arguments such as `(foo, bar)` to a string, this strips any trailing comma that may be present.
    pub fn to_string(&self, source: &str) -> String {
        // The serialization of the arguments is the same as the serialization of a sequence expr
        // Therefore this method just constructs an `Expr::Sequence(..)` and serializes it
        let contents = Expr::Sequence(SequenceExpr {
            span: self.span.to_owned(),
            exprs: self.arguments.to_owned(),
            comma_whitespace: self.comma_whitespaces.to_owned(),
        })
        .to_string(source);

        format!("({})", contents.trim())
    }
}

impl Expr {
    pub fn to_string(&self, source: &str) -> String {
        match self {
            Expr::This(data)
            | Expr::Number(data)
            | Expr::String(data)
            | Expr::Null(data)
            | Expr::Regex(data)
            | Expr::Identifier(data)
            | Expr::True(data)
            | Expr::False(data) => format!(" {} ", data.span.content(source).to_string()),

            Expr::Member(data) => format!(
                " {}.{}",
                data.object.to_string(source).trim(),
                data.property.to_string(source).trim()
            ),

            Expr::Update(data) => {
                if data.prefix {
                    format!(
                        " {}{}",
                        data.op.to_string(),
                        data.object.to_string(source).trim_start()
                    )
                } else {
                    format!(
                        "{}{} ",
                        data.object.to_string(source).trim_end(),
                        data.op.to_string()
                    )
                }
            }

            Expr::New(data) => format!(
                " new {}{} ",
                data.target.to_string(source).trim(),
                data.args
                    .as_ref()
                    .map_or("".to_string(), |args| args.to_string(source))
            ),

            Expr::Unary(data) => format!(
                " {} {}",
                data.op.to_string(),
                data.object.to_string(source).trim_start()
            ),

            Expr::Binary(data) => format!(
                "{}{}{}",
                data.left.to_string(source),
                data.op.to_string(),
                data.right.to_string(source)
            ),

            Expr::Conditional(data) => format!(
                "{} ? {} : {}",
                data.condition.to_string(source).trim_end(),
                data.if_true.to_string(source).trim(),
                data.if_false.to_string(source).trim_start()
            ),

            Expr::Assign(data) => format!(
                "{}{}{}",
                data.left.to_string(source),
                data.op.to_string(),
                data.right.to_string(source)
            ),

            Expr::Call(data) => format!(
                "{}{} ",
                data.callee.to_string(source).trim_end(),
                data.arguments.to_string(source)
            ),

            Expr::Bracket(data) => format!(
                "{}[{}] ",
                data.object.to_string(source).trim_end(),
                data.property.to_string(source).trim()
            ),

            Expr::Grouping(data) => format!(" ({}) ", data.expr.to_string(source).trim()),

            Expr::Array(data) => {
                let exprs = data
                    .exprs
                    .iter()
                    .map(|expr| {
                        expr.as_ref()
                            .map_or(",".to_string(), |expr| expr.to_string(source))
                    })
                    .collect::<Vec<String>>();

                // Calculate the approximate size of the final string to avoid allocating on every iteration
                // This ends up being a bit larger than the actual size but that is negligible
                let alloc_size = (exprs.iter().map(|x| x.len()).sum::<usize>() //The total size of the expressions
                    // Assume each element is an expression with a comma (there can be no exprs, so we cant just subtract without risking an overflow error)
                    + exprs.len().checked_sub(1).unwrap_or(exprs.len()))
                    // The leading ` [` and trailing `] `
                    + 4;

                let mut ret = String::with_capacity(alloc_size);
                ret.push_str(" [");
                for (idx, expr) in exprs.iter().enumerate() {
                    if idx == exprs.len() - 1 || expr == &"," {
                        if expr == &"," {
                            ret = ret.trim_end().to_string();
                        }
                        ret.push_str(&expr.trim());
                    } else {
                        ret.push_str(expr.trim());
                        ret.push_str(", ");
                    }
                }
                ret.push_str("] ");
                println!("{}, {}", ret.len(), alloc_size);
                ret
            }

            Expr::Object(data) => {
                let props = data
                    .props
                    .iter()
                    .map(|prop| {
                        format!(
                            "{}: {}",
                            prop.key.to_string(source).trim(),
                            prop.value.to_string(source).trim()
                        )
                    })
                    .collect::<Vec<String>>();
                
                // Calculate the approximate final size of the string to avoid allocating on each iteration
                let alloc_size = (props.iter().map(|x| x.len()).sum::<usize>() + (data.props.len() - 1) * 2) + 4;

                let mut ret = String::with_capacity(alloc_size);
                ret.push_str(" {");
                for (idx, prop) in props.iter().enumerate() {
                    if idx == props.len() - 1 {
                        ret.push_str(&prop);
                    } else {
                        ret.push_str(&prop);
                        ret.push_str(", ");
                    }
                }
                ret.push_str("} ");
                ret
            }

            Expr::Sequence(data) => {
                let exprs = data
                    .exprs
                    .iter()
                    .map(|expr| expr.to_string(source))
                    .collect::<Vec<String>>();

                // Calculate the size needed for the final string.
                // This avoids having to allocate more space on the string for every iteration, which is slow
                let alloc_size = exprs.iter().map(|x| x.len()).sum::<usize>() + exprs.len() - 1;

                let mut ret = String::with_capacity(alloc_size);
                for (idx, expr) in exprs.iter().enumerate() {
                    // if this is the last expression we shouldnt add a trailing comma
                    if idx == exprs.len() - 1 {
                        ret.push_str(&expr);
                    } else {
                        ret.push_str(expr.trim_end());
                        ret.push_str(",");
                    }
                }
                ret
            }
        }
    }
}

impl Token {
    /// Serializes a token into its source code
    #[allow(dead_code)]
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
            | TokenType::Whitespace => self.lexeme.content(source).to_string(),

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
        assert_eq!(expr!(src).to_string(src), " new new foo.bar ");
    }

    #[test]
    fn new_with_args() {
        let src = "new foo   (bar, baz,) ";
        println!("{}", expr!(src).to_string(src));
        assert_eq!(expr!(src).to_string(src), " new foo(bar, baz) ");
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

    #[test]
    fn assign() {
        let src = "foo +=7 = foo%=bar       ";
        assert_eq!(expr!(src).to_string(src), " foo += 7 = foo %= bar ");
    }

    #[test]
    fn sequence() {
        let src = "1 ? 2 :\nnew foo,6,7,\n\n8\n";
        assert_eq!(expr!(src).to_string(src), " 1 ? 2 : new foo, 6, 7, 8 ");
    }

    #[test]
    fn bracket() {
        let src = "foo [ bar ]\n['foo']";
        assert_eq!(expr!(src).to_string(src), " foo[bar]['foo'] ");
    }

    #[test]
    fn arguments() {
        use crate::parser::Parser;
        let src = "(foo,bar,     baz, 6 + 7, 9,  )";
        let mut parser = Parser::with_source(src, "tests", true).unwrap();
        assert_eq!(
            parser.parse_args(None).unwrap().to_string(src),
            "(foo, bar, baz, 6 + 7, 9)"
        );
    }

    #[test]
    fn call() {
        let src = "\nfoo  (bar, baz,)\n";
        assert_eq!(expr!(src).to_string(src), " foo(bar, baz) ");
    }

    #[test]
    fn grouping() {
        let src = " ( foo * ( bar ))\n\n";
        assert_eq!(expr!(src).to_string(src), " (foo * (bar)) ");
    }

    #[test]
    fn array() {
        let src = " [ foo, 1, 4, ,,, 7,  8, [66]] ";
        assert_eq!(expr!(src).to_string(src), " [foo, 1, 4,,,,7, 8, [66]] ");
    }

    #[test]
    fn object() {
        let src = " { foo: bar, 1  : 5, var: 7 += 7, \"a\": /aa/g   }";
        assert_eq!(expr!(src).to_string(src), " {foo: bar, 1: 5, var: 7 += 7, \"a\": /aa/g} ");
    }
}
