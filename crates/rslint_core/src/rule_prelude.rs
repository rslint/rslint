//! Commonly used items by rules. These include some AST definitions, and utilities.

#[doc(no_inline)]
pub use crate::{
    declare_lint, rule_tests, util, CstRule, Diagnostic, Outcome, RuleCtx, RuleResult,
};

#[doc(no_inline)]
pub use rslint_parser::{
    ast, op, token_set, util as parseutil, util::color, AstNode, AstToken, BigInt, JsNum,
    SyntaxElement, SyntaxKind, SyntaxNode, SyntaxNodeExt, SyntaxToken, SyntaxTokenExt, TokenSet, T,
};
