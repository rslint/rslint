//! Provider for LSP diagnostics.

use crate::core::{
    document::{Document, RuleResult},
    session::Session,
};
use rslint_core::{
    apply_top_level_directives, directives::DirectiveResult, run_rule, DirectiveParser,
};
use rslint_errors::{lsp::convert_to_lsp_diagnostic, Diagnostic as RslintDiagnostic};
use std::{collections::HashMap, sync::Arc};
use tower_lsp::lsp_types::*;

fn process_diagnostics(
    document: &Document,
    uri: Url,
    diagnostics: Vec<RslintDiagnostic>,
    out: &mut Vec<Diagnostic>,
) {
    let files = document.files.clone();
    let file_id = document.file.id;

    for diagnostic in diagnostics {
        if let Some(lsp_diag) = convert_to_lsp_diagnostic(
            diagnostic,
            &files,
            file_id,
            uri.clone(),
            Some("rslint".to_string()),
        ) {
            out.push(lsp_diag);
        }
    }
}

pub async fn publish_diagnostics(session: &Session, uri: Url) -> anyhow::Result<()> {
    let diags = {
        let mut document = session.get_mut_document(&uri)?;

        let mut new_store = session.store.clone();
        let DirectiveResult {
            directives,
            diagnostics: mut directive_diagnostics,
        } = DirectiveParser::new_with_store(document.root.clone(), &document.file, &session.store)
            .get_file_directives();

        apply_top_level_directives(
            directives.as_slice(),
            &mut new_store,
            &mut directive_diagnostics,
            document.file.id,
        );

        let verbose = false;
        let src = Arc::from(document.file.source.clone());
        let rule_results: HashMap<&str, rslint_core::RuleResult> = new_store
            .rules
            .iter()
            .map(|rule| {
                (
                    rule.name(),
                    run_rule(
                        &**rule,
                        document.file.id,
                        document.root.clone(),
                        verbose,
                        &directives,
                        Arc::clone(&src),
                    ),
                )
            })
            .collect();

        let mut diags = vec![];

        process_diagnostics(
            &document,
            uri.clone(),
            directive_diagnostics
                .into_iter()
                .map(|x| x.diagnostic)
                .collect(),
            &mut diags,
        );

        process_diagnostics(
            &document,
            uri.clone(),
            document.parsing_errors.to_owned(),
            &mut diags,
        );

        for diagnostics in rule_results.clone().into_iter().map(|(_, r)| r.diagnostics) {
            process_diagnostics(&document, uri.clone(), diagnostics, &mut diags);
        }

        document.rule_results = rule_results
            .into_iter()
            .map(|(_, v)| v)
            .map(|res| RuleResult {
                diagnostics: diags.to_owned(),
                fixer: res.fixer,
            })
            .collect();

        diags
    };

    let version = Default::default();
    session
        .client()?
        .publish_diagnostics(uri, diags, version)
        .await;

    Ok(())
}
