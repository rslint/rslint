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
mod parser;

use std::fmt;

pub use self::parser::*;

use crate::{rule_tests, CstRule, CstRuleStore, Diagnostic, SyntaxNode};
use rslint_parser::{util::*, SmolStr, TextRange, TextSize};

// TODO: More complex warnings, things like ignoring node directives because of file level directives

#[derive(Debug, Clone)]
pub enum ComponentKind {
    ///  This component is a rule name (e.g. `no-extra-boolean-cast` or `no-empty-block`)
    RuleName(SmolStr),
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
    Repetition(Box<Instruction>, &'static str),
    Either(Box<Instruction>, Box<Instruction>),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let slice = match self {
            Instruction::RuleName => "rule name",
            Instruction::Number => "number",
            Instruction::CommandName(_) => "command name",
            Instruction::Literal(_) => "literal",
            Instruction::Optional(all) => {
                let one_of = all.iter().map(ToString::to_string).collect::<Vec<_>>();
                return write!(f, "one of {}", one_of.join(", "));
            }
            Instruction::Repetition(insn, _) => return write!(f, "sequence of {}", insn),
            Instruction::Either(left, right) => return write!(f, "{} or {}", left, right),
        };
        write!(f, "{}", slice)
    }
}

/// Any command that is given to the linter using an inline comment.
#[derive(Debug, Clone)]
pub struct Directive {
    comment: Comment,
    components: Vec<Component>,
}

impl Directive {
    /// Finds the component which contains the given index in his span.
    pub fn component_at(&self, idx: TextSize) -> Option<&Component> {
        self.components.iter().find(|c| c.range.contains(idx))
    }
}

/// Apply file level directives to a store and add their respective diagnostics to the pool of diagnostics.
/// for file level ignores this will clear all the rules from the store.
///
/// This method furthermore issues more contextual warnings like disabling a rule after
/// the entire file has been disabled.
pub fn apply_top_level_directives(
    _directives: &[Directive],
    _store: &mut CstRuleStore,
    _diagnostics: &mut Vec<Diagnostic>,
    _file_id: usize,
) {
    todo!()
}

pub fn apply_node_directives(
    _directives: &[Directive],
    _node: &SyntaxNode,
    _store: &CstRuleStore,
) -> Option<CstRuleStore> {
    todo!()
}

pub fn skip_node(_directives: &[Directive], _node: &SyntaxNode, _rule: &dyn CstRule) -> bool {
    todo!()
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
