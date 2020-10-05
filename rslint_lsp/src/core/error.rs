//! Core definitions related to runtime errors.

use std::path::PathBuf;
use thiserror::Error;
use tower_lsp::lsp_types::*;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error, PartialEq)]
pub(crate) enum Error {
    #[error("client not initialized")]
    ClientNotInitialized,
    #[error("document not found: {0}")]
    DocumentNotFound(Url),
    #[error("invalid language extension: {0}")]
    InvalidLanguageExtension(String),
    #[error("invalid language id: {0}")]
    InvalidLanguageId(String),
    #[error("conversion to &str failed")]
    ToStrFailed,
    #[error("failed to get file extension for PathBuf: {0}")]
    PathExtensionFailed(PathBuf),
}

pub(crate) struct IntoJsonRpcError(pub(crate) anyhow::Error);

impl From<IntoJsonRpcError> for tower_lsp::jsonrpc::Error {
    fn from(error: IntoJsonRpcError) -> Self {
        let mut rpc_error = tower_lsp::jsonrpc::Error::internal_error();
        rpc_error.data = Some(serde_json::to_value(format!("{}", error.0)).unwrap());
        rpc_error
    }
}
