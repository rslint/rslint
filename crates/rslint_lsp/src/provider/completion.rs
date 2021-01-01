//! Autocompletion support for directives

use crate::core::session::Session;
use anyhow::Result;
use once_cell::sync::Lazy;
use rslint_core::{
    directives::{get_command_descriptors, CommandDescriptor, Instruction},
    util::levenshtein_distance,
    CstRuleStore, DirectiveErrorKind,
};
use rslint_parser::{util::*, TextRange, TextSize};
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Documentation,
    MarkupContent, MarkupKind,
};

static DESCRIPTORS: Lazy<Box<[CommandDescriptor]>> = Lazy::new(get_command_descriptors);

fn command_name_completions() -> CompletionResponse {
    CompletionResponse::Array(
        DESCRIPTORS
            .iter()
            .map(|x| {
                let mut label = String::with_capacity(7 + x.name.len());
                label.push_str("rslint-");
                label.push_str(x.name);
                CompletionItem {
                    detail: Some(x.docs.to_string()),
                    label,
                    kind: Some(CompletionItemKind::Snippet),
                    ..Default::default()
                }
            })
            .collect(),
    )
}

pub async fn complete(
    session: &Session,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let document = session
        .get_document(&params.text_document_position.text_document.uri)
        .await?;

    let loc = rslint_errors::lsp::position_to_byte_index(
        &document.files,
        document.file.id,
        &params.text_document_position.position,
    )?;

    if let Some(err) = document
        .directive_errors
        .iter()
        .find(|x| x.range().end == loc)
    {
        return Ok(Some(match err.kind {
            DirectiveErrorKind::ExpectedCommand | DirectiveErrorKind::InvalidCommandName => {
                command_name_completions()
            }
            DirectiveErrorKind::InvalidRule => {
                let wrong_text = &document.file.source[err.range()];
                let available_rules = CstRuleStore::new().builtins().rules.into_iter();

                let mut list = available_rules
                    .map(|r| (r.name(), r.docs()))
                    .collect::<Vec<_>>();

                list.sort_by(|(l_name, _), (r_name, _)| {
                    levenshtein_distance(wrong_text, l_name)
                        .cmp(&levenshtein_distance(wrong_text, r_name))
                });
                completion_list(list, true)
            }
            DirectiveErrorKind::ExpectedNotFound(Instruction::RuleName) => completion_list(
                CstRuleStore::new()
                    .builtins()
                    .rules
                    .into_iter()
                    .map(|x| (x.name(), x.docs()))
                    .collect(),
                true,
            ),
            _ => return Ok(None),
        }));
    }

    let start = TextSize::from(loc as u32);
    if let Some(comment) = document
        .root
        .covering_element(TextRange::at(start, 0.into()))
        .into_token()
    {
        if let Some(c) = comment.comment() {
            let content = c.content.trim();
            if content.len() <= 7 && "rslint-".find(content) == Some(0) {
                return Ok(Some(command_name_completions()));
            }
        }
    }

    Ok(None)
}

fn completion_list(items: Vec<(impl ToString, impl ToString)>, rules: bool) -> CompletionResponse {
    CompletionResponse::Array(
        items
            .into_iter()
            .map(|(label, detail)| {
                string_to_completion_item(label.to_string(), detail.to_string(), rules)
            })
            .collect(),
    )
}

fn string_to_completion_item(label: String, detail: String, rules: bool) -> CompletionItem {
    if rules {
        let mut split = detail.split('\n');
        let header = split.next().unwrap_or("");
        let body = split.next().unwrap_or("").to_string();
        let documentation = Some(Documentation::MarkupContent(MarkupContent {
            kind: MarkupKind::Markdown,
            value: body,
        }));
        CompletionItem {
            documentation,
            detail: Some(header.to_string()),
            kind: Some(CompletionItemKind::Field),
            label,
            ..Default::default()
        }
    } else {
        CompletionItem {
            label,
            detail: Some(detail),
            kind: Some(CompletionItemKind::Field),
            ..Default::default()
        }
    }
}
