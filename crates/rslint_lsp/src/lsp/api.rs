//! Definitions for the LSP server instance.

use crate::{lsp::server::Server, provider, service::synchronizer};
use tower_lsp::{
    jsonrpc::{self, Result},
    lsp_types::*,
    LanguageServer,
};

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let capabilities = crate::lsp::server::capabilities();
        Ok(InitializeResult {
            capabilities,
            ..InitializeResult::default()
        })
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        provider::hover::on_hover(self.session.clone(), params)
            .await
            .map_err(|_| jsonrpc::Error::internal_error())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        provider::completion::complete(self.session.clone(), params)
            .await
            .map_err(|_| jsonrpc::Error::internal_error())
    }

    async fn initialized(&self, _: InitializedParams) {
        let typ = MessageType::Info;
        let message = "RSLint Language Server initialized!";
        self.client.log_message(typ, message).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let session = self.session.clone();
        synchronizer::document::open(session, params).await.unwrap()
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let session = self.session.clone();
        synchronizer::document::change(session, params)
            .await
            .unwrap()
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let session = self.session.clone();
        synchronizer::document::close(session, params)
            .await
            .unwrap()
    }
}
