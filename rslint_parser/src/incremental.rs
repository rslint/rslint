use crate::syntax::{decl::*, expr::*, pat::object_binding_pattern, stmt::*};
use crate::{SyntaxKind::*, *};
use rowan::GreenToken;

/// insert-delete, a single change to text which does not overlap with
/// other indels
#[derive(Debug, Clone)]
pub struct Indel {
    pub insert: String,
    pub delete: TextRange,
}

impl Indel {
    pub fn insert(offset: TextSize, text: String) -> Indel {
        Indel::replace(TextRange::empty(offset), text)
    }
    pub fn delete(range: TextRange) -> Indel {
        Indel::replace(range, String::new())
    }
    pub fn replace(range: TextRange, replace_with: String) -> Indel {
        Indel {
            delete: range,
            insert: replace_with,
        }
    }

    pub fn apply(&self, text: &mut String) {
        let start: usize = self.delete.start().into();
        let end: usize = self.delete.end().into();
        text.replace_range(start..end, &self.insert);
    }
}

pub(crate) fn incremental_reparse(
    node: &SyntaxNode,
    edit: &Indel,
    errors: Vec<ParserError>,
    file_id: usize,
) -> Option<(GreenNode, Vec<ParserError>, TextRange)> {
    if let Some((green, new_errors, old_range)) = reparse_token(node, &edit, file_id) {
        return Some((
            green,
            merge_errors(errors, new_errors, old_range, edit),
            old_range,
        ));
    }

    if let Some((green, new_errors, old_range)) = reparse_block(node, &edit, file_id) {
        return Some((
            green,
            merge_errors(errors, new_errors, old_range, edit),
            old_range,
        ));
    }
    None
}

fn merge_errors(
    mut old_errors: Vec<ParserError>,
    new_errors: Vec<ParserError>,
    range_before_reparse: TextRange,
    edit: &Indel,
) -> Vec<ParserError> {
    for old_err in old_errors.iter_mut() {
        let inserted_len = TextSize::of(&edit.insert);
        for label in old_err.labels.iter_mut() {
            let old_range = TextRange::new(
                (label.range.start as u32).into(),
                (label.range.end as u32).into(),
            );
            if old_range.end() >= range_before_reparse.start() {
                label.range = ((old_range + inserted_len) - edit.delete.len()).into();
            }
        }
    }

    old_errors.extend(new_errors.into_iter().map(|mut new_err| {
        for label in new_err.labels.iter_mut() {
            let old_range = TextRange::new(
                (label.range.start as u32).into(),
                (label.range.end as u32).into(),
            );
            label.range = (old_range + range_before_reparse.start()).into();
        }
        new_err
    }));
    old_errors
}

fn reparse_token<'node>(
    root: &'node SyntaxNode,
    edit: &Indel,
    file_id: usize,
) -> Option<(GreenNode, Vec<ParserError>, TextRange)> {
    let prev_token = root.covering_element(edit.delete).as_token()?.clone();
    let prev_token_kind = prev_token.kind();
    match prev_token_kind {
        WHITESPACE | COMMENT | IDENT | STRING | TEMPLATE_CHUNK => {
            if prev_token_kind == WHITESPACE || prev_token_kind == COMMENT {
                // removing a new line may extend the previous token
                let deleted_range = edit.delete - prev_token.text_range().start();
                if util::contains_js_linebreak(&prev_token.text()[deleted_range]) {
                    return None;
                }
            }

            let mut new_text = get_text_after_edit(prev_token.clone().into(), &edit);
            let (new_token_kind, new_err) = lex_single_syntax_kind(&new_text, file_id)?;

            if new_token_kind != prev_token_kind
                || (new_token_kind == IDENT && is_contextual_kw(&new_text))
            {
                return None;
            }

            // Check that edited token is not a part of the bigger token.
            if let Some(next_char) = root.text().char_at(prev_token.text_range().end()) {
                new_text.push(next_char);
                let token_with_next_char = lex_single_syntax_kind(&new_text, file_id);
                if let Some((_kind, _error)) = token_with_next_char {
                    return None;
                }
                new_text.pop();
            }

            let new_token =
                GreenToken::new(rowan::SyntaxKind(prev_token_kind.into()), new_text.into());
            Some((
                prev_token.replace_with(new_token),
                new_err.into_iter().collect(),
                prev_token.text_range(),
            ))
        }
        _ => None,
    }
}

fn reparse_block<'node>(
    root: &'node SyntaxNode,
    edit: &Indel,
    file_id: usize,
) -> Option<(GreenNode, Vec<ParserError>, TextRange)> {
    let (node, function) = find_reparsable_node(root, edit.delete)?;
    let text = get_text_after_edit(node.clone().into(), edit);

    let (tokens, new_lexer_errors) = tokenize(&text, file_id);
    let vec = tokens
        .iter()
        .map(|t| t.kind)
        .filter(|k| !k.is_trivia())
        .collect::<Vec<_>>();

    // skip eof
    if !is_balanced(&vec[..vec.len() - 1]) {
        return None;
    }
    let token_source = TokenSource::new(&text, &tokens);
    let mut tree_sink = LosslessTreeSink::new(&text, &tokens);
    reparse(function, token_source, &mut tree_sink, file_id);

    let (green, mut new_parser_errors) = tree_sink.finish();
    new_parser_errors.extend(new_lexer_errors);

    Some((
        node.replace_with(green),
        new_parser_errors,
        node.text_range(),
    ))
}

#[allow(clippy::type_complexity)]
fn find_reparsable_node(
    node: &SyntaxNode,
    range: TextRange,
) -> Option<(SyntaxNode, fn(&mut Parser) -> CompletedMarker)> {
    let node = node.covering_element(range);

    let mut ancestors = match node {
        NodeOrToken::Token(it) => it.parent().ancestors(),
        NodeOrToken::Node(it) => it.ancestors(),
    };
    ancestors.find_map(|node| {
        let parent = node.parent().map(|it| it.kind());
        let function = get_reparser_fn(node.kind(), parent)?;
        Some((node, function))
    })
}

fn is_balanced(tokens: &[SyntaxKind]) -> bool {
    if tokens.is_empty()
        || tokens.first().unwrap() != &T!['{']
        || tokens.last().unwrap() != &T!['}']
    {
        return false;
    }
    let mut balance = 0usize;
    for t in &tokens[1..tokens.len() - 1] {
        match t {
            T!['{'] => balance += 1,
            T!['}'] => {
                balance = match balance.checked_sub(1) {
                    Some(b) => b,
                    None => return false,
                }
            }
            _ => (),
        }
    }
    balance == 0
}

fn lex_single_syntax_kind(
    string: &str,
    file_id: usize,
) -> Option<(SyntaxKind, Option<ParserError>)> {
    rslint_lexer::Lexer::from_str(string, file_id)
        .next()
        .map(|(t, e)| (t.kind, e))
}

fn get_text_after_edit(element: SyntaxElement, edit: &Indel) -> String {
    let edit = Indel::replace(
        edit.delete - element.text_range().start(),
        edit.insert.clone(),
    );

    let mut text = match element {
        NodeOrToken::Token(token) => token.text().to_string(),
        NodeOrToken::Node(node) => node.text().to_string(),
    };
    edit.apply(&mut text);
    text
}

fn is_contextual_kw(string: &str) -> bool {
    matches!(string, "await" | "async" | "yield")
}

fn block_stmt_reparse(p: &mut Parser) -> CompletedMarker {
    block_stmt(p, false, None)
}

fn block_stmt_reparse_fn(p: &mut Parser) -> CompletedMarker {
    block_stmt(p, true, None)
}

// TODO: switch stmt, import, and export reparsing
fn get_reparser_fn(
    node: SyntaxKind,
    parent: Option<SyntaxKind>,
) -> Option<fn(&mut Parser) -> CompletedMarker> {
    let res = match node {
        BLOCK_STMT => {
            if let Some(FN_DECL) | Some(FN_EXPR) = parent {
                block_stmt_reparse_fn
            } else {
                block_stmt_reparse
            }
        }
        OBJECT_EXPR => object_expr,
        OBJECT_PATTERN => object_binding_pattern,
        CLASS_BODY => class_body,
        _ => return None,
    };

    Some(res)
}

fn reparse(
    function: fn(&mut Parser) -> CompletedMarker,
    token_source: TokenSource,
    sink: &mut dyn TreeSink,
    file_id: usize,
) {
    let mut p = Parser::new(token_source, file_id);
    function(&mut p);
    let events = p.finish();
    process(sink, events);
}
