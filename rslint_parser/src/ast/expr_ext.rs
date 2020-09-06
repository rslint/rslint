//! Extensions for things which are not easily generated in ast expr nodes

use crate::{ast::*, util::*, SyntaxText, TextSize, TokenSet, T};
use SyntaxKind::*;

impl BracketExpr {
    pub fn object(&self) -> Option<Expr> {
        support::children(self.syntax()).next()
    }

    pub fn prop(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }
}

impl CondExpr {
    pub fn test(&self) -> Option<Expr> {
        support::children(self.syntax()).next()
    }

    pub fn cons(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }

    pub fn alt(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(2)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PropName {
    Computed(ComputedPropertyName),
    Literal(Literal),
    Ident(Name),
}

impl AstNode for PropName {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, NAME | LITERAL | COMPUTED_PROPERTY_NAME)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(syntax.kind()) {
            None
        } else {
            Some(match syntax.kind() {
                LITERAL => PropName::Literal(Literal::cast(syntax).unwrap()),
                NAME => PropName::Ident(Name::cast(syntax).unwrap()),
                COMPUTED_PROPERTY_NAME => {
                    PropName::Computed(ComputedPropertyName::cast(syntax).unwrap())
                }
                _ => unreachable!(),
            })
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            PropName::Ident(s) => s.syntax(),
            PropName::Literal(s) => s.syntax(),
            PropName::Computed(s) => s.syntax(),
        }
    }
}

impl PropName {
    pub fn as_string(&self) -> Option<std::string::String> {
        Some(self.syntax().text().to_string())
    }
}

impl LiteralProp {
    pub fn key(&self) -> Option<PropName> {
        if PropName::can_cast(
            support::children::<Expr>(self.syntax())
                .next()?
                .syntax()
                .kind(),
        ) {
            PropName::cast(
                support::children::<Expr>(self.syntax())
                    .next()
                    .unwrap()
                    .syntax()
                    .to_owned(),
            )
        } else {
            None
        }
    }

    pub fn value(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }
}

/// A binary operation applied to two expressions
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum BinOp {
    /// `<`
    LessThan,
    /// `>`
    GreaterThan,
    /// `<=`
    LessThanOrEqual,
    /// `>=`
    GreaterThanOrEqual,
    /// `==`
    Equality,
    /// `===`
    StrictEquality,
    /// `!=`
    Inequality,
    /// `!==`
    StrictInequality,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Times,
    /// `/`
    Divide,
    /// `%`
    Remainder,
    /// `**`
    Exponent,
    /// `<<`
    LeftShift,
    /// `>>`
    RightShift,
    /// `>>>`
    UnsignedRightShift,
    /// `&`
    BitwiseAnd,
    /// `|`
    BitwiseOr,
    /// `^`
    BitwiseXor,
    /// `??`
    NullishCoalescing,
    /// `||`
    LogicalOr,
    /// `&&`
    LogicalAnd,
    /// `in`
    In,
    /// `instanceof`
    Instanceof
}

impl BinExpr {
    pub fn op_details(&self) -> Option<(SyntaxToken, BinOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|x| x.into_token())
            .find_map(|t| {
                let op = match t.kind() {
                    T![<] => BinOp::LessThan,
                    T![>] => BinOp::GreaterThan,
                    T![<=] => BinOp::LessThanOrEqual,
                    T![>=] => BinOp::GreaterThanOrEqual,
                    T![==] => BinOp::Equality,
                    T![===] => BinOp::StrictEquality,
                    T![!=] => BinOp::Inequality,
                    T![!==] => BinOp::StrictInequality,
                    T![+] => BinOp::Plus,
                    T![-] => BinOp::Minus,
                    T![*] => BinOp::Times,
                    T![/] => BinOp::Divide,
                    T![%] => BinOp::Remainder,
                    T![**] => BinOp::Exponent,
                    T![<<] => BinOp::LeftShift,
                    T![>>] => BinOp::RightShift,
                    T![>>>] => BinOp::UnsignedRightShift,
                    T![&] => BinOp::BitwiseAnd,
                    T![|] => BinOp::BitwiseOr,
                    T![^] => BinOp::BitwiseXor,
                    T![??] => BinOp::NullishCoalescing,
                    T![||] => BinOp::LogicalOr,
                    T![&&] => BinOp::LogicalAnd,
                    T![in] => BinOp::In,
                    T![instanceof] => BinOp::Instanceof,
                    _ => return None,
                };
                Some((t, op))
            })
    }

    pub fn op(&self) -> Option<BinOp> {
        self.op_details().map(|t| t.1)
    }

    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op_details().map(|t| t.0)
    }

    pub fn lhs(&self) -> Option<Expr> {
        support::children(self.syntax()).next()
    }

    pub fn rhs(&self) -> Option<Expr> {
        support::children(self.syntax()).nth(1)
    }

    /// Whether this binary expr is a `||` or `&&` expression. 
    pub fn conditional(&self) -> bool {
        token_set![T![||], T![&&]].contains(self.op_token().map(|x| x.kind()).unwrap_or(T![&]))
    }

    /// Whether this is a comparison operation, such as `>`, `<`, `==`, `!=`, `===`, etc.
    pub fn comparison(&self) -> bool {
        const SET: TokenSet = token_set![T![>], T![<], T![>=], T![<=], T![==], T![===], T![!=], T![!==]];
        SET.contains(self.op_token().map(|x| x.kind()).unwrap_or(T![&]))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    /// `++`
    Increment,
    /// `--`
    Decrement,
    /// `delete`
    Delete,
    /// `void`
    Void,
    /// `typeof`
    Typeof,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `~`
    BitwiseNot,
    /// `!`
    LogicalNot,
    /// `await`
    Await,
}

impl UnaryExpr {
    pub fn op_details(&self) -> Option<(SyntaxToken, UnaryOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|x| x.into_token())
            .find_map(|t| {
                let op = match t.kind() {
                    T![++] => UnaryOp::Increment,
                    T![--] => UnaryOp::Decrement,
                    T![delete] => UnaryOp::Delete,
                    T![void] => UnaryOp::Void,
                    T![typeof] => UnaryOp::Typeof,
                    T![+] => UnaryOp::Plus,
                    T![-] => UnaryOp::Minus,
                    T![~] => UnaryOp::BitwiseNot,
                    T![!] => UnaryOp::LogicalNot,
                    T![await] => UnaryOp::Await,
                    _ => return None,
                };
                Some((t, op))
            })
    }

    pub fn op(&self) -> Option<UnaryOp> {
        self.op_details().map(|t| t.1)
    }

    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op_details().map(|t| t.0)
    }
}

impl KeyValuePattern {
    pub fn value(&self) -> Option<Pattern> {
        // This is to easily handle both `NAME NAME` and `: NAME`
        if self.syntax().children().count() == 2 {
            Pattern::cast(self.syntax().last_child().unwrap())
        } else {
            self.colon_token()?
                .next_sibling_or_token()?
                .into_node()?
                .try_to::<Pattern>()
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubtractAssign,
    TimesAssign,
    RemainderAssign,
    ExponentAssign,
    LeftShiftAssign,
    RightShiftAssign,
    UnsignedRightShiftAssign,
    BitwiseAndAssign,
    BitwiseOrAssign,
    BitwiseXorAssign,
    LogicalAndAssign,
    LogicalOrAssign,
    NullishCoalescingAssign,
}

impl AssignExpr {
    pub fn op_details(&self) -> Option<(SyntaxToken, AssignOp)> {
        self.syntax()
            .children_with_tokens()
            .filter_map(|x| x.into_token())
            .find_map(|t| {
                let op = match t.kind() {
                    T![=] => AssignOp::Assign,
                    T![+=] => AssignOp::AddAssign,
                    T![-=] => AssignOp::SubtractAssign,
                    T![*=] => AssignOp::TimesAssign,
                    T![%=] => AssignOp::RemainderAssign,
                    T![**=] => AssignOp::ExponentAssign,
                    T![>>=] => AssignOp::LeftShiftAssign,
                    T![<<=] => AssignOp::RightShiftAssign,
                    T![>>>=] => AssignOp::UnsignedRightShiftAssign,
                    T![&=] => AssignOp::BitwiseAndAssign,
                    T![|=] => AssignOp::BitwiseOrAssign,
                    T![^=] => AssignOp::BitwiseXorAssign,
                    T![&&=] => AssignOp::LogicalAndAssign,
                    T![||=] => AssignOp::LogicalOrAssign,
                    T![??=] => AssignOp::NullishCoalescingAssign,
                    _ => return None,
                };
                Some((t, op))
            })
    }

    pub fn op(&self) -> Option<AssignOp> {
        self.op_details().map(|t| t.1)
    }

    pub fn op_token(&self) -> Option<SyntaxToken> {
        self.op_details().map(|t| t.0)
    }

    pub fn lhs(&self) -> Option<PatternOrExpr> {
        self.syntax.children().next().and_then(|n| n.try_to())
    }

    pub fn rhs(&self) -> Option<Expr> {
        self.syntax.children().nth(1).and_then(|n| n.try_to())
    }
}

impl ArrayExpr {
    pub fn has_trailing_comma(&self) -> bool {
        if let Some(last) = self.elements().last().map(|it| it.syntax().to_owned()) {
            if let Some(tok) = last
                .next_sibling_or_token()
                .map(|it| it.into_token())
                .flatten()
            {
                return tok.kind() == T![,];
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExprOrSpread {
    Expr(Expr),
    Spread(SpreadElement),
}

impl AstNode for ExprOrSpread {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            SPREAD_ELEMENT => true,
            _ => Expr::can_cast(kind),
        }
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if !Self::can_cast(syntax.kind()) {
            None
        } else {
            Some(if syntax.kind() == SPREAD_ELEMENT {
                ExprOrSpread::Spread(SpreadElement::cast(syntax).unwrap())
            } else {
                ExprOrSpread::Expr(Expr::cast(syntax).unwrap())
            })
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            ExprOrSpread::Expr(it) => it.syntax(),
            ExprOrSpread::Spread(it) => it.syntax(),
        }
    }
}

impl ExprOrSpread {
    pub fn is_spread(&self) -> bool {
        matches!(self, ExprOrSpread::Spread(_))
    }

    pub fn is_expr(&self) -> bool {
        matches!(self, ExprOrSpread::Expr(_))
    }
}

impl ObjectExpr {
    pub fn has_trailing_comma(&self) -> bool {
        if let Some(last) = self.props().last().map(|it| it.syntax().to_owned()) {
            if let Some(tok) = last
                .next_sibling_or_token()
                .map(|it| it.into_token())
                .flatten()
            {
                return tok.kind() == T![,];
            }
        }
        false
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LiteralKind {
    Number,
    String,
    Null,
    Bool(bool),
    Regex,
}

impl Literal {
    pub fn token(&self) -> SyntaxToken {
        self.syntax()
            .children_with_tokens()
            .find(|e| !e.kind().is_trivia())
            .and_then(|e| e.into_token())
            .unwrap()
    }

    pub fn kind(&self) -> LiteralKind {
        match self.token().kind() {
            T![null] => LiteralKind::Null,
            NUMBER => LiteralKind::Number,
            STRING => LiteralKind::String,
            TRUE_KW => LiteralKind::Bool(true),
            FALSE_KW => LiteralKind::Bool(false),
            _ => unreachable!(),
        }
    }

    pub fn is_number(&self) -> bool {
        self.kind() == LiteralKind::Number
    }
    pub fn is_string(&self) -> bool {
        self.kind() == LiteralKind::String
    }
    pub fn is_null(&self) -> bool {
        self.kind() == LiteralKind::Null
    }
    pub fn is_bool(&self) -> bool {
        matches!(self.kind(), LiteralKind::Bool(_))
    }
    pub fn is_regex(&self) -> bool {
        self.kind() == LiteralKind::Regex
    }

    /// Get the inner text of a string not including the quotes
    pub fn inner_string_text(&self) -> Option<SyntaxText> {
        if !self.is_string() {
            return None;
        }

        let start = self.syntax().text_range().start() + TextSize::from(1);
        let end_char = self
            .syntax()
            .text()
            .char_at(self.syntax().text().len() - TextSize::from(1))
            .unwrap();
        let end = if end_char == '"' || end_char == '\'' {
            self.syntax().text_range().end() - TextSize::from(1)
        } else {
            self.syntax().text_range().end()
        };

        Some(self.syntax().text().slice(start..end))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExprOrBlock {
    Expr(Expr),
    Block(BlockStmt)
}

impl AstNode for ExprOrBlock {
    fn can_cast(kind: SyntaxKind) -> bool {
        if kind == BLOCK_STMT {
            true
        } else {
            Expr::can_cast(kind)
        }
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if syntax.kind() == BLOCK_STMT {
            Some(ExprOrBlock::Block(BlockStmt::cast(syntax).unwrap()))
        } else {
            Some(ExprOrBlock::Expr(Expr::cast(syntax)?))
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            ExprOrBlock::Expr(it) => it.syntax(),
            ExprOrBlock::Block(it) => it.syntax()
        }
    }
}

impl ArrowExpr {
    pub fn body(&self) -> Option<ExprOrBlock> {
        ExprOrBlock::cast(self.syntax().children().last()?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternOrExpr {
    Pattern(Pattern),
    Expr(Expr),
}

impl AstNode for PatternOrExpr {
    fn can_cast(kind: SyntaxKind) -> bool {
        Expr::can_cast(kind) || Pattern::can_cast(kind)
    }

    fn cast(syntax: SyntaxNode) -> Option<Self> {
        Some(if Pattern::can_cast(syntax.kind()) {
            PatternOrExpr::Pattern(Pattern::cast(syntax).unwrap())
        } else {
            PatternOrExpr::Expr(Expr::cast(syntax).unwrap())
        })
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            PatternOrExpr::Pattern(it) => it.syntax(),
            PatternOrExpr::Expr(it) => it.syntax()
        }
    }
}
