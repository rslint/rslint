//! Synchronizes document changes between editor and server.

/// Functions related to processing events for a document.
pub(crate) mod document {
    use crate::{
        core::{document::Document, session::Session},
        provider,
    };
    use rslint_core::DirectiveParser;
    use rslint_errors::file::SimpleFiles;
    use tower_lsp::lsp_types::*;

    /// Handle a document "change" event.
    pub(crate) async fn change(
        session: &Session,
        params: DidChangeTextDocumentParams,
    ) -> anyhow::Result<()> {
        let DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier { uri, .. },
            content_changes,
        } = params;
        let TextDocumentContentChangeEvent { text, .. } = content_changes[0].clone();

        // NOTE: We use an explicit scope here because document (below) must be
        // dropped before calling publish_diagnostics (further below) otherwise
        // the server will block.
        {
            let mut files = SimpleFiles::new();
            let file_id = files.add(uri.to_string(), text.clone());

            let mut document = session.get_mut_document(&uri)?;
            document.files = files;
            document.file.id = file_id;
            document.file.source = text.clone();

            let (parsing_errors, root) = document.file.parse_with_errors();

            let res = DirectiveParser::new(root.clone(), &document.file).get_file_directives();

            document.root = root;
            document.directives = res.directives;
            document.directive_errors = res.diagnostics;
            document.parsing_errors = parsing_errors;
        }

        provider::diagnostics::publish_diagnostics(session, uri).await?;

        Ok(())
    }

    /// Handle a document "close" event.
    pub(crate) async fn close(
        session: &Session,
        params: DidCloseTextDocumentParams,
    ) -> anyhow::Result<()> {
        let DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier { uri },
        } = params;

        session.remove_document(&uri)?;

        let diagnostics = Default::default();
        let version = Default::default();
        session
            .client()?
            .publish_diagnostics(uri, diagnostics, version)
            .await;

        Ok(())
    }

    /// Handle a document "open" event.
    pub(crate) async fn open(
        session: &Session,
        params: DidOpenTextDocumentParams,
    ) -> anyhow::Result<()> {
        let DidOpenTextDocumentParams {
            text_document:
                TextDocumentItem {
                    uri,
                    language_id,
                    text,
                    ..
                },
        } = params;

        let document = Document::new(uri.clone(), language_id, text)?;
        session.insert_document(uri.clone(), document)?;

        provider::diagnostics::publish_diagnostics(session, uri).await?;

        Ok(())
    }
}
