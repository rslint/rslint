use super::lexer::{Lexer, Token};
use crate::CstRuleStore;
use rslint_lexer::SyntaxKind;
use rslint_parser::SyntaxNode;

/// A string that denotes that start of a directive (`rslint-`).
pub const DECLARATOR: &str = "rslint-";

pub struct DirectivesParser {
    /// The root node of a file, `SCRIPT` or `MODULE`.
    root: SyntaxNode,
    file_id: usize,
}

impl DirectivesParser {
    /// Create a new `DirectivesParser` with a root of a file.
    ///
    /// # Panics
    ///
    /// If the given `root` is not `SCRIPT` or `MODULE`.
    pub fn new(root: SyntaxNode, file_id: usize) -> Self {
        assert!(matches!(
            root.kind(),
            SyntaxKind::SCRIPT | SyntaxKind::MODULE
        ));

        Self { root, file_id }
    }
}
