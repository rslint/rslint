//! Commonly used items by rules. These include some AST definitions, and utilities.

pub use crate::{
    declare_lint, CstRule, Diagnostic, DiagnosticBuilder, Label, Outcome, RuleCtx, RuleResult,
    util, rule_tests
};
pub use rslint_parser::{
    ast, token_set, util as parseutil, AstNode, AstToken, SyntaxElement, SyntaxKind, SyntaxNode,
    SyntaxNodeExt, SyntaxToken, SyntaxTokenExt, TokenSet, T, util::color
};
