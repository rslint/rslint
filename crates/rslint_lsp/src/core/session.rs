//! Core definitions related to the LSP server session.

use crate::core::{document::Document, error::Error};
use crate::provider::config::LinterConfig;
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use futures::executor::block_on;
use rslint_config::{ConfigRepr, ConfigStyle};
use rslint_core::CstRuleStore;
use serde::Deserialize;
use serde_json::Value;
use std::{fs::read_to_string, sync::RwLock};
use taplo::{parser::Parse, util::coords::Mapper};
use tower_lsp::lsp_types::ConfigurationItem;
use tower_lsp::{lsp_types::*, Client};

#[serde(rename_all = "camelCase")]
#[derive(Debug, Clone, Deserialize)]
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

fn parse_config_str(string: &str, style: ConfigStyle) -> Option<ConfigRepr> {
    match style {
        ConfigStyle::Toml => toml::from_str(&string).ok(),
        ConfigStyle::Json => serde_json::from_str(&string).ok(),
    }
}

fn try_find_config_file(client: Option<Client>) -> Option<(ConfigRepr, ConfigStyle)> {
    // workspace_folders is async, however, this method is only called once on initialization therefore it is fine to block
    let folders = block_on(client?.workspace_folders()).ok()??;
    let folder = folders.first()?;
    let dir = folder.uri.to_file_path().ok()?;
    let (config_path, style) = rslint_config::Config::find_config(true, Some(dir))?;
    let config_str = read_to_string(config_path).ok()?;

    (parse_config_str(&config_str, style), style)
}

/// Represents the current state of the LSP session.
pub struct Session {
    client: Option<Client>,
    documents: DashMap<Url, Document>,
    pub(crate) store: CstRuleStore,
    pub(crate) config: RwLock<Config>,
    pub(crate) linter_config: RwLock<Option<LinterConfig>>,
}

impl Session {
    /// Create a new session.
    pub fn new(client: Option<Client>) -> anyhow::Result<Self> {
        let documents = DashMap::new();
        let store = CstRuleStore::new().recommended();
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

        let linter_config = if let Some((repr, style)) = try_find_config_file(client.clone()) {
        } else {
            None
        };

        Ok(Session {
            client,
            documents,
            store,
            config,
            config_doc: RwLock::new(None),
            linter_config,
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
    pub async fn get_mut_document(&self, uri: &Url) -> anyhow::Result<RefMut<'_, Url, Document>> {
        self.documents
            .get_mut(uri)
            .ok_or_else(|| Error::DocumentNotFound(uri.clone()).into())
    }

    pub fn update_config_file(&self, new_string: &str, style: ConfigStyle) {
        if let Some(repr) = parse_config_str(new_string, style) {
            *self.linter_config.write().unwrap() = Some(repr);
        }
    }
}
