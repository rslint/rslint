use crate::lexer::token::{BinToken, TokenType};
use crate::parser::cst::expr::*;
use crate::parser::Parser;
use crate::parser::error::ParseDiagnosticType::DisallowedIdentifier;

impl Expr {
    /// Validate that the expression is a valid assign target.  
    // Productions such as `++--` or `++this` are invalid
    pub fn is_valid_assign_target(&self, parser: &mut Parser) -> bool {
        match self {
            Expr::Identifier(LiteralExpr { span, ..}) => {
                let content = span.content(parser.source);
                if parser.state.strict.is_some() && ["eval", "arguments"].contains(&content) {
                    let err = parser.error(DisallowedIdentifier, &format!("`{}` cannot be assigned to in strict mode code", content))
                        .primary(span.to_owned(), &format!("Assignment to `{}` is not allowed", content));
                    
                    parser.errors.push(err);
                }
                true
            },

            // You cant run update expressions on literals
            Expr::This(_)
            | Expr::String(_)
            | Expr::Number(_)
            | Expr::Regex(_)
            | Expr::False(_)
            | Expr::True(_)
            | Expr::Null(_) => false,

            Expr::Member(_) | Expr::Bracket(_) => true,

            Expr::New(_)
            | Expr::Update(_)
            | Expr::Unary(_)
            | Expr::Binary(_)
            | Expr::Conditional(_)
            | Expr::Assign(_)
            | Expr::Sequence(_)
            | Expr::Call(_)
            | Expr::Array(_) 
            | Expr::Object(_) 
            | Expr::Function(_) => false,

            Expr::Grouping(GroupingExpr { ref expr, .. }) => expr.is_valid_assign_target(parser),
        }
    }
}

impl TokenType {
    pub fn precedence(&self) -> Option<u8> {
        if let TokenType::BinOp(ref data) = self {
            Some(data.precedence())
        } else {
            match self {
                TokenType::In => Some(7),
                TokenType::Instanceof => Some(7),
                _ => None,
            }
        }
    }

    pub fn is_identifier_name(&self) -> bool {
        if let TokenType::Identifier = self {
            true
        } else {
            self.is_keyword()
        }
    }

    pub fn is_binop(&self) -> bool {
        match self {
            TokenType::In | TokenType::Instanceof | TokenType::BinOp(_) => true,
            _ => false,
        }
    }
}

impl BinToken {
    pub fn precedence(&self) -> u8 {
        use crate::lexer::token::BinToken::*;
        match self {
            Equality => 6,
            Inequality => 6,
            StrictEquality => 6,
            StrictInequality => 6,
            LessThan => 7,
            LessThanOrEqual => 7,
            GreaterThan => 7,
            GreaterThanOrEqual => 7,
            LeftBitshift => 8,
            RightBitshift => 8,
            UnsignedRightBitshift => 8,

            Add => 9,
            Subtract => 9,
            Multiply => 10,
            Divide => 10,
            Modulo => 11,

            BitwiseOr => 3,
            BitwiseXor => 4,
            BitwiseAnd => 5,
            LogicalOr => 1,
            LogicalAnd => 2,

            Exponent => 11,
        }
    }
}
