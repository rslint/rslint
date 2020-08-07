//! Extra utlities for untyped syntax nodes, syntax tokens, and AST nodes.

use crate::*;

/// Extensions to rowan's SyntaxNode
pub trait SyntaxNodeExt {
    fn to_node(&self) -> &SyntaxNode;

    /// Get all of the tokens of this node, recursively, including whitespace.
    fn tokens(&self) -> Vec<SyntaxToken> {
        self.to_node()
            .descendants_with_tokens()
            .filter_map(|x| x.into_token())
            .collect()
    }

    /// Get all the tokens of this node, recursively, not including whitespace.
    fn lossy_tokens(&self) -> Vec<SyntaxToken> {
        self.to_node()
            .descendants_with_tokens()
            .filter_map(|x| x.into_token().filter(|token| !token.kind().is_trivia()))
            .collect()
    }

    /// Check if the node is a certain AST node and that it can be casted to it.
    fn is<T: AstNode>(&self) -> bool {
        T::can_cast(self.to_node().kind())
    }

    /// Cast this node to a certain AST node.
    fn to<T: AstNode>(&self) -> Option<T> {
        T::cast(self.to_node().to_owned())
    }
}

impl SyntaxNodeExt for SyntaxNode {
    fn to_node(&self) -> &SyntaxNode {
        self
    }
}

/// Concatenate tokens into a string
pub fn concat_tokens(tokens: &Vec<SyntaxToken>) -> String {
    tokens
        .iter()
        .map(|token| token.text().to_string())
        .collect()
}
