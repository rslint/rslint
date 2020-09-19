//! Commonly used items by rules. These include some AST definitions, and utilities.

#[doc(no_inline)]
pub use crate::{
    declare_lint, CstRule, Diagnostic, DiagnosticBuilder, Label, Outcome, RuleCtx, RuleResult,
    util, rule_tests
};

#[doc(no_inline)]
pub use rslint_parser::{
    ast, token_set, util as parseutil, AstNode, AstToken, SyntaxElement, SyntaxKind, SyntaxNode,
    SyntaxNodeExt, SyntaxToken, SyntaxTokenExt, TokenSet, T, util::color, JsNum, BigInt, op
};
