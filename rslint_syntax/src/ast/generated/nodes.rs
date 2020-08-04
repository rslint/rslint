//! Generated file, do not edit by hand, see `xtask/src/codegen`

use crate::{
    ast::{support, AstChildren, AstNode, ForHead, StmtListItem},
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Program {
    pub(crate) syntax: SyntaxNode,
}
impl Program {
    pub fn items(&self) -> AstChildren<StmtListItem> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub(crate) syntax: SyntaxNode,
}
impl Literal {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockStmt {
    pub(crate) syntax: SyntaxNode,
}
impl BlockStmt {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn stmts(&self) -> AstChildren<Stmt> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarStmt {
    pub(crate) syntax: SyntaxNode,
}
impl VarStmt {
    pub fn var_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![var]) }
    pub fn declared(&self) -> AstChildren<Declarator> { support::children(&self.syntax) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Declarator {
    pub(crate) syntax: SyntaxNode,
}
impl Declarator {
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
    pub fn eq_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![=]) }
    pub fn value(&self) -> Option<AssignExpr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmptyStmt {
    pub(crate) syntax: SyntaxNode,
}
impl EmptyStmt {
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ExprStmt {
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IfStmt {
    pub(crate) syntax: SyntaxNode,
}
impl IfStmt {
    pub fn if_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![if]) }
    pub fn condition(&self) -> Option<Condition> { support::child(&self.syntax) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
    pub fn else_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![else]) }
    pub fn alt(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Condition {
    pub(crate) syntax: SyntaxNode,
}
impl Condition {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn condition(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DoWhileStmt {
    pub(crate) syntax: SyntaxNode,
}
impl DoWhileStmt {
    pub fn do_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![do]) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
    pub fn while_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![while]) }
    pub fn condition(&self) -> Option<Condition> { support::child(&self.syntax) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhileStmt {
    pub(crate) syntax: SyntaxNode,
}
impl WhileStmt {
    pub fn while_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![while]) }
    pub fn condition(&self) -> Option<Condition> { support::child(&self.syntax) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ForStmt {
    pub fn for_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![for]) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn init(&self) -> Option<ForHead> { support::child(&self.syntax) }
    pub fn test(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn update(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForInStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ForInStmt {
    pub fn for_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![for]) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn left(&self) -> Option<ForHead> { support::child(&self.syntax) }
    pub fn in_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![in]) }
    pub fn right(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContinueStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ContinueStmt {
    pub fn continue_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, T![continue])
    }
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BreakStmt {
    pub(crate) syntax: SyntaxNode,
}
impl BreakStmt {
    pub fn break_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![break]) }
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReturnStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ReturnStmt {
    pub fn return_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![return]) }
    pub fn value(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WithStmt {
    pub(crate) syntax: SyntaxNode,
}
impl WithStmt {
    pub fn with_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![with]) }
    pub fn condition(&self) -> Option<Condition> { support::child(&self.syntax) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SwitchStmt {
    pub(crate) syntax: SyntaxNode,
}
impl SwitchStmt {
    pub fn switch_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![switch]) }
    pub fn test(&self) -> Option<Condition> { support::child(&self.syntax) }
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn cases(&self) -> AstChildren<CaseClause> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseClause {
    pub(crate) syntax: SyntaxNode,
}
impl CaseClause {
    pub fn default_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![default]) }
    pub fn case_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![case]) }
    pub fn test(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
    pub fn cons(&self) -> AstChildren<Stmt> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelledStmt {
    pub(crate) syntax: SyntaxNode,
}
impl LabelledStmt {
    pub fn label(&self) -> Option<Name> { support::child(&self.syntax) }
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
    pub fn stmt(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThrowStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ThrowStmt {
    pub fn throw_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![throw]) }
    pub fn exception(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TryStmt {
    pub(crate) syntax: SyntaxNode,
}
impl TryStmt {
    pub fn try_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![try]) }
    pub fn test(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
    pub fn handler(&self) -> Option<CatchClause> { support::child(&self.syntax) }
    pub fn finally_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![finally]) }
    pub fn finalizer(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CatchClause {
    pub(crate) syntax: SyntaxNode,
}
impl CatchClause {
    pub fn catch_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![catch]) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn error(&self) -> Option<Name> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
    pub fn cons(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DebuggerStmt {
    pub(crate) syntax: SyntaxNode,
}
impl DebuggerStmt {
    pub fn debugger_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, T![debugger])
    }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnDecl {
    pub(crate) syntax: SyntaxNode,
}
impl FnDecl {
    pub fn function_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, T![function])
    }
    pub fn name(&self) -> Option<Name> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<FnBody> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub(crate) syntax: SyntaxNode,
}
impl Name {
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterList {
    pub(crate) syntax: SyntaxNode,
}
impl ParameterList {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn parameters(&self) -> AstChildren<Name> { support::children(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnBody {
    pub(crate) syntax: SyntaxNode,
}
impl FnBody {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn body(&self) -> AstChildren<StmtListItem> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThisExpr {
    pub(crate) syntax: SyntaxNode,
}
impl ThisExpr {
    pub fn this_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![this]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayExpr {
    pub(crate) syntax: SyntaxNode,
}
impl ArrayExpr {
    pub fn l_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['[']) }
    pub fn elements(&self) -> AstChildren<Expr> { support::children(&self.syntax) }
    pub fn r_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![']']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectExpr {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectExpr {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn props(&self) -> AstChildren<ObjectProp> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralProp {
    pub(crate) syntax: SyntaxNode,
}
impl LiteralProp {
    pub fn key(&self) -> Option<Literal> { support::child(&self.syntax) }
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
    pub fn value(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GetterProp {
    pub(crate) syntax: SyntaxNode,
}
impl GetterProp {
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
    pub fn key(&self) -> Option<Literal> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<FnBody> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SetterProp {
    pub(crate) syntax: SyntaxNode,
}
impl SetterProp {
    pub fn key(&self) -> Option<Literal> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<FnBody> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GroupingExpr {
    pub(crate) syntax: SyntaxNode,
}
impl GroupingExpr {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn inner(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnExpr {
    pub(crate) syntax: SyntaxNode,
}
impl FnExpr {
    pub fn function_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, T![function])
    }
    pub fn name(&self) -> Option<Name> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<FnBody> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BracketExpr {
    pub(crate) syntax: SyntaxNode,
}
impl BracketExpr {
    pub fn object(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn l_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['[']) }
    pub fn prop(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![']']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DotExpr {
    pub(crate) syntax: SyntaxNode,
}
impl DotExpr {
    pub fn object(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn dot_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![.]) }
    pub fn prop(&self) -> Option<Name> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewExpr {
    pub(crate) syntax: SyntaxNode,
}
impl NewExpr {
    pub fn new_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![new]) }
    pub fn object(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn arguments(&self) -> Option<ArgList> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArgList {
    pub(crate) syntax: SyntaxNode,
}
impl ArgList {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn args(&self) -> AstChildren<Expr> { support::children(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CallExpr {
    pub(crate) syntax: SyntaxNode,
}
impl CallExpr {
    pub fn callee(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn arguments(&self) -> Option<ArgList> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostfixExpr {
    pub(crate) syntax: SyntaxNode,
}
impl PostfixExpr {
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnaryExpr {
    pub(crate) syntax: SyntaxNode,
}
impl UnaryExpr {
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr {
    pub(crate) syntax: SyntaxNode,
}
impl BinExpr {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CondExpr {
    pub(crate) syntax: SyntaxNode,
}
impl CondExpr {
    pub fn test(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn question_mark_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![?]) }
    pub fn cons(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
    pub fn alt(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssignExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AssignExpr {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SequenceExpr {
    pub(crate) syntax: SyntaxNode,
}
impl SequenceExpr {
    pub fn exprs(&self) -> AstChildren<Expr> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectProp {
    LiteralProp(LiteralProp),
    GetterProp(GetterProp),
    SetterProp(SetterProp),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Declaration {
    FnDecl(FnDecl),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    BlockStmt(BlockStmt),
    VarStmt(VarStmt),
    EmptyStmt(EmptyStmt),
    ExprStmt(ExprStmt),
    IfStmt(IfStmt),
    DoWhileStmt(DoWhileStmt),
    WhileStmt(WhileStmt),
    ForStmt(ForStmt),
    ForInStmt(ForInStmt),
    ContinueStmt(ContinueStmt),
    BreakStmt(BreakStmt),
    ReturnStmt(ReturnStmt),
    WithStmt(WithStmt),
    LabelledStmt(LabelledStmt),
    SwitchStmt(SwitchStmt),
    ThrowStmt(ThrowStmt),
    TryStmt(TryStmt),
    DebuggerStmt(DebuggerStmt),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Name(Name),
    ThisExpr(ThisExpr),
    ArrayExpr(ArrayExpr),
    ObjectExpr(ObjectExpr),
    GroupingExpr(GroupingExpr),
    BracketExpr(BracketExpr),
    DotExpr(DotExpr),
    NewExpr(NewExpr),
    CallExpr(CallExpr),
    PostfixExpr(PostfixExpr),
    UnaryExpr(UnaryExpr),
    BinExpr(BinExpr),
    CondExpr(CondExpr),
    AssignExpr(AssignExpr),
    SequenceExpr(SequenceExpr),
}
impl AstNode for Program {
    fn can_cast(kind: SyntaxKind) -> bool { kind == PROGRAM }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Literal {
    fn can_cast(kind: SyntaxKind) -> bool { kind == LITERAL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for BlockStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == BLOCK_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for VarStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == VAR_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Declarator {
    fn can_cast(kind: SyntaxKind) -> bool { kind == DECLARATOR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for EmptyStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EMPTY_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ExprStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EXPR_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for IfStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == IF_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Condition {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CONDITION }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for DoWhileStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == DO_WHILE_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for WhileStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == WHILE_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ForStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FOR_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ForInStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FOR_IN_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ContinueStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CONTINUE_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for BreakStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == BREAK_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ReturnStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == RETURN_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for WithStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == WITH_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for SwitchStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SWITCH_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for CaseClause {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CASE_CLAUSE }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for LabelledStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == LABELLED_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ThrowStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == THROW_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for TryStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == TRY_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for CatchClause {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CATCH_CLAUSE }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for DebuggerStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == DEBUGGER_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for FnDecl {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FN_DECL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Name {
    fn can_cast(kind: SyntaxKind) -> bool { kind == NAME }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ParameterList {
    fn can_cast(kind: SyntaxKind) -> bool { kind == PARAMETER_LIST }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for FnBody {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FN_BODY }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ThisExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == THIS_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ArrayExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == ARRAY_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ObjectExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == OBJECT_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for LiteralProp {
    fn can_cast(kind: SyntaxKind) -> bool { kind == LITERAL_PROP }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for GetterProp {
    fn can_cast(kind: SyntaxKind) -> bool { kind == GETTER_PROP }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for SetterProp {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SETTER_PROP }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for GroupingExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == GROUPING_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for FnExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FN_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for BracketExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == BRACKET_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for DotExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == DOT_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for NewExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == NEW_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ArgList {
    fn can_cast(kind: SyntaxKind) -> bool { kind == ARG_LIST }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for CallExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CALL_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for PostfixExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == POSTFIX_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for UnaryExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == UNARY_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for BinExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == BIN_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for CondExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == COND_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for AssignExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == ASSIGN_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for SequenceExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SEQUENCE_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl From<LiteralProp> for ObjectProp {
    fn from(node: LiteralProp) -> ObjectProp { ObjectProp::LiteralProp(node) }
}
impl From<GetterProp> for ObjectProp {
    fn from(node: GetterProp) -> ObjectProp { ObjectProp::GetterProp(node) }
}
impl From<SetterProp> for ObjectProp {
    fn from(node: SetterProp) -> ObjectProp { ObjectProp::SetterProp(node) }
}
impl AstNode for ObjectProp {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            LITERAL_PROP | GETTER_PROP | SETTER_PROP => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            LITERAL_PROP => ObjectProp::LiteralProp(LiteralProp { syntax }),
            GETTER_PROP => ObjectProp::GetterProp(GetterProp { syntax }),
            SETTER_PROP => ObjectProp::SetterProp(SetterProp { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ObjectProp::LiteralProp(it) => &it.syntax,
            ObjectProp::GetterProp(it) => &it.syntax,
            ObjectProp::SetterProp(it) => &it.syntax,
        }
    }
}
impl From<FnDecl> for Declaration {
    fn from(node: FnDecl) -> Declaration { Declaration::FnDecl(node) }
}
impl AstNode for Declaration {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            FN_DECL => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            FN_DECL => Declaration::FnDecl(FnDecl { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Declaration::FnDecl(it) => &it.syntax,
        }
    }
}
impl From<BlockStmt> for Stmt {
    fn from(node: BlockStmt) -> Stmt { Stmt::BlockStmt(node) }
}
impl From<VarStmt> for Stmt {
    fn from(node: VarStmt) -> Stmt { Stmt::VarStmt(node) }
}
impl From<EmptyStmt> for Stmt {
    fn from(node: EmptyStmt) -> Stmt { Stmt::EmptyStmt(node) }
}
impl From<ExprStmt> for Stmt {
    fn from(node: ExprStmt) -> Stmt { Stmt::ExprStmt(node) }
}
impl From<IfStmt> for Stmt {
    fn from(node: IfStmt) -> Stmt { Stmt::IfStmt(node) }
}
impl From<DoWhileStmt> for Stmt {
    fn from(node: DoWhileStmt) -> Stmt { Stmt::DoWhileStmt(node) }
}
impl From<WhileStmt> for Stmt {
    fn from(node: WhileStmt) -> Stmt { Stmt::WhileStmt(node) }
}
impl From<ForStmt> for Stmt {
    fn from(node: ForStmt) -> Stmt { Stmt::ForStmt(node) }
}
impl From<ForInStmt> for Stmt {
    fn from(node: ForInStmt) -> Stmt { Stmt::ForInStmt(node) }
}
impl From<ContinueStmt> for Stmt {
    fn from(node: ContinueStmt) -> Stmt { Stmt::ContinueStmt(node) }
}
impl From<BreakStmt> for Stmt {
    fn from(node: BreakStmt) -> Stmt { Stmt::BreakStmt(node) }
}
impl From<ReturnStmt> for Stmt {
    fn from(node: ReturnStmt) -> Stmt { Stmt::ReturnStmt(node) }
}
impl From<WithStmt> for Stmt {
    fn from(node: WithStmt) -> Stmt { Stmt::WithStmt(node) }
}
impl From<LabelledStmt> for Stmt {
    fn from(node: LabelledStmt) -> Stmt { Stmt::LabelledStmt(node) }
}
impl From<SwitchStmt> for Stmt {
    fn from(node: SwitchStmt) -> Stmt { Stmt::SwitchStmt(node) }
}
impl From<ThrowStmt> for Stmt {
    fn from(node: ThrowStmt) -> Stmt { Stmt::ThrowStmt(node) }
}
impl From<TryStmt> for Stmt {
    fn from(node: TryStmt) -> Stmt { Stmt::TryStmt(node) }
}
impl From<DebuggerStmt> for Stmt {
    fn from(node: DebuggerStmt) -> Stmt { Stmt::DebuggerStmt(node) }
}
impl AstNode for Stmt {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            BLOCK_STMT | VAR_STMT | EMPTY_STMT | EXPR_STMT | IF_STMT | DO_WHILE_STMT
            | WHILE_STMT | FOR_STMT | FOR_IN_STMT | CONTINUE_STMT | BREAK_STMT | RETURN_STMT
            | WITH_STMT | LABELLED_STMT | SWITCH_STMT | THROW_STMT | TRY_STMT | DEBUGGER_STMT => {
                true
            }
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            BLOCK_STMT => Stmt::BlockStmt(BlockStmt { syntax }),
            VAR_STMT => Stmt::VarStmt(VarStmt { syntax }),
            EMPTY_STMT => Stmt::EmptyStmt(EmptyStmt { syntax }),
            EXPR_STMT => Stmt::ExprStmt(ExprStmt { syntax }),
            IF_STMT => Stmt::IfStmt(IfStmt { syntax }),
            DO_WHILE_STMT => Stmt::DoWhileStmt(DoWhileStmt { syntax }),
            WHILE_STMT => Stmt::WhileStmt(WhileStmt { syntax }),
            FOR_STMT => Stmt::ForStmt(ForStmt { syntax }),
            FOR_IN_STMT => Stmt::ForInStmt(ForInStmt { syntax }),
            CONTINUE_STMT => Stmt::ContinueStmt(ContinueStmt { syntax }),
            BREAK_STMT => Stmt::BreakStmt(BreakStmt { syntax }),
            RETURN_STMT => Stmt::ReturnStmt(ReturnStmt { syntax }),
            WITH_STMT => Stmt::WithStmt(WithStmt { syntax }),
            LABELLED_STMT => Stmt::LabelledStmt(LabelledStmt { syntax }),
            SWITCH_STMT => Stmt::SwitchStmt(SwitchStmt { syntax }),
            THROW_STMT => Stmt::ThrowStmt(ThrowStmt { syntax }),
            TRY_STMT => Stmt::TryStmt(TryStmt { syntax }),
            DEBUGGER_STMT => Stmt::DebuggerStmt(DebuggerStmt { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Stmt::BlockStmt(it) => &it.syntax,
            Stmt::VarStmt(it) => &it.syntax,
            Stmt::EmptyStmt(it) => &it.syntax,
            Stmt::ExprStmt(it) => &it.syntax,
            Stmt::IfStmt(it) => &it.syntax,
            Stmt::DoWhileStmt(it) => &it.syntax,
            Stmt::WhileStmt(it) => &it.syntax,
            Stmt::ForStmt(it) => &it.syntax,
            Stmt::ForInStmt(it) => &it.syntax,
            Stmt::ContinueStmt(it) => &it.syntax,
            Stmt::BreakStmt(it) => &it.syntax,
            Stmt::ReturnStmt(it) => &it.syntax,
            Stmt::WithStmt(it) => &it.syntax,
            Stmt::LabelledStmt(it) => &it.syntax,
            Stmt::SwitchStmt(it) => &it.syntax,
            Stmt::ThrowStmt(it) => &it.syntax,
            Stmt::TryStmt(it) => &it.syntax,
            Stmt::DebuggerStmt(it) => &it.syntax,
        }
    }
}
impl From<Name> for Expr {
    fn from(node: Name) -> Expr { Expr::Name(node) }
}
impl From<ThisExpr> for Expr {
    fn from(node: ThisExpr) -> Expr { Expr::ThisExpr(node) }
}
impl From<ArrayExpr> for Expr {
    fn from(node: ArrayExpr) -> Expr { Expr::ArrayExpr(node) }
}
impl From<ObjectExpr> for Expr {
    fn from(node: ObjectExpr) -> Expr { Expr::ObjectExpr(node) }
}
impl From<GroupingExpr> for Expr {
    fn from(node: GroupingExpr) -> Expr { Expr::GroupingExpr(node) }
}
impl From<BracketExpr> for Expr {
    fn from(node: BracketExpr) -> Expr { Expr::BracketExpr(node) }
}
impl From<DotExpr> for Expr {
    fn from(node: DotExpr) -> Expr { Expr::DotExpr(node) }
}
impl From<NewExpr> for Expr {
    fn from(node: NewExpr) -> Expr { Expr::NewExpr(node) }
}
impl From<CallExpr> for Expr {
    fn from(node: CallExpr) -> Expr { Expr::CallExpr(node) }
}
impl From<PostfixExpr> for Expr {
    fn from(node: PostfixExpr) -> Expr { Expr::PostfixExpr(node) }
}
impl From<UnaryExpr> for Expr {
    fn from(node: UnaryExpr) -> Expr { Expr::UnaryExpr(node) }
}
impl From<BinExpr> for Expr {
    fn from(node: BinExpr) -> Expr { Expr::BinExpr(node) }
}
impl From<CondExpr> for Expr {
    fn from(node: CondExpr) -> Expr { Expr::CondExpr(node) }
}
impl From<AssignExpr> for Expr {
    fn from(node: AssignExpr) -> Expr { Expr::AssignExpr(node) }
}
impl From<SequenceExpr> for Expr {
    fn from(node: SequenceExpr) -> Expr { Expr::SequenceExpr(node) }
}
impl AstNode for Expr {
    fn can_cast(kind: SyntaxKind) -> bool {
        match kind {
            NAME | THIS_EXPR | ARRAY_EXPR | OBJECT_EXPR | GROUPING_EXPR | BRACKET_EXPR
            | DOT_EXPR | NEW_EXPR | CALL_EXPR | POSTFIX_EXPR | UNARY_EXPR | BIN_EXPR
            | COND_EXPR | ASSIGN_EXPR | SEQUENCE_EXPR => true,
            _ => false,
        }
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            NAME => Expr::Name(Name { syntax }),
            THIS_EXPR => Expr::ThisExpr(ThisExpr { syntax }),
            ARRAY_EXPR => Expr::ArrayExpr(ArrayExpr { syntax }),
            OBJECT_EXPR => Expr::ObjectExpr(ObjectExpr { syntax }),
            GROUPING_EXPR => Expr::GroupingExpr(GroupingExpr { syntax }),
            BRACKET_EXPR => Expr::BracketExpr(BracketExpr { syntax }),
            DOT_EXPR => Expr::DotExpr(DotExpr { syntax }),
            NEW_EXPR => Expr::NewExpr(NewExpr { syntax }),
            CALL_EXPR => Expr::CallExpr(CallExpr { syntax }),
            POSTFIX_EXPR => Expr::PostfixExpr(PostfixExpr { syntax }),
            UNARY_EXPR => Expr::UnaryExpr(UnaryExpr { syntax }),
            BIN_EXPR => Expr::BinExpr(BinExpr { syntax }),
            COND_EXPR => Expr::CondExpr(CondExpr { syntax }),
            ASSIGN_EXPR => Expr::AssignExpr(AssignExpr { syntax }),
            SEQUENCE_EXPR => Expr::SequenceExpr(SequenceExpr { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::Name(it) => &it.syntax,
            Expr::ThisExpr(it) => &it.syntax,
            Expr::ArrayExpr(it) => &it.syntax,
            Expr::ObjectExpr(it) => &it.syntax,
            Expr::GroupingExpr(it) => &it.syntax,
            Expr::BracketExpr(it) => &it.syntax,
            Expr::DotExpr(it) => &it.syntax,
            Expr::NewExpr(it) => &it.syntax,
            Expr::CallExpr(it) => &it.syntax,
            Expr::PostfixExpr(it) => &it.syntax,
            Expr::UnaryExpr(it) => &it.syntax,
            Expr::BinExpr(it) => &it.syntax,
            Expr::CondExpr(it) => &it.syntax,
            Expr::AssignExpr(it) => &it.syntax,
            Expr::SequenceExpr(it) => &it.syntax,
        }
    }
}
impl std::fmt::Display for ObjectProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BlockStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for VarStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Declarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for EmptyStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ExprStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IfStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for DoWhileStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for WhileStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ForStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ForInStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ContinueStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BreakStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ReturnStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for WithStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for SwitchStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for CaseClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LabelledStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ThrowStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TryStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for CatchClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for DebuggerStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for FnDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ParameterList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for FnBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ThisExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArrayExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ObjectExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LiteralProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for GetterProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for SetterProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for GroupingExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for FnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BracketExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for DotExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for NewExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArgList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for CallExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for PostfixExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for UnaryExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BinExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for CondExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AssignExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for SequenceExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
