pub mod file;

mod diagnostic;
mod suggestion;

pub use diagnostic::{Diagnostic, SubDiagnostic};
pub use suggestion::CodeSuggestion;

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{self, Annotation, AnnotationType, Snippet},
};
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{self, Write},
};

/// Indicicates how a tool should manage this suggestion.
#[derive(Clone, Copy, Debug)]
pub enum Applicability {
    /// The suggestion is definitely what the user intended.
    /// This suggestion should be automatically applied.
    Always,
    /// The suggestion may be what the user intended, but it is uncertain.
    /// The suggestion should result in valid Rust code if it is applied.
    MaybeIncorrect,
    /// The suggestion contains placeholders like `(...)` or `{ /* fields */ }`.
    /// The suggestion cannot be applied automatically because it will not result in valid Rust code.
    /// The user will need to fill in the placeholders.
    HasPlaceholders,
    /// The applicability of the suggestion is unknown.
    Unspecified,
}

/// Types of severity.
#[derive(Clone, Copy, Debug)]
pub enum Severity {
    Error,
    Warning,
    Help,
    Note,
    Info,
}

impl Into<snippet::AnnotationType> for Severity {
    fn into(self) -> snippet::AnnotationType {
        use snippet::AnnotationType::*;

        match self {
            Severity::Error => Error,
            Severity::Warning => Warning,
            Severity::Help => Help,
            Severity::Note => Note,
            Severity::Info => Info,
        }
    }
}

fn convert<'d, 'f: 'd>(
    diagnostic: &'d Diagnostic,
    files: &'f dyn file::Files,
    color: bool,
) -> Vec<Snippet<'d>> {
    let mut slices: HashMap<file::FileId, snippet::Slice<'d>> = HashMap::new();

    for child in &diagnostic.children {
        let (span, file_id) = (child.span.span.clone(), child.span.file);
        let annotation = snippet::SourceAnnotation {
            range: (span.start, span.end),
            label: &child.msg,
            annotation_type: child.severity.into(),
        };
        match slices.entry(file_id) {
            Entry::Vacant(entry) => {
                let source = files.source(file_id);
                let name = files.name(file_id);

                let slice = snippet::Slice {
                    source,
                    line_start: files.line_index(child.span.span.start),
                    origin: Some(name),
                    annotations: vec![annotation],
                    fold: true,
                };

                entry.insert(slice);
            }
            Entry::Occupied(mut entry) => {
                let slice = entry.get_mut();
                slice.annotations.push(annotation);
            }
        }
    }

    let mut snippets = vec![];

    for sug in &diagnostic.suggestions {
        let (file_id, span) = (sug.substitution.0.file, sug.substitution.0.span.clone());

        let snippet = Snippet {
            title: Some(Annotation {
                label: Some(&sug.label),
                id: None,
                annotation_type: AnnotationType::Help,
            }),
            slices: vec![snippet::Slice {
                source: files.source(file_id),
                line_start: files.line_index(span.start),
                origin: Some(files.name(file_id)),
                annotations: vec![snippet::SourceAnnotation {
                    range: (span.start, span.end),
                    label: "",
                    annotation_type: AnnotationType::Help,
                }],
                fold: true,
            }],
            footer: vec![],
            opt: FormatOptions {
                color,
                ..Default::default()
            },
        };
        snippets.push(snippet);
    }

    let snippet = Snippet {
        title: Some(Annotation {
            label: Some(&diagnostic.title),
            id: diagnostic.code.as_deref(),
            annotation_type: diagnostic.severity.into(),
        }),
        slices: slices.into_iter().map(|(_, v)| v).collect(),
        footer: diagnostic
            .footer
            .iter()
            .map(|footer| Annotation {
                id: None,
                label: Some(&footer.label),
                annotation_type: footer.severity.into(),
            })
            .collect(),

        opt: FormatOptions {
            color,
            ..Default::default()
        },
    };
    snippets.push(snippet);

    snippets
}

/// Takes a list of `Diagnostic`s and prints them to given output.
pub fn emit(
    out: &mut dyn Write,
    files: &dyn file::Files,
    color: bool,
    diagnostics: Vec<Diagnostic>,
) -> Result<(), io::Error> {
    diagnostics
        .iter()
        .flat_map(|d| convert(d, files, color))
        .map(|snippet| {
            let dl = DisplayList::from(snippet);
            writeln!(out, "{}", dl)
        })
        .collect::<Result<(), io::Error>>()
}
