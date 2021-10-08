use crate::rule_prelude::*;
use rslint_parser::ast::FnDecl;
use ast::{Expr, Stmt};

declare_lint! {
  #[derive(Default)]
  RequireYield,
  errors,
  "require_yeild"
}

//&& node.parent().map_or(true, |parent| !ALLOWED.contains(&parent.kind())




#[typetag::serde]
impl CstRule for RequireYield {
  fn check_node(&self, node: &SyntaxNode, context: &mut RuleCtx) -> Option<()> {
      if node.kind() == SyntaxKind::FN_DECL  {
        // if the function doesn't have a star token, it is not a generator
        let fn_decl_node = node.to::<FnDecl>();
        if !fn_decl_node.star_token().is_none() {

           // if the fn body is empty, return early
           if fn_decl_node.body()?.stmts().count() == 0 {
               return None
           }

           let yield_expr = fn_decl_node.body()?.stmts()
              // get all the expressions in the function block stmts
              .filter_map(|x| match x {
                Stmt::ExprStmt(expr) => expr.expr(),
                _ => None,
              })
              // find a yield expression if one exists
              .find(|x| matches!(x, Expr::YieldExpr(_)));

            // if none of the statements in the fn block
            // were a yield statement, report the lint
            if yield_expr.is_none() {
                let err = context.err(self.name(), "This generator function does not have 'yield'.")
                  .primary(node.trimmed_range(), "Add a 'yield' statement.");
                context.add_err(err);
            }
        }
      }
    None
  }

}



rule_tests! {
  RequireYield::default(),
  err: {
    "
    function* foo(){
      return 10;
    }
    ",
  },
  ok: {
   // Generator fn with yeild statement
   "
   function* foo(){
       yield 5;
       return 10;
   }
   ",
   // Regular fn
   "
   function foo() {
     return 10;
   }
   ",
   // Empty generator fn is also valid
   "
   function* foo() { }
   "
  }
}
