//! Implementation of converting, and emitting diagnostics
//! using `annotate-snippets-rs`.

use crate::annotate_snippets::{display_list as dl, snippet};
use crate::{
    file::{FileId, Files},
    CodeSuggestion, Diagnostic,
};
use std::{
    collections::{hash_map::Entry, HashMap},
    io::{self, BufWriter, Write},
    ops::Range,
};

/// The emitter is responsible for emitting
/// diagnostics to a given output.
pub struct Emitter<'files> {
    color: bool,
    files: &'files dyn Files,
    out: Box<dyn Write>,
}

impl<'files> Emitter<'files> {
    /// Creates a new `Emitter`.
    pub fn new(out: Box<dyn Write>, files: &'files dyn Files, color: bool) -> Self {
        Self { color, files, out }
    }

    /// Creates a new `Emitter` that will output the diagnostics
    /// to stdout.
    pub fn stdout(files: &'files dyn Files, color: bool) -> Self {
        let out = io::stdout();
        let out = BufWriter::new(out);
        Self::new(Box::new(out), files, color)
    }

    /// Creates a new `Emitter` that will output the diagnostics
    /// to stderr.
    pub fn stderr(files: &'files dyn Files, color: bool) -> Self {
        let out = io::stderr();
        let out = BufWriter::new(out);
        Self::new(Box::new(out), files, color)
    }
}

impl Emitter<'_> {
    /// Emit a diagnostic and write it to the output of this `Emitter`.
    ///
    /// Diagnostics that have no primary label, will be displayed without any spans.
    /// Not even secondary ones.
    pub fn emit_diagnostic(&mut self, d: &Diagnostic) -> io::Result<()> {
        let mut slices: HashMap<FileId, snippet::Slice<'_>> = HashMap::new();

        for child in d.primary.iter().chain(&d.children) {
            let Range { start, end } = child.span.range;

            let source = self.files.source(d.file_id);

            let entry = slices.entry(child.span.file);
            let mut tuple = (source, entry);
            let slice = match tuple {
                (Some(source), Entry::Vacant(entry)) => entry.insert(snippet::Slice {
                    source,
                    origin: self.files.name(d.file_id),
                    line_start: 1,
                    annotations: vec![],
                    fold: true,
                }),
                (_, Entry::Occupied(ref mut entry)) => entry.get_mut(),
                _ => continue,
            };

            let annotation = snippet::SourceAnnotation {
                range: (start, end),
                label: &child.msg,
                annotation_type: child.severity.into(),
            };
            slice.annotations.push(annotation);
        }

        enum Suggestion {
            Inline(String),
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
            let inline = msg.len() <= 25;

            if inline {
                let label = if replacement.is_empty() {
                    msg.to_string()
                } else {
                    format!("{}: {}", msg, replacement)
                };

                let suggestion = Suggestion::Inline(label);
                suggestions.push(suggestion);
            } else {
                use std::cmp;

                let label = msg.to_string();
                let source = if let Some(source) = self.files.source(file.unwrap_or(d.file_id)) {
                    let mut source = source.to_string();
                    source.replace_range(start..end, replacement);
                    source
                } else {
                    continue;
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
                    file: *file,
                    span: (start, end),
                };
                suggestions.push(suggestion);
            }
        }

        let mut suggestion_snippets = vec![];
        let mut additional_footers = vec![];

        for sug in suggestions.iter() {
            match sug {
                Suggestion::Inline(label) => {
                    additional_footers.push(snippet::Annotation {
                        id: None,
                        label: Some(label),
                        annotation_type: snippet::AnnotationType::Help,
                    });
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
                        line_start: 1,
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

        let mut footer = Some(
            d.footers
                .iter()
                .map(|footer| snippet::Annotation {
                    id: None,
                    label: Some(&footer.msg),
                    annotation_type: footer.severity.into(),
                })
                .collect::<Vec<_>>(),
        );

        if let Some(last) = suggestion_snippets.last_mut() {
            last.footer = footer.take().into_iter().flatten().collect();
        }

        let snippet = snippet::Snippet {
            title: Some(snippet::Annotation {
                label: Some(&d.title),
                id: d.code.as_deref().filter(|code| !code.is_empty()),
                annotation_type: d.severity.into(),
            }),
            slices: slices.into_iter().map(|(_, v)| v).collect(),
            footer: footer
                .into_iter()
                .flatten()
                .chain(additional_footers)
                .collect(),
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
