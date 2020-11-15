//! Autocompletion support for directives

use crate::core::session::Session;
use anyhow::Result;
use rslint_core::{
    directives::ComponentKind, directives::Instruction, util::levenshtein_distance, CstRuleStore,
    DirectiveErrorKind,
};
use std::sync::Arc;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse, Documentation,
    MarkupContent, MarkupKind,
};

pub async fn complete(
    session: Arc<Session>,
    params: CompletionParams,
) -> Result<Option<CompletionResponse>> {
    let document = session
        .get_document(&params.text_document_position.text_document.uri)
        .await?;

    log::info!("---- {:#?}", document.directive_errors);
    if !document.directive_errors.is_empty() {
        let loc = rslint_errors::lsp::position_to_byte_index(
            &document.files,
            document.file_id,
            &params.text_document_position.position,
        )?;
        log::info!("{}", loc);
        if let Some(err) = document
            .directive_errors
            .iter()
            .find(|x| x.range().end == loc)
        {
            return Ok(Some(match err.kind {
                DirectiveErrorKind::ExpectedCommand => completion_list(
                    vec![(
                        "rslint-ignore",
                        ComponentKind::CommandName("ignore".into())
                            .documentation()
                            .unwrap(),
                    )],
                    false,
                ),
                DirectiveErrorKind::InvalidRule => {
                    let wrong_text = &document.text[err.range()];
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
                DirectiveErrorKind::InvalidCommandName => {
                    let available = vec![(
                        "rslint-ignore",
                        ComponentKind::CommandName("ignore".into())
                            .documentation()
                            .unwrap(),
                    )];
                    completion_list(available, false)
                }
                _ => return Ok(None),
            }));
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
