//! Style and lossless tree utilities.

use crate::rule_prelude::*;
use rslint_parser::NodeOrToken;
use std::iter;
use SyntaxKind::*;

/// Extensions to nodes and tokens used for stylistic linting which cares about trivia (whitespace)
pub trait StyleExt {
    fn to_elem(&self) -> SyntaxElement;
    fn trailing_trivia_has_linebreak(&self, stop_at_comment: bool) -> bool {
        self.trailing_whitespace(stop_at_comment)
            .into_iter()
            .any(|x| parseutil::contains_js_linebreak(x.text().as_str()))
    }

    fn leading_trivia_has_linebreak(&self, stop_at_comment: bool) -> bool {
        self.leading_whitespace(stop_at_comment)
            .into_iter()
            .any(|x| parseutil::contains_js_linebreak(x.text().as_str()))
    }

    fn trailing_trivia(&self, stop_at_comment: bool) -> Vec<SyntaxToken> {
        let elem = self.to_elem();
        let token = elem
            .clone()
            .into_token()
            .or_else(|| elem.into_node().unwrap().first_token());

        iter::successors(
            token.and_then(|x| {
                if x.kind() == WHITESPACE || (x.kind() == COMMENT && !stop_at_comment) {
                    Some(x)
                } else {
                    x.next_token()
                }
            }),
            |next| next.next_token(),
        )
        .take_while(|x| x.kind().is_trivia())
        .collect()
    }

    fn trailing_whitespace(&self, stop_at_comment: bool) -> Vec<SyntaxToken> {
        self.trailing_trivia(stop_at_comment)
            .into_iter()
            .take_while(|x| x.kind() == WHITESPACE)
            .collect()
    }

    fn leading_trivia(&self, stop_at_comment: bool) -> Vec<SyntaxToken> {
        let elem = self.to_elem();
        let token = elem
            .clone()
            .into_token()
            .or_else(|| match elem.prev_sibling_or_token() {
                Some(elem) => match elem {
                    NodeOrToken::Node(n) => n.last_token(),
                    NodeOrToken::Token(t) => Some(t),
                },
                None => elem
                    .ancestors()
                    .find_map(|it| it.prev_sibling_or_token())
                    .and_then(|e| match e {
                        NodeOrToken::Node(n) => n.last_token(),
                        NodeOrToken::Token(t) => Some(t),
                    }),
            });

        iter::successors(
            token.and_then(|x| {
                if x.kind() == WHITESPACE || (x.kind() == COMMENT && !stop_at_comment) {
                    Some(x)
                } else {
                    x.prev_token()
                }
            }),
            |next| next.prev_token(),
        )
        .take_while(|x| x.kind().is_trivia())
        .collect()
    }

    fn leading_whitespace(&self, stop_at_comment: bool) -> Vec<SyntaxToken> {
        self.leading_trivia(stop_at_comment)
            .into_iter()
            .take_while(|x| x.kind() == WHITESPACE)
            .collect()
    }

    fn has_leading_whitespace(&self, include_linebreaks: bool, stop_at_comment: bool) -> bool {
        let leading = self.leading_whitespace(stop_at_comment);
        tokens_have_whitespace(&leading, include_linebreaks)
    }

    fn has_trailing_whitespace(&self, include_linebreaks: bool, stop_at_comment: bool) -> bool {
        let trailing = self.trailing_whitespace(stop_at_comment);
        tokens_have_whitespace(&trailing, include_linebreaks)
    }
}

impl StyleExt for SyntaxNode {
    fn to_elem(&self) -> SyntaxElement {
        SyntaxElement::from(self.to_owned())
    }
}

impl StyleExt for SyntaxToken {
    fn to_elem(&self) -> SyntaxElement {
        SyntaxElement::from(self.to_owned())
    }
}

impl StyleExt for SyntaxElement {
    fn to_elem(&self) -> SyntaxElement {
        self.to_owned()
    }
}

pub fn tokens_have_whitespace(tokens: &[SyntaxToken], include_linebreaks: bool) -> bool {
    for ws in tokens {
        if parseutil::contains_js_whitespace(ws.text().as_str())
            || (include_linebreaks && parseutil::contains_js_linebreak(ws.text().as_str()))
        {
            return true;
        }
    }
    false
}
