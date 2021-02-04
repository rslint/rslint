//! Utilities for working with RegEx pattern analysis

use rslint_regex::{AssertionKind, CharacterClassMember, ClassPerlKind, Node, QuantifierKind};
use std::ops::Range;

/// An intermediate-level representation of a RegEx node, this IR is made up of
/// frames instead of being a single node. e.g. `abcd` is one node but 4 HIR frames, this helps
/// greatly in analysis for rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hir {
    pub frames: Vec<HirFrame>,
    pub range: Range<usize>,
}

impl Hir {
    /// Make a new HIR from a node.
    ///
    /// Returns `None` if the node is empty
    pub fn from_node(node: &Node) -> Option<Self> {
        if node == &Node::Empty {
            return None;
        }

        Some(Self {
            frames: HirFrame::frames_from_node(node),
            range: node.span().unwrap().as_range(),
        })
    }

    /// Get all of the frames descending from this HIR node, including frames from other frames
    pub fn descendant_frames(&self) -> impl Iterator<Item = &HirFrame> {
        let mut frames = Vec::with_capacity(self.frames.len());
        for frame in &self.frames {
            frames.push(frame);
            match &frame.ty {
                HirFrameKind::Disjunction(hirs) => {
                    frames.extend(hirs.iter().flat_map(|x| x.descendant_frames()));
                }
                HirFrameKind::Assertion(kind) => match kind {
                    HirAssertionKind::Lookahead(hir)
                    | HirAssertionKind::Lookbehind(hir)
                    | HirAssertionKind::NegativeLookahead(hir)
                    | HirAssertionKind::NegativeLookbehind(hir) => {
                        frames.extend(hir.descendant_frames())
                    }
                    _ => {}
                },
                HirFrameKind::Class(_, members) => {
                    for member in members {
                        match member {
                            HirCharacterClassMember::Range(a, b) => {
                                frames.extend(a.descendant_frames().chain(b.descendant_frames()))
                            }
                            HirCharacterClassMember::Single(a) => {
                                frames.extend(a.descendant_frames())
                            }
                        }
                    }
                }
                HirFrameKind::Quantifier(hir, _, _) | HirFrameKind::Group(_, hir, _) => {
                    frames.extend(hir.descendant_frames())
                }
                _ => {}
            }
        }
        frames.into_iter()
    }

    pub fn is(&self, src: impl AsRef<str>, text: impl AsRef<str>) -> bool {
        self.text(src.as_ref()) == text.as_ref()
    }

    pub fn text<'src>(&self, src: &'src str) -> &'src str {
        &src[self.range.clone()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirFrame {
    pub ty: HirFrameKind,
    pub range: Range<usize>,
}

impl HirFrame {
    pub fn frames_from_node(node: &Node) -> Vec<Self> {
        match node {
            Node::Empty => unreachable!(),
            Node::Disjunction(span, nodes) => {
                let hirs = nodes.iter().filter_map(|x| Hir::from_node(x)).collect();
                vec![HirFrame {
                    ty: HirFrameKind::Disjunction(hirs),
                    range: span.as_range(),
                }]
            }
            Node::Assertion(span, kind) => {
                let new_kind = match kind {
                    AssertionKind::StartOfLine => HirAssertionKind::StartOfLine,
                    AssertionKind::EndOfLine => HirAssertionKind::EndOfLine,
                    AssertionKind::WordBoundary => HirAssertionKind::WordBoundary,
                    AssertionKind::NonWordBoundary => HirAssertionKind::NonWordBoundary,
                    AssertionKind::Lookahead(node) => {
                        if **node == Node::Empty {
                            return vec![];
                        }
                        HirAssertionKind::Lookahead(Box::new(Hir::from_node(node).unwrap()))
                    }
                    AssertionKind::Lookbehind(node) => {
                        if **node == Node::Empty {
                            return vec![];
                        }
                        HirAssertionKind::Lookbehind(Box::new(Hir::from_node(node).unwrap()))
                    }
                    AssertionKind::NegativeLookahead(node) => {
                        if **node == Node::Empty {
                            return vec![];
                        }
                        HirAssertionKind::NegativeLookahead(Box::new(Hir::from_node(node).unwrap()))
                    }
                    AssertionKind::NegativeLookbehind(node) => {
                        if **node == Node::Empty {
                            return vec![];
                        }
                        HirAssertionKind::NegativeLookbehind(Box::new(
                            Hir::from_node(node).unwrap(),
                        ))
                    }
                };
                vec![HirFrame {
                    ty: HirFrameKind::Assertion(new_kind),
                    range: span.as_range(),
                }]
            }
            Node::Alternative(_, nodes) => nodes
                .iter()
                .filter_map(|x| {
                    if x == &Node::Empty {
                        None
                    } else {
                        Some(HirFrame::frames_from_node(node))
                    }
                })
                .flatten()
                .collect(),
            Node::Literal(span, c) => {
                vec![HirFrame {
                    ty: HirFrameKind::Char(*c),
                    range: span.as_range(),
                }]
            }
            Node::PerlClass(span, kind, negated) => {
                vec![HirFrame {
                    ty: HirFrameKind::PerlClass(*negated, kind.to_owned()),
                    range: span.as_range(),
                }]
            }
            Node::BackReference(span, num) => {
                vec![HirFrame {
                    ty: HirFrameKind::Backref(num.to_string()),
                    range: span.as_range(),
                }]
            }
            Node::NamedBackReference(span, string) => {
                vec![HirFrame {
                    ty: HirFrameKind::Backref(string.clone()),
                    range: span.as_range(),
                }]
            }
            Node::Dot(span) => {
                vec![HirFrame {
                    ty: HirFrameKind::Any,
                    range: span.as_range(),
                }]
            }
            Node::CharacterClass(span, class) => {
                vec![HirFrame {
                    ty: HirFrameKind::Class(
                        class.negated,
                        class
                            .members
                            .iter()
                            .filter_map(|x| {
                                Some(match x {
                                    CharacterClassMember::Single(node) => {
                                        HirCharacterClassMember::Single(Hir::from_node(node)?)
                                    }
                                    CharacterClassMember::Range(a, b) => {
                                        HirCharacterClassMember::Range(
                                            Hir::from_node(a)?,
                                            Hir::from_node(b)?,
                                        )
                                    }
                                })
                            })
                            .collect(),
                    ),
                    range: span.as_range(),
                }]
            }
            Node::Group(span, group) => {
                vec![HirFrame {
                    ty: HirFrameKind::Group(
                        group.name.to_owned(),
                        if let Some(node) = Hir::from_node(&group.inner) {
                            Box::new(node)
                        } else {
                            return vec![];
                        },
                        group.noncapturing,
                    ),
                    range: span.as_range(),
                }]
            }
            Node::Quantifier(span, node, kind, lazy) => {
                vec![HirFrame {
                    ty: HirFrameKind::Quantifier(
                        Box::new(Hir::from_node(node).expect("empty node with a quantifier")),
                        kind.to_owned(),
                        *lazy,
                    ),
                    range: span.as_range(),
                }]
            }
        }
    }

    pub fn is(&self, src: impl AsRef<str>, text: impl AsRef<str>) -> bool {
        self.text(src.as_ref()) == text.as_ref()
    }

    pub fn text<'src>(&self, src: &'src str) -> &'src str {
        &src[self.range.clone()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirFrameKind {
    Empty,
    Disjunction(Vec<Hir>),
    Assertion(HirAssertionKind),
    Char(char),
    Any,
    Backref(String),
    Class(bool, Vec<HirCharacterClassMember>),
    Quantifier(Box<Hir>, QuantifierKind, bool),
    Group(Option<String>, Box<Hir>, bool),
    PerlClass(bool, ClassPerlKind),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirAssertionKind {
    StartOfLine,
    EndOfLine,
    WordBoundary,
    NonWordBoundary,
    Lookahead(Box<Hir>),
    NegativeLookahead(Box<Hir>),
    Lookbehind(Box<Hir>),
    NegativeLookbehind(Box<Hir>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirCharacterClassMember {
    Range(Hir, Hir),
    Single(Hir),
}

impl HirCharacterClassMember {
    /// Check if this class member equals a string.
    ///
    /// # Warning
    ///
    /// This does not check for the presence of `-` in the case of a range because it assumes
    /// it is always there because of the grammar, therefore omitting the `-` may yield strange/incorrect results
    pub fn is(&self, src: impl AsRef<str>, text: impl AsRef<str>) -> bool {
        match self {
            HirCharacterClassMember::Range(a, b) => {
                let text = text.as_ref();
                let src = src.as_ref();
                text.starts_with(a.text(src)) && text.ends_with(b.text(src))
            }
            HirCharacterClassMember::Single(hir) => hir.is(src, text),
        }
    }
}
