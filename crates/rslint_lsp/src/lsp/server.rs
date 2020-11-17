//! Definitions for the LSP server instance.

use crate::core::session::Session;
use std::sync::Arc;
use tower_lsp::{lsp_types::*, Client};

/// The RSLint Language Server instance.
pub struct Server {
    /// The LSP client handle.
    pub client: Client,
    /// The current state of the LSP session.
    pub session: Arc<Session>,
}

impl Server {
    /// Create a new server.
    pub fn new(client: Client) -> anyhow::Result<Self> {
        let session = Arc::new(Session::new(Some(client.clone()))?);
        Ok(Server { client, session })
    }
}

/// Compute the server capabilities.
pub fn capabilities() -> ServerCapabilities {
    let text_document_sync = Some(TextDocumentSyncCapability::Options(
        TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::Full),
            ..Default::default()
        },
    ));

    ServerCapabilities {
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        completion_provider: Some(CompletionOptions {
            trigger_characters: Some(
                ('a'..='z')
                    .into_iter()
                    .chain('A'..='Z')
                    .map(|x| x.to_string())
                    .collect(),
            ),
            ..Default::default()
        }),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        text_document_sync,
        ..Default::default()
    }
}
