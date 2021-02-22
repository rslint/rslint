use cstree::{
    Direction, GreenNode, GreenToken, SyntaxKind, TextRange, TextSize, TokenAtOffset, WalkEvent,
};
use fxhash::FxHasher;
use lasso::{Rodeo, Spur};
use std::hash::BuildHasherDefault;
use std::{fmt, ops::Deref};
use std::{
    fmt::{Display, Formatter},
    mem::transmute,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsLanguage;

impl cstree::Language for JsLanguage {
    type Kind = rslint_syntax::SyntaxKind;

    fn kind_from_raw(raw: cstree::SyntaxKind) -> rslint_syntax::SyntaxKind {
        rslint_syntax::SyntaxKind::from(raw.0)
    }

    fn kind_to_raw(kind: rslint_syntax::SyntaxKind) -> cstree::SyntaxKind {
        cstree::SyntaxKind(kind.into())
    }
}

pub type Interner = Rodeo<Spur, BuildHasherDefault<FxHasher>>;

type NewSyntaxNode = cstree::SyntaxNode<JsLanguage, (), Interner>;
type NewSyntaxToken = cstree::SyntaxToken<JsLanguage, (), Interner>;
type NewSyntaxText<'a, 'b> = cstree::SyntaxText<'a, 'b, Interner, JsLanguage, (), Interner>;

// SAFETY: all newtypes are repr(transparent), therefore transmuting to and from the newtype
// and the cstree type is sound.

macro_rules! auto_impl_methods {
    ($($name:ident($($param:ident: $ty:ty),*) -> $return_ty:ty),*) => {
        $(
            #[inline]
            pub fn $name(&self, $($param:$ty),*) -> $return_ty {
                self.0.$name($($param),*)
            }
        )*
    }
}

macro_rules! transmutable_methods {
    ($($name:ident -> $return_ty:ty),*) => {
        $(
            #[inline]
            pub fn $name(&self) -> $return_ty {
                // SAFETY: see comment at top of file
                self.0.$name().map(|x| unsafe { transmute(x) })
            }
        )*
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxNode(NewSyntaxNode);

impl SyntaxNode {
    auto_impl_methods! {
        replace_with(replacement: GreenNode) -> GreenNode,
        syntax_kind() -> SyntaxKind,
        kind() -> rslint_syntax::SyntaxKind,
        text_range() -> TextRange,
        green() -> &GreenNode
    }

    transmutable_methods! {
        parent -> Option<&SyntaxNode>,
        ancestors -> impl Iterator<Item = &SyntaxNode>,
        children -> impl Iterator<Item = &SyntaxNode>,
        children_with_tokens -> impl Iterator<Item = SyntaxElementRef>,
        first_child -> Option<&SyntaxNode>,
        first_child_or_token -> Option<SyntaxElementRef>,
        last_child -> Option<&SyntaxNode>,
        last_child_or_token -> Option<SyntaxElementRef>,
        next_sibling -> Option<&SyntaxNode>,
        next_sibling_or_token -> Option<SyntaxElementRef>,
        prev_sibling -> Option<&SyntaxNode>,
        prev_sibling_or_token -> Option<SyntaxElementRef>,
        first_token -> Option<&SyntaxToken>,
        last_token -> Option<&SyntaxToken>,
        descendants -> impl Iterator<Item = &SyntaxNode>,
        descendants_with_tokens -> impl Iterator<Item = SyntaxElementRef>
    }

    #[inline]
    pub fn new_with_resolver(green: GreenNode, resolver: Interner) -> Self {
        Self(NewSyntaxNode::new_root_with_resolver(green, resolver))
    }

    #[inline]
    pub fn next_child_after(&self, n: usize, offset: TextSize) -> Option<&SyntaxNode> {
        self.0
            .next_child_after(n, offset)
            .map(|x| unsafe { transmute(x) })
    }

    #[inline]
    pub fn prev_child_before(&self, n: usize, offset: TextSize) -> Option<&SyntaxNode> {
        self.0
            .prev_child_before(n, offset)
            .map(|x| unsafe { transmute(x) })
    }

    #[inline]
    pub fn next_child_or_token_after(
        &self,
        n: usize,
        offset: TextSize,
    ) -> Option<SyntaxElementRef> {
        self.0
            .next_child_or_token_after(n, offset)
            .map(|x| unsafe { transmute(x) })
    }

    #[inline]
    pub fn prev_child_or_token_before(
        &self,
        n: usize,
        offset: TextSize,
    ) -> Option<SyntaxElementRef> {
        self.0
            .prev_child_or_token_before(n, offset)
            .map(|x| unsafe { transmute(x) })
    }

    #[inline]
    pub fn siblings(&self, direction: Direction) -> impl Iterator<Item = &SyntaxNode> {
        self.0.siblings(direction).map(|x| unsafe { transmute(x) })
    }

    #[inline]
    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElementRef> {
        self.0
            .siblings_with_tokens(direction)
            .map(|x| unsafe { transmute(x) })
    }

    #[inline]
    pub fn token_at_offset(&self, offset: TextSize) -> TokenAtOffset<SyntaxToken> {
        self.0
            .token_at_offset(offset)
            .map(|x| unsafe { transmute(x) })
    }

    #[inline]
    pub fn covering_element(&self, range: TextRange) -> SyntaxElementRef {
        unsafe { transmute(self.0.covering_element(range)) }
    }

    #[inline]
    pub fn text(&self) -> SyntaxText {
        unsafe { transmute(self.0.text()) }
    }

    #[inline]
    pub fn preorder_with_tokens(&self) -> impl Iterator<Item = WalkEvent<SyntaxElementRef>> {
        self.0.preorder_with_tokens().map(|x| match x {
            WalkEvent::Enter(t) => WalkEvent::Enter(t.into()),
            WalkEvent::Leave(t) => WalkEvent::Leave(t.into()),
        })
    }
}

impl Display for SyntaxNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxToken(NewSyntaxToken);

impl SyntaxToken {
    auto_impl_methods! {
        syntax_kind() -> SyntaxKind,
        kind() -> rslint_syntax::SyntaxKind,
        text_range() -> TextRange,
        green() -> &GreenToken,
        replace_with(replacement: GreenToken) -> GreenNode,
        text() -> &str
    }

    transmutable_methods! {
        ancestors -> impl Iterator<Item = &SyntaxNode>,
        next_sibling_or_token -> Option<SyntaxElementRef>,
        prev_sibling_or_token -> Option<SyntaxElementRef>,
        next_token -> Option<&SyntaxToken>,
        prev_token -> Option<&SyntaxToken>
    }

    #[inline]
    pub fn parent(&self) -> &SyntaxNode {
        unsafe { transmute(self.0.parent()) }
    }

    #[inline]
    pub fn siblings_with_tokens(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = SyntaxElementRef> {
        self.0
            .siblings_with_tokens(direction)
            .map(|x| unsafe { transmute(x) })
    }
}

impl Display for SyntaxToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type SyntaxElement = NodeOrToken<SyntaxNode, SyntaxToken>;
pub type SyntaxElementRef<'a> = NodeOrToken<&'a SyntaxNode, &'a SyntaxToken>;

impl SyntaxElement {
    #[inline]
    pub fn text_range(&self) -> TextRange {
        match self {
            NodeOrToken::Node(it) => it.text_range(),
            NodeOrToken::Token(it) => it.text_range(),
        }
    }

    #[inline]
    pub fn syntax_kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.syntax_kind(),
            NodeOrToken::Token(it) => it.syntax_kind(),
        }
    }

    #[inline]
    pub fn kind(&self) -> rslint_syntax::SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    #[inline]
    pub fn parent(&self) -> Option<&SyntaxNode> {
        match self {
            NodeOrToken::Node(it) => it.parent(),
            NodeOrToken::Token(it) => Some(it.parent()),
        }
    }

    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = &SyntaxNode> {
        match self {
            NodeOrToken::Node(it) => it.ancestors(),
            NodeOrToken::Token(it) => it.parent().ancestors(),
        }
    }

    #[inline]
    pub fn first_token(&self) -> Option<&SyntaxToken> {
        match self {
            NodeOrToken::Node(it) => it.first_token(),
            NodeOrToken::Token(it) => Some(it),
        }
    }

    #[inline]
    pub fn last_token(&self) -> Option<&SyntaxToken> {
        match self {
            NodeOrToken::Node(it) => it.last_token(),
            NodeOrToken::Token(it) => Some(it),
        }
    }

    #[inline]
    pub fn next_sibling_or_token(&self) -> Option<SyntaxElementRef<'_>> {
        match self {
            NodeOrToken::Node(it) => it.next_sibling_or_token(),
            NodeOrToken::Token(it) => it.next_sibling_or_token(),
        }
    }

    #[inline]
    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElementRef<'_>> {
        match self {
            NodeOrToken::Node(it) => it.prev_sibling_or_token(),
            NodeOrToken::Token(it) => it.prev_sibling_or_token(),
        }
    }
}

impl<'a> SyntaxElementRef<'a> {
    #[inline]
    pub fn text_range(&self) -> TextRange {
        match self {
            NodeOrToken::Node(it) => it.text_range(),
            NodeOrToken::Token(it) => it.text_range(),
        }
    }

    #[inline]
    pub fn syntax_kind(&self) -> SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.syntax_kind(),
            NodeOrToken::Token(it) => it.syntax_kind(),
        }
    }

    #[inline]
    pub fn kind(&self) -> rslint_syntax::SyntaxKind {
        match self {
            NodeOrToken::Node(it) => it.kind(),
            NodeOrToken::Token(it) => it.kind(),
        }
    }

    #[inline]
    pub fn parent(&self) -> Option<&'a SyntaxNode> {
        match self {
            NodeOrToken::Node(it) => it.parent(),
            NodeOrToken::Token(it) => Some(it.parent()),
        }
    }

    #[inline]
    pub fn ancestors(&self) -> impl Iterator<Item = &'a SyntaxNode> {
        match self {
            NodeOrToken::Node(it) => it.ancestors(),
            NodeOrToken::Token(it) => it.parent().ancestors(),
        }
    }

    #[inline]
    pub fn first_token(&self) -> Option<&'a SyntaxToken> {
        match self {
            NodeOrToken::Node(it) => it.first_token(),
            NodeOrToken::Token(it) => Some(it),
        }
    }

    #[inline]
    pub fn last_token(&self) -> Option<&'a SyntaxToken> {
        match self {
            NodeOrToken::Node(it) => it.last_token(),
            NodeOrToken::Token(it) => Some(it),
        }
    }

    #[inline]
    pub fn next_sibling_or_token(&self) -> Option<SyntaxElementRef<'a>> {
        match self {
            NodeOrToken::Node(it) => it.next_sibling_or_token(),
            NodeOrToken::Token(it) => it.next_sibling_or_token(),
        }
    }

    #[inline]
    pub fn prev_sibling_or_token(&self) -> Option<SyntaxElementRef<'a>> {
        match self {
            NodeOrToken::Node(it) => it.prev_sibling_or_token(),
            NodeOrToken::Token(it) => it.prev_sibling_or_token(),
        }
    }

    #[inline]
    pub fn token_at_offset(&self, offset: TextSize) -> TokenAtOffset<SyntaxToken> {
        assert!(self.text_range().start() <= offset && offset <= self.text_range().end());
        match self {
            NodeOrToken::Token(token) => TokenAtOffset::Single((*token).clone()),
            NodeOrToken::Node(node) => node.token_at_offset(offset),
        }
    }
}

impl From<SyntaxElementRef<'_>> for SyntaxElement {
    fn from(elem: SyntaxElementRef<'_>) -> Self {
        match elem {
            NodeOrToken::Node(n) => NodeOrToken::Node(n.to_owned()),
            NodeOrToken::Token(t) => NodeOrToken::Token(t.to_owned()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeOrToken<N, T> {
    Node(N),
    Token(T),
}

impl<N, T> NodeOrToken<N, T> {
    pub fn into_node(self) -> Option<N> {
        match self {
            NodeOrToken::Node(node) => Some(node),
            NodeOrToken::Token(_) => None,
        }
    }

    pub fn into_token(self) -> Option<T> {
        match self {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(token) => Some(token),
        }
    }

    pub fn as_node(&self) -> Option<&N> {
        match self {
            NodeOrToken::Node(node) => Some(node),
            NodeOrToken::Token(_) => None,
        }
    }

    pub fn as_token(&self) -> Option<&T> {
        match self {
            NodeOrToken::Node(_) => None,
            NodeOrToken::Token(token) => Some(token),
        }
    }
}

impl<N: Display, T: Display> Display for NodeOrToken<N, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NodeOrToken::Node(n) => n.fmt(f),
            NodeOrToken::Token(t) => t.fmt(f),
        }
    }
}

impl From<cstree::NodeOrToken<NewSyntaxNode, NewSyntaxToken>>
    for NodeOrToken<SyntaxNode, SyntaxToken>
{
    fn from(other: cstree::NodeOrToken<NewSyntaxNode, NewSyntaxToken>) -> Self {
        unsafe {
            match other {
                cstree::NodeOrToken::Node(n) => NodeOrToken::Node(transmute(n)),
                cstree::NodeOrToken::Token(t) => NodeOrToken::Token(transmute(t)),
            }
        }
    }
}

impl<'a, 'b> From<cstree::NodeOrToken<&'a NewSyntaxNode, &'b NewSyntaxToken>>
    for NodeOrToken<&'a SyntaxNode, &'b SyntaxToken>
{
    fn from(other: cstree::NodeOrToken<&'a NewSyntaxNode, &'b NewSyntaxToken>) -> Self {
        unsafe {
            match other {
                cstree::NodeOrToken::Node(n) => NodeOrToken::Node(transmute(n)),
                cstree::NodeOrToken::Token(t) => NodeOrToken::Token(transmute(t)),
            }
        }
    }
}

impl<T> From<SyntaxNode> for NodeOrToken<SyntaxNode, T> {
    fn from(node: SyntaxNode) -> Self {
        Self::Node(node)
    }
}

impl<N> From<SyntaxToken> for NodeOrToken<N, SyntaxToken> {
    fn from(token: SyntaxToken) -> Self {
        Self::Token(token)
    }
}

impl<'a, T> From<&'a SyntaxNode> for NodeOrToken<&'a SyntaxNode, T> {
    fn from(node: &'a SyntaxNode) -> Self {
        Self::Node(node)
    }
}

impl<'a, N> From<&'a SyntaxToken> for NodeOrToken<N, &'a SyntaxToken> {
    fn from(token: &'a SyntaxToken) -> Self {
        Self::Token(token)
    }
}

impl<'a> From<&'a NodeOrToken<SyntaxNode, SyntaxToken>>
    for NodeOrToken<&'a SyntaxNode, &'a SyntaxToken>
{
    fn from(elem: &'a NodeOrToken<SyntaxNode, SyntaxToken>) -> Self {
        match elem {
            NodeOrToken::Node(n) => NodeOrToken::Node(n),
            NodeOrToken::Token(t) => NodeOrToken::Token(t),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq)]
pub struct SyntaxText<'a, 'b>(NewSyntaxText<'a, 'b>);

impl<'a, 'b> Deref for SyntaxText<'a, 'b> {
    type Target = NewSyntaxText<'a, 'b>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'b> SyntaxText<'a, 'b> {
    pub fn slice(&self, range: TextRange) -> Self {
        Self(self.0.slice(range))
    }
}

impl<'a, 'b> PartialEq<&'_ str> for SyntaxText<'a, 'b> {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl Display for SyntaxText<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq<SyntaxText<'_, '_>> for &'_ str {
    fn eq(&self, other: &SyntaxText) -> bool {
        *self == other.0
    }
}
