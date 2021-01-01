//! Code action support, for example, actions to automatically fix an error.

use crate::core::{document::Document, session::Session};
use anyhow::Result;
use rslint_errors::Severity;
use rslint_parser::{util::*, T};
use std::ops::Range;
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeActionParams, CodeActionResponse,
    Diagnostic, Position, Range as LspRange, TextEdit, Url, WorkspaceEdit,
};

pub async fn actions(
    session: &Session,
    params: CodeActionParams,
) -> Result<Option<CodeActionResponse>> {
    let document = session.get_document(&params.text_document.uri).await?;
    if document
        .parsing_errors
        .iter()
        .any(|d| d.severity == Severity::Error)
        && !session.config.read().unwrap().incorrect_file_autofixes
    {
        return Ok(None);
    }

    let action_range =
        rslint_errors::lsp::range_to_byte_span(&document.files, document.file.id, &params.range)?;

    let mut actions = vec![];
    let diagnostics = document
        .rule_results
        .iter()
        .flat_map(|x| x.diagnostics.clone())
        .collect();

    actions.push(CodeActionOrCommand::CodeAction(ignore_file_action(
        document.value(),
        document.key(),
        diagnostics,
    )));

    for res in document.rule_results.iter() {
        let matched_diag = res.diagnostics.iter().find(|d| {
            rslint_errors::lsp::range_to_byte_span(&document.files, document.file.id, &d.range).ok()
                == Some(action_range.to_owned())
        });

        if let Some(fixer) = res.fixer.as_ref() {
            if matched_diag.is_some() {
                let edits = fixer
                    .indels
                    .iter()
                    .filter_map(|i| {
                        Some(TextEdit {
                            range: rslint_errors::lsp::byte_span_to_range(
                                &document.files,
                                document.file.id,
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
                                document.file.id,
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

fn ignore_file_action(document: &Document, uri: &Url, diagnostics: Vec<Diagnostic>) -> CodeAction {
    // if the file has a shebang we cant insert a comment at the start without causing a syntax error
    let line = document
        .root
        .token_with_kind(T![shebang])
        .map_or(0u64, |_| 1);

    let first_edit = TextEdit {
        range: LspRange::new(Position::new(line, 0), Position::new(line, 0)),
        new_text: "// rslint-ignore\n".to_string(),
    };

    CodeAction {
        title: "Ignore this file".to_string(),
        edit: Some(WorkspaceEdit::new(
            vec![(uri.to_owned(), vec![first_edit])]
                .into_iter()
                .collect(),
        )),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(diagnostics),
        ..Default::default()
    }
}
