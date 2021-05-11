//! Core definitions related to the LSP server session.

use crate::core::{document::Document, error::Error};
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use futures::executor::block_on;
use rslint_core::CstRuleStore;
use serde::Deserialize;
use serde_json::Value;
use std::sync::RwLock;
use taplo::{parser::Parse, util::coords::Mapper};
use tower_lsp::lsp_types::ConfigurationItem;
use tower_lsp::{lsp_types::*, Client};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub incorrect_file_autofixes: bool,
}

impl Config {
    pub fn from_value(value: Value) -> anyhow::Result<Self> {
        Ok(serde_json::from_value(value)?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            incorrect_file_autofixes: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TomlDocument {
    pub parse: Parse,
    pub mapper: Mapper,
}

impl TomlDocument {
    pub fn new(parse: Parse, mapper: Mapper) -> Self {
        Self { parse, mapper }
    }
}

/// Represents the current state of the LSP session.
pub struct Session {
    client: Option<Client>,
    documents: DashMap<Url, Document>,
    pub(crate) store: CstRuleStore,
    pub(crate) config: RwLock<Config>,
    pub(crate) config_doc: RwLock<Option<TomlDocument>>,
}

impl Session {
    /// Create a new session.
    pub fn new(client: Option<Client>) -> anyhow::Result<Self> {
        let documents = DashMap::new();
        let store = CstRuleStore::new().builtins();
        let config = RwLock::new(
            client
                .as_ref()
                .and_then(|x| {
                    let config = block_on(x.configuration(vec![ConfigurationItem {
                        scope_uri: None,
                        section: Some("vscode-rslint".to_owned()),
                    }]))
                    .ok()?;

                    Config::from_value(config.first()?.to_owned()).ok()
                })
                .unwrap_or_default(),
        );

        Ok(Session {
            client,
            documents,
            store,
            config,
            config_doc: RwLock::new(None),
        })
    }

    pub(crate) fn client(&self) -> anyhow::Result<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| Error::ClientNotInitialized.into())
    }

    /// Insert an opened document into the session.
    pub fn insert_document(
        &self,
        uri: Url,
        document: Document,
    ) -> anyhow::Result<Option<Document>> {
        let result = self.documents.insert(uri, document);
        Ok(result)
    }

    /// Remove a closed document from the session.
    pub fn remove_document(&self, uri: &Url) -> anyhow::Result<Option<(Url, Document)>> {
        let result = self.documents.remove(uri);
        Ok(result)
    }

    /// Get a reference to a document associated with the session, if possible.
    pub async fn get_document(&self, uri: &Url) -> anyhow::Result<Ref<'_, Url, Document>> {
        self.documents
            .get(uri)
            .ok_or_else(|| Error::DocumentNotFound(uri.clone()).into())
    }

    /// Get a mutable reference to a document associated with the session, if possible.
    pub fn get_mut_document(&self, uri: &Url) -> anyhow::Result<RefMut<'_, Url, Document>> {
        self.documents
            .get_mut(uri)
            .ok_or_else(|| Error::DocumentNotFound(uri.clone()).into())
    }
}
