use crate::file::FileId;
use std::ops::Range;

/// Types of severity.
#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Error,
    Warning,
    Help,
    Note,
    Info,
}

/// Styles for a [`Label`].
#[derive(Clone, Copy, Debug)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

/// Any object that can be turned into a [`Range`](std::ops::Range).
pub trait Span {
    fn as_range(&self) -> Range<usize>;
}

/// A diagnostic message that can give information
/// like errors or warnings.
pub struct Diagnostic<'str> {
    file_id: FileId,
    code: &'str str,
    title: &'str str,
    severity: Severity,
    labels: Vec<Label<'str>>,
}

impl<'str> Diagnostic<'str> {
    /// Creates a new [`Diagnostic`] that will be used in a builder-like way
    /// to modify labels, and suggestions.
    pub fn new<'code: 'str, 'title: 'str>(
        file_id: FileId,
        severity: Severity,
        code: &'code str,
        title: &'title str,
    ) -> Self {
        Self {
            file_id,
            code,
            title,
            severity,
            labels: vec![],
        }
    }

    /// Attaches a primary label to this [`Diagnostic`].
    pub fn primary<'label: 'str>(mut self, span: impl Span, label: &'label str) -> Self {
        self.labels.push(Label {
            range: span.as_range(),
            style: LabelStyle::Primary,
            label,
        });
        self
    }

    /// Attaches a secondary label to this [`Diagnostic`].
    pub fn secondary<'label: 'str>(mut self, span: impl Span, label: &'label str) -> Self {
        self.labels.push(Label {
            range: span.as_range(),
            style: LabelStyle::Secondary,
            label,
        });
        self
    }
}

/// Structure that represents a range of text to be highlighted with a label.
pub struct Label<'str> {
    label: &'str str,
    range: Range<usize>,
    style: LabelStyle,
}
