mod diagnostic;
mod rule;
mod store;

pub mod groups;
pub mod rule_prelude;
pub mod testing;
pub mod util;

pub use self::{
    diagnostic::DiagnosticBuilder,
    groups::CstRuleGroup,
    rule::{CstRule, Outcome, Rule, RuleCtx, RuleResult},
    store::CstRuleStore,
};
pub use codespan_reporting::diagnostic::{Label, Severity};

use dyn_clone::clone_box;
use rayon::prelude::*;
use rslint_parser::{parse_module, parse_text, SyntaxNode};
use std::collections::HashMap;

/// The type of errors, warnings, and notes emitted by the linter.
pub type Diagnostic = codespan_reporting::diagnostic::Diagnostic<usize>;

/// The result of linting a file.
#[derive(Debug)]
pub struct LintResult {
    pub parser_diagnostics: Vec<Diagnostic>,
    pub store: CstRuleStore,
    pub rule_diagnostics: HashMap<&'static str, Vec<Diagnostic>>,
}

impl LintResult {
    /// Get all of the diagnostics thrown during linting, in the order of parser diagnostics, then
    /// the diagnostics of each rule sequentially.
    pub fn diagnostics(&self) -> impl Iterator<Item = &Diagnostic> {
        self.parser_diagnostics
            .iter()
            .chain(self.rule_diagnostics.values().map(|x| x.iter()).flatten())
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
    store: CstRuleStore,
    verbose: bool,
) -> LintResult {
    let (parser_diagnostics, green) = if module {
        let parse = parse_module(file_source.as_ref(), file_id);
        (parse.errors().to_owned(), parse.green())
    } else {
        let parse = parse_text(file_source.as_ref(), file_id);
        (parse.errors().to_owned(), parse.green())
    };

    let rule_diagnostics = store
        .par_rules()
        .map(|rule| {
            let root = SyntaxNode::new_root(green.clone());

            (rule.name(), run_rule(rule, file_id, root, verbose))
        })
        .collect();

    LintResult {
        parser_diagnostics,
        store,
        rule_diagnostics
    }
}

pub fn run_rule(
    rule: &Box<dyn CstRule>,
    file_id: usize,
    root: SyntaxNode,
    verbose: bool,
) -> Vec<Diagnostic> {
    let mut ctx = RuleCtx {
        file_id,
        verbose,
        diagnostics: vec![],
    };

    rule.check_root(&root, &mut ctx);

    root.descendants_with_tokens().for_each(|elem| {
        match elem {
            rslint_parser::NodeOrToken::Node(node) => rule.check_node(&node, &mut ctx),
            rslint_parser::NodeOrToken::Token(tok) => rule.check_token(&tok, &mut ctx),
        };
    });

    ctx.diagnostics
}

/// Get a rule by its kebab-case name. 
pub fn get_rule_by_name(name: &str) -> Option<Box<dyn CstRule>> {
    CstRuleStore::new()
        .builtins()
        .rules()
        .find(|rule| rule.name() == name)
        .map(|rule| clone_box(&**rule))
}
