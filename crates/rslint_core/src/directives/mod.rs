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

pub(self) mod lexer;

mod commands;
mod parser;

pub use self::commands::*;
pub use self::parser::*;

use crate::{rule_tests, CstRule, CstRuleStore, Diagnostic, SyntaxNode};
use rslint_lexer::SyntaxKind;
use rslint_parser::{util::*, SmolStr, TextRange, TextSize};

// TODO: More complex warnings, things like ignoring node directives because of file level directives

#[derive(Debug, Clone)]
pub enum ComponentKind {
    /// This component is a rule name (e.g. `no-extra-boolean-cast` or `no-empty-block`)
    ///
    /// The directive parser will not verify if the rule name is valid. This has to be done
    /// separately.
    Rule(Box<dyn CstRule>),
    /// This component is the name of a directive command (e.g. `ignore`)
    CommandName(SmolStr),
    /// A number that is parsed by the [`Number`] instruction.
    ///
    /// [`Number`]: ./enum.Instruction.html
    Number(u64),
    /// Any literal that was parsed by the [`Literal`] instruction.
    ///
    /// [`Literal`]: ./enum.Instruction.html
    Literal(&'static str),
    /// A sequence list of parsed `ComponentKind`s.
    Repetition(Vec<Component>),
}

impl ComponentKind {
    /// Returns the documentation that should be shown for this document.
    pub fn documentation(&self) -> Option<&'static str> {
        match self {
            ComponentKind::Rule(rule) => Some(rule.docs()),
            ComponentKind::CommandName(name) => match name.as_ref() {
                "ignore" => Some(
                    "`ignore` will ignore all rules, or any given rules in some range or node.",
                ),
                _ => None,
            },
            _ => None,
        }
    }
}

impl ComponentKind {
    pub fn rule(&self) -> Option<Box<dyn CstRule>> {
        match self {
            ComponentKind::Rule(rule) => Some(rule.clone()),
            _ => None,
        }
    }

    pub fn command_name(&self) -> Option<&str> {
        match self {
            ComponentKind::CommandName(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn literal(&self) -> Option<&str> {
        match self {
            ComponentKind::Literal(val) => Some(*val),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<u64> {
        match self {
            ComponentKind::Number(val) => Some(*val),
            _ => None,
        }
    }

    pub fn repetition(&self) -> Option<&[Component]> {
        match self {
            ComponentKind::Repetition(components) => Some(components.as_slice()),
            _ => None,
        }
    }
}

/// A `Component` represents a parsed `Instruction`, that also has a span,
/// so you can find the `Component` at any span in the directive.
#[derive(Debug, Clone)]
pub struct Component {
    pub kind: ComponentKind,
    pub range: TextRange,
}

/// `Instruction`s are used to add directives to the parser.
///
/// Directives are parsed based off all registered instructions.
///
/// # Example
///
/// To add an `ignore` rule, you can add the following instructions:
/// ```ignore
/// # use rslint_core::directives::Instruction::*;
/// # fn main() {
/// vec![
///   CommandName("ignore"),
///   Repetition(RuleName, ","),
///   Optional(vec![
///     Literal("until"),
///     Either(Literal("eof"), Number)
///   ])
/// ]
/// # }
/// ```
#[derive(Debug, Clone)]
pub enum Instruction {
    RuleName,
    Number,

    CommandName(&'static str),
    Literal(&'static str),
    Optional(Vec<Instruction>),
    Repetition(Box<Instruction>, SyntaxKind),
    Either(Box<Instruction>, Box<Instruction>),
}

/// Any command that is given to the linter using an inline comment.
#[derive(Debug, Clone)]
pub struct Directive {
    /// The line number in which the directive comment was parsed.
    pub line: usize,
    pub comment: Comment,
    pub components: Vec<Component>,
    /// Contains the parsed `Command`, but is `None` if the `components`
    /// failed to be parsed as a valid `Command`.
    pub command: Option<Command>,
}

impl Directive {
    /// Finds the component which contains the given index in his span.
    pub fn component_at(&self, idx: TextSize) -> Option<&Component> {
        self.components
            .iter()
            .find(|c| c.range.contains(idx))
            .and_then(|component| {
                if let ComponentKind::Repetition(components) = &component.kind {
                    components.iter().find(|c| c.range.contains(idx))
                } else {
                    Some(component)
                }
            })
    }
}

/// Apply file level directives to a store and add their respective diagnostics to the pool of diagnostics.
/// for file level ignores this will clear all the rules from the store.
///
/// This method furthermore issues more contextual warnings like disabling a rule after
/// the entire file has been disabled.
pub fn apply_top_level_directives(
    directives: &[Directive],
    store: &mut CstRuleStore,
    diagnostics: &mut Vec<DirectiveError>,
    file_id: usize,
) {
    // TODO: More complex warnings, things like ignoring node directives because of file level directives

    let mut ignored = vec![];
    let mut cleared = None;

    for directive in directives {
        match &directive.command {
            Some(Command::IgnoreFile) => {
                store.rules.clear();
                cleared = Some(directive.comment.token.text_range());
            }
            Some(Command::IgnoreFileRules(rules)) => {
                ignored.push(directive.comment.token.text_range());
                store
                    .rules
                    .retain(|rule| !rules.iter().any(|allowed| allowed.name() == rule.name()));
            }
            _ => {}
        }
    }

    if let Some(range) = cleared {
        for ignored_range in ignored {
            let warn = Diagnostic::warning(
                file_id,
                "linter",
                "ignoring redundant rule ignore directive",
            )
            .secondary(range, "this directive ignores all rules")
            .primary(ignored_range, "this directive is ignored")
            .unnecessary();

            diagnostics.push(DirectiveError::new(warn, DirectiveErrorKind::Other));
        }
    }
}

pub fn apply_node_directives(
    directives: &[Directive],
    node: &SyntaxNode,
    store: &CstRuleStore,
) -> Option<CstRuleStore> {
    let comment = node.first_token().and_then(|t| t.comment())?;
    let directive = directives.iter().find(|dir| dir.comment == comment)?;
    let mut store = store.clone();

    match &directive.command {
        Some(Command::IgnoreNode(_)) => {
            store.rules.clear();
        }
        Some(Command::IgnoreNodeRules(_, rules)) => {
            store
                .rules
                .retain(|rule| !rules.iter().any(|allowed| allowed.name() == rule.name()));
        }
        _ => {}
    }
    Some(store)
}

pub fn skip_node(directives: &[Directive], node: &SyntaxNode, rule: &dyn CstRule) -> bool {
    if let Some(comment) = node.first_token().and_then(|t| t.comment()) {
        if let Some(directive) = directives.iter().find(|dir| dir.comment == comment) {
            match &directive.command {
                Some(Command::IgnoreNode(_)) => {
                    return true;
                }
                Some(Command::IgnoreNodeRules(_, rules)) => {
                    if rules.iter().any(|allowed| allowed.name() == rule.name()) {
                        return true;
                    }
                }
                _ => {}
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
