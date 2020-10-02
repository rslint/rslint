//! A bit-set of `SyntaxKind`s.

use crate::SyntaxKind;

/// A bit-set of `SyntaxKind`s
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenSet(u128);

impl TokenSet {
    pub const EMPTY: TokenSet = TokenSet(0);

    pub const fn singleton(kind: SyntaxKind) -> TokenSet {
        TokenSet(mask(kind))
    }

    pub const fn union(self, other: TokenSet) -> TokenSet {
        TokenSet(self.0 | other.0)
    }

    pub fn contains(&self, kind: SyntaxKind) -> bool {
        self.0 & mask(kind) != 0
    }
}

const fn mask(kind: SyntaxKind) -> u128 {
    1u128 << (kind as usize)
}

/// Utility macro for making a new token set
#[macro_export]
macro_rules! token_set {
    ($($t:expr),*) => { TokenSet::EMPTY$(.union(TokenSet::singleton($t)))* };
    ($($t:expr),* ,) => { token_set!($($t),*) };
}
