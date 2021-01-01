//! All directive command implementations.

use super::{Component, ComponentKind, Instruction};
use crate::{CstRule, File};
use rslint_lexer::SyntaxKind;
use rslint_parser::SyntaxNode;

/// A structure describing a command.
#[derive(Debug, Clone)]
pub struct CommandDescriptor {
    pub instructions: Box<[Instruction]>,
    pub docs: &'static str,
    pub name: &'static str,
}

/// Get all of the possible command descriptors
pub fn get_command_descriptors() -> Box<[CommandDescriptor]> {
    vec![
        ignore_command_descriptor(),
        disable_command_descriptor(),
        enable_command_descriptor(),
    ]
    .into_boxed_slice()
}

pub fn ignore_command_descriptor() -> CommandDescriptor {
    use Instruction::*;

    CommandDescriptor {
        instructions: vec![
            CommandName("ignore"),
            Repetition(Box::new(RuleName), SyntaxKind::COMMA),
            Optional(vec![
                Literal("until"),
                Either(Box::new(Literal("eof")), Box::new(Number)),
            ]),
        ]
        .into_boxed_slice(),
        docs: "ignore all or some rules on a file or a statement/declaration",
        name: "ignore",
    }
}

pub fn disable_command_descriptor() -> CommandDescriptor {
    use Instruction::*;

    CommandDescriptor {
        instructions: vec![
            CommandName("disable"),
            Repetition(Box::new(RuleName), SyntaxKind::COMMA),
        ]
        .into_boxed_slice(),
        docs: "disable the linter for some rules or all rules until a matching enable command is found",
        name: "disable",
    }
}

pub fn enable_command_descriptor() -> CommandDescriptor {
    use Instruction::*;

    CommandDescriptor {
        instructions: vec![CommandName("enable")].into_boxed_slice(),
        docs: "enable the linter after a disable command",
        name: "enable",
    }
}

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

    /// Disable the linter for some rules or all rules from some line until the next enable command
    Disable(usize, Vec<Box<dyn CstRule>>),
    /// Enable the linter after a disable command
    Enable(usize),
}

impl Command {
    /// Takes a parsed `Directive`, and tries to convert it into a `Command`.
    pub fn parse(
        components: &[Component],
        line: usize,
        node: Option<SyntaxNode>,
        top_level: bool,
        file: &File,
    ) -> Option<Self> {
        let Component { kind, .. } = components.first()?;
        let name = match kind {
            ComponentKind::CommandName(name) => name.as_str(),
            _ => return None,
        };

        match name {
            "ignore" => parse_ignore_command(components, node, top_level),
            "disable" => Some(parse_disable_command(components, line, file)),
            "enable" => Some(parse_enable_command(line, file)),
            _ => None,
        }
    }
}

fn parse_ignore_command(
    components: &[Component],
    node: Option<SyntaxNode>,
    top_level: bool,
) -> Option<Command> {
    if let Some(rules) = components.get(1).and_then(|c| c.kind.repetition()) {
        let rules = rules.iter().flat_map(|c| c.kind.rule()).collect::<Vec<_>>();

        if let Some(node) = node {
            Some(Command::IgnoreNodeRules(node, rules))
        } else {
            Some(Command::IgnoreFileRules(rules))
        }
    } else if let Some(node) = node {
        Some(Command::IgnoreNode(node))
    } else if top_level {
        Some(Command::IgnoreFile)
    } else {
        None
    }
}

fn parse_disable_command(components: &[Component], line: usize, file: &File) -> Command {
    let line_start = file
        .line_start(line)
        .expect("parse_disable_command was given an out of bounds line index");

    if let Some(rules) = components.get(1).and_then(|c| c.kind.repetition()) {
        let rules = rules.iter().flat_map(|c| c.kind.rule()).collect::<Vec<_>>();
        Command::Disable(line_start, rules)
    } else {
        Command::Disable(line_start, vec![])
    }
}

fn parse_enable_command(line: usize, file: &File) -> Command {
    let line_start = file
        .line_start(line)
        .expect("parse_enable_command was given an out of bounds line index");

    Command::Enable(line_start)
}
