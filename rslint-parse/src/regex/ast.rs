use crate::span::Span;

/// A structure representing an ECMAScript regex pattern as a syntax tree
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Pattern {
    pub items: Vec<PatternItem>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum RegexFlags {
    Global,
    Insensitive,
    Multiline,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum PatternItem {
    Disjunction(Disjunction),
    Alternative(Alternative),
    Placeholder,
}

/// A pattern accepting "either or", e.g. `a|b` (either `a` or `b`)
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Disjunction {
    pub span: Span,
    pub left: Box<PatternItem>,
    pub right: Box<PatternItem>,
}

/// A pattern providing a list of terms, e.g `\Ba`
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Alternative {
    pub span: Span,
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Term {
    Assertion,
    Atom,
}

/// A pattern which either succeeds or fails in matching a pattern, composed of Anchors and Lookarounds, e.g. `$` and `^` and `(?= ...)`
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Assertion {
    /// Anchors
    /// ^
    StartOfString,
    /// $
    EndOfString,
    /// \b
    WordBoundary,
    /// \B
    NonWordBoundary,

    /// Lookarounds
    /// (?= ...)
    PositiveLookahead(Disjunction),
    /// (?! ...)
    NegativeLookahead(Disjunction),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Atom {
    /// A literal source character such as `a`
    Character(char),
    /// A dot (any character)
    Dot,
    /// An escaped source character such as `\]`
    Escaped(char),
    /// A group such as `(a)`
    Group(Disjunction),
    /// `(?: ...)` matches everything inside without creating a group
    NonGroup(Disjunction),
    // TODO: class
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Quantifier {
    /// Whether the quantifier is lazy, e.g. `a+?`
    pub lazy: bool,
    pub prefix: QuantifierPrefix,
}

// TODO: Currently we use u64s which should be enough for any sane human being, 
/// but this might need to become a bigint to be sPeC cOmPlIaNt
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum QuantifierPrefix {
    /// *
    ZeroOrMore,
    /// +
    OneOrMore,
    /// ?
    ZeroOrOne,
    /// {n}
    ExactlyN(u64),
    /// {n,}
    NOrMore(u64),
    /// {n,m}
    BetweenNAndM(u64, u64),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct CharacterClass {
    pub span: Span,
    /// Whether a character class indicates "not in" or "not in the range of", e.g. `[^a]`
    pub inverted: bool,
    /// The items included in this character class, e.g. in `[a-zbc]` the items are `a-z`, `b`, and `c`
    pub items: Vec<CharacterClassMember>,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum CharacterClassMember {
    /// A range such as `a-Z`
    Range(CharacterClassAtom, CharacterClassAtom),
    Character(CharacterClassAtom),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum CharacterClassAtom {
    Character(char),
    Escaped(char),
}

impl CharacterClassAtom {
    pub fn char(&self) -> char {
        match self {
            CharacterClassAtom::Character(data) => *data,
            CharacterClassAtom::Escaped(data) => *data,
        }
    }
}