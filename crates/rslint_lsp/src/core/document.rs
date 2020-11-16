//! Core definitions related to documents.

use crate::core::language::{Language, LanguageId};
use rslint_core::{Directive, DirectiveError, DirectiveParser};
use rslint_errors::file::SimpleFiles;
use rslint_parser::{ast, parse_module, parse_text, GreenNode, Parse, ParserError, SyntaxNode};
use std::convert::TryFrom;
use tower_lsp::lsp_types::*;

/// Trait for working with Parse<T> for a document.
pub trait DocumentParse: Send + Sync {
    /// The GreenNode for a document.
    fn green(&self) -> GreenNode;
    /// The parser diagnostics for a document.
    fn parser_diagnostics(&self) -> &[ParserError];
}

impl DocumentParse for Parse<ast::Module> {
    fn green(&self) -> GreenNode {
        Parse::green(self.clone())
    }

    fn parser_diagnostics(&self) -> &[ParserError] {
        self.errors()
    }
}

impl DocumentParse for Parse<ast::Script> {
    fn green(&self) -> GreenNode {
        Parse::green(self.clone())
    }

    fn parser_diagnostics(&self) -> &[ParserError] {
        self.errors()
    }
}

/// The current state of a document.
pub struct Document {
    /// The files database containing the document.
    pub files: SimpleFiles,
    /// The file id of the document.
    pub file_id: usize,
    /// The language type of the document (e.g., JavaScript (script) or JavaScript (module)).
    pub language: Language,
    /// The language id of the document (e.g., "javascript").
    pub language_id: LanguageId,
    /// The result of parsing a document.
    pub parse: Box<dyn DocumentParse>,
    /// All directives in this document.
    pub directives: Vec<Directive>,
    /// The textual content of the document.
    pub text: String,
    /// The errors which occured while parsing the directive
    pub directive_errors: Vec<DirectiveError>,
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

        let parse = if language == Language::JavaScriptModule {
            Box::new(parse_module(&text, file_id)) as Box<dyn DocumentParse>
        } else {
            Box::new(parse_text(&text, file_id)) as Box<dyn DocumentParse>
        };

        let res = DirectiveParser::new(SyntaxNode::new_root(parse.green()), file_id)
            .get_file_directives();

        let document = Document {
            files,
            file_id,
            language,
            language_id: LanguageId(language_id),
            directives: res.directives,
            parse,
            text,
            directive_errors: res.diagnostics,
        };

        Ok(document)
    }
}
