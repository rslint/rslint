//! All directive command implementations.

use super::{Component, ComponentKind, Instruction};
use crate::CstRule;
use rslint_lexer::SyntaxKind;
use rslint_parser::SyntaxNode;
use std::ops::Range;

/// All different directive commands.
#[derive(Debug, Clone)]
pub enum Command {
    /// Ignore all rules in the whole file.
    IgnoreFile,
    /// Ignore only a subset of rules in the whole file.
    IgnoreFileRules(Vec<Box<dyn CstRule>>),
    /// Ignore all rules for a specific `SyntaxNode`.
    IgnoreNode(SyntaxNode),
    /// Ignore only a subset of rules for a specific `SyntaxNode`.
    IgnoreNodeRules(SyntaxNode, Vec<Box<dyn CstRule>>),
    /// Ignore all rules in a range of lines.
    IgnoreUntil(Range<usize>),
    /// Ignore only a subset of rules in a range of lines.
    IgnoreUntilRules(Range<usize>, Vec<Box<dyn CstRule>>),
}

impl Command {
    pub fn instructions() -> Box<[Box<[Instruction]>]> {
        use Instruction::*;

        vec![vec![
            CommandName("ignore"),
            Repetition(Box::new(RuleName), SyntaxKind::COMMA),
            Optional(vec![
                Literal("until"),
                Either(Box::new(Literal("eof")), Box::new(Number)),
            ]),
        ]
        .into_boxed_slice()]
        .into_boxed_slice()
    }

    /// Takes a parsed `Directive`, and tries to convert it into a `Command`.
    pub fn parse(components: &[Component], line: usize, node: Option<SyntaxNode>) -> Option<Self> {
        let Component { kind, .. } = components.first()?;
        let name = match kind {
            ComponentKind::CommandName(name) => name.as_str(),
            _ => return None,
        };

        match name {
            "ignore" => parse_ignore_command(components, line, node),
            _ => None,
        }
    }
}

fn parse_ignore_command(
    components: &[Component],
    line: usize,
    node: Option<SyntaxNode>,
) -> Option<Command> {
    // TODO: We can probably warn the user about directives like this:
    // ```
    // // rslint-ignore no-empty until eof
    // if (true) {}
    // ```
    // because this will be parsed as a `IgnoreNodeRules` and thus
    // ignores the `until eof` part, which may not be obivous when looking at it.

    if let Some(rules) = components.get(1).and_then(|c| c.kind.repetition()) {
        let rules = rules
            .iter()
            .flat_map(|c| c.kind.rule())
            .collect::<Vec<_>>();

        if components
            .get(2)
            .and_then(|c| c.kind.literal())
            .map_or(false, |l| l == "until")
        {
            match components.get(3).map(|c| &c.kind)? {
                ComponentKind::Literal("eof") => {
                    Some(Command::IgnoreUntilRules(line..usize::max_value(), rules))
                }
                ComponentKind::Number(val) => {
                    Some(Command::IgnoreUntilRules(line..line + *val as usize, rules))
                }
                _ => None,
            }
        } else if let Some(node) = node {
            Some(Command::IgnoreNodeRules(node, rules))
        } else {
            Some(Command::IgnoreFileRules(rules))
        }
    } else if components
        .get(1)
        .and_then(|c| c.kind.literal())
        .map_or(false, |l| l == "until")
    {
        match components.get(2).map(|c| &c.kind)? {
            ComponentKind::Literal("eof") => Some(Command::IgnoreUntil(line..usize::max_value())),
            ComponentKind::Number(val) => Some(Command::IgnoreUntil(line..line + *val as usize)),
            _ => None,
        }
    } else if let Some(node) = node {
        Some(Command::IgnoreNode(node))
    } else {
        Some(Command::IgnoreFile)
    }
}
