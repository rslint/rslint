//! Core definitions related to language types for documents.

use crate::core::error::Error;
use std::{convert::TryFrom, path::Path};

/// A language type for a document (e.g., JavaScript (script) or JavaScript (module) or TypeScript).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    /// A JavaScript (script)
    JavaScriptScript,
    /// JavaScript (module)
    JavaScriptModule,
    /// TypeScript
    TypeScript,
}

/// A language id for a document (e.g., JavaScript (script) or JavaScript (module)).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageId(pub String);

impl From<Language> for LanguageId {
    fn from(language: Language) -> Self {
        match language {
            Language::JavaScriptScript => LanguageId("javascript".into()),
            Language::JavaScriptModule => LanguageId("javascript".into()),
            Language::TypeScript => LanguageId("typescript".into()),
        }
    }
}

impl TryFrom<&Path> for Language {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> anyhow::Result<Self> {
        let file_ext = path
            .extension()
            .ok_or_else(|| Error::PathExtensionFailed(path.into()))?;
        let file_ext = file_ext.to_str().ok_or(Error::ToStrFailed)?;
        match file_ext {
            "mjs" => Ok(Language::JavaScriptModule),
            "js" => Ok(Language::JavaScriptScript),
            "ts" => Ok(Language::TypeScript),
            _ => Err(Error::InvalidLanguageExtension(file_ext.into()).into()),
        }
    }
}

impl TryFrom<LanguageId> for Language {
    type Error = anyhow::Error;

    fn try_from(id: LanguageId) -> anyhow::Result<Self> {
        // NOTE: unfortunately there isn't a separate commonly used id for modules, so we just default to module.
        match id.0.as_str() {
            "javascript" => Ok(Language::JavaScriptModule),
            "typescript" => Ok(Language::TypeScript),
            _ => Err(Error::InvalidLanguageId(id.0).into()),
        }
    }
}
