//! The core runner for RSLint responsible for the bulk of the linter's work.
//!
//! The crate is not RSLint-specific and can be used from any project. The runner is responsible
//! for taking a list of rules, and source code and running the linter on it. It is important to decouple
//! the CLI work and the low level linting work from eachother to be able to reuse the linter facilities.
//! Therefore, the core runner should never do anything `rslint_cli`-specific.
//!
//! The structure at the core of the crate is the [`CstRule`] and [`Rule`] traits.
//! CST rules run on a single file and its concrete syntax tree produced by [`rslint_parser`].
//! The rules have a couple of restrictions for clarity and speed, these include:
//! - all cst rules must be [`Send`](std::marker::Send) and [`Sync`](std::marker::Sync) so they can be run in parallel
//! - rules may never rely on the results of other rules, this is impossible because rules are run in parallel
//! - rules should never make any network or file requests
//!
//! ## Using the runner
//!
//! To run the runner you must first create a [`CstRuleStore`], which is the structure used for storing what rules
//! to run. Then you can use [`lint_file`].
//!
//! ## Running a single rule
//!
//! To run a single rule you can find the rule you want in the `groups` module and submodules within. Then
//! to run a rule in full on a syntax tree you can use [`run_rule`].
//!
//! Rules can also be run on individual nodes using the functions on [`CstRule`].
//! ⚠️ note however that many rules rely on checking tokens or the root and running on single nodes
//! may yield incorrect results, you should only do this if you know about the rule's implementation.

// FIXME: Workaround for https://github.com/GREsau/schemars/pull/65
#![allow(clippy::field_reassign_with_default)]

mod file;
mod rule;
mod store;
mod testing;

pub mod autofix;
pub mod directives;
pub mod groups;
pub mod rule_prelude;
pub mod util;

pub use self::{
    file::File,
    rule::{CstRule, Inferable, Outcome, Rule, RuleCtx, RuleLevel, RuleResult, Tag},
    store::CstRuleStore,
};
pub use rslint_errors::{Diagnostic, Severity, Span};

pub use crate::directives::{
    apply_top_level_directives, skip_node, Directive, DirectiveError, DirectiveErrorKind,
    DirectiveParser,
};

use dyn_clone::clone_box;
use rslint_parser::{util::SyntaxNodeExt, SyntaxKind, SyntaxNode};
use std::collections::HashMap;
use std::sync::Arc;

/// The result of linting a file.
// TODO: A lot of this stuff can be shoved behind a "linter options" struct
#[derive(Debug, Clone)]
pub struct LintResult<'s> {
    /// Any diagnostics (errors, warnings, etc) emitted from the parser
    pub parser_diagnostics: Vec<Diagnostic>,
    /// The diagnostics emitted by each rule run
    pub rule_results: HashMap<&'static str, RuleResult>,
    /// Any warnings or errors emitted by the directive parser
    pub directive_diagnostics: Vec<DirectiveError>,
    pub store: &'s CstRuleStore,
    pub parsed: SyntaxNode,
    pub file_id: usize,
    pub verbose: bool,
    pub fixed_code: Option<String>,
}

impl LintResult<'_> {
    /// Get all of the diagnostics thrown during linting, in the order of parser diagnostics, then
    /// the diagnostics of each rule sequentially.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.parser_diagnostics
            .iter()
            .chain(
                self.rule_results
                    .values()
                    .map(|x| x.diagnostics.iter())
                    .flatten(),
            )
            .chain(self.directive_diagnostics.iter().map(|x| &x.diagnostic))
    }

    /// The overall outcome of linting this file (failure, warning, success, etc)
    pub fn outcome(&self) -> Outcome {
        self.diagnostics().into()
    }

    /// Attempt to automatically fix any fixable issues and return the fixed code.
    ///
    /// This will not run if there are syntax errors unless `dirty` is set to true.
    pub fn fix(&mut self, dirty: bool, file: &File) -> Option<String> {
        if self
            .parser_diagnostics
            .iter()
            .any(|x| x.severity == Severity::Error)
            && !dirty
        {
            None
        } else {
            Some(autofix::recursively_apply_fixes(self, file))
        }
    }
}

/// Lint a file with a specific rule store.
pub fn lint_file<'s>(file: &File, store: &'s CstRuleStore, verbose: bool) -> LintResult<'s> {
    let (diagnostics, node) = file.parse_with_errors();
    lint_file_inner(node, diagnostics, file, store, verbose)
}

/// used by lint_file and incrementally_relint to not duplicate code
pub(crate) fn lint_file_inner<'s>(
    node: SyntaxNode,
    parser_diagnostics: Vec<Diagnostic>,
    file: &File,
    store: &'s CstRuleStore,
    verbose: bool,
) -> LintResult<'s> {
    let mut new_store = store.clone();
    let directives::DirectiveResult {
        directives,
        diagnostics: mut directive_diagnostics,
    } = { DirectiveParser::new_with_store(node.clone(), file, store).get_file_directives() };

    apply_top_level_directives(
        directives.as_slice(),
        &mut new_store,
        &mut directive_diagnostics,
        file.id,
    );

    let src: Arc<str> = Arc::from(node.to_string());

    // FIXME: Replace with thread pool
    let results = new_store
        .rules
        .into_iter()
        .map(|rule| {
            (
                rule.name(),
                run_rule(
                    &*rule,
                    file.id,
                    node.clone(),
                    verbose,
                    &directives,
                    src.clone(),
                ),
            )
        })
        .collect();

    LintResult {
        parser_diagnostics,
        rule_results: results,
        directive_diagnostics,
        store,
        parsed: node,
        file_id: file.id,
        verbose,
        fixed_code: None,
    }
}

/// Run a single run on an entire parsed file.
///
/// # Panics
/// Panics if `root`'s kind is not `SCRIPT` or `MODULE`
pub fn run_rule(
    rule: &dyn CstRule,
    file_id: usize,
    root: SyntaxNode,
    verbose: bool,
    directives: &[Directive],
    src: Arc<str>,
) -> RuleResult {
    assert!(root.kind() == SyntaxKind::SCRIPT || root.kind() == SyntaxKind::MODULE);
    let mut ctx = RuleCtx {
        file_id,
        verbose,
        diagnostics: vec![],
        fixer: None,
        src,
    };

    rule.check_root(&root, &mut ctx);

    root.descendants_with_tokens_with(&mut |elem| {
        match elem {
            rslint_parser::NodeOrToken::Node(node) => {
                if skip_node(directives, &node, rule) || node.kind() == SyntaxKind::ERROR {
                    return false;
                }
                rule.check_node(&node, &mut ctx);
            }
            rslint_parser::NodeOrToken::Token(tok) => {
                let _ = rule.check_token(&tok, &mut ctx);
            }
        };
        true
    });
    RuleResult::new(ctx.diagnostics, ctx.fixer)
}

/// Get a rule by its kebab-case name.
pub fn get_rule_by_name(name: &str) -> Option<Box<dyn CstRule>> {
    CstRuleStore::new()
        .builtins()
        .rules
        .iter()
        .find(|rule| rule.name() == name)
        .map(|rule| clone_box(&**rule))
}

/// Get a group's rules by the group name.
// TODO: there should be a good way to not have to hardcode all of this
pub fn get_group_rules_by_name(group_name: &str) -> Option<Vec<Box<dyn CstRule>>> {
    use groups::*;

    Some(match group_name {
        "errors" => errors(),
        "style" => style(),
        "regex" => regex(),
        _ => return None,
    })
}

/// Get a suggestion for an incorrect rule name for things such as "did you mean ...?"
pub fn get_rule_suggestion(incorrect_rule_name: &str) -> Option<&str> {
    let rules = CstRuleStore::new()
        .builtins()
        .rules
        .into_iter()
        .map(|rule| rule.name());
    util::find_best_match_for_name(rules, incorrect_rule_name, None)
}

/// Get a rule and its documentation.
///
/// This will always be `Some` for valid rule names and it will be an empty string
/// if the rule has no docs
pub fn get_rule_docs(rule: &str) -> Option<&'static str> {
    get_rule_by_name(rule).map(|rule| rule.docs())
}

macro_rules! trait_obj_helper {
    ($($name:ident),* $(,)?) => {
        vec![
            $(
                Box::new($name::default()) as Box<dyn Inferable>
            ),*
        ]
    }
}

/// Get all of the built in rules which can have their options inferred using multiple syntax nodes
/// see [`Inferable`] for more information.
pub fn get_inferable_rules() -> Vec<Box<dyn Inferable>> {
    use groups::style::*;

    trait_obj_helper![BlockSpacing]
}
