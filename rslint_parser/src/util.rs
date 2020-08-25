//! Extra utlities for untyped syntax nodes, syntax tokens, and AST nodes.

use crate::*;

/// Extensions to rowan's SyntaxNode
pub trait SyntaxNodeExt {
    #[doc(hidden)]
    fn to_node(&self) -> &SyntaxNode;

    /// Get all of the tokens of this node, recursively, including whitespace and comments.
    fn tokens(&self) -> Vec<SyntaxToken> {
        self.to_node()
            .descendants_with_tokens()
            .filter_map(|x| x.into_token())
            .collect()
    }

    /// Get all the tokens of this node, recursively, not including whitespace and comments.
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
    ///
    /// # Panics
    /// Panics if the underlying node cannot be cast to the AST node
    fn to<T: AstNode>(&self) -> T {
        T::cast(self.to_node().to_owned()).expect(&format!(
            "Tried to cast node as `{:?}` but was unable to cast",
            stringify!(T)
        ))
    }

    /// Try to cast this node to a certain AST node
    fn try_to<T: AstNode>(&self) -> Option<T> {
        T::cast(self.to_node().to_owned())
    }

    /// Compare two syntax nodes by comparing their underlying non-whitespace tokens.
    ///
    /// This is a more accurate way of comparing nodes because it does not count whitespace.
    /// Text based equality counts `foo. bar` and `foo.bar` as different, while this counts them as the same.
    ///
    /// # Examples
    ///
    /// ```
    /// use rslint_parser::{SyntaxNodeExt, parse_expr};
    ///
    /// let left = parse_expr("foo. bar", 0).syntax();
    /// let right = parse_expr("foo.bar", 0).syntax();
    ///
    /// assert!(left.lexical_eq(&right));
    ///
    /// assert_ne!(left.text(), right.text());
    /// ```
    fn lexical_eq(&self, right: &SyntaxNode) -> bool {
        let left = self.lossy_tokens();
        let right = right.lossy_tokens();

        if left.len() == right.len() {
            left.iter()
                .zip(right.iter())
                .all(|(l, r)| l.text() == r.text())
        } else {
            false
        }
    }

    /// Syntax highlight the node's text into an ANSI string.
    /// If stdout and stderr are not terminals, this will return the raw
    /// node text.
    fn color(&self) -> String {
        color(&self.to_node().text().to_string())
    }

    /// Get the text range of this node, not including any leading or trailing whitespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use rslint_parser::{SyntaxNodeExt, parse_expr, TextRange};
    ///
    /// let node = parse_expr(" foo. bar  ", 0).syntax();
    ///
    /// assert_eq!(node.trimmed_range(), TextRange::new(1.into(), 9.into()));
    ///
    /// assert_eq!(node.text_range(), TextRange::new(0.into(), 11.into()));
    /// ```
    fn trimmed_range(&self) -> TextRange {
        let node = self.to_node();
        let tokens = node.lossy_tokens();
        let start = tokens
            .first()
            .map(|t| t.text_range().start())
            .unwrap_or_else(|| 0.into());
        let end = tokens
            .last()
            .map(|t| t.text_range().end())
            .unwrap_or_else(|| 0.into());

        TextRange::new(start, end)
    }

    /// Get the text of this node, not including leading or trailing whitespace
    ///
    /// # Examples
    /// ```
    /// use rslint_parser::{SyntaxNodeExt, parse_expr, TextRange};
    ///
    /// let node = parse_expr(" foo. bar  ", 0).syntax();
    ///
    /// assert_eq!(node.trimmed_text(), "foo. bar");
    /// ```
    fn trimmed_text(&self) -> SyntaxText {
        self.to_node().text().slice(self.to_node().trimmed_range())
    }

    /// Get the directly adjacent previous token before the node.
    /// This could be whitespace (and most of the time it will be)
    /// therefore it is usually more useful to use the lossy version of this
    ///
    /// If the previous element is a node without tokens, the return value will be `None`
    fn prev_adjacent_token(&self) -> Option<SyntaxToken> {
        let node = self.to_node();
        let prev_element = node.prev_sibling_or_token()?;

        match prev_element {
            NodeOrToken::Node(node) => node.tokens().last().cloned(),
            NodeOrToken::Token(token) => Some(token),
        }
    }

    /// Get the directly adjacent next token after the node.
    /// This could be whitespace (and most of the time it will be)
    /// therefore it is usually more useful to use the lossy version of this
    ///
    /// If the next element is a node without tokens, the return value will be `None`
    fn next_adjacent_token(&self) -> Option<SyntaxToken> {
        let node = self.to_node();
        let next_element = node.next_sibling_or_token()?;

        match next_element {
            NodeOrToken::Node(node) => node.tokens().first().cloned(),
            NodeOrToken::Token(token) => Some(token),
        }
    }

    /// Get the directly adjacent previous (non whitespace) token before the node.
    ///
    /// # Examples
    /// ```
    /// use rslint_parser::{SyntaxNodeExt, parse_expr, ast::BinExpr, AstNode};
    ///
    /// let node = parse_expr("2 + 3 * 2", 0).syntax().to::<BinExpr>().rhs().unwrap().syntax().to_owned();
    ///
    /// assert_eq!(node.prev_adjacent_token_lossy().unwrap().text(), "2");
    /// ```
    fn prev_adjacent_token_lossy(&self) -> Option<SyntaxToken> {
        let node = self.to_node();
        let prev_element = node.prev_sibling_or_token()?;

        match prev_element {
            NodeOrToken::Node(node) => node.lossy_tokens().last().cloned(),
            NodeOrToken::Token(token) => {
                if !token.kind().is_trivia() {
                    Some(token)
                } else {
                    for element in node.siblings_with_tokens(Direction::Prev) {
                        match element {
                            NodeOrToken::Token(token) if !token.kind().is_trivia() => {
                                return Some(token)
                            }
                            NodeOrToken::Node(node) => return node.lossy_tokens().last().cloned(),
                            _ => {}
                        }
                    }
                    None
                }
            }
        }
    }

    /// Get the directly adjacent next (non whitespace) token after the node.
    ///
    /// # Examples
    /// ```
    /// use rslint_parser::{SyntaxNodeExt, parse_expr, ast::BinExpr, AstNode};
    ///
    /// let node = parse_expr("2 + 3 * 2", 0).syntax().to::<BinExpr>().lhs().unwrap().syntax().to_owned();
    ///
    /// assert_eq!(node.next_adjacent_token_lossy().unwrap().text(), "3");
    /// ```
    fn next_adjacent_token_lossy(&self) -> Option<SyntaxToken> {
        let node = self.to_node();
        let next_element = node.next_sibling_or_token()?;

        match next_element {
            NodeOrToken::Node(node) => node.lossy_tokens().first().cloned(),
            NodeOrToken::Token(token) => {
                if !token.kind().is_trivia() {
                    Some(token)
                } else {
                    for element in node.siblings_with_tokens(Direction::Next) {
                        match element {
                            NodeOrToken::Token(token) if !token.kind().is_trivia() => {
                                return Some(token)
                            }
                            NodeOrToken::Node(node) => return node.lossy_tokens().first().cloned(),
                            _ => {}
                        }
                    }
                    None
                }
            }
        }
    }
}

impl SyntaxNodeExt for SyntaxNode {
    fn to_node(&self) -> &SyntaxNode {
        self
    }
}

/// Concatenate tokens into a string
pub fn concat_tokens(tokens: &[SyntaxToken]) -> String {
    tokens
        .iter()
        .map(|token| token.text().to_string())
        .collect()
}
