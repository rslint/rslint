//! The intermediate representation of a RegEx
//! in a tree based structure.

use crate::Span;
use bitflags::bitflags;
use rslint_errors::Diagnostic;

bitflags! {
    pub struct Flags: u8 {
        /// With this flag the search looks for all matches, without this flag
        /// only the first match is returned
        const G = 0b00000001;
        /// Multiline mode
        const M = 0b00000010;
        /// Case-insensitive search
        const I = 0b00000100;
        /// "dotall" mode, that allows `.` to match newlines (`\n`)
        const S = 0b00001000;
        /// Enables full unicode support
        const U = 0b00010000;
        /// "Sticky" mode
        const Y = 0b00100000;
    }
}

impl Flags {
    /// Tries to parse a set of regex flags from a given string.
    ///
    /// The `offset` and `file_id` arguments are used in the resulting `Diagnostic`
    /// if an error occurrs.
    pub fn parse(raw_flags: &str, offset: usize, file_id: usize) -> Result<Self, Diagnostic> {
        // TODO: Probably support mulitple errors?
        let mut indicies = [0usize; 6];
        let mut flags = Flags::empty();

        for (idx, c) in raw_flags.chars().enumerate() {
            let flag = match c {
                'g' => Flags::G,
                'm' => Flags::M,
                'i' => Flags::I,
                's' => Flags::S,
                'u' => Flags::U,
                'y' => Flags::Y,
                c => {
                    let idx = idx + offset;
                    let d =
                        Diagnostic::error(file_id, "regex", format!("invalid regex flag: `{}`", c))
                            .primary(idx..idx + 1, "");
                    return Err(d);
                }
            };

            if flags.contains(flag) {
                let first_idx = indicies[flag.ffs()];
                let idx = idx + offset;
                let d =
                    Diagnostic::error(file_id, "regex", format!("duplicate regex flag: `{}`", c))
                        .primary(idx..idx + 1, "flag defined here...")
                        .secondary(
                            first_idx..first_idx + 1,
                            "...but it already was defined here",
                        );
                return Err(d);
            }
            indicies[flag.ffs()] = offset + idx;

            flags |= flag;
        }

        Ok(flags)
    }

    /// Find-First-Set implementation
    fn ffs(&self) -> usize {
        match *self {
            Self::G => 0,
            Self::M => 1,
            Self::I => 2,
            Self::S => 3,
            Self::U => 4,
            Self::Y => 5,
            _ => unreachable!(),
        }
    }
}

/// The structure that represents a regular expression.
///
/// It contains the actual RegEx node, and the flags for this expression.
#[derive(Debug, Clone)]
pub struct RegEx {
    pub node: Node,
    pub flags: Flags,
}

/// The tree structure that is used to represent parsed
/// RegEx patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// An empty regex node.
    Empty,
    /// Alternation node (e.g. `a|b|c`).
    Alternation(Alternation),
    /// A single assertion.
    Assertion(Assertion),
    /// A concatination of regex nodes.
    Concat(Concat),
    /// A single character literal.
    Literal(Literal),
    /// Matches a character class (e.g. `\d` or `\w`).
    Class(Class),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssertionKind {
    /// `^`
    StartOfLine,
    /// `$`
    EndOfLine,
    /// `\b`
    WordBoundary,
    /// `\B`
    NonWordBoundary,
    /// `x(?=y)`
    Lookahead(Box<Node>),
    /// `x(?!y)`
    NegativeLookahead(Box<Node>),
    /// `(?<=y)x`
    Lookbehind(Box<Node>),
    /// `(?<!y)x`
    NegativeLookbehind(Box<Node>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Alternation {
    pub span: Span,
    pub alternatives: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assertion {
    pub span: Span,
    pub kind: AssertionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concat {
    pub span: Span,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Literal {
    pub span: Span,
    pub c: char,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub span: Span,
    pub kind: ClassKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassKind {
    Dot,
    Digit,
    NotDigit,
    Word,
    NotWord,
    Whitespace,
    NotWhitespace,
    HorizontalTab,
    CarriageReturn,
    Linefeed,
    VerticalTab,
    FormFeed,
    Null,
}
