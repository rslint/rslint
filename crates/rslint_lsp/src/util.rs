use rslint_errors::Span;
use rslint_errors::{file::Files, lsp::*};
use tower_lsp::lsp_types::{CodeAction, CodeActionKind, Diagnostic, TextEdit, Url, WorkspaceEdit};

#[derive(Clone, PartialEq)]
pub struct ActionBuilder {
    inner: CodeAction,
}

impl ActionBuilder {
    pub fn new(title: impl ToString, kind: CodeActionKind) -> Self {
        Self {
            inner: CodeAction {
                title: title.to_string(),
                kind: Some(kind),
                ..Default::default()
            },
        }
    }

    pub fn edit(mut self, edit: impl Into<WorkspaceEdit>) -> Self {
        self.inner.edit = Some(edit.into());
        self
    }

    pub fn preferred(mut self) -> Self {
        self.inner.is_preferred = Some(true);
        self
    }

    pub fn diagnostics(mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        self.inner.diagnostics = Some(diagnostics.into_iter().collect());
        self
    }

    pub fn end(self) -> CodeAction {
        self.inner
    }
}

#[derive(Clone, PartialEq)]
pub struct EditBuilder<'a, F: Files> {
    inner: Vec<TextEdit>,
    files: &'a F,
    file_id: usize,
}

impl<'a, T: Files> EditBuilder<'a, T> {
    pub fn new(files: &'a T, file_id: usize) -> Self {
        Self {
            inner: Default::default(),
            files,
            file_id,
        }
    }

    pub fn delete(mut self, range: impl Span) -> Self {
        self.inner.push(TextEdit {
            range: byte_span_to_range(self.files, self.file_id, range.as_range())
                .expect("Invalid range"),
            new_text: String::new(),
        });
        self
    }

    pub fn insert(mut self, range: impl Span, new_text: impl ToString) -> Self {
        self.inner.push(TextEdit {
            range: byte_span_to_range(self.files, self.file_id, range.as_range())
                .expect("Invalid range"),
            new_text: new_text.to_string(),
        });
        self
    }

    pub fn end(self, url: Url) -> WorkspaceEdit {
        WorkspaceEdit::new(vec![(url, self.inner)].into_iter().collect())
    }
}
