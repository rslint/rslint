//! The intermediate representation of a RegEx
//! in a tree based structure.

use crate::Span;
use bitflags::bitflags;

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

/// The structure that represents a regular expression.
///
/// It contains the actual RegEx node, and the flags for this expression.
#[derive(Debug, Clone)]
pub struct Regex {
    pub node: Node,
    pub flags: Flags,
}

/// The tree structure that is used to represent parsed
/// RegEx patterns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    /// An empty regex node.
    Empty,
    /// An "either or". (e.g. `a|b`)
    Disjunction(Span, Vec<Node>),
    /// A single assertion.
    Assertion(Span, AssertionKind),
    /// A concatenation of regex nodes. (e.g. `ab`)
    Alternative(Span, Vec<Node>),
    /// A single character literal.
    Literal(Span, char),
    /// Matches a character class (e.g. `\d` or `\w`).
    ///
    /// The bool argument indicates if this perl class is negated.
    PerlClass(Span, ClassPerlKind, bool),
    /// A back reference to a previous group (`\1`, `\2`, ...).
    BackReference(Span, u32),
    /// A `.` that matches everything.
    Dot(Span),
    /// A class of multiple characters such as `[A-Z0-9]`
    CharacterClass(Span, CharacterClass),
    /// A grouped pattern
    Group(Span, Group),
    /// A quantifier which optionally matches or matches multiple times.
    /// `bool` indicates whether a lazy quantifier (`?`) is present after it.
    Quantifier(Span, Box<Node>, QuantifierKind, bool),
    /// A reference to a group using a name
    NamedBackReference(Span, String),
}

/// A grouped pattern which can later be referred to
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    /// Whether this group cannot be later referred to with `$0` for example
    pub noncapturing: bool,
    pub inner: Box<Node>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuantifierKind {
    /// `?`
    Optional,
    /// `*`
    Multiple,
    /// `+`
    AtLeastOne,
    /// `{number}`
    Number(u32),
    /// `{number,number}`. if the second option is None it is "between X and unlimited times"
    Between(u32, Option<u32>),
}

/// A class matching multiple characters or ranges of characters
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharacterClass {
    pub negated: bool,
    pub members: Vec<CharacterClassMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterClassMember {
    Range(Node, Node),
    Single(Node),
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
pub enum ClassPerlKind {
    Digit,
    Word,
    Space,
    Unicode(Option<String>, String),
}
