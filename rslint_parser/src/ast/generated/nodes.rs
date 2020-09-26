//! Generated file, do not edit by hand, see `xtask/src/codegen`

use crate::{
    ast::*,
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, T,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Script {
    pub(crate) syntax: SyntaxNode,
}
impl Script {
    pub fn items(&self) -> AstChildren<Stmt> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Module {
    pub(crate) syntax: SyntaxNode,
}
impl Module {
    pub fn items(&self) -> AstChildren<ModuleItem> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportDecl {
    pub(crate) syntax: SyntaxNode,
}
impl ImportDecl {
    pub fn import_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![import]) }
    pub fn imports(&self) -> AstChildren<ImportClause> { support::children(&self.syntax) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WildcardImport {
    pub(crate) syntax: SyntaxNode,
}
impl WildcardImport {
    pub fn star_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![*]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedImports {
    pub(crate) syntax: SyntaxNode,
}
impl NamedImports {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn specifiers(&self) -> AstChildren<Specifier> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Specifier {
    pub(crate) syntax: SyntaxNode,
}
impl Specifier {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportDecl {
    pub(crate) syntax: SyntaxNode,
}
impl ExportDecl {
    pub fn export_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![export]) }
    pub fn decl(&self) -> Option<Decl> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportNamed {
    pub(crate) syntax: SyntaxNode,
}
impl ExportNamed {
    pub fn export_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![export]) }
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn specifiers(&self) -> AstChildren<Specifier> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportWildcard {
    pub(crate) syntax: SyntaxNode,
}
impl ExportWildcard {
    pub fn export_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![export]) }
    pub fn star_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![*]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportDefaultDecl {
    pub(crate) syntax: SyntaxNode,
}
impl ExportDefaultDecl {
    pub fn export_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![export]) }
    pub fn default_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![default]) }
    pub fn decl(&self) -> Option<DefaultDecl> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExportDefaultExpr {
    pub(crate) syntax: SyntaxNode,
}
impl ExportDefaultExpr {
    pub fn export_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![export]) }
    pub fn default_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![default]) }
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
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
pub struct VarDecl {
    pub(crate) syntax: SyntaxNode,
}
impl VarDecl {
    pub fn var_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![var]) }
    pub fn const_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![const]) }
    pub fn declared(&self) -> AstChildren<Declarator> { support::children(&self.syntax) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Declarator {
    pub(crate) syntax: SyntaxNode,
}
impl Declarator {
    pub fn pattern(&self) -> Option<Pattern> { support::child(&self.syntax) }
    pub fn eq_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![=]) }
    pub fn value(&self) -> Option<Expr> { support::child(&self.syntax) }
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
    pub fn else_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![else]) }
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
    pub fn init(&self) -> Option<ForStmtInit> { support::child(&self.syntax) }
    pub fn test(&self) -> Option<ForStmtTest> { support::child(&self.syntax) }
    pub fn update(&self) -> Option<ForStmtUpdate> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForStmtInit {
    pub(crate) syntax: SyntaxNode,
}
impl ForStmtInit {
    pub fn inner(&self) -> Option<ForHead> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForStmtTest {
    pub(crate) syntax: SyntaxNode,
}
impl ForStmtTest {
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForStmtUpdate {
    pub(crate) syntax: SyntaxNode,
}
impl ForStmtUpdate {
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForInStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ForInStmt {
    pub fn for_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![for]) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn left(&self) -> Option<ForStmtInit> { support::child(&self.syntax) }
    pub fn in_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![in]) }
    pub fn right(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
    pub fn cons(&self) -> Option<Stmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ForOfStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ForOfStmt {
    pub fn for_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![for]) }
    pub fn await_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![await]) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn left(&self) -> Option<ForStmtInit> { support::child(&self.syntax) }
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
    pub fn cases(&self) -> AstChildren<SwitchCase> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CaseClause {
    pub(crate) syntax: SyntaxNode,
}
impl CaseClause {
    pub fn case_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![case]) }
    pub fn test(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
    pub fn cons(&self) -> AstChildren<Stmt> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultClause {
    pub(crate) syntax: SyntaxNode,
}
impl DefaultClause {
    pub fn default_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![default]) }
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
    pub fn finalizer(&self) -> Option<Finalizer> { support::child(&self.syntax) }
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
pub struct Finalizer {
    pub(crate) syntax: SyntaxNode,
}
impl Finalizer {
    pub fn finally_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![finally]) }
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
    pub fn star_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![*]) }
    pub fn name(&self) -> Option<Name> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub(crate) syntax: SyntaxNode,
}
impl Name {
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NameRef {
    pub(crate) syntax: SyntaxNode,
}
impl NameRef {
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParameterList {
    pub(crate) syntax: SyntaxNode,
}
impl ParameterList {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn parameters(&self) -> AstChildren<Pattern> { support::children(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
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
    pub fn elements(&self) -> AstChildren<ExprOrSpread> { support::children(&self.syntax) }
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
pub struct SpreadProp {
    pub(crate) syntax: SyntaxNode,
}
impl SpreadProp {
    pub fn dotdotdot_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![...]) }
    pub fn value(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InitializedProp {
    pub(crate) syntax: SyntaxNode,
}
impl InitializedProp {
    pub fn key(&self) -> Option<Name> { support::child(&self.syntax) }
    pub fn eq_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![=]) }
    pub fn value(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentProp {
    pub(crate) syntax: SyntaxNode,
}
impl IdentProp {
    pub fn name(&self) -> Option<Name> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LiteralProp {
    pub(crate) syntax: SyntaxNode,
}
impl LiteralProp {
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Getter {
    pub(crate) syntax: SyntaxNode,
}
impl Getter {
    pub fn key(&self) -> Option<PropName> { support::child(&self.syntax) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
    pub fn body(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Setter {
    pub(crate) syntax: SyntaxNode,
}
impl Setter {
    pub fn key(&self) -> Option<PropName> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
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
pub struct BracketExpr {
    pub(crate) syntax: SyntaxNode,
}
impl BracketExpr {
    pub fn super_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![super]) }
    pub fn l_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['[']) }
    pub fn r_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![']']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DotExpr {
    pub(crate) syntax: SyntaxNode,
}
impl DotExpr {
    pub fn super_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![super]) }
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
pub struct SuperCall {
    pub(crate) syntax: SyntaxNode,
}
impl SuperCall {
    pub fn super_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![super]) }
    pub fn arguments(&self) -> Option<ArgList> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportCall {
    pub(crate) syntax: SyntaxNode,
}
impl ImportCall {
    pub fn import_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![import]) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn argument(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NewTarget {
    pub(crate) syntax: SyntaxNode,
}
impl NewTarget {
    pub fn new_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![new]) }
    pub fn dot_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![.]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportMeta {
    pub(crate) syntax: SyntaxNode,
}
impl ImportMeta {
    pub fn import_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![import]) }
    pub fn dot_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![.]) }
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
    pub fn question_mark_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![?]) }
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
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
pub struct Template {
    pub(crate) syntax: SyntaxNode,
}
impl Template {
    pub fn tag(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn elements(&self) -> AstChildren<TemplateElement> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TemplateElement {
    pub(crate) syntax: SyntaxNode,
}
impl TemplateElement {
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpreadElement {
    pub(crate) syntax: SyntaxNode,
}
impl SpreadElement {
    pub fn dotdotdot_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![...]) }
    pub fn element(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrayPattern {
    pub(crate) syntax: SyntaxNode,
}
impl ArrayPattern {
    pub fn l_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['[']) }
    pub fn elements(&self) -> AstChildren<Pattern> { support::children(&self.syntax) }
    pub fn r_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![']']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectPattern {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectPattern {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn elements(&self) -> AstChildren<ObjectPatternProp> { support::children(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RestPattern {
    pub(crate) syntax: SyntaxNode,
}
impl RestPattern {
    pub fn dotdotdot_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![...]) }
    pub fn pat(&self) -> Option<Pattern> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssignPattern {
    pub(crate) syntax: SyntaxNode,
}
impl AssignPattern {
    pub fn key(&self) -> Option<Pattern> { support::child(&self.syntax) }
    pub fn eq_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![=]) }
    pub fn value(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyValuePattern {
    pub(crate) syntax: SyntaxNode,
}
impl KeyValuePattern {
    pub fn key(&self) -> Option<PropName> { support::child(&self.syntax) }
    pub fn colon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![:]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ComputedPropertyName {
    pub(crate) syntax: SyntaxNode,
}
impl ComputedPropertyName {
    pub fn l_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['[']) }
    pub fn prop(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn r_brack_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![']']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SinglePattern {
    pub(crate) syntax: SyntaxNode,
}
impl SinglePattern {
    pub fn name(&self) -> Option<Name> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArrowExpr {
    pub(crate) syntax: SyntaxNode,
}
impl ArrowExpr {
    pub fn params(&self) -> Option<ArrowExprParams> { support::child(&self.syntax) }
    pub fn fat_arrow_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![=>]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct YieldExpr {
    pub(crate) syntax: SyntaxNode,
}
impl YieldExpr {
    pub fn yield_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![yield]) }
    pub fn star_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![*]) }
    pub fn value(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FnExpr {
    pub(crate) syntax: SyntaxNode,
}
impl FnExpr {
    pub fn function_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, T![function])
    }
    pub fn star_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![*]) }
    pub fn name(&self) -> Option<Name> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Method {
    pub(crate) syntax: SyntaxNode,
}
impl Method {
    pub fn star_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![*]) }
    pub fn name(&self) -> Option<PropName> { support::child(&self.syntax) }
    pub fn parameters(&self) -> Option<ParameterList> { support::child(&self.syntax) }
    pub fn body(&self) -> Option<BlockStmt> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StaticMethod {
    pub(crate) syntax: SyntaxNode,
}
impl StaticMethod {
    pub fn static_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![static]) }
    pub fn method(&self) -> Option<Method> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassDecl {
    pub(crate) syntax: SyntaxNode,
}
impl ClassDecl {
    pub fn class_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![class]) }
    pub fn extends_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![extends]) }
    pub fn body(&self) -> Option<ClassBody> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassExpr {
    pub(crate) syntax: SyntaxNode,
}
impl ClassExpr {
    pub fn class_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![class]) }
    pub fn extends_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![extends]) }
    pub fn body(&self) -> Option<ClassBody> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ClassBody {
    pub(crate) syntax: SyntaxNode,
}
impl ClassBody {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn elements(&self) -> Option<ClassElement> { support::child(&self.syntax) }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AwaitExpr {
    pub(crate) syntax: SyntaxNode,
}
impl AwaitExpr {
    pub fn await_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![await]) }
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectProp {
    LiteralProp(LiteralProp),
    Getter(Getter),
    Setter(Setter),
    SpreadProp(SpreadProp),
    InitializedProp(InitializedProp),
    IdentProp(IdentProp),
    Method(Method),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Pattern {
    SinglePattern(SinglePattern),
    RestPattern(RestPattern),
    AssignPattern(AssignPattern),
    ObjectPattern(ObjectPattern),
    ArrayPattern(ArrayPattern),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SwitchCase {
    CaseClause(CaseClause),
    DefaultClause(DefaultClause),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ObjectPatternProp {
    AssignPattern(AssignPattern),
    KeyValuePattern(KeyValuePattern),
    RestPattern(RestPattern),
    SinglePattern(SinglePattern),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ArrowExprParams {
    Name(Name),
    ParameterList(ParameterList),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MethodDefinition {
    Method(Method),
    Getter(Getter),
    Setter(Setter),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClassElement {
    EmptyStmt(EmptyStmt),
    Method(Method),
    StaticMethod(StaticMethod),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ImportClause {
    WildcardImport(WildcardImport),
    NamedImports(NamedImports),
    Name(Name),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DefaultDecl {
    FnDecl(FnDecl),
    ClassDecl(ClassDecl),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decl {
    FnDecl(FnDecl),
    ClassDecl(ClassDecl),
    VarDecl(VarDecl),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    ArrowExpr(ArrowExpr),
    Literal(Literal),
    Template(Template),
    NameRef(NameRef),
    ThisExpr(ThisExpr),
    ArrayExpr(ArrayExpr),
    ObjectExpr(ObjectExpr),
    GroupingExpr(GroupingExpr),
    BracketExpr(BracketExpr),
    DotExpr(DotExpr),
    NewExpr(NewExpr),
    CallExpr(CallExpr),
    UnaryExpr(UnaryExpr),
    BinExpr(BinExpr),
    CondExpr(CondExpr),
    AssignExpr(AssignExpr),
    SequenceExpr(SequenceExpr),
    FnExpr(FnExpr),
    ClassExpr(ClassExpr),
    NewTarget(NewTarget),
    ImportMeta(ImportMeta),
    SuperCall(SuperCall),
    ImportCall(ImportCall),
    YieldExpr(YieldExpr),
    AwaitExpr(AwaitExpr),
}
impl AstNode for Script {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SCRIPT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Module {
    fn can_cast(kind: SyntaxKind) -> bool { kind == MODULE }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ImportDecl {
    fn can_cast(kind: SyntaxKind) -> bool { kind == IMPORT_DECL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for WildcardImport {
    fn can_cast(kind: SyntaxKind) -> bool { kind == WILDCARD_IMPORT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for NamedImports {
    fn can_cast(kind: SyntaxKind) -> bool { kind == NAMED_IMPORTS }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Specifier {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SPECIFIER }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ExportDecl {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EXPORT_DECL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ExportNamed {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EXPORT_NAMED }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ExportWildcard {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EXPORT_WILDCARD }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ExportDefaultDecl {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EXPORT_DEFAULT_DECL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ExportDefaultExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EXPORT_DEFAULT_EXPR }
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
impl AstNode for VarDecl {
    fn can_cast(kind: SyntaxKind) -> bool { kind == VAR_DECL }
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
impl AstNode for ForStmtInit {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FOR_STMT_INIT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ForStmtTest {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FOR_STMT_TEST }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ForStmtUpdate {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FOR_STMT_UPDATE }
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
impl AstNode for ForOfStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FOR_OF_STMT }
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
impl AstNode for DefaultClause {
    fn can_cast(kind: SyntaxKind) -> bool { kind == DEFAULT_CLAUSE }
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
impl AstNode for Finalizer {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FINALIZER }
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
impl AstNode for NameRef {
    fn can_cast(kind: SyntaxKind) -> bool { kind == NAME_REF }
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
impl AstNode for SpreadProp {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SPREAD_PROP }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for InitializedProp {
    fn can_cast(kind: SyntaxKind) -> bool { kind == INITIALIZED_PROP }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for IdentProp {
    fn can_cast(kind: SyntaxKind) -> bool { kind == IDENT_PROP }
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
impl AstNode for Getter {
    fn can_cast(kind: SyntaxKind) -> bool { kind == GETTER }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Setter {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SETTER }
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
impl AstNode for SuperCall {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SUPER_CALL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ImportCall {
    fn can_cast(kind: SyntaxKind) -> bool { kind == IMPORT_CALL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for NewTarget {
    fn can_cast(kind: SyntaxKind) -> bool { kind == NEW_TARGET }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ImportMeta {
    fn can_cast(kind: SyntaxKind) -> bool { kind == IMPORT_META }
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
impl AstNode for Template {
    fn can_cast(kind: SyntaxKind) -> bool { kind == TEMPLATE }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for TemplateElement {
    fn can_cast(kind: SyntaxKind) -> bool { kind == TEMPLATE_ELEMENT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for SpreadElement {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SPREAD_ELEMENT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ArrayPattern {
    fn can_cast(kind: SyntaxKind) -> bool { kind == ARRAY_PATTERN }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ObjectPattern {
    fn can_cast(kind: SyntaxKind) -> bool { kind == OBJECT_PATTERN }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for RestPattern {
    fn can_cast(kind: SyntaxKind) -> bool { kind == REST_PATTERN }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for AssignPattern {
    fn can_cast(kind: SyntaxKind) -> bool { kind == ASSIGN_PATTERN }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for KeyValuePattern {
    fn can_cast(kind: SyntaxKind) -> bool { kind == KEY_VALUE_PATTERN }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ComputedPropertyName {
    fn can_cast(kind: SyntaxKind) -> bool { kind == COMPUTED_PROPERTY_NAME }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for SinglePattern {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SINGLE_PATTERN }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ArrowExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == ARROW_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for YieldExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == YIELD_EXPR }
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
impl AstNode for Method {
    fn can_cast(kind: SyntaxKind) -> bool { kind == METHOD }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for StaticMethod {
    fn can_cast(kind: SyntaxKind) -> bool { kind == STATIC_METHOD }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ClassDecl {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CLASS_DECL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ClassExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CLASS_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ClassBody {
    fn can_cast(kind: SyntaxKind) -> bool { kind == CLASS_BODY }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for AwaitExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == AWAIT_EXPR }
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
impl From<Getter> for ObjectProp {
    fn from(node: Getter) -> ObjectProp { ObjectProp::Getter(node) }
}
impl From<Setter> for ObjectProp {
    fn from(node: Setter) -> ObjectProp { ObjectProp::Setter(node) }
}
impl From<SpreadProp> for ObjectProp {
    fn from(node: SpreadProp) -> ObjectProp { ObjectProp::SpreadProp(node) }
}
impl From<InitializedProp> for ObjectProp {
    fn from(node: InitializedProp) -> ObjectProp { ObjectProp::InitializedProp(node) }
}
impl From<IdentProp> for ObjectProp {
    fn from(node: IdentProp) -> ObjectProp { ObjectProp::IdentProp(node) }
}
impl From<Method> for ObjectProp {
    fn from(node: Method) -> ObjectProp { ObjectProp::Method(node) }
}
impl AstNode for ObjectProp {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            LITERAL_PROP | GETTER | SETTER | SPREAD_PROP | INITIALIZED_PROP | IDENT_PROP | METHOD
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            LITERAL_PROP => ObjectProp::LiteralProp(LiteralProp { syntax }),
            GETTER => ObjectProp::Getter(Getter { syntax }),
            SETTER => ObjectProp::Setter(Setter { syntax }),
            SPREAD_PROP => ObjectProp::SpreadProp(SpreadProp { syntax }),
            INITIALIZED_PROP => ObjectProp::InitializedProp(InitializedProp { syntax }),
            IDENT_PROP => ObjectProp::IdentProp(IdentProp { syntax }),
            METHOD => ObjectProp::Method(Method { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ObjectProp::LiteralProp(it) => &it.syntax,
            ObjectProp::Getter(it) => &it.syntax,
            ObjectProp::Setter(it) => &it.syntax,
            ObjectProp::SpreadProp(it) => &it.syntax,
            ObjectProp::InitializedProp(it) => &it.syntax,
            ObjectProp::IdentProp(it) => &it.syntax,
            ObjectProp::Method(it) => &it.syntax,
        }
    }
}
impl From<SinglePattern> for Pattern {
    fn from(node: SinglePattern) -> Pattern { Pattern::SinglePattern(node) }
}
impl From<RestPattern> for Pattern {
    fn from(node: RestPattern) -> Pattern { Pattern::RestPattern(node) }
}
impl From<AssignPattern> for Pattern {
    fn from(node: AssignPattern) -> Pattern { Pattern::AssignPattern(node) }
}
impl From<ObjectPattern> for Pattern {
    fn from(node: ObjectPattern) -> Pattern { Pattern::ObjectPattern(node) }
}
impl From<ArrayPattern> for Pattern {
    fn from(node: ArrayPattern) -> Pattern { Pattern::ArrayPattern(node) }
}
impl AstNode for Pattern {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            SINGLE_PATTERN | REST_PATTERN | ASSIGN_PATTERN | OBJECT_PATTERN | ARRAY_PATTERN
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            SINGLE_PATTERN => Pattern::SinglePattern(SinglePattern { syntax }),
            REST_PATTERN => Pattern::RestPattern(RestPattern { syntax }),
            ASSIGN_PATTERN => Pattern::AssignPattern(AssignPattern { syntax }),
            OBJECT_PATTERN => Pattern::ObjectPattern(ObjectPattern { syntax }),
            ARRAY_PATTERN => Pattern::ArrayPattern(ArrayPattern { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Pattern::SinglePattern(it) => &it.syntax,
            Pattern::RestPattern(it) => &it.syntax,
            Pattern::AssignPattern(it) => &it.syntax,
            Pattern::ObjectPattern(it) => &it.syntax,
            Pattern::ArrayPattern(it) => &it.syntax,
        }
    }
}
impl From<CaseClause> for SwitchCase {
    fn from(node: CaseClause) -> SwitchCase { SwitchCase::CaseClause(node) }
}
impl From<DefaultClause> for SwitchCase {
    fn from(node: DefaultClause) -> SwitchCase { SwitchCase::DefaultClause(node) }
}
impl AstNode for SwitchCase {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, CASE_CLAUSE | DEFAULT_CLAUSE) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            CASE_CLAUSE => SwitchCase::CaseClause(CaseClause { syntax }),
            DEFAULT_CLAUSE => SwitchCase::DefaultClause(DefaultClause { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            SwitchCase::CaseClause(it) => &it.syntax,
            SwitchCase::DefaultClause(it) => &it.syntax,
        }
    }
}
impl From<AssignPattern> for ObjectPatternProp {
    fn from(node: AssignPattern) -> ObjectPatternProp { ObjectPatternProp::AssignPattern(node) }
}
impl From<KeyValuePattern> for ObjectPatternProp {
    fn from(node: KeyValuePattern) -> ObjectPatternProp { ObjectPatternProp::KeyValuePattern(node) }
}
impl From<RestPattern> for ObjectPatternProp {
    fn from(node: RestPattern) -> ObjectPatternProp { ObjectPatternProp::RestPattern(node) }
}
impl From<SinglePattern> for ObjectPatternProp {
    fn from(node: SinglePattern) -> ObjectPatternProp { ObjectPatternProp::SinglePattern(node) }
}
impl AstNode for ObjectPatternProp {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            ASSIGN_PATTERN | KEY_VALUE_PATTERN | REST_PATTERN | SINGLE_PATTERN
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            ASSIGN_PATTERN => ObjectPatternProp::AssignPattern(AssignPattern { syntax }),
            KEY_VALUE_PATTERN => ObjectPatternProp::KeyValuePattern(KeyValuePattern { syntax }),
            REST_PATTERN => ObjectPatternProp::RestPattern(RestPattern { syntax }),
            SINGLE_PATTERN => ObjectPatternProp::SinglePattern(SinglePattern { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ObjectPatternProp::AssignPattern(it) => &it.syntax,
            ObjectPatternProp::KeyValuePattern(it) => &it.syntax,
            ObjectPatternProp::RestPattern(it) => &it.syntax,
            ObjectPatternProp::SinglePattern(it) => &it.syntax,
        }
    }
}
impl From<Name> for ArrowExprParams {
    fn from(node: Name) -> ArrowExprParams { ArrowExprParams::Name(node) }
}
impl From<ParameterList> for ArrowExprParams {
    fn from(node: ParameterList) -> ArrowExprParams { ArrowExprParams::ParameterList(node) }
}
impl AstNode for ArrowExprParams {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, NAME | PARAMETER_LIST) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            NAME => ArrowExprParams::Name(Name { syntax }),
            PARAMETER_LIST => ArrowExprParams::ParameterList(ParameterList { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ArrowExprParams::Name(it) => &it.syntax,
            ArrowExprParams::ParameterList(it) => &it.syntax,
        }
    }
}
impl From<Method> for MethodDefinition {
    fn from(node: Method) -> MethodDefinition { MethodDefinition::Method(node) }
}
impl From<Getter> for MethodDefinition {
    fn from(node: Getter) -> MethodDefinition { MethodDefinition::Getter(node) }
}
impl From<Setter> for MethodDefinition {
    fn from(node: Setter) -> MethodDefinition { MethodDefinition::Setter(node) }
}
impl AstNode for MethodDefinition {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, METHOD | GETTER | SETTER) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            METHOD => MethodDefinition::Method(Method { syntax }),
            GETTER => MethodDefinition::Getter(Getter { syntax }),
            SETTER => MethodDefinition::Setter(Setter { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            MethodDefinition::Method(it) => &it.syntax,
            MethodDefinition::Getter(it) => &it.syntax,
            MethodDefinition::Setter(it) => &it.syntax,
        }
    }
}
impl From<EmptyStmt> for ClassElement {
    fn from(node: EmptyStmt) -> ClassElement { ClassElement::EmptyStmt(node) }
}
impl From<Method> for ClassElement {
    fn from(node: Method) -> ClassElement { ClassElement::Method(node) }
}
impl From<StaticMethod> for ClassElement {
    fn from(node: StaticMethod) -> ClassElement { ClassElement::StaticMethod(node) }
}
impl AstNode for ClassElement {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, EMPTY_STMT | METHOD | STATIC_METHOD) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            EMPTY_STMT => ClassElement::EmptyStmt(EmptyStmt { syntax }),
            METHOD => ClassElement::Method(Method { syntax }),
            STATIC_METHOD => ClassElement::StaticMethod(StaticMethod { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ClassElement::EmptyStmt(it) => &it.syntax,
            ClassElement::Method(it) => &it.syntax,
            ClassElement::StaticMethod(it) => &it.syntax,
        }
    }
}
impl From<WildcardImport> for ImportClause {
    fn from(node: WildcardImport) -> ImportClause { ImportClause::WildcardImport(node) }
}
impl From<NamedImports> for ImportClause {
    fn from(node: NamedImports) -> ImportClause { ImportClause::NamedImports(node) }
}
impl From<Name> for ImportClause {
    fn from(node: Name) -> ImportClause { ImportClause::Name(node) }
}
impl AstNode for ImportClause {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, WILDCARD_IMPORT | NAMED_IMPORTS | NAME) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            WILDCARD_IMPORT => ImportClause::WildcardImport(WildcardImport { syntax }),
            NAMED_IMPORTS => ImportClause::NamedImports(NamedImports { syntax }),
            NAME => ImportClause::Name(Name { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            ImportClause::WildcardImport(it) => &it.syntax,
            ImportClause::NamedImports(it) => &it.syntax,
            ImportClause::Name(it) => &it.syntax,
        }
    }
}
impl From<FnDecl> for DefaultDecl {
    fn from(node: FnDecl) -> DefaultDecl { DefaultDecl::FnDecl(node) }
}
impl From<ClassDecl> for DefaultDecl {
    fn from(node: ClassDecl) -> DefaultDecl { DefaultDecl::ClassDecl(node) }
}
impl AstNode for DefaultDecl {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, FN_DECL | CLASS_DECL) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            FN_DECL => DefaultDecl::FnDecl(FnDecl { syntax }),
            CLASS_DECL => DefaultDecl::ClassDecl(ClassDecl { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            DefaultDecl::FnDecl(it) => &it.syntax,
            DefaultDecl::ClassDecl(it) => &it.syntax,
        }
    }
}
impl From<FnDecl> for Decl {
    fn from(node: FnDecl) -> Decl { Decl::FnDecl(node) }
}
impl From<ClassDecl> for Decl {
    fn from(node: ClassDecl) -> Decl { Decl::ClassDecl(node) }
}
impl From<VarDecl> for Decl {
    fn from(node: VarDecl) -> Decl { Decl::VarDecl(node) }
}
impl AstNode for Decl {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, FN_DECL | CLASS_DECL | VAR_DECL) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            FN_DECL => Decl::FnDecl(FnDecl { syntax }),
            CLASS_DECL => Decl::ClassDecl(ClassDecl { syntax }),
            VAR_DECL => Decl::VarDecl(VarDecl { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Decl::FnDecl(it) => &it.syntax,
            Decl::ClassDecl(it) => &it.syntax,
            Decl::VarDecl(it) => &it.syntax,
        }
    }
}
impl From<ArrowExpr> for Expr {
    fn from(node: ArrowExpr) -> Expr { Expr::ArrowExpr(node) }
}
impl From<Literal> for Expr {
    fn from(node: Literal) -> Expr { Expr::Literal(node) }
}
impl From<Template> for Expr {
    fn from(node: Template) -> Expr { Expr::Template(node) }
}
impl From<NameRef> for Expr {
    fn from(node: NameRef) -> Expr { Expr::NameRef(node) }
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
impl From<FnExpr> for Expr {
    fn from(node: FnExpr) -> Expr { Expr::FnExpr(node) }
}
impl From<ClassExpr> for Expr {
    fn from(node: ClassExpr) -> Expr { Expr::ClassExpr(node) }
}
impl From<NewTarget> for Expr {
    fn from(node: NewTarget) -> Expr { Expr::NewTarget(node) }
}
impl From<ImportMeta> for Expr {
    fn from(node: ImportMeta) -> Expr { Expr::ImportMeta(node) }
}
impl From<SuperCall> for Expr {
    fn from(node: SuperCall) -> Expr { Expr::SuperCall(node) }
}
impl From<ImportCall> for Expr {
    fn from(node: ImportCall) -> Expr { Expr::ImportCall(node) }
}
impl From<YieldExpr> for Expr {
    fn from(node: YieldExpr) -> Expr { Expr::YieldExpr(node) }
}
impl From<AwaitExpr> for Expr {
    fn from(node: AwaitExpr) -> Expr { Expr::AwaitExpr(node) }
}
impl AstNode for Expr {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            ARROW_EXPR
                | LITERAL
                | TEMPLATE
                | NAME_REF
                | THIS_EXPR
                | ARRAY_EXPR
                | OBJECT_EXPR
                | GROUPING_EXPR
                | BRACKET_EXPR
                | DOT_EXPR
                | NEW_EXPR
                | CALL_EXPR
                | UNARY_EXPR
                | BIN_EXPR
                | COND_EXPR
                | ASSIGN_EXPR
                | SEQUENCE_EXPR
                | FN_EXPR
                | CLASS_EXPR
                | NEW_TARGET
                | IMPORT_META
                | SUPER_CALL
                | IMPORT_CALL
                | YIELD_EXPR
                | AWAIT_EXPR
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            ARROW_EXPR => Expr::ArrowExpr(ArrowExpr { syntax }),
            LITERAL => Expr::Literal(Literal { syntax }),
            TEMPLATE => Expr::Template(Template { syntax }),
            NAME_REF => Expr::NameRef(NameRef { syntax }),
            THIS_EXPR => Expr::ThisExpr(ThisExpr { syntax }),
            ARRAY_EXPR => Expr::ArrayExpr(ArrayExpr { syntax }),
            OBJECT_EXPR => Expr::ObjectExpr(ObjectExpr { syntax }),
            GROUPING_EXPR => Expr::GroupingExpr(GroupingExpr { syntax }),
            BRACKET_EXPR => Expr::BracketExpr(BracketExpr { syntax }),
            DOT_EXPR => Expr::DotExpr(DotExpr { syntax }),
            NEW_EXPR => Expr::NewExpr(NewExpr { syntax }),
            CALL_EXPR => Expr::CallExpr(CallExpr { syntax }),
            UNARY_EXPR => Expr::UnaryExpr(UnaryExpr { syntax }),
            BIN_EXPR => Expr::BinExpr(BinExpr { syntax }),
            COND_EXPR => Expr::CondExpr(CondExpr { syntax }),
            ASSIGN_EXPR => Expr::AssignExpr(AssignExpr { syntax }),
            SEQUENCE_EXPR => Expr::SequenceExpr(SequenceExpr { syntax }),
            FN_EXPR => Expr::FnExpr(FnExpr { syntax }),
            CLASS_EXPR => Expr::ClassExpr(ClassExpr { syntax }),
            NEW_TARGET => Expr::NewTarget(NewTarget { syntax }),
            IMPORT_META => Expr::ImportMeta(ImportMeta { syntax }),
            SUPER_CALL => Expr::SuperCall(SuperCall { syntax }),
            IMPORT_CALL => Expr::ImportCall(ImportCall { syntax }),
            YIELD_EXPR => Expr::YieldExpr(YieldExpr { syntax }),
            AWAIT_EXPR => Expr::AwaitExpr(AwaitExpr { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::ArrowExpr(it) => &it.syntax,
            Expr::Literal(it) => &it.syntax,
            Expr::Template(it) => &it.syntax,
            Expr::NameRef(it) => &it.syntax,
            Expr::ThisExpr(it) => &it.syntax,
            Expr::ArrayExpr(it) => &it.syntax,
            Expr::ObjectExpr(it) => &it.syntax,
            Expr::GroupingExpr(it) => &it.syntax,
            Expr::BracketExpr(it) => &it.syntax,
            Expr::DotExpr(it) => &it.syntax,
            Expr::NewExpr(it) => &it.syntax,
            Expr::CallExpr(it) => &it.syntax,
            Expr::UnaryExpr(it) => &it.syntax,
            Expr::BinExpr(it) => &it.syntax,
            Expr::CondExpr(it) => &it.syntax,
            Expr::AssignExpr(it) => &it.syntax,
            Expr::SequenceExpr(it) => &it.syntax,
            Expr::FnExpr(it) => &it.syntax,
            Expr::ClassExpr(it) => &it.syntax,
            Expr::NewTarget(it) => &it.syntax,
            Expr::ImportMeta(it) => &it.syntax,
            Expr::SuperCall(it) => &it.syntax,
            Expr::ImportCall(it) => &it.syntax,
            Expr::YieldExpr(it) => &it.syntax,
            Expr::AwaitExpr(it) => &it.syntax,
        }
    }
}
impl std::fmt::Display for ObjectProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for SwitchCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ObjectPatternProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArrowExprParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for MethodDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ClassElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ImportClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for DefaultDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Decl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ImportDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for WildcardImport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for NamedImports {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Specifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ExportDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ExportNamed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ExportWildcard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ExportDefaultDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ExportDefaultExpr {
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
impl std::fmt::Display for VarDecl {
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
impl std::fmt::Display for ForStmtInit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ForStmtTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ForStmtUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ForInStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ForOfStmt {
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
impl std::fmt::Display for DefaultClause {
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
impl std::fmt::Display for Finalizer {
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
impl std::fmt::Display for NameRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ParameterList {
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
impl std::fmt::Display for SpreadProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for InitializedProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for IdentProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for LiteralProp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Getter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Setter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for GroupingExpr {
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
impl std::fmt::Display for SuperCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ImportCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for NewTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ImportMeta {
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
impl std::fmt::Display for Template {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for TemplateElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for SpreadElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArrayPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ObjectPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for RestPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AssignPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for KeyValuePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ComputedPropertyName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for SinglePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ArrowExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for YieldExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for FnExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for StaticMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ClassDecl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ClassExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ClassBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for AwaitExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
