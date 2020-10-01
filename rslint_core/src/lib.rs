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

mod diagnostic;
mod rule;
mod store;
mod testing;

pub mod directives;
pub mod groups;
pub mod incremental;
pub mod rule_prelude;
pub mod util;

pub use self::{
    diagnostic::{DiagnosticBuilder, Span},
    rule::{CstRule, Outcome, Rule, RuleCtx, RuleLevel, RuleResult},
    store::CstRuleStore,
};
pub use codespan_reporting::diagnostic::{Label, Severity};
pub use rslint_parser::Indel;

use crate::directives::{apply_top_level_directives, skip_node, Directive, DirectiveParser};
use dyn_clone::clone_box;
use rayon::prelude::*;
use rslint_parser::{parse_module, parse_text, util::SyntaxNodeExt, SyntaxNode};
use std::collections::HashMap;

/// The type of errors, warnings, and notes emitted by the linter.
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<usize>;

/// The result of linting a file.
// TODO: A lot of this stuff can be shoved behind a "linter options" struct
#[derive(Debug, Clone)]
pub struct LintResult<'s> {
    pub parser_diagnostics: Vec<Diagnostic>,
    pub store: &'s CstRuleStore,
    pub rule_diagnostics: HashMap<&'static str, Vec<Diagnostic>>,
    pub directive_diagnostics: Vec<Diagnostic>,
    pub parsed: SyntaxNode,
    pub file_id: usize,
    pub verbose: bool,
}

impl LintResult<'_> {
    /// Get all of the diagnostics thrown during linting, in the order of parser diagnostics, then
    /// the diagnostics of each rule sequentially.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.parser_diagnostics
            .iter()
            .chain(self.rule_diagnostics.values().map(|x| x.iter()).flatten())
            .chain(self.directive_diagnostics.iter())
    }

    /// The overall outcome of linting this file (failure, warning, success, etc)
    pub fn outcome(&self) -> Outcome {
        self.diagnostics().into()
    }
}

/// Lint a file with a specific rule store.
pub fn lint_file(
    file_id: usize,
    file_source: impl AsRef<str>,
    module: bool,
    store: &CstRuleStore,
    verbose: bool,
) -> Result<LintResult, Diagnostic> {
    let (parser_diagnostics, green) = if module {
        let parse = parse_module(file_source.as_ref(), file_id);
        (parse.errors().to_owned(), parse.green())
    } else {
        let parse = parse_text(file_source.as_ref(), file_id);
        (parse.errors().to_owned(), parse.green())
    };
    lint_file_inner(
        SyntaxNode::new_root(green),
        parser_diagnostics,
        file_id,
        store,
        verbose,
    )
}

/// used by lint_file and incrementally_relint to not duplicate code
pub(crate) fn lint_file_inner(
    node: SyntaxNode,
    parser_diagnostics: Vec<Diagnostic>,
    file_id: usize,
    store: &CstRuleStore,
    verbose: bool,
) -> Result<LintResult, Diagnostic> {
    let mut new_store = store.clone();
    let results = DirectiveParser::new(node.clone(), file_id, store).get_file_directives()?;
    let mut directive_diagnostics = vec![];

    let directives = results
        .into_iter()
        .map(|res| {
            directive_diagnostics.extend(res.diagnostics);
            res.directive
        })
        .collect::<Vec<_>>();

    apply_top_level_directives(
        directives.as_slice(),
        &mut new_store,
        &mut directive_diagnostics,
        file_id,
    );

    let rule_diagnostics = new_store
        .rules
        .par_iter()
        .map(|rule| {
            (
                rule.name(),
                run_rule(&**rule, file_id, node.clone(), verbose, &directives),
            )
        })
        .collect();

    Ok(LintResult {
        parser_diagnostics,
        store,
        rule_diagnostics,
        directive_diagnostics,
        parsed: node,
        file_id,
        verbose,
    })
}

pub fn run_rule(
    rule: &dyn CstRule,
    file_id: usize,
    root: SyntaxNode,
    verbose: bool,
    directives: &[Directive],
) -> Vec<Diagnostic> {
    let mut ctx = RuleCtx {
        file_id,
        verbose,
        diagnostics: vec![],
    };

    rule.check_root(&root, &mut ctx);

    root.descendants_with_tokens_with(&mut |elem| {
        match elem {
            rslint_parser::NodeOrToken::Node(node) => {
                if skip_node(directives, &node, rule) {
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

    ctx.diagnostics
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
