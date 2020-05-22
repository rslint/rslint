use crate::lexer::token::{BinToken, TokenType};
use crate::parser::cst::expr::*;

impl Expr {
    /// Validate that the expression is a valid assign target.  
    // Productions such as `++--` or `++this` are invalid
    pub fn is_valid_assign_target(&self) -> bool {
        match self {
            // TODO: Handle strict mode `eval` and `arguments`
            Expr::Identifier(_) => true,

            // You cant run update expressions on literals
            Expr::This(_)
            | Expr::String(_)
            | Expr::Number(_)
            | Expr::Regex(_)
            | Expr::False(_)
            | Expr::True(_)
            | Expr::Null(_) => false,

            Expr::Member(_) => true,

            Expr::New(_) => false,

            Expr::Update(_) => false,

            Expr::Unary(_) => false,

            Expr::Binary(_) => false,

            Expr::Conditional(_) => false,
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
