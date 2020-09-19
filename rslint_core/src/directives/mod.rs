//! Directives used to configure or ignore rules.
//! These take place of comments over nodes or comments at the top level.
//!
//! Directives can contain multiple commands separated by `-`. For example:
//!
//! ```text
//! // rslint-ignore for-direction, no-await-in-loop - deny no-empty -- because why not
//!   |      |                                     |  |            |    |             |
//!   |      +-------------------------------------+  +------------+    +-------------+
//!   |                      command                     command            comment   |
//!   +-------------------------------------------------------------------------------+
//!                                      Directive
//! ```

mod parser;

pub use self::parser::*;

use crate::{CstRuleStore, Diagnostic, DiagnosticBuilder, SyntaxNode, CstRule, rule_tests};
use rslint_parser::util::*;

// TODO: More complex warnings, things like ignoring node directives because of file level directives

/// Apply file level directives to a store and add their respective diagnostics to the pool of diagnostics.
/// for file level ignores this will clear all the rules from the store.
///
/// This method furthermore issues more contextual warnings like disabling a rule after
/// the entire file has been disabled.
pub fn apply_top_level_directives(
    directives: &[Directive],
    store: &mut CstRuleStore,
    diagnostics: &mut Vec<Diagnostic>,
    file_id: usize
) {
    let mut ignored = Vec::new();
    let mut cleared = None;

    for directive in directives {
        for command in &directive.commands {
            if command.top_level() {
                match command {
                    Command::IgnoreFile => {
                        store.rules.clear();
                        cleared = Some(directive.comment.token.text_range());
                    }
                    Command::IgnoreRulesFile(rules) => {
                        ignored.push(directive.comment.token.text_range());
                        store.rules.retain(|rule| !rules.iter().any(|allowed| allowed.name() == rule.name()));
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    if let Some(range) = cleared {
        for ignored_range in ignored {
            let warn = DiagnosticBuilder::warning(file_id, "linter", "ignoring redundant rule ignore directive")
                .secondary(range, "this directive ignores all rules")
                .primary(ignored_range, "this directive is ignored");

            diagnostics.push(warn.into());
        }
    }
}

pub fn apply_node_directives(
    directives: &[Directive],
    node: &SyntaxNode,
    store: &CstRuleStore
) -> Option<CstRuleStore> {
    let comment = node.first_token().and_then(|t| t.comment())?;
    let directive = directives.iter().find(|dir| dir.comment == comment)?;
    let mut store = store.clone();

    for command in &directive.commands {
        match command {
            Command::IgnoreNode(_) => {
                store.rules.clear();
            },
            Command::IgnoreRules(rules, _) => {
                store.rules.retain(|rule| !rules.iter().any(|allowed| allowed.name() == rule.name()));
            },
            _ => {}
        }
    }
    Some(store)
}

pub fn skip_node(
    directives: &[Directive],
    node: &SyntaxNode,
    rule: &Box<dyn CstRule>
) -> bool {
    if let Some(comment) = node.first_token().and_then(|t| t.comment()) {
        if let Some(directive) = directives.iter().find(|dir| dir.comment == comment) {
            for command in &directive.commands {
                match command {
                    Command::IgnoreNode(_) => {
                        return true;
                    },
                    Command::IgnoreRules(rules, _) => {
                        if rules.iter().any(|allowed| allowed.name() == rule.name()) {
                            return true;
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    false
}

rule_tests! {
    crate::groups::errors::NoEmpty::default(),
    err: {
        "{}",
        "
        // rslint-ignore no-empty
        {}

        {}
        "
    },
    ok: {
        "
        // rslint-ignore no-empty

        {}
        ",
        "
        // rslint-ignore no-empty
        {}
        ",
        "
        // rslint-ignore 
        {}
        "
    }
}
