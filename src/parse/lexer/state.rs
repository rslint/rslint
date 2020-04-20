use super::token::TokenType;
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::iter::FromIterator;
use log::trace;

/// A structure for keeping track of context for template and regex literals
/// State ignores whitespace and linebreaks, state is not updated for whitespace or linebreak tokens
#[derive(Debug)]
pub struct LexerState {
  pub expr_allowed: bool,
  pub prev: Option<TokenType>,
  /// if the previous token and the next have a Linebreak token in between them
  pub had_linebreak: bool,
  pub context: TokenContext,
}

impl LexerState {
  pub fn new() -> Self {
    Self {
      expr_allowed: true,
      prev: None,
      had_linebreak: false,
      context: TokenContext(vec![Context::BraceStmt]),
    }
  }

  pub fn update(&mut self, next: Option<TokenType>) {
    self.expr_allowed = self.update_expr_allowed(next);
    trace!("Updating state: \n next: {:?} | prev: {:?} | expr_allowed_next: {}",
    next, self.prev, self.expr_allowed
    );
    self.prev = next;
  }

  fn update_expr_allowed(&mut self, next: Option<TokenType>) -> bool {

    if next.filter(|tt| tt.is_keyword()).is_some() && self.prev == Some(TokenType::Period) {
      return false;
    }

    match next {
      Some(TokenType::ParenClose) | Some(TokenType::BraceClose) => {
        if self.context.len() == 1 { return true; }

        let closed = self.context.pop()
          .expect("Tried update state with ) or } but context is somehow empty"); //should be unreachable

        // const foo = function(){}
        //                         ^ expression cannot follow
        if closed == Context::BraceStmt && self.context.cur() == Some(Context::FnExpr) {
          self.context.pop();
          return false;
        }

        // `${} `
        //    ^ after template literal
        if closed == Context::TplInternal {
          match self.context.cur() {
            Some(Context::Template) => return false,
            _ => return true,
          }
        }

        // 6 /.+/
        // ^  ^ expr cannot follow expr
        !closed.is_expr()
      },

      Some(TokenType::Function) => {

        if self.expr_allowed && !self.context.is_brace_block(self.prev, self.had_linebreak, self.expr_allowed) {
          self.context.push(Context::FnExpr);
        }
        false
      },

      // for(a of /a/g)
      //          ^ expr after of is allowed even if potentially wrong
      Some(TokenType::Of) if self.context.cur() == Some(Context::ParenStmt { for_loop: true }) => {
        !self.prev.expect("Unreachable condition where previous is None inside a for loop").is_before_expr()
      },
      
      Some(TokenType::Identifier) => {
        //TODO es6
        match self.prev {
          Some(TokenType::Var) if self.had_linebreak => true,
          _ => false
        }
      },

      Some(TokenType::BraceOpen) => {
        let next_context = 
          if self.context.is_brace_block(self.prev, self.had_linebreak, self.expr_allowed) {
            Context::BraceStmt
          } else {
            Context::BraceExpr
          };
        self.context.push(next_context);
        true
      },

      // Some(TokenType::TemplateOpen) => {
      //   self.context.push(Context::TplInternal);
      //   true
      // }

      Some(TokenType::ParenOpen) => {
        self.context.push(match self.prev {
          Some(t) if t.is_keyword() => match t {
            TokenType::If | TokenType::With | TokenType::While => Context::ParenStmt { for_loop: false },
            TokenType::For => Context::ParenStmt { for_loop: true },
            _ => Context::ParenExpr
          },
          _ => Context::ParenExpr
        });
        true
      },

      Some(TokenType::Increment) | Some(TokenType::Decrement) => self.expr_allowed,

      None => false,

      _ => next.unwrap().is_before_expr()
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Context {
  BraceStmt,
  BraceExpr,
  TplInternal,
  ParenStmt {
    for_loop: bool
  },
  ParenExpr,
  Template,
  FnExpr
}

impl Context {
  pub fn is_expr(&self) -> bool {
    EXPR_CTXTS.contains(self)
  }
}

static EXPR_CTXTS: Lazy<HashSet<Context>> = Lazy::new(|| {
  HashSet::from_iter(vec![
    Context::BraceExpr,
    Context::TplInternal,
    Context::ParenExpr,
    Context::Template,
    Context::FnExpr
  ])
});

#[derive(Debug)]
pub struct TokenContext(Vec<Context>);

impl TokenContext {
  pub fn is_brace_block(&self, prev: Option<TokenType>, had_line_break: bool, expr_allowed: bool) -> bool {
    if prev == Some(TokenType::Colon) {
      match self.cur() {
        Some(Context::BraceStmt) => return true,
        // { a: {} }
        //   ^  ^
        Some(Context::BraceExpr) => return false,
        _ => unreachable!(),
      }
    }

    match prev {
      /* 
        function a() {
          return { b: "" };
        }

        function a() {
          return {
            function(b){}
          };
        }

        function* gen() {
          yield { b: "" };
        }

      */
      Some(TokenType::Return) | Some(TokenType::Yield) => return had_line_break,

      /*
        if() {} else {}
                 ^   ^
        
        ; {}
       ^  ^

       ) {}
      ^  ^

      {}
      ^ start of program
      */
      Some(TokenType::Else) | Some(TokenType::Semicolon) | Some(TokenType::ParenClose) | None => return true,

      Some(TokenType::BraceOpen) => return self.cur() == Some(Context::BraceStmt),

      _ => {}
    }

    !expr_allowed
  }

  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  pub fn len(&self) -> usize {
    self.0.len()
  }

  pub fn pop(&mut self) -> Option<Context> {
    let popped = self.0.pop();
    trace!("Popped context: {:?}", popped);
    popped
  }

  pub fn cur(&self) -> Option<Context> {
    self.0.last().cloned()
  }

  pub fn push(&mut self, context: Context) {
    trace!("Pushed context: {:?}", context);
    self.0.push(context);
  }
}