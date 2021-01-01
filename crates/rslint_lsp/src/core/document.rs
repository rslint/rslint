//! Core definitions related to documents.

use crate::core::language::{Language, LanguageId};
use rslint_core::{autofix::Fixer, Directive, DirectiveError, DirectiveParser, File};
use rslint_errors::file::SimpleFiles;
use rslint_parser::{FileKind, SyntaxNode};
use std::convert::TryFrom;
use tower_lsp::lsp_types::*;

pub struct RuleResult {
    pub diagnostics: Vec<Diagnostic>,
    pub fixer: Option<Fixer>,
}

/// The current state of a document.
pub struct Document {
    /// The file backing of this document
    pub file: File,
    /// The files database containing the document.
    pub files: SimpleFiles,
    /// The language type of the document (e.g., JavaScript (script) or JavaScript (module)).
    pub language: Language,
    /// The language id of the document (e.g., "javascript").
    pub language_id: LanguageId,
    /// The errors from parsing a document.
    pub parsing_errors: Vec<rslint_errors::Diagnostic>,
    /// All directives in this document.
    pub directives: Vec<Directive>,
    /// The errors which occured while parsing the directive
    pub directive_errors: Vec<DirectiveError>,
    /// The result of running rules on the document
    pub rule_results: Vec<RuleResult>,
    pub root: SyntaxNode,
}

impl Document {
    /// Create a new Document.
    pub fn new(uri: Url, language_id: String, text: String) -> anyhow::Result<Self> {
        let language = {
            if let Ok(path) = uri.to_file_path() {
                Language::try_from(path.as_path())?
            } else {
                Language::try_from(LanguageId(language_id.clone()))?
            }
        };

        let mut files = SimpleFiles::new();
        let file_id = files.add(uri.to_string(), text.clone());
        let kind = match language {
            Language::JavaScriptModule => FileKind::Module,
            Language::JavaScriptScript => FileKind::Script,
            Language::TypeScript => FileKind::TypeScript,
        };
        let mut file = File::from_string(text, kind, uri.path());
        file.id = file_id;

        let (parsing_errors, root) = file.parse_with_errors();

        let res = DirectiveParser::new(root.clone(), &file).get_file_directives();

        let document = Document {
            files,
            file,
            language,
            language_id: LanguageId(language_id),
            directives: res.directives,
            parsing_errors,
            directive_errors: res.diagnostics,
            rule_results: vec![],
            root,
        };

        Ok(document)
    }
}
