use std::{iter::FusedIterator, slice, sync::Arc};

use erasable::Thin;
use slice_dst::SliceWithHeader;

use crate::{
    green::{GreenElement, GreenElementRef, PackedGreenElement, SyntaxKind},
    TextSize,
};

#[repr(align(2))] // NB: this is an at-least annotation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct GreenNodeHead {
    kind: SyntaxKind,
    text_len: TextSize,
}

/// Internal node in the immutable tree.
/// It has other nodes and tokens as children.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct GreenNode {
    pub(super) data: Thin<Arc<SliceWithHeader<GreenNodeHead, PackedGreenElement>>>,
}

impl GreenNode {
    /// Creates new Node.
    #[inline]
    pub fn new<I>(kind: SyntaxKind, children: I) -> GreenNode
    where
        I: IntoIterator<Item = GreenElement>,
        I::IntoIter: ExactSizeIterator,
    {
        let mut text_len: TextSize = 0.into();
        let children = children
            .into_iter()
            .inspect(|it| text_len += it.text_len())
            .map(PackedGreenElement::from);
        let mut data: Arc<_> = SliceWithHeader::new(
            GreenNodeHead {
                kind,
                text_len: 0.into(),
            },
            children,
        );

        // XXX: fixup `text_len` after construction, because we can't iterate
        // `children` twice.
        Arc::get_mut(&mut data).unwrap().header.text_len = text_len;

        GreenNode { data: data.into() }
    }

    /// Kind of this node.
    #[inline]
    pub fn kind(&self) -> SyntaxKind {
        self.data.header.kind
    }

    /// Returns the length of the text covered by this node.
    #[inline]
    pub fn text_len(&self) -> TextSize {
        self.data.header.text_len
    }

    /// Children of this node.
    #[inline]
    pub fn children(&self) -> Children<'_> {
        Children {
            inner: self.data.slice.iter(),
        }
    }

    pub(crate) fn ptr(&self) -> *const u8 {
        let r: &SliceWithHeader<_, _> = &*self.data;
        r as *const _ as _
    }
}

#[derive(Debug, Clone)]
pub struct Children<'a> {
    inner: slice::Iter<'a, PackedGreenElement>,
}

// NB: forward everything stable that iter::Slice specializes as of Rust 1.39.0
impl ExactSizeIterator for Children<'_> {
    #[inline(always)]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = GreenElementRef<'a>;

    #[inline]
    fn next(&mut self) -> Option<GreenElementRef<'a>> {
        self.inner.next().map(PackedGreenElement::as_ref)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.inner.count()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth(n).map(PackedGreenElement::as_ref)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    #[inline]
    fn fold<Acc, Fold>(self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        for x in self {
            accum = f(accum, x);
        }
        accum
    }
}

impl<'a> DoubleEndedIterator for Children<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(PackedGreenElement::as_ref)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.inner.nth_back(n).map(PackedGreenElement::as_ref)
    }

    #[inline]
    fn rfold<Acc, Fold>(mut self, init: Acc, mut f: Fold) -> Acc
    where
        Fold: FnMut(Acc, Self::Item) -> Acc,
    {
        let mut accum = init;
        while let Some(x) = self.next_back() {
            accum = f(accum, x);
        }
        accum
    }
}

impl FusedIterator for Children<'_> {}
