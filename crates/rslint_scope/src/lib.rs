mod datalog;
mod visit;

pub use datalog::{
    Datalog, DatalogBuilder, DatalogFunction, DatalogResult, DatalogScope, DatalogTransaction,
    DerivedFacts,
};

use rslint_core::{
    rule_prelude::{
        ast::{
            Decl, Expr, FnDecl, Literal, LiteralKind, NameRef, Pattern, ReturnStmt, Stmt, VarDecl,
        },
        AstNode, SyntaxNode, SyntaxNodeExt, TextRange,
    },
    CstRule, Rule, RuleCtx,
};
use serde::{Deserialize, Serialize};
use types::{
    internment::{self, Intern},
    ExprId, InvalidNameUse, Pattern as DatalogPattern,
};
use visit::Visit;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScopeAnalyzer {
    #[serde(skip)]
    datalog: Datalog,
}

impl ScopeAnalyzer {
    pub fn new() -> DatalogResult<Self> {
        Ok(Self {
            datalog: Datalog::new()?,
        })
    }

    pub fn analyze(&self, syntax: &SyntaxNode, ctx: &mut RuleCtx) -> DatalogResult<()> {
        let analyzer = AnalyzerInner;

        let facts = self.datalog.transaction(|trans| {
            let scope = trans.scope();
            for stmt in syntax.children().filter_map(|node| node.try_to::<Stmt>()) {
                analyzer.visit(&scope, stmt);
            }

            Ok(())
        })?;

        for InvalidNameUse { name, span, .. } in facts.invalid_name_uses {
            let error = ctx
                .err(
                    "datalog-scoping",
                    format!("cannot find value `{}` in this scope", name),
                )
                .primary(span, "not found in this scope".to_owned());

            ctx.add_err(error);
        }

        Ok(())
    }
}

impl Rule for ScopeAnalyzer {
    fn name(&self) -> &'static str {
        "scope-analysis"
    }

    fn group(&self) -> &'static str {
        "errors"
    }
}

#[typetag::serde]
impl CstRule for ScopeAnalyzer {
    fn check_root(&self, root: &SyntaxNode, ctx: &mut RuleCtx) -> Option<()> {
        if let Err(err) = self.analyze(root, ctx) {
            eprintln!("Datalog error: {:?}", err);
        }

        Some(())
    }
}

struct AnalyzerInner;

impl AnalyzerInner {
    fn visit_pattern(&self, pattern: Pattern) -> Intern<DatalogPattern> {
        match pattern {
            Pattern::SinglePattern(single) => internment::intern(&DatalogPattern {
                name: internment::intern(&single.text()),
            }),

            _ => todo!(),
        }
    }
}

impl<'ddlog> Visit<'ddlog, Stmt> for AnalyzerInner {
    type Output = Option<DatalogScope<'ddlog>>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, stmt: Stmt) -> Self::Output {
        match stmt {
            Stmt::BlockStmt(block) => {
                let mut scope = scope.scope();
                for stmt in block.stmts() {
                    if let Some(new_scope) = self.visit(&scope, stmt) {
                        scope = new_scope;
                    }
                }
                // TODO: How to connect blocks with the downstream nodes?
            }
            Stmt::EmptyStmt(_) => {}
            Stmt::ExprStmt(expr) => {
                expr.expr().map(|expr| self.visit(scope, expr));
            }
            Stmt::IfStmt(_) => {}
            Stmt::DoWhileStmt(_) => {}
            Stmt::WhileStmt(_) => {}
            Stmt::ForStmt(_) => {}
            Stmt::ForInStmt(_) => {}
            Stmt::ContinueStmt(_) => {}
            Stmt::BreakStmt(_) => {}
            Stmt::ReturnStmt(ret) => self.visit(scope, ret),
            Stmt::WithStmt(_) => {}
            Stmt::LabelledStmt(_) => {}
            Stmt::SwitchStmt(_) => {}
            Stmt::ThrowStmt(_) => {}
            Stmt::TryStmt(_) => {}
            Stmt::DebuggerStmt(_) => {}
            Stmt::Decl(decl) => return self.visit(scope, decl),
        }

        None
    }
}

impl<'ddlog> Visit<'ddlog, Decl> for AnalyzerInner {
    type Output = Option<DatalogScope<'ddlog>>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, decl: Decl) -> Self::Output {
        match decl {
            Decl::FnDecl(func) => {
                self.visit(scope, func);
                None
            }
            Decl::ClassDecl(_) => None,
            Decl::VarDecl(var) => Some(self.visit(scope, var)),
        }
    }
}

impl<'ddlog> Visit<'ddlog, FnDecl> for AnalyzerInner {
    type Output = ();

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, func: FnDecl) -> Self::Output {
        let function_id = scope.next_function_id();
        let name = func.name().map(|name| internment::intern(&name.text()));

        let function = scope.decl_function(function_id, name);

        if let Some(params) = func.parameters() {
            for param in params.parameters() {
                function.argument(self.visit_pattern(param));
            }
        }

        if let Some(body) = func.body() {
            let mut scope: Box<dyn DatalogBuilder<'_>> = Box::new(function);

            for stmt in body.stmts() {
                // Enter a new scope after each statement that requires one
                if let Some(new_scope) = self.visit(&*scope, stmt) {
                    scope = Box::new(new_scope);
                }
            }
        }
    }
}

impl<'ddlog> Visit<'ddlog, VarDecl> for AnalyzerInner {
    type Output = DatalogScope<'ddlog>;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, var: VarDecl) -> Self::Output {
        let (mut last_scope, span) = (None, var.syntax().trimmed_range());

        for decl in var.declared() {
            let pattern = decl.pattern().map(|pat| self.visit_pattern(pat));
            let value = self.visit(scope, decl.value());

            last_scope = Some(if var.is_let() {
                scope.decl_let(pattern, value, span)
            } else if var.is_const() {
                scope.decl_const(pattern, value, span)
            } else if var.is_var() {
                scope.decl_var(pattern, value, span)
            } else {
                unreachable!("a variable declaration was neither `let`, `const` or `var`");
            });
        }

        last_scope.expect("at least one variable was declared, right?")
    }
}

impl<'ddlog> Visit<'ddlog, Expr> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, expr: Expr) -> Self::Output {
        match expr {
            Expr::Literal(literal) => self.visit(scope, literal),
            Expr::NameRef(name) => self.visit(scope, name),

            // FIXME: This is here so things can function before everything is 100%
            //        translatable into datalog, mostly for my sanity
            _ => 0,
        }
    }
}

impl<'ddlog> Visit<'ddlog, Literal> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, literal: Literal) -> Self::Output {
        let span = literal.syntax().trimmed_range();

        match literal.kind() {
            LiteralKind::Number(number) => scope.number(number, span),
            LiteralKind::BigInt(bigint) => scope.bigint(bigint, span),
            LiteralKind::String => {
                scope.string(literal.inner_string_text().unwrap().to_string(), span)
            }
            LiteralKind::Null => scope.null(span),
            LiteralKind::Bool(boolean) => scope.boolean(boolean, span),
            LiteralKind::Regex => scope.regex(span),
        }
    }
}

impl<'ddlog> Visit<'ddlog, NameRef> for AnalyzerInner {
    type Output = ExprId;

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, name: NameRef) -> Self::Output {
        scope.name_ref(name.to_string(), name.syntax().trimmed_range())
    }
}

impl<'ddlog> Visit<'ddlog, ReturnStmt> for AnalyzerInner {
    type Output = ();

    fn visit(&self, scope: &dyn DatalogBuilder<'ddlog>, ret: ReturnStmt) -> Self::Output {
        let value = ret.value().map(|val| self.visit(scope, val));
        scope.ret(value, ret.syntax().text_range());
    }
}
