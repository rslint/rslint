use crate::lexer::token::TokenType;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    This(LiteralExpr),
    Number(LiteralExpr),
    String(LiteralExpr),
    Null(LiteralExpr),
    Regex(LiteralExpr),
    Identifier(LiteralExpr),
    True(LiteralExpr),
    False(LiteralExpr),
    Member(MemberExpr),
    New(NewExpr),
    Update(UpdateExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
}

impl Expr {
    /// Get the span of a returned expression.  
    /// This is required for binary, ternary, and member expressions
    pub fn span(&self) -> &Span {
        match self {
            Expr::This(data)
            | Expr::Number(data)
            | Expr::String(data)
            | Expr::Null(data)
            | Expr::Regex(data)
            | Expr::Identifier(data)
            | Expr::True(data)
            | Expr::False(data) => &data.span,

            Expr::Member(data) => &data.span,
            Expr::New(data) => &data.span,
            Expr::Update(data) => &data.span,
            Expr::Unary(data) => &data.span,
            Expr::Binary(data) => &data.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub span: Span,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub op: TokenType,
    pub whitespace: OperatorWhitespace,
}
/// An expression such as `++foo` or `--foo`
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateExpr {
    pub span: Span,
    pub prefix: bool,
    pub object: Box<Expr>,
    pub op: TokenType,
    pub whitespace: OperatorWhitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub span: Span,
    pub object: Box<Expr>,
    pub op: TokenType,
    pub whitespace: OperatorWhitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorWhitespace {
    pub before_op: Span,
    pub after_op: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpr {
    pub span: Span,
    pub object: Box<Expr>,
    pub property: Box<Expr>,
    pub whitespace: MemberExprWhitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemberExprWhitespace {
    pub before_dot: Span,
    pub after_dot: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr {
    pub span: Span,
    pub target: Box<Expr>,
    pub whitespace: NewExprWhitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewExprWhitespace {
    pub before_new: Span,
    pub after_new: Span,
}

/// An expression which can be described as a single token with leading and trailing whitespace
#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpr {
    pub span: Span,
    pub whitespace: ExprWhitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExprWhitespace {
    pub before: Span,
    pub after: Span,
}

/// An object literal such as `{}` or `{"a": b}`
#[derive(Debug, Clone, PartialEq)]
pub struct Object {
    pub span: Span,
    pub props: Vec<ObjProp>,
    pub has_trailing_comma: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjProp {
    pub span: Span,
    pub key: ObjPropKey,
    pub val: ObjPropVal,
    pub whitespace: Option<ObjPropWhitespace>,
}

/// Whitespace for getter or setter properties.
/// Whitespace for other properties are defined in terms of their expr whitespace.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjPropWhitespace {
    pub ident: ExprWhitespace,
    /// Before the `get` or `set`
    pub before_declarator: Span,
    /// After the `(...)`, before the `{...}`
    pub after_parameter_list: Span,
    /// After the `{...}`, before the next `,` or `}`
    pub after_stmt_list: Span,
}

/// A key inside of an object literal, this may be things such as:  
/// `get a() {}` // a is the key, the body of the get is the val.  
/// `set a() {}` // same as above  
/// `"foo"`  
/// `foo`  
/// `15`  
#[derive(Debug, Clone, PartialEq)]
pub enum ObjPropKey {
    // TODO: Perhaps it would be beneficial to add the actual token to the params
    // TODO: Add computed for ES6
    Identifier(Span, ExprWhitespace),
    LiteralString(Span, ExprWhitespace),
    LiteralNumber(Span, ExprWhitespace),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjPropVal {
    /// Values which are evaluated each time the key is accessed such as `1 + 2`.
    Initialized(Span, Expr),
    Get(Span), // TODO: add statement list
    Set(Span), // TODO: same as above
}
