//! Definitions for the LSP server instance.

use crate::{
    core::session::{Config, TomlDocument},
    lsp::server::Server,
    provider,
    service::synchronizer,
};
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
        if uri_is_toml(&params.text_document_position_params.text_document.uri) {
            return Ok(None);
        }
        provider::hover::on_hover(&*self.session, params)
            .await
            .map_err(|_| jsonrpc::Error::internal_error())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        if is_config_doc(&params.text_document_position.text_document.uri) {
            if let Some(ref doc) = *self.session.config_doc.read().unwrap() {
                let completions = provider::toml_completion::toml_completions(
                    doc,
                    params.text_document_position.position,
                    schemars::schema_for!(rslint_config::ConfigRepr),
                );
                Ok(Some(CompletionResponse::Array(completions)))
            } else {
                Ok(None)
            }
        } else {
            if uri_is_toml(&params.text_document_position.text_document.uri) {
                return Ok(None);
            }
            provider::completion::complete(&*self.session, params)
                .await
                .map_err(|_| jsonrpc::Error::internal_error())
        }
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        if uri_is_toml(&params.text_document.uri) {
            return Ok(None);
        }
        provider::actions::actions(&*self.session, params)
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
        if is_config_doc(&params.text_document.uri) {
            let doc = params.text_document;
            let parse = taplo::parser::parse(&doc.text);
            let mapper = taplo::util::coords::Mapper::new_utf16(&doc.text, false);

            *self.session.config_doc.write().unwrap() = Some(TomlDocument::new(parse, mapper));
        } else {
            if params.text_document.language_id == "toml" {
                return;
            }

            synchronizer::document::open(&*self.session, params)
                .await
                .unwrap()
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if is_config_doc(&params.text_document.uri) {
            if let Some(ref mut doc) = *self.session.config_doc.write().unwrap() {
                let TextDocumentContentChangeEvent { text, .. } = params.content_changes[0].clone();
                doc.parse = taplo::parser::parse(&text);
                doc.mapper = taplo::util::coords::Mapper::new_utf16(&text, false);
            }
        } else {
            if uri_is_toml(&params.text_document.uri) {
                return;
            }

            synchronizer::document::change(&*self.session, params)
                .await
                .unwrap()
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        synchronizer::document::close(&*self.session, params)
            .await
            .unwrap()
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        if let Ok(config) = Config::from_value(params.settings) {
            *self.session.config.write().unwrap() = config;
        }
    }
}

fn uri_is_toml(uri: &Url) -> bool {
    uri.to_file_path()
        .ok()
        .and_then(|x| Some(x.extension()?.to_string_lossy().to_string()))
        == Some("toml".to_string())
}

fn is_config_doc(uri: &Url) -> bool {
    uri.to_file_path()
        .ok()
        .and_then(|x| Some(x.file_name()?.to_string_lossy().to_string()))
        == Some("rslintrc.toml".to_string())
}
