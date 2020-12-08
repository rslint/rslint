//! Code action support, for example, actions to automatically fix an error.

use crate::core::session::Session;
use anyhow::Result;
use lspower::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    TextEdit, WorkspaceEdit,
};
use rslint_errors::Severity;
use std::ops::Range;

pub async fn actions(
    session: &Session,
    params: CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let document = session.get_document(&params.text_document.uri).await?;
    if document
        .parse
        .parser_diagnostics()
        .iter()
        .any(|d| d.severity == Severity::Error)
        && !session.config.read().unwrap().incorrect_file_autofixes
    {
        return Ok(None);
    }

    let action_range =
        rslint_errors::lsp::range_to_byte_span(&document.files, document.file_id, &params.range)?;

    let mut actions = vec![];
    for res in document.rule_results.iter() {
        if let Some(fixer) = res.fixer.as_ref() {
            let has_match = res.diagnostics.iter().any(|d| {
                rslint_errors::lsp::range_to_byte_span(&document.files, document.file_id, &d.range)
                    .ok()
                    == Some(action_range.to_owned())
            });

            if has_match {
                let edits = fixer
                    .indels
                    .iter()
                    .filter_map(|i| {
                        Some(TextEdit {
                            range: rslint_errors::lsp::byte_span_to_range(
                                &document.files,
                                document.file_id,
                                Range::<usize>::from(i.delete),
                            )
                            .ok()?,
                            new_text: i.insert.to_owned(),
                        })
                    })
                    .collect::<Vec<_>>();

                let edit = Some(WorkspaceEdit::new(
                    vec![(params.text_document.uri.to_owned(), edits)]
                        .into_iter()
                        .collect(),
                ));

                let diagnostics = Some(
                    res.diagnostics
                        .iter()
                        .filter(|d| {
                            rslint_errors::lsp::range_to_byte_span(
                                &document.files,
                                document.file_id,
                                &d.range,
                            )
                            .ok()
                                == Some(action_range.to_owned())
                        })
                        .cloned()
                        .collect(),
                );

                let action = CodeAction {
                    title: "Fix this issue".to_string(),
                    edit,
                    is_preferred: Some(true),
                    diagnostics,
                    kind: Some(CodeActionKind::QUICKFIX),
                    ..Default::default()
                };
                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }
    }
    Ok(Some(actions))
}
