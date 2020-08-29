//! Directives used to configure or ignore rules.
//! These take place of comments over nodes or comments at the top level.

use crate::{Diagnostic, DiagnosticBuilder};
use rslint_parser::{
    SyntaxKind, SyntaxNode, SyntaxNodeExt, SyntaxToken, SyntaxTokenExt, TextRange,
};
use std::str::CharIndices;

pub type DirectiveParseResult<'src> = (Vec<Directive<'src>>, Vec<Diagnostic>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandKind<'src> {
    /// Disable linting for the entire file.
    DisableFile,
    /// Disable one or more rules on a node.
    DisableRules(Vec<&'src str>),
    /// Disable any rules on a node.
    DisableNode(SyntaxNode),
    /// Disable rules for an entire file.
    DisableRulesFile(Vec<&'src str>),
}

/// A command given to the linter by an inline comment.
/// A single command may include multiple commands inside of it.
/// A directive constitutes a single comment, which may have one or more commands inside of it.
pub struct Directive<'src> {
    pub commands: Vec<Command<'src>>,
    pub raw: &'src str,
    pub comment: SyntaxToken,
}

pub struct Command<'src> {
    pub kind: CommandKind<'src>,
    /// The span of this command, not including any leading or trailing whitespace.
    pub range: TextRange,
}

pub struct DirectiveParser<'src> {
    pub root_node: SyntaxNode,
    /// A string denoting the start of a directive, `@rslint` by default.
    pub declarator: &'src str,
    diagnostics: Vec<Diagnostic>,
    file_id: usize
}

impl DirectiveParser<'_> {
    /// Make a new directive parser from the root node of a file.
    ///
    /// # Panics
    /// Panics if the node's kind is not SCRIPT or MODULE
    pub fn new(root_node: SyntaxNode, file_id: usize) -> Self {
        assert!(matches!(
            root_node.kind(),
            SyntaxKind::SCRIPT | SyntaxKind::MODULE
        ));

        Self {
            root_node,
            declarator: "@rslint",
            diagnostics: Vec::new(),
            file_id
        }
    }

    fn err(&self, message: impl AsRef<str>) -> DiagnosticBuilder {
        DiagnosticBuilder::error(self.file_id, "LinterError", message.as_ref())
    }

    /// Extract directives which apply to the whole file such as `@rslint ignore` or `@rslint disable rule`.
    pub fn extract_top_level_directives(&self) -> DirectiveParseResult<'_> {
        let comments = self
            .root_node
            .tokens()
            .into_iter()
            .take_while(|t| t.kind().is_trivia())
            .filter(|t| {
                t.kind() == SyntaxKind::COMMENT
                    && t.comment()
                        .unwrap()
                        .content
                        .trim_start()
                        .starts_with(self.declarator)
            });

        
    }

    fn offset_range(token: &SyntaxToken, word: &str) -> TextRange {
        let start = token.text_range();
        TextRange::new(start + word.as_ptr() as usize, start)
    }

    /// An individual command. 
    // TODO: allow for `rslint-ignore` too.
    fn command<'a>(&self, comment: SyntaxToken, offset: usize, top_level: bool) -> Result<Command<'a>, Diagnostic> {
        let words = comment.text()[offset..].split_whitespace();

        match words.next() {
            Some("ignore") => {
                if !top_level {
                    let err = self.err("`ignore` directives must be placed at the top of a file")
                        .primary(comment.text_range(), message);
                }
            }
        }
    }
}
