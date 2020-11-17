//! Provider for LSP diagnostics.

use crate::core::{
    document::{Document, RuleResult},
    session::Session,
};
use rayon::prelude::*;
use rslint_core::{
    apply_top_level_directives, directives::DirectiveResult, run_rule, DirectiveParser,
};
use rslint_errors::{lsp::convert_to_lsp_diagnostic, Diagnostic as RslintDiagnostic};
use rslint_parser::SyntaxNode;
use std::{collections::HashMap, sync::Arc};
use tower_lsp::lsp_types::*;

fn process_diagnostics(
    document: &Document,
    uri: Url,
    diagnostics: Vec<RslintDiagnostic>,
    out: &mut Vec<Diagnostic>,
) {
    let files = document.files.clone();
    let file_id = document.file_id;

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

pub async fn publish_diagnostics(session: Arc<Session>, uri: Url) -> anyhow::Result<()> {
    let mut document = session.get_mut_document(&uri).await?;
    let file_id = document.file_id;

    let mut new_store = session.store.clone();
    let DirectiveResult {
        directives,
        diagnostics: mut directive_diagnostics,
    } = DirectiveParser::new_with_store(
        SyntaxNode::new_root(document.parse.green()),
        file_id,
        &session.store,
    )
    .get_file_directives();

    apply_top_level_directives(
        directives.as_slice(),
        &mut new_store,
        &mut directive_diagnostics,
        file_id,
    );

    let verbose = false;
    let src = Arc::new(document.text.clone());
    let rule_results: HashMap<&str, rslint_core::RuleResult> = new_store
        .rules
        .par_iter()
        .map(|rule| {
            let root = SyntaxNode::new_root(document.parse.green());
            (
                rule.name(),
                run_rule(&**rule, file_id, root, verbose, &directives, src.clone()),
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
        document.parse.parser_diagnostics().to_owned(),
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

    let version = Default::default();
    session
        .client()?
        .publish_diagnostics(uri, diags, version)
        .await;

    Ok(())
}
