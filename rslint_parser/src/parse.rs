//! Utilities for high level parsing of js code.

use crate::{
    ast::Program, AstNode, GreenNode, ParserError, SyntaxKind, SyntaxNode, LosslessTreeSink,
    TokenSource,
    Event,
    LossyTreeSink
};
use std::{marker::PhantomData, sync::Arc};

/// A utility struct for managing the result of a parser job
#[derive(Debug)]
pub struct Parse<T> {
    green: GreenNode,
    errors: Arc<Vec<ParserError>>,
    _ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Parse<T> {
        Parse {
            green: self.green.clone(),
            errors: self.errors.clone(),
            _ty: PhantomData,
        }
    }
}

impl<T> Parse<T> {
    pub fn new(green: GreenNode, errors: Vec<ParserError>) -> Parse<T> {
        Parse {
            green,
            errors: Arc::new(errors),
            _ty: PhantomData,
        }
    }

    /// The syntax node represented by this Parse result
    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }

    /// Get the errors which ocurred when parsing
    pub fn errors(&self) -> &[ParserError] {
        &*self.errors
    }
}

impl<T: AstNode> Parse<T> {
    /// Convert the result to an untyped SyntaxNode parse
    pub fn to_syntax(self) -> Parse<SyntaxNode> {
        Parse {
            green: self.green,
            errors: self.errors,
            _ty: PhantomData,
        }
    }

    /// Convert this parse result into a typed AST node.
    ///
    /// # Panics
    /// Panics if the node represented by this parse result mismatches.
    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).unwrap()
    }

    /// Try to convert this parse's untyped syntax node into an AST node.
    pub fn try_tree(&self) -> Option<T> {
        T::cast(self.syntax_node())
    }

    /// Convert this parse into a result
    pub fn ok(self) -> Result<T, Arc<Vec<ParserError>>> {
        if self.errors.is_empty() {
            Ok(self.tree())
        } else {
            Err(self.errors)
        }
    }
}

/// Run the rslint_lexer lexer to turn source code into tokens and errors produced by the lexer
pub fn tokenize(text: &str, file_id: usize) -> (Vec<rslint_lexer::Token>, Vec<ParserError>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for (tok, error) in rslint_lexer::Lexer::from_str(text, file_id) {
        tokens.push(tok);
        if let Some(err) = error {
            errors.push(err)
        }
    }
    (tokens, errors)
}

fn parse_common(text: &str, file_id: usize) -> (Vec<Event>, Vec<ParserError>, Vec<rslint_lexer::Token>) {
    let (tokens, errors) = tokenize(&text, file_id);

    let tok_source = TokenSource::new(text, &tokens);

    let mut parser = crate::Parser::new(tok_source, file_id);
    /* PLACEHOLDER */
    let m = parser.start();
    crate::syntax::expr::expr(&mut parser);
    m.complete(&mut parser, SyntaxKind::PROGRAM);
    (parser.finish(), errors, tokens)
}

/// Parse text into a [`Parse`](Parse) which can then be turned into an untyped root [`SyntaxNode`](SyntaxNode).
/// Or turned into a typed [`Program`](crate::ast::Program) with [`tree`](Parse::tree).
///
/// ```
/// use rslint_parser::{ast::BracketExpr, parse_text, AstNode, SyntaxToken, SyntaxNodeExt, util};
///
/// let parse = parse_text("foo. bar[2]", 0);
/// // The untyped syntax node of `foo.bar[2]`, the root node is `Program`.
/// let untyped_expr_node = parse.syntax_node().first_child().unwrap();
///
/// // SyntaxNodes can be turned into a nice string representation.
/// println!("{:#?}", untyped_expr_node);
///
/// // You can then cast syntax nodes into a typed AST node.
/// let typed_ast_node = BracketExpr::cast(untyped_expr_node.to_owned()).unwrap();
///
/// // Everything on every ast node is optional because of error recovery.
/// let prop = typed_ast_node.prop().unwrap();
///
/// // You can then go back to an untyped SyntaxNode and get its range, text, parents, children, etc.
/// assert_eq!(prop.syntax().text(), "2");
///
/// // Util has a function for yielding all tokens of a node.
/// let tokens = untyped_expr_node.tokens();
///
/// assert_eq!(&util::concat_tokens(&tokens), "foo. bar[2]")
/// ```
pub fn parse_text(text: &str, file_id: usize) -> Parse<Program> {
    let (events, mut errors, tokens) = parse_common(text, file_id);
    let mut tree_sink = LosslessTreeSink::new(text, &tokens);
    crate::process(&mut tree_sink, events);
    let (green, parse_errors) = tree_sink.finish();
    errors.extend(parse_errors);
    Parse::new(green, errors)
}

/// Lossly parse text into a [`Parse`](Parse) which can then be turned into an untyped root [`SyntaxNode`](SyntaxNode).
/// Or turned into a typed [`Program`](crate::ast::Program) with [`tree`](Parse::tree).
///
/// Unlike [`parse_text`], the final parse result includes no whitespace, it does however include errors. 
/// 
/// ```
/// use rslint_parser::{ast::BracketExpr, parse_text_lossy, AstNode, SyntaxToken, SyntaxNodeExt, util};
///
/// let parse = parse_text_lossy("foo. bar[2]", 0);
/// // The untyped syntax node of `foo.bar[2]`, the root node is `Program`.
/// let untyped_expr_node = parse.syntax_node().first_child().unwrap();
///
/// // SyntaxNodes can be turned into a nice string representation.
/// println!("{:#?}", untyped_expr_node);
///
/// // You can then cast syntax nodes into a typed AST node.
/// let typed_ast_node = BracketExpr::cast(untyped_expr_node.to_owned()).unwrap();
///
/// // Everything on every ast node is optional because of error recovery.
/// let prop = typed_ast_node.prop().unwrap();
///
/// // You can then go back to an untyped SyntaxNode and get its range, text, parents, children, etc.
/// assert_eq!(prop.syntax().text(), "2");
///
/// // Util has a function for yielding all tokens of a node.
/// let tokens = untyped_expr_node.tokens();
///
/// // End result does not include whitespace because the parsing is lossy in this case
/// assert_eq!(&util::concat_tokens(&tokens), "foo.bar[2]")
/// ```
pub fn parse_text_lossy(text: &str, file_id: usize) -> Parse<Program> {
    let (events, mut errors, tokens) = parse_common(text, file_id);
    let mut tree_sink = LossyTreeSink::new(text, &tokens);
    crate::process(&mut tree_sink, events);
    let (green, parse_errors) = tree_sink.finish();
    errors.extend(parse_errors);
    Parse::new(green, errors)
}

#[test]
fn it_works() {
    let parse = parse_text("{
        a: foo,
        b: 6 += 3
        5: 1
    }", 0);
    println!("{:#?}", parse.syntax_node());
    println!("{:#?}", parse.syntax_node().text());
    println!("{:#?}", parse.errors());
}
