//! Core definitions related to the LSP server session.

use crate::core::{document::Document, error::Error};
use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use rslint_core::CstRuleStore;
use tower_lsp::{lsp_types::*, Client};

/// Represents the current state of the LSP session.
pub struct Session {
    client: Option<Client>,
    documents: DashMap<Url, Document>,
    pub(crate) store: CstRuleStore,
}

impl Session {
    /// Create a new session.
    pub fn new(client: Option<Client>) -> anyhow::Result<Self> {
        let documents = DashMap::new();
        let store = CstRuleStore::new().builtins();
        Ok(Session {
            client,
            documents,
            store,
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
}
