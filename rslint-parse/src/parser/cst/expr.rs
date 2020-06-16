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
    Conditional(ConditionalExpr),
    Assign(AssignmentExpr),
    Sequence(SequenceExpr),
    Call(CallExpr),
    Bracket(BracketExpr),
    Grouping(GroupingExpr),
    Array(ArrayExpr),
    Object(Object),
}

impl Expr {
    /// Get the span of a returned expression.  
    /// This is required for expressions which need to know about the previous expression's
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
            Expr::Conditional(data) => &data.span,
            Expr::Assign(data) => &data.span,
            Expr::Sequence(data) => &data.span,
            Expr::Call(data) => &data.span,
            Expr::Bracket(data) => &data.span,
            Expr::Grouping(data) => &data.span,
            Expr::Array(data) => &data.span,
            Expr::Object(data) => &data.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpr {
    pub span: Span,
    /// This is an option because undefined values can be declared
    pub exprs: Vec<Option<Expr>>,
    pub comma_whitespaces: Vec<OperatorWhitespace>,
    pub opening_bracket_whitespace: OperatorWhitespace,
    pub closing_bracket_whitespace: OperatorWhitespace,
}

/// An expression enclosed by parentheses
#[derive(Debug, Clone, PartialEq)]
pub struct GroupingExpr {
    pub span: Span,
    pub expr: Box<Expr>,
    pub opening_paren_whitespace: OperatorWhitespace,
    pub closing_paren_whitespace: OperatorWhitespace,
}

/// A member access expression with brackets, such as `foo["bar"]`.
#[derive(Debug, Clone, PartialEq)]
pub struct BracketExpr {
    pub span: Span,
    pub object: Box<Expr>,
    pub property: Box<Expr>,
    pub opening_bracket_whitespace: OperatorWhitespace,
    pub closing_bracket_whitespace: OperatorWhitespace,
}

/// A call to a function with arguments such as `foo(bar, baz,)`.
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub span: Span,
    pub callee: Box<Expr>,
    pub arguments: Arguments,
}

/// A list of expressions delimited by commas such as `a, b, c`.
#[derive(Debug, Clone, PartialEq)]
pub struct SequenceExpr {
    pub span: Span,
    pub exprs: Vec<Expr>,
    /// A vector of the whitespace of each comma in the sequence.  
    /// The length of this vector should always be `exprs.len() - 1`.  
    /// if you find this is not the case, please open an issue at https://github.com/RDambrosio016/RSLint
    pub comma_whitespace: Vec<OperatorWhitespace>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub span: Span,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub op: TokenType,
    pub whitespace: OperatorWhitespace
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalExpr {
    pub span: Span,
    pub condition: Box<Expr>,
    pub if_false: Box<Expr>,
    pub if_true: Box<Expr>,
    pub whitespace: ConditionalWhitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionalWhitespace {
    pub before_qmark: Span,
    pub after_qmark: Span,
    pub before_colon: Span,
    pub after_colon: Span,
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
    pub args: Option<Arguments>,
    pub whitespace: NewExprWhitespace,
}

/// Arguments like `(foo, bar,)`, You can find if there was a trailing comma by checking  
/// `if comma_whitespace.len() == arguments.len()`
#[derive(Debug, Clone, PartialEq)]
pub struct Arguments {
    pub span: Span,
    pub arguments: Vec<Expr>,
    pub open_paren_whitespace: OperatorWhitespace,
    pub close_paren_whitespace: OperatorWhitespace,
    pub comma_whitespaces: Vec<OperatorWhitespace>,
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
    pub comma_whitespaces: Vec<OperatorWhitespace>,
    pub open_brace_whitespace: OperatorWhitespace,
    pub close_brace_whitespace: OperatorWhitespace,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjProp {
    pub span: Span,
    pub key: Box<Expr>,
    pub value: Box<Expr>,
    /// The whitespace of the colon
    pub whitespace: OperatorWhitespace,
}