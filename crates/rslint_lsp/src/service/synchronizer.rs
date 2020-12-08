//! Synchronizes document changes between editor and server.

/// Functions related to processing events for a document.
pub(crate) mod document {
    use crate::{
        core::{
            document::{Document, DocumentParse},
            language::Language,
            session::Session,
        },
        provider,
    };
    use lspower::lsp_types::*;
    use rslint_core::DirectiveParser;
    use rslint_errors::file::SimpleFiles;
    use rslint_parser::{parse_module, parse_text, SyntaxNode};

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

            let mut document = session.get_mut_document(&uri).await?;
            document.files = files;
            document.file_id = file_id;
            document.text = text.clone();

            document.parse = if document.language == Language::JavaScriptModule {
                Box::new(parse_module(&text, file_id)) as Box<dyn DocumentParse>
            } else {
                Box::new(parse_text(&text, file_id)) as Box<dyn DocumentParse>
            };

            let res = DirectiveParser::new(SyntaxNode::new_root(document.parse.green()), file_id)
                .get_file_directives();

            document.directives = res.directives;
            document.directive_errors = res.diagnostics;
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
