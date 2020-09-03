use crate::rule_prelude::*;
use rslint_parser::TextRange;
use SyntaxKind::*;

declare_lint! {
    #[derive(Default)]
    GetterReturn,
    "getter-return",
    /// Whether to allow implicitly returning undefined with `return;`
    allow_implicit: bool
}

impl CstRule for GetterReturn {
    fn check_node(&self, node: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        match node.kind() {
            CALL_EXPR => {
                let expr = node.to::<ast::CallExpr>();
                if expr.callee().map_or(false, |e| {
                    e.syntax()
                        .structural_lossy_token_eq(&["Object", ".", "defineProperty"])
                }) && expr.arguments()?.args().count() == 3
                {
                    let args: Vec<ast::Expr> = expr.arguments().unwrap().args().collect();
                    if let Some(obj) = args
                        .iter()
                        .nth(2)
                        .and_then(|expr| expr.syntax().try_to::<ast::ObjectExpr>())
                    {
                        for prop in obj.props() {
                            if let ast::ObjectProp::LiteralProp(literal_prop) = prop {
                                if literal_prop.key()?.syntax().text() != "get" {
                                    continue;
                                }
                                match literal_prop.value()? {
                                    ast::Expr::FnExpr(decl) => {
                                        self.check_stmts(args[1].syntax(), decl.body()?.syntax(), decl.body()?.stmts(), ctx);
                                    },
                                    ast::Expr::ArrowExpr(arrow) => {
                                        if let ast::ExprOrBlock::Block(block) = arrow.body()? {
                                            self.check_stmts(args[1].syntax(), block.syntax(), block.stmts(), ctx);
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            GETTER => {
                let getter = node.to::<ast::Getter>();
                if let Some(body) = getter.body() {
                    if let Some(key) = getter.key() {
                        self.check_stmts(key.syntax(), body.syntax(), body.stmts(), ctx);
                    }
                }
            },
            _ => {}
        }
        None
    }
}

impl GetterReturn {
    fn check_stmts(&self, key: &SyntaxNode, body: &SyntaxNode, mut stmts: impl Iterator<Item = ast::Stmt>, ctx: &mut RuleCtx) {
        if !stmts.any(|stmt| self.check_stmt(&stmt)) {
            let err = ctx.err(self.name(), format!("Getter properties must always return a value, but `{}` does not.", key.trimmed_text()))
                .secondary(key.trimmed_range(), "this key is sometimes or always undefined...")
                .primary(body.trimmed_range(), "...because this getter does not always return a value");

            ctx.add_err(err);
        }
    }

    fn check_stmt(&self, stmt: &ast::Stmt) -> bool {
        match stmt {
            ast::Stmt::IfStmt(if_stmt) => self.check_if(if_stmt),
            ast::Stmt::BlockStmt(block) => block.stmts().any(|stmt| self.check_stmt(&stmt)),
            ast::Stmt::ReturnStmt(stmt) => stmt.value().is_some() || self.allow_implicit,
            ast::Stmt::SwitchStmt(switch) => {
                dbg!(switch).cases().any(|case| match dbg!(case) {
                    ast::SwitchCase::CaseClause(clause) => clause.cons().any(|s| self.check_stmt(&s)),
                    ast::SwitchCase::DefaultClause(clause) => dbg!(clause).cons().any(|s| self.check_stmt(&s))
                })
            }
            _ => false
        }
    }

    /// Check if an if statement unconditionally returns from the statement.
    fn check_if(&self, stmt: &ast::IfStmt) -> bool {
        if stmt.alt().is_none() {
            return false;
        }

        if let Some(cons) = stmt.cons() {
            if !self.check_stmt(&cons) {
                return false;
            }
            return self.check_stmt(&stmt.alt().unwrap());
        }
        false
    }
}
