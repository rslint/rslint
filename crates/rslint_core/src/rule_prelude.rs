//! Commonly used items by rules. These include some AST definitions, and utilities.

#[doc(no_inline)]
pub use crate::{
    autofix::{Fixer, Unwrappable, Wrapping},
    declare_lint, rule_tests, util, CstRule, Diagnostic, Outcome, RuleCtx, RuleResult, Span,
};

#[doc(no_inline)]
pub use rslint_parser::{
    ast, op, token_set, util as parseutil, util::color, AstNode, AstToken, BigInt, JsNum,
    SyntaxElement, SyntaxKind, SyntaxNode, SyntaxNodeExt, SyntaxToken, SyntaxTokenExt, TokenSet, T,
};

#[doc(no_inline)]
pub use rslint_errors::{Applicability, Severity};
