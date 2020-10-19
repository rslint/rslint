//! Implementation of converting, and emitting diagnostics
//! using `annotate-snippets-rs`.

use crate::annotate_snippets::{display_list as dl, snippet};
use crate::{
    file::{FileId, FileSpan, Files},
    suggestion::*,
    Diagnostic,
};
use rslint_text_edit::*;
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

        let children = || d.primary.iter().chain(&d.children);
        let source_from_to = |file, from, to| {
            let start_line = self.files.line_index(file, from)?;
            let end_line = self.files.line_index(file, to)?;

            self.files
                .line_range(file, start_line)
                .and_then(|range| (range.start, self.files.line_range(file, end_line)?.end).into())
        };

        let start_line = children().map(|child| child.span.range.start).min();
        let end_line = children().map(|child| child.span.range.end).max();

        let source_range = start_line
            .zip(end_line)
            .and_then(|(from, to)| source_from_to(d.file_id, from, to));

        let source = self.files.source(d.file_id).map(|source| {
            if let Some((from, to)) = source_range {
                &source[from..to]
            } else {
                source
            }
        });
        let source = source.expect("no source code for file is availabe");

        for child in children() {
            let Range { start, end } = child.span.range;
            let offset = source_range.unwrap().0;

            let start = start - offset;
            let end = end - offset;

            let entry = slices.entry(child.span.file);
            let line_index = self.files.line_index(d.file_id, start_line.unwrap());
            let mut triple = (line_index, entry);

            let slice = match triple {
                (Some(line_start), Entry::Vacant(entry)) => entry.insert(snippet::Slice {
                    source,
                    origin: self.files.name(d.file_id),
                    line_start: line_start + 1,
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
                line_start: usize,
                labels: Vec<Range<usize>>,
            },
        }

        let mut suggestions = vec![];

        for CodeSuggestion {
            substitution,
            span: FileSpan { file, range },
            style,
            labels,
            msg,
            ..
        } in &d.suggestions
        {
            let replacement = match substitution {
                SuggestionChange::Indels(indels) => {
                    let mut old = self.files.source(*file).expect("Non existant file id")
                        [range.clone()]
                    .to_owned();
                    apply_indels(&indels, &mut old);
                    old
                }
                SuggestionChange::String(string) => string.clone(),
            };

            let Range { mut start, mut end } = range.clone();
            if let SuggestionStyle::Inline | SuggestionStyle::HideCode = style {
                let label = if replacement.is_empty() || *style == SuggestionStyle::HideCode {
                    msg.clone()
                } else {
                    format!("{}: {}", msg, replacement)
                };

                let suggestion = Suggestion::Inline(label);
                suggestions.push(suggestion);
            } else {
                use std::cmp;

                let label = msg.to_string();
                let (source, offset) = if let Some(source) = self.files.source(*file) {
                    let (source_start, source_end) = source_from_to(*file, start, end)
                        .expect("failed to get source range for file");
                    start -= source_start;
                    end -= source_start;

                    let source = &source[source_start..source_end];
                    let mut source = source.to_string();
                    source.replace_range(start..end, &replacement);
                    (source, source_start)
                } else {
                    continue;
                };

                let labels = labels
                    .iter()
                    .cloned()
                    .map(|range| range.start - offset..range.end - offset)
                    .collect::<Vec<_>>();

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
                    file: Some(*file).filter(|id| *id != d.file_id),
                    span: (start, end),
                    line_start: self.files.line_index(*file, start + offset).unwrap() + 1,
                    labels,
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
                    line_start,
                    file,
                    labels,
                } => {
                    let annotations: Vec<_> = labels
                        .iter()
                        .map(|x| snippet::SourceAnnotation {
                            range: (x.start, x.end),
                            label: "",
                            annotation_type: snippet::AnnotationType::Info,
                        })
                        .collect();

                    let slice = snippet::Slice {
                        source,
                        line_start: *line_start,
                        origin: file.and_then(|file| self.files.name(file)),
                        annotations: if annotations.is_empty() {
                            vec![snippet::SourceAnnotation {
                                range: (*start, *end),
                                label: "",
                                annotation_type: snippet::AnnotationType::Info,
                            }]
                        } else {
                            annotations
                        },
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
