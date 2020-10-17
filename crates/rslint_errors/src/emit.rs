//! Implementation of converting, and emitting diagnostics
//! using `annotate-snippets-rs`.

use crate::{
    file::{FileId, Files},
    CodeSuggestion, Diagnostic,
};
use annotate_snippets::{display_list as dl, snippet};
use std::{
    collections::HashMap,
    io::{self, BufWriter, Write},
    ops::Range,
};

/// The emitter is responsible for emitting
/// diagnostics to a given output.
pub struct Emitter {
    color: bool,
    files: Box<dyn Files>,
    out: Box<dyn Write>,
}

impl Emitter {
    /// Creates a new `Emitter`.
    pub fn new(out: Box<dyn Write>, files: Box<dyn Files>, color: bool) -> Self {
        Self { color, files, out }
    }

    /// Creates a new `Emitter` that will output the diagnostics
    /// to stdout.
    pub fn stdout(files: Box<dyn Files>, color: bool) -> Self {
        let out = io::stdout();
        let out = BufWriter::new(out);
        Self::new(Box::new(out), files, color)
    }

    /// Creates a new `Emitter` that will output the diagnostics
    /// to stderr.
    pub fn stderr(files: Box<dyn Files>, color: bool) -> Self {
        let out = io::stderr();
        let out = BufWriter::new(out);
        Self::new(Box::new(out), files, color)
    }
}

impl Emitter {
    pub fn emit_diagnostic(&mut self, d: &Diagnostic) -> io::Result<()> {
        if d.primary.is_none() {
            return Ok(());
        }

        let mut slices: HashMap<FileId, snippet::Slice<'_>> = HashMap::new();

        for child in d.primary.iter().chain(&d.children) {
            let Range { start, end } = child.span.span;

            let entry = slices
                .entry(child.span.file)
                .or_insert_with(|| snippet::Slice {
                    source: self.files.source(d.file_id),
                    origin: self.files.name(d.file_id),
                    line_start: self.files.line_index(d.file_id, start).max(1),
                    annotations: vec![],
                    fold: true,
                });

            let annotation = snippet::SourceAnnotation {
                range: (start, end),
                label: &child.msg,
                annotation_type: child.severity.into(),
            };
            entry.annotations.push(annotation);
        }

        enum Suggestion {
            Inline {
                label: String,
                file: FileId,
                span: (usize, usize),
            },
            Additional {
                label: String,
                source: String,
                file: Option<FileId>,
                span: (usize, usize),
            },
        }

        let mut suggestions = vec![];

        for CodeSuggestion {
            substitution: (file, range, replacement),
            msg,
            ..
        } in &d.suggestions
        {
            let Range { start, end } = range.clone();
            let inline = msg.len() + replacement.len() + 2 <= 25;

            if inline {
                let label = if replacement.is_empty() {
                    msg.to_string()
                } else {
                    format!("{}: {}", msg, replacement)
                };

                let suggestion = Suggestion::Inline {
                    label,
                    file: file.unwrap_or(d.file_id),
                    span: (start, end),
                };
                suggestions.push(suggestion);
            } else {
                use std::cmp;

                let label = msg.to_string();
                let source = {
                    let mut source = self.files.source(file.unwrap_or(d.file_id)).to_string();
                    source.replace_range(start..end, replacement);
                    source
                };

                let (start, end) = (
                    start,
                    cmp::min(start + replacement.len().max(1), source.len()),
                );

                let start = if start > source.len() - 1 {
                    cmp::max(start, source.len()) - 1
                } else {
                    start
                };

                let suggestion = Suggestion::Additional {
                    label,
                    source,
                    file: file.clone(),
                    span: dbg!((start, end)),
                };
                suggestions.push(suggestion);
            }
        }

        let mut suggestion_snippets = vec![];

        for sug in suggestions.iter() {
            match sug {
                Suggestion::Inline { label, file, span } => {
                    let entry = slices
                        .get_mut(&file)
                        .expect("invalid file id provided for suggestion");
                    let annotation = snippet::SourceAnnotation {
                        range: span.clone(),
                        label,
                        annotation_type: snippet::AnnotationType::Help,
                    };
                    entry.annotations.push(annotation);
                }
                Suggestion::Additional {
                    label,
                    source,
                    span: (start, end),
                    file,
                } => {
                    let annotation = snippet::SourceAnnotation {
                        range: (*start, *end),
                        label: "",
                        annotation_type: snippet::AnnotationType::Help,
                    };

                    let slice = snippet::Slice {
                        source,
                        line_start: self
                            .files
                            .line_index(file.unwrap_or(d.file_id), *start)
                            .max(1),
                        origin: file.and_then(|file| self.files.name(file)),
                        annotations: vec![annotation],
                        fold: true,
                    };

                    let snippet = snippet::Snippet {
                        title: Some(snippet::Annotation {
                            label: Some(&label),
                            id: None,
                            annotation_type: snippet::AnnotationType::Help,
                        }),
                        slices: vec![slice],
                        footer: vec![],
                        opt: dl::FormatOptions {
                            color: self.color,
                            ..Default::default()
                        },
                    };

                    suggestion_snippets.push(snippet);
                }
            }
        }

        let footer = d
            .footer
            .iter()
            .map(|footer| snippet::Annotation {
                id: None,
                label: Some(&footer.label),
                annotation_type: footer.severity.into(),
            })
            .collect::<Vec<_>>();

        let snippet = snippet::Snippet {
            title: Some(snippet::Annotation {
                label: Some(&d.title),
                id: d.code.as_deref(),
                annotation_type: d.severity.into(),
            }),
            slices: slices.into_iter().map(|(_, v)| v).collect(),
            footer,
            opt: dl::FormatOptions {
                color: self.color,
                ..Default::default()
            },
        };

        let strings = std::iter::once(snippet)
            .chain(suggestion_snippets)
            .map(|snippet| dl::DisplayList::from(snippet).to_string())
            // FIXME(Stupremee): Please somehow avoid this stupid allocatin.
            .collect::<Vec<_>>();

        for snippet in strings {
            writeln!(self.out, "{}", snippet)?;
        }
        Ok(())
    }
}
