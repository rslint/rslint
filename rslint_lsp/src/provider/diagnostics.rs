//! Provider for LSP diagnostics.

use crate::core::{document::Document, session::Session};
use codespan_lsp::byte_span_to_range;
use codespan_reporting::diagnostic::LabelStyle;
use itertools::Itertools;
use rayon::prelude::*;
use rslint_core::{apply_top_level_directives, run_rule, DirectiveParser};
use rslint_parser::SyntaxNode;
use std::{collections::HashMap, sync::Arc};
use tower_lsp::lsp_types::*;

fn process_diagnostics(
    document: &Document,
    uri: Url,
    rule: Option<&str>,
    diagnostics: &[codespan_reporting::diagnostic::Diagnostic<usize>],
    out: &mut Vec<Diagnostic>,
) -> anyhow::Result<()> {
    let files = document.files.clone();
    let file_id = document.file_id;

    for diagnostic in diagnostics {
        let mut primary_label = None;
        let mut related_information = vec![];

        for label in diagnostic
            .labels
            .iter()
            .filter(|label| label.file_id == document.file_id)
            .sorted_by(|a, b| a.range.clone().cmp(b.range.clone()))
        {
            if label.style == LabelStyle::Primary {
                primary_label = Some(label);
            }

            let range = match byte_span_to_range(&files, file_id, label.range.clone()) {
                Err(codespan_lsp::Error::ColumnOutOfBounds { max, .. }) => {
                    let start = std::cmp::min(max, label.range.start);
                    let end = std::cmp::min(max, label.range.end);
                    byte_span_to_range(&files, file_id, start..end)
                }
                range => range,
            }?;

            related_information.push(DiagnosticRelatedInformation {
                location: Location {
                    uri: uri.clone(),
                    range,
                },
                message: label.message.clone(),
            });
        }

        if let Some(primary_label) = primary_label {
            let primary_range =
                match byte_span_to_range(&files, file_id, primary_label.range.clone()) {
                    Err(codespan_lsp::Error::ColumnOutOfBounds { max, .. }) => {
                        let start = std::cmp::min(max, primary_label.range.start);
                        let end = std::cmp::min(max, primary_label.range.end);
                        byte_span_to_range(&files, file_id, start..end)
                    }
                    range => range,
                }?;

            let severity = Default::default();
            let code = if let Some(rule) = rule {
                Some(NumberOrString::String(rule.into()))
            } else {
                Some(NumberOrString::String("parser".into()))
            };
            let source = Some("rslint".into());
            let message = diagnostic.message.clone();
            let related_information = Some(related_information);
            let tags = Default::default();

            out.push(Diagnostic::new(
                primary_range,
                severity,
                code,
                source,
                message,
                related_information,
                tags,
            ));
        }
    }

    Ok(())
}

pub async fn publish_diagnostics(session: Arc<Session>, uri: Url) -> anyhow::Result<()> {
    let document = session.get_document(&uri).await?;
    let file_id = document.file_id;

    let mut new_store = session.store.clone();
    let results = DirectiveParser::new(
        SyntaxNode::new_root(document.parse.green()),
        file_id,
        &session.store,
    )
    .get_file_directives();

    match results {
        Ok(results) => {
            let mut directive_diagnostics = vec![];
            let directives = results
                .into_iter()
                .map(|res| {
                    directive_diagnostics.extend(res.diagnostics);
                    res.directive
                })
                .collect::<Vec<_>>();

            apply_top_level_directives(
                directives.as_slice(),
                &mut new_store,
                &mut directive_diagnostics,
                file_id,
            );

            let verbose = false;
            let rule_diagnostics: HashMap<&str, Vec<rslint_core::Diagnostic>> = new_store
                .rules
                .par_iter()
                .map(|rule| {
                    let root = SyntaxNode::new_root(document.parse.green());
                    (
                        rule.name(),
                        run_rule(&**rule, file_id, root, verbose, &directives),
                    )
                })
                .collect();

            let mut diags = vec![];

            process_diagnostics(
                &document,
                uri.clone(),
                None,
                &document.parse.parser_diagnostics(),
                &mut diags,
            )?;

            for (rule, diagnostics) in rule_diagnostics {
                process_diagnostics(&document, uri.clone(), Some(rule), &diagnostics, &mut diags)?;
            }

            let version = Default::default();
            session
                .client()?
                .publish_diagnostics(uri, diags, version)
                .await;

            Ok(())
        }

        Err(diagnostic) => {
            let diagnostics = vec![{
                let range = Default::default();
                let severity = Default::default();
                let code = Default::default();
                let source = Some("rslint".into());
                let message = diagnostic.message;
                let related_information = Default::default();
                let tags = Default::default();
                Diagnostic::new(
                    range,
                    severity,
                    code,
                    source,
                    message,
                    related_information,
                    tags,
                )
            }];
            let version = Default::default();

            session
                .client()?
                .publish_diagnostics(uri, diagnostics, version)
                .await;

            Ok(())
        }
    }
}
