use swc_ecma_visit_macros::define;
use std::any::Any;
use rslint_parse::parser::cst::{declaration::*, expr::*, stmt::*, *};
use rslint_parse::lexer::token::TokenType;
use rslint_parse::span::Span;

pub trait Node: Any {
    fn as_any(&self) -> &dyn Any;
}

impl<T> Node for T where T: Any {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

define!({
    pub struct CST {
        pub statements: Vec<StmtListItem>,
        pub shebang: Option<Span>,
        pub eof_whitespace: Span,
    }

    pub enum Stmt {
        Variable(VarStmt),
        Empty(EmptyStmt),
        Block(BlockStmt),
        Expr(ExprStmt),
        If(IfStmt),
        Switch(SwitchStmt),
        Throw(ThrowStmt),
        While(WhileStmt),
        DoWhile(DoWhileStmt),
        Labelled(LabelledStmt),
        Break(BreakStmt),
        Continue(ContinueStmt),
        Return(ReturnStmt),
        Try(TryStmt),
        For(ForStmt),
        ForIn(ForInStmt),
        With(WithStmt),
    }

    pub enum StmtListItem {
        Declaration(Declaration),
        Stmt(Stmt),
    }

    pub enum Semicolon {
        Implicit,
        Explicit(LiteralWhitespace),
    }
    
    pub struct Declarator {
        pub span: Span,
        pub name: LiteralExpr,
        pub value: Option<Expr>,
        pub initializer_whitespace: Option<LiteralWhitespace>,
    }
    
    pub struct VarStmt {
        pub span: Span,
        pub declared: Vec<Declarator>,
        pub comma_whitespaces: Vec<LiteralWhitespace>,
        pub var_whitespace: LiteralWhitespace,
        pub semi: Semicolon,
    }
    
    pub struct BlockStmt {
        pub span: Span,
        pub stmts: Vec<Stmt>,
        pub open_brace_whitespace: LiteralWhitespace,
        pub close_brace_whitespace: LiteralWhitespace,
    }
    
    pub struct EmptyStmt {
        pub span: Span,
        pub semi_whitespace: LiteralWhitespace,
    }
    
    pub struct ExprStmt {
        pub span: Span,
        pub expr: Expr,
        pub semi: Semicolon,
    }
    
    pub struct IfStmt {
        pub span: Span,
        pub if_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub condition: Expr,
        pub cons: Box<Stmt>,
        pub else_whitespace: Option<LiteralWhitespace>,
        pub alt: Option<Box<Stmt>>,
    }
    
    pub struct Case {
        pub span: Span,
        pub default: bool,
        pub whitespace: LiteralWhitespace,
        pub colon_whitespace: LiteralWhitespace,
        pub test: Option<Expr>,
        pub cons: Vec<Stmt>
    }
    
    pub struct SwitchStmt {
        pub span: Span,
        pub switch_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub test: Expr,
        pub open_brace_whitespace: LiteralWhitespace,
        pub close_brace_whitespace: LiteralWhitespace,
        pub cases: Vec<Case>,
    }
    
    pub struct ThrowStmt {
        pub span: Span,
        pub arg: Expr,
        pub semi: Semicolon,
        pub throw_whitespace: LiteralWhitespace,
    }
    
    pub struct WhileStmt {
        pub span: Span,
        pub while_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub condition: Expr,
        pub cons: Box<Stmt>,
    }
    
    pub struct DoWhileStmt {
        pub span: Span,
        pub do_whitespace: LiteralWhitespace,
        pub while_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub condition: Expr,
        pub cons: Box<Stmt>,
    }
    
    pub struct LabelledStmt {
        pub span: Span,
        pub label: LiteralExpr,
        pub colon_whitespace: LiteralWhitespace,
        pub body: Box<Stmt>,
    }
    
    pub struct BreakStmt {
        pub span: Span,
        pub break_whitespace: LiteralWhitespace,
        pub label: Option<LiteralExpr>,
        pub semi: Semicolon,
    }
    
    pub struct ContinueStmt {
        pub span: Span,
        pub continue_whitespace: LiteralWhitespace,
        pub label: Option<LiteralExpr>,
        pub semi: Semicolon,
    }
    
    pub struct ReturnStmt {
        pub span: Span,
        pub return_whitespace: LiteralWhitespace,
        pub value: Option<Expr>,
        pub semi: Semicolon,
    }
    
    pub struct CatchClause {
        pub span: Span,
        pub catch_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub param: LiteralExpr,
        pub body: BlockStmt,
    }
    
    pub struct TryStmt {
        pub span: Span,
        pub try_whitespace: LiteralWhitespace,
        pub test: BlockStmt,
        pub handler: Option<CatchClause>,
        pub finalizer: Option<BlockStmt>,
        pub final_whitespace: Option<LiteralWhitespace>,
    }
    
    pub enum ForStmtInit {
        Expr(Expr),
        Var(VarStmt),
    }
    
    pub struct ForStmt {
        pub span: Span,
        pub for_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub init: Option<ForStmtInit>,
        pub test: Option<Expr>,
        pub update: Option<Expr>,
        pub body: Box<Stmt>,
        pub init_semicolon_whitespace: LiteralWhitespace,
        pub test_semicolon_whitespace: LiteralWhitespace,
    }
    
    pub struct ForInStmt {
        pub span: Span,
        pub for_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub left: ForStmtInit,
        pub right: Expr,
        pub in_whitespace: LiteralWhitespace,
        pub body: Box<Stmt>,
    }
    
    pub struct WithStmt {
        pub span: Span,
        pub with_whitespace: LiteralWhitespace,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub object: Expr,
        pub body: Box<Stmt>,
    }
    
    
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
        Object(ObjectExpr),
        Function(FunctionDecl),
    }

    pub struct ArrayExpr {
        pub span: Span,
        pub exprs: Vec<Option<Expr>>,
        pub comma_whitespaces: Vec<LiteralWhitespace>,
        pub opening_bracket_whitespace: LiteralWhitespace,
        pub closing_bracket_whitespace: LiteralWhitespace,
    }

    pub struct GroupingExpr {
        pub span: Span,
        pub expr: Box<Expr>,
        pub opening_paren_whitespace: LiteralWhitespace,
        pub closing_paren_whitespace: LiteralWhitespace,
    }

    pub struct BracketExpr {
        pub span: Span,
        pub object: Box<Expr>,
        pub property: Box<Expr>,
        pub opening_bracket_whitespace: LiteralWhitespace,
        pub closing_bracket_whitespace: LiteralWhitespace,
    }

    pub struct CallExpr {
        pub span: Span,
        pub callee: Box<Expr>,
        pub arguments: Arguments,
    }

    pub struct SequenceExpr {
        pub span: Span,
        pub exprs: Vec<Expr>,
        pub comma_whitespace: Vec<LiteralWhitespace>,
    }

    pub struct AssignmentExpr {
        pub span: Span,
        pub left: Box<Expr>,
        pub right: Box<Expr>,
        pub op: TokenType,
        pub whitespace: LiteralWhitespace
    }

    pub struct ConditionalExpr {
        pub span: Span,
        pub condition: Box<Expr>,
        pub if_false: Box<Expr>,
        pub if_true: Box<Expr>,
        pub whitespace: ConditionalWhitespace,
    }

    pub struct ConditionalWhitespace {
        pub before_qmark: Span,
        pub after_qmark: Span,
        pub before_colon: Span,
        pub after_colon: Span,
    }

    pub struct BinaryExpr {
        pub span: Span,
        pub left: Box<Expr>,
        pub right: Box<Expr>,
        pub op: TokenType,
        pub whitespace: LiteralWhitespace,
    }

    pub struct UpdateExpr {
        pub span: Span,
        pub prefix: bool,
        pub object: Box<Expr>,
        pub op: TokenType,
        pub whitespace: LiteralWhitespace,
    }

    pub struct UnaryExpr {
        pub span: Span,
        pub object: Box<Expr>,
        pub op: TokenType,
        pub whitespace: LiteralWhitespace,
    }

    pub struct MemberExpr {
        pub span: Span,
        pub object: Box<Expr>,
        pub property: Box<Expr>,
        pub whitespace: LiteralWhitespace,
    }

    pub struct NewExpr {
        pub span: Span,
        pub target: Box<Expr>,
        pub args: Option<Arguments>,
        pub whitespace: LiteralWhitespace,
    }

    pub struct Arguments {
        pub span: Span,
        pub arguments: Vec<Expr>,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub comma_whitespaces: Vec<LiteralWhitespace>,
    }

    pub struct LiteralExpr {
        pub span: Span,
        pub whitespace: LiteralWhitespace,
    }

    pub struct LiteralWhitespace {
        pub before: Span,
        pub after: Span,
    }

    pub struct ObjectExpr {
        pub span: Span,
        pub props: Vec<ObjProp>,
        pub comma_whitespaces: Vec<LiteralWhitespace>,
        pub open_brace_whitespace: LiteralWhitespace,
        pub close_brace_whitespace: LiteralWhitespace,
    }

    pub struct LiteralObjProp {
        pub span: Span,
        pub key: Box<Expr>,
        pub value: Box<Expr>,
        pub whitespace: LiteralWhitespace,
    }

    pub struct ComputedObjProp {
        pub span: Span,
        pub identifier_whitespace: LiteralWhitespace,
        pub key: Box<Expr>,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
        pub argument: Option<LiteralExpr>,
        pub open_brace_whitespace: LiteralWhitespace,
        pub close_brace_whitespace: LiteralWhitespace,
        pub body: Vec<StmtListItem>,
    }

    pub enum ObjProp {
        Literal(LiteralObjProp),
        Getter(ComputedObjProp),
        Setter(ComputedObjProp),
    }

    pub enum Declaration {
        Function(FunctionDecl),
    }

    pub struct FunctionDecl {
        pub span: Span,
        pub function_whitespace: LiteralWhitespace,
        pub name: Option<LiteralExpr>,
        pub parameters: Parameters,
        pub open_brace_whitespace: LiteralWhitespace,
        pub close_brace_whitespace: LiteralWhitespace,
        pub body: Vec<StmtListItem>,
    }

    pub struct Parameters {
        pub span: Span,
        pub parameters: Vec<LiteralExpr>,
        pub comma_whitespaces: Vec<LiteralWhitespace>,
        pub open_paren_whitespace: LiteralWhitespace,
        pub close_paren_whitespace: LiteralWhitespace,
    }
});