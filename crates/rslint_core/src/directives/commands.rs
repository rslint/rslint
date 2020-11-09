//! All directive command implementations.

use super::{Component, ComponentKind, Directive};
use crate::CstRule;
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
    /// Takes a parsed `Directive`, and tries to convert it into a `Command`.
    pub fn parse(
        Directive {
            node, components, ..
        }: Directive,
    ) -> Option<Self> {
        let Component { kind, range } = components.first()?;
        let name = match kind {
            ComponentKind::CommandName(name) => name.as_str(),
            _ => return None,
        };

        let new_components = &components[1..];
        let cmd = match name {
            "ignore" => todo!(),
            _ => return None,
        };
        Some(cmd)
    }
}
