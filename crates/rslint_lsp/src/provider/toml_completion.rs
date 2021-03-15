//! Autocompletion for rslintrc.toml
//!
//! This is taken from `taplo_ide`, all credit for the implementation goes to them

use std::collections::HashSet;

use crate::core::session::TomlDocument;
use itertools::Itertools;
use schemars::{
    schema::{InstanceType, RootSchema, Schema, SingleOrVec},
    Map,
};
use serde_json::Value;
use taplo::{
    analytics::NodeRef,
    dom::{self, RootNode},
    schema::util::{get_schema_objects, ExtendedSchema},
    syntax::SyntaxKind,
};
use tower_lsp::lsp_types::*;

pub(crate) fn toml_completions(
    doc: &TomlDocument,
    position: Position,
    root_schema: RootSchema,
) -> Vec<CompletionItem> {
    let dom = doc.parse.clone().into_dom();
    let paths: HashSet<dom::Path> = dom.iter().map(|(p, _)| p).collect();

    let offset = doc.mapper.offset(position).unwrap();

    let query = dom.query_position(offset);

    if !query.is_completable() {
        return Vec::new();
    }

    match &query.before {
        Some(before) => {
            if query.is_inside_header() {
                let mut query_path = before.path.clone();
                if query.is_empty_header() {
                    query_path = dom::Path::new();
                } else if query_path.is_empty() {
                    query_path = query.after.path.clone();
                }

                // We always include the current object as well.
                query_path = query_path.skip_right(1);

                let range = before
                    .syntax
                    .range
                    .map(|range| doc.mapper.range(range).unwrap())
                    .or_else(|| {
                        query
                            .after
                            .syntax
                            .range
                            .map(|range| doc.mapper.range(range).unwrap())
                    });

                return get_schema_objects(query_path.clone(), &root_schema, true)
                    .into_iter()
                    .map(|s| s.descendants(&root_schema.definitions, 10))
                    .flatten()
                    .filter(|(_, s, _)| !s.is_hidden())
                    .filter(|(p, ..)| {
                        if let Some(same_path) = before.syntax.key_path.as_ref() {
                            if p == same_path {
                                true
                            } else {
                                valid_key(&query_path.extend(p.clone()), &paths, &dom)
                            }
                        } else {
                            valid_key(&query_path.extend(p.clone()), &paths, &dom)
                        }
                    })
                    .filter(|(_, s, _)| {
                        if query
                            .after
                            .syntax
                            .syntax_kinds
                            .iter()
                            .any(|kind| *kind == SyntaxKind::TABLE_ARRAY_HEADER)
                        {
                            s.is_array_of_objects(&root_schema.definitions)
                        } else {
                            s.is(InstanceType::Object)
                        }
                    })
                    .unique_by(|(p, ..)| p.clone())
                    .map(|(path, schema, required)| {
                        key_completion(
                            &root_schema.definitions,
                            query_path.without_index().extend(path),
                            schema,
                            required,
                            range,
                            false,
                            None,
                            false,
                        )
                    })
                    .collect();
            } else {
                let node = before
                    .nodes
                    .last()
                    .cloned()
                    .unwrap_or_else(|| query.after.nodes.last().cloned().unwrap());

                match node {
                    node @ NodeRef::Table(_) | node @ NodeRef::Root(_) => {
                        let mut query_path = before.path.clone();

                        if node.is_root() {
                            // Always full path.
                            query_path = dom::Path::new();
                        }

                        let range = before
                            .syntax
                            .range
                            .map(|range| doc.mapper.range(range).unwrap())
                            .or_else(|| {
                                query
                                    .after
                                    .syntax
                                    .range
                                    .map(|range| doc.mapper.range(range).unwrap())
                            });

                        let comma_before = false;
                        let additional_edits = Vec::new();

                        // FIXME: comma insertion before entry
                        // if inline_table {
                        //     if let Some((tok_range, tok)) = before.syntax.first_token_before() {
                        //         if tok.kind() != SyntaxKind::COMMA
                        //             && tok.kind() != SyntaxKind::BRACE_START
                        //         {
                        //             let range_after = TextRange::new(
                        //                 tok_range.end(),
                        //                 tok_range.end() + TextSize::from(1),
                        //             );

                        //             additional_edits.push(TextEdit {
                        //                 range: doc.mapper.range(range_after).unwrap(),
                        //                 new_text: ",".into(),
                        //             })
                        //         }
                        //     }

                        //     let current_token =
                        //         before.syntax.element.as_ref().unwrap().as_token().unwrap();

                        //     if current_token.kind() != SyntaxKind::WHITESPACE
                        //         && current_token.kind() != SyntaxKind::COMMA
                        //     {
                        //         comma_before = true;
                        //     }

                        //     range = None;
                        // }

                        return get_schema_objects(query_path.clone(), &root_schema, true)
                            .into_iter()
                            .map(|s| s.descendants(&root_schema.definitions, 10))
                            .flatten()
                            .filter(|(_, s, _)| !s.is_hidden())
                            .filter(|(p, ..)| {
                                if let Some(same_path) = before.syntax.key_path.as_ref() {
                                    if p == same_path {
                                        true
                                    } else {
                                        valid_key(
                                            &query_path.extend(if node.is_root() {
                                                query_path.extend(p.clone())
                                            } else {
                                                p.clone()
                                            }),
                                            &paths,
                                            &dom,
                                        )
                                    }
                                } else {
                                    valid_key(
                                        &query_path.extend(if node.is_root() {
                                            query_path.extend(p.clone())
                                        } else {
                                            p.clone()
                                        }),
                                        &paths,
                                        &dom,
                                    )
                                }
                            })
                            .unique_by(|(p, ..)| p.clone())
                            .map(|(path, schema, required)| {
                                key_completion(
                                    &root_schema.definitions,
                                    if node.is_root() {
                                        query_path.extend(path)
                                    } else {
                                        path
                                    },
                                    schema,
                                    required,
                                    range,
                                    true,
                                    if !additional_edits.is_empty() {
                                        Some(additional_edits.clone())
                                    } else {
                                        None
                                    },
                                    comma_before,
                                )
                            })
                            .collect();
                    }
                    NodeRef::Entry(_) => {
                        // Value completion.
                        let query_path = before.path.clone();

                        let range = before
                            .syntax
                            .range
                            .map(|range| doc.mapper.range(range).unwrap())
                            .or_else(|| {
                                query
                                    .after
                                    .syntax
                                    .range
                                    .map(|range| doc.mapper.range(range).unwrap())
                            });

                        return get_schema_objects(query_path, &root_schema, true)
                            .into_iter()
                            .map(|schema| {
                                value_completions(
                                    &root_schema.definitions,
                                    schema,
                                    range,
                                    None,
                                    false,
                                    true,
                                )
                            })
                            .flatten()
                            .unique_by(|comp| comp.insert_text.clone())
                            .collect();
                    }
                    NodeRef::Array(_) => {
                        // Value completion inside an array.
                        let query_path = before.path.clone();

                        let comma_before = false;
                        let additional_edits = Vec::new();

                        // FIXME: comma insertion before entry
                        // if let Some((tok_range, tok)) = before.syntax.first_token_before() {
                        //     if tok.kind() != SyntaxKind::COMMA
                        //         && tok.kind() != SyntaxKind::BRACKET_START
                        //     {
                        //         let range_after = TextRange::new(
                        //             tok_range.end(),
                        //             tok_range.end() + TextSize::from(1),
                        //         );

                        //         additional_edits.push(TextEdit {
                        //             range: doc.mapper.range(range_after).unwrap(),
                        //             new_text: ",".into(),
                        //         })
                        //     }
                        // }

                        // let current_token =
                        //     before.syntax.element.as_ref().unwrap().as_token().unwrap();

                        // if current_token.kind() != SyntaxKind::WHITESPACE
                        //     && current_token.kind() != SyntaxKind::COMMA
                        // {
                        //     comma_before = true;
                        // }

                        return get_schema_objects(query_path.clone(), &root_schema, true)
                            .into_iter()
                            .filter_map(|s| match query_path.last() {
                                Some(k) => {
                                    if k.is_key() {
                                        s.schema.array.as_ref().and_then(|arr| match &arr.items {
                                            Some(items) => match items {
                                                SingleOrVec::Single(item) => {
                                                    Some(ExtendedSchema::resolved(
                                                        &root_schema.definitions,
                                                        &*item,
                                                    ))
                                                }
                                                SingleOrVec::Vec(_) => None, // FIXME: handle this (hard).
                                            },
                                            None => None,
                                        })
                                    } else {
                                        None
                                    }
                                }
                                None => s.schema.array.as_ref().and_then(|arr| match &arr.items {
                                    Some(items) => match items {
                                        SingleOrVec::Single(item) => {
                                            Some(ExtendedSchema::resolved(
                                                &root_schema.definitions,
                                                &*item,
                                            ))
                                        }
                                        SingleOrVec::Vec(_) => None, // FIXME: handle this (hard).
                                    },
                                    None => None,
                                }),
                            })
                            .flatten()
                            .map(|schema| {
                                value_completions(
                                    &root_schema.definitions,
                                    schema,
                                    None,
                                    if additional_edits.is_empty() {
                                        None
                                    } else {
                                        Some(additional_edits.clone())
                                    },
                                    comma_before,
                                    false,
                                )
                            })
                            .flatten()
                            .unique_by(|comp| comp.insert_text.clone())
                            .collect();
                    }
                    NodeRef::Value(_) => {
                        let query_path = before.path.clone();

                        let range = before
                            .syntax
                            .element
                            .as_ref()
                            .map(|el| doc.mapper.range(el.text_range()).unwrap());

                        return get_schema_objects(query_path, &root_schema, true)
                            .into_iter()
                            .map(|schema| {
                                value_completions(
                                    &root_schema.definitions,
                                    schema,
                                    range,
                                    None,
                                    false,
                                    false,
                                )
                            })
                            .flatten()
                            .unique_by(|comp| comp.insert_text.clone())
                            .collect();
                    }
                    _ => {
                        // Look for an incomplete key.
                        if let Some(before_node) = before.nodes.last() {
                            if before_node.is_key() {
                                let query_path = before.path.skip_right(1);

                                let mut is_root = true;

                                for node in &before.nodes {
                                    if let NodeRef::Table(t) = node {
                                        if !t.is_pseudo() {
                                            is_root = false;
                                        }
                                    };
                                }

                                let range = before
                                    .syntax
                                    .range
                                    .map(|range| doc.mapper.range(range).unwrap());

                                return get_schema_objects(query_path.clone(), &root_schema, true)
                                    .into_iter()
                                    .map(|s| s.descendants(&root_schema.definitions, 10))
                                    .flatten()
                                    .filter(|(_, s, _)| !s.is_hidden())
                                    .unique_by(|(p, ..)| p.clone())
                                    .map(|(path, schema, required)| {
                                        key_completion(
                                            &root_schema.definitions,
                                            if is_root {
                                                query_path.extend(path)
                                            } else {
                                                path
                                            },
                                            schema,
                                            required,
                                            range,
                                            false,
                                            None,
                                            false,
                                        )
                                    })
                                    .collect();
                            }
                        }
                    }
                }
            }
        }
        None => {
            // Start of the document
            let node = query.after.nodes.last().cloned().unwrap();

            if node.is_root() {
                let mut query_path = query.after.path.clone();

                query_path = query_path.skip_right(1);

                let range = query
                    .after
                    .syntax
                    .range
                    .map(|range| doc.mapper.range(range).unwrap());

                return get_schema_objects(query_path.clone(), &root_schema, true)
                    .into_iter()
                    .map(|s| s.descendants(&root_schema.definitions, 10))
                    .flatten()
                    .filter(|(_, s, _)| !s.is_hidden())
                    .unique_by(|(p, ..)| p.clone())
                    .map(|(path, schema, required)| {
                        key_completion(
                            &root_schema.definitions,
                            query_path.extend(path),
                            schema,
                            required,
                            range,
                            true,
                            None,
                            false,
                        )
                    })
                    .collect();
            }
        }
    }

    Vec::new()
}

fn detail_text(schema: Option<ExtendedSchema>, text: Option<&str>) -> Option<String> {
    if schema.is_none() && text.is_none() {
        return None;
    }

    let schema_title = schema
        .and_then(|o| o.schema.metadata.as_ref())
        .and_then(|meta| meta.title.clone())
        .unwrap_or_default();

    Some(format!(
        "{text}{schema}",
        schema = if schema_title.is_empty() {
            "".into()
        } else if text.is_none() {
            format!("({})", schema_title)
        } else {
            format!(" ({})", schema_title)
        },
        text = text.map(|t| t.to_string()).unwrap_or_default()
    ))
}

fn key_documentation(schema: ExtendedSchema) -> Option<Documentation> {
    schema
        .ext
        .docs
        .as_ref()
        .and_then(|docs| docs.main.as_ref())
        .map(|doc| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.clone(),
            })
        })
        .or_else(|| {
            schema
                .schema
                .metadata
                .as_ref()
                .and_then(|meta| meta.description.clone())
                .map(|desc| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: desc,
                    })
                })
        })
}

#[allow(clippy::too_many_arguments)]
fn key_completion(
    _defs: &Map<String, Schema>,
    path: dom::Path,
    schema: ExtendedSchema,
    required: bool,
    range: Option<Range>,
    eq: bool,
    additional_text_edits: Option<Vec<TextEdit>>,
    comma_before: bool,
) -> CompletionItem {
    let insert_text = if eq {
        with_comma(format!("{} = ", path.dotted()), comma_before)
    } else {
        with_comma(path.dotted(), comma_before)
    };

    CompletionItem {
        label: path.dotted(),
        additional_text_edits,
        sort_text: if required {
            Some(required_text(&path.dotted()))
        } else {
            None
        },
        text_edit: range.map(|range| {
            CompletionTextEdit::Edit(TextEdit {
                range,
                new_text: insert_text.clone(),
            })
        }),
        insert_text: Some(insert_text),
        kind: if schema.is(InstanceType::Object) {
            Some(CompletionItemKind::Struct)
        } else {
            Some(CompletionItemKind::Variable)
        },
        detail: detail_text(
            Some(schema.clone()),
            if required { Some("required") } else { None },
        ),
        documentation: key_documentation(schema.clone()),
        preselect: Some(true),
        ..Default::default()
    }
}

fn const_value_documentation(schema: ExtendedSchema) -> Option<Documentation> {
    schema.ext.docs.as_ref().and_then(|d| {
        d.const_value.as_ref().map(|doc| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.clone(),
            })
        })
    })
}

fn default_value_documentation(schema: ExtendedSchema) -> Option<Documentation> {
    schema.ext.docs.as_ref().and_then(|d| {
        d.default_value.as_ref().map(|doc| {
            Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc.clone(),
            })
        })
    })
}

fn enum_documentation(schema: ExtendedSchema, idx: usize) -> Option<Documentation> {
    schema.ext.docs.as_ref().and_then(|d| {
        d.enum_values.as_ref().and_then(|doc| {
            doc.get(idx).and_then(|d| {
                d.as_ref().map(|doc| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: doc.clone(),
                    })
                })
            })
        })
    })
}

fn value_completions(
    defs: &Map<String, Schema>,
    schema: ExtendedSchema,
    range: Option<Range>,
    additional_text_edits: Option<Vec<TextEdit>>,
    comma_before: bool,
    space_before: bool,
) -> Vec<CompletionItem> {
    // Only one constant allowed.
    if let Some(c) = &schema.schema.const_value {
        return value_insert(c, range, comma_before, space_before)
            .map(|value_completion| {
                vec![CompletionItem {
                    additional_text_edits,
                    detail: detail_text(Some(schema.clone()), None),
                    documentation: const_value_documentation(schema.clone()),
                    preselect: Some(true),
                    ..value_completion
                }]
            })
            .unwrap_or_default();
    }

    // Enums only if there are any.
    if let Some(e) = &schema.schema.enum_values {
        return e
            .iter()
            .enumerate()
            .map(|(i, e)| {
                value_insert(e, range, comma_before, space_before).map(|value_completion| {
                    CompletionItem {
                        additional_text_edits: additional_text_edits.clone(),
                        detail: detail_text(Some(schema.clone()), None),
                        documentation: enum_documentation(schema.clone(), i),
                        preselect: Some(true),
                        ..value_completion
                    }
                })
            })
            .flatten()
            .collect();
    }

    if let Some(default) = schema
        .schema
        .metadata
        .as_ref()
        .and_then(|m| m.default.as_ref())
    {
        if let Some(value_completion) = value_insert(default, range, comma_before, space_before) {
            return vec![CompletionItem {
                additional_text_edits,
                detail: detail_text(Some(schema.clone()), None),
                documentation: default_value_documentation(schema.clone()),
                preselect: Some(true),
                sort_text: Some(format!("{}", 1 as char)),
                ..value_completion
            }];
        }
    }

    let mut completions = Vec::new();

    // Default values.
    match &schema.schema.instance_type {
        Some(tys) => match tys {
            SingleOrVec::Single(ty) => {
                if let Some(c) = empty_value_inserts(
                    defs,
                    schema.clone(),
                    **ty,
                    range,
                    comma_before,
                    space_before,
                ) {
                    for value_completion in c {
                        completions.push(CompletionItem {
                            additional_text_edits: additional_text_edits.clone(),
                            detail: detail_text(Some(schema.clone()), None),
                            preselect: Some(true),
                            ..value_completion
                        });
                    }
                }
            }
            SingleOrVec::Vec(tys) => {
                for ty in tys {
                    if let Some(c) = empty_value_inserts(
                        defs,
                        schema.clone(),
                        *ty,
                        range,
                        comma_before,
                        space_before,
                    ) {
                        for value_completion in c {
                            completions.push(CompletionItem {
                                additional_text_edits: additional_text_edits.clone(),
                                detail: detail_text(Some(schema.clone()), None),
                                preselect: Some(true),
                                ..value_completion
                            });
                        }
                    }
                }
            }
        },
        None => {}
    }

    completions
}

fn with_comma(text: String, comma_before: bool) -> String {
    if comma_before {
        format!(", {}", text)
    } else {
        text
    }
}

fn with_leading_space(text: String, space_before: bool) -> String {
    if space_before {
        format!(" {}", text)
    } else {
        text
    }
}

// To make sure required completions are at the top, we prefix it
// with an invisible character
fn required_text(key: &str) -> String {
    format!("{}{}", 1 as char, key)
}

fn value_insert(
    value: &Value,
    range: Option<Range>,
    comma_before: bool,
    space_before: bool,
) -> Option<CompletionItem> {
    match value {
        Value::Object(_) => {
            let insert_text = format_value(value, true, 0);

            Some(CompletionItem {
                label: "table".into(),
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(insert_text.clone(), space_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Struct),
                insert_text_format: Some(InsertTextFormat::Snippet),
                insert_text: Some(with_leading_space(
                    with_comma(insert_text, comma_before),
                    space_before,
                )),
                ..Default::default()
            })
        }
        Value::Bool(_) => {
            let insert_text = format_value(value, true, 0);

            Some(CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(insert_text.clone(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Constant),
                insert_text_format: Some(InsertTextFormat::Snippet),
                insert_text: Some(with_leading_space(
                    with_comma(insert_text, comma_before),
                    space_before,
                )),
                label: format_value(value, false, 0),
                ..Default::default()
            })
        }
        Value::Number(_) => {
            let insert_text = format_value(value, true, 0);

            Some(CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(insert_text.clone(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Constant),
                insert_text_format: Some(InsertTextFormat::Snippet),
                insert_text: Some(with_leading_space(
                    with_comma(insert_text, comma_before),
                    space_before,
                )),
                label: format_value(value, false, 0),
                ..Default::default()
            })
        }
        Value::String(_) => {
            let insert_text = format_value(value, true, 0);

            Some(CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(insert_text.clone(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Constant),
                insert_text: Some(with_leading_space(
                    with_comma(insert_text, comma_before),
                    space_before,
                )),
                label: format_value(value, false, 0),
                insert_text_format: Some(InsertTextFormat::Snippet),
                ..Default::default()
            })
        }
        Value::Array(_) => {
            let insert_text = format_value(value, true, 0);

            Some(CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(insert_text.clone(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Constant),
                insert_text_format: Some(InsertTextFormat::Snippet),
                insert_text: Some(with_leading_space(
                    with_comma(insert_text, comma_before),
                    space_before,
                )),
                label: "array".into(),
                ..Default::default()
            })
        }
        Value::Null => None,
    }
}

fn empty_value_inserts(
    defs: &Map<String, Schema>,
    schema: ExtendedSchema,
    ty: InstanceType,
    range: Option<Range>,
    comma_before: bool,
    space_before: bool,
) -> Option<Vec<CompletionItem>> {
    match ty {
        InstanceType::Boolean => Some(vec![
            CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma("true".into(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Value),
                insert_text: Some(with_leading_space(
                    with_comma("true".into(), comma_before),
                    space_before,
                )),
                insert_text_format: Some(InsertTextFormat::Snippet),
                label: "true".into(),
                ..Default::default()
            },
            CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma("false".into(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Value),
                insert_text: Some(with_leading_space(
                    with_comma("${0:false}".into(), comma_before),
                    space_before,
                )),
                insert_text_format: Some(InsertTextFormat::Snippet),
                label: "false".into(),
                ..Default::default()
            },
        ]),
        InstanceType::Array => Some(vec![CompletionItem {
            text_edit: range.map(|range| {
                CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: with_leading_space(
                        with_comma("[ $0 ]".into(), comma_before),
                        space_before,
                    ),
                })
            }),
            kind: Some(CompletionItemKind::Value),
            insert_text: Some(with_leading_space(
                with_comma("[ $0 ]".into(), comma_before),
                space_before,
            )),
            insert_text_format: Some(InsertTextFormat::Snippet),
            label: "empty array".into(),
            ..Default::default()
        }]),
        InstanceType::Number => Some(vec![CompletionItem {
            text_edit: range.map(|range| {
                CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: with_comma("${0:0.0}".into(), comma_before),
                })
            }),
            kind: Some(CompletionItemKind::Value),
            insert_text: Some(with_leading_space(
                with_comma("${0:0.0}".into(), comma_before),
                space_before,
            )),
            insert_text_format: Some(InsertTextFormat::Snippet),
            label: "number".into(),
            ..Default::default()
        }]),
        InstanceType::String => Some(vec![
            CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(r#""$0""#.into(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Value),
                insert_text: Some(with_leading_space(
                    with_comma(r#""$0""#.into(), comma_before),
                    space_before,
                )),
                insert_text_format: Some(InsertTextFormat::Snippet),
                sort_text: Some(required_text("1string")),
                label: "string".into(),
                ..Default::default()
            },
            CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(r#""""$0""""#.into(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Value),
                insert_text: Some(with_leading_space(
                    with_comma(r#""""$0""""#.into(), comma_before),
                    space_before,
                )),
                insert_text_format: Some(InsertTextFormat::Snippet),
                sort_text: Some(required_text("2multi-line string")),
                label: "multi-line string".into(),
                ..Default::default()
            },
            CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(r#"'$0'"#.into(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Value),
                insert_text: Some(with_leading_space(
                    with_comma(r#"'$0'"#.into(), comma_before),
                    space_before,
                )),
                insert_text_format: Some(InsertTextFormat::Snippet),
                sort_text: Some("3literal string".into()),
                label: "literal string".into(),
                ..Default::default()
            },
            CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(r#"'''$0'''"#.into(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Value),
                insert_text: Some(with_leading_space(
                    with_comma(r#"'''$0'''"#.into(), comma_before),
                    space_before,
                )),
                insert_text_format: Some(InsertTextFormat::Snippet),
                sort_text: Some("4multi-line literal string".into()),
                label: "multi-line literal string".into(),
                ..Default::default()
            },
        ]),
        InstanceType::Integer => Some(vec![CompletionItem {
            text_edit: range.map(|range| {
                CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: with_leading_space(
                        with_comma("${0:0}".into(), comma_before),
                        space_before,
                    ),
                })
            }),
            kind: Some(CompletionItemKind::Value),
            insert_text: Some(with_leading_space(
                with_comma("${0:0}".into(), comma_before),
                space_before,
            )),
            insert_text_format: Some(InsertTextFormat::Snippet),
            label: "integer".into(),
            ..Default::default()
        }]),
        InstanceType::Object => match &schema.schema.object {
            Some(o) => {
                if o.properties.is_empty() {
                    Some(vec![CompletionItem {
                        text_edit: range.map(|range| {
                            CompletionTextEdit::Edit(TextEdit {
                                range,
                                new_text: with_leading_space(
                                    with_comma(r#"{ $0 }"#.into(), comma_before),
                                    space_before,
                                ),
                            })
                        }),
                        kind: Some(CompletionItemKind::Value),
                        insert_text: Some(with_leading_space(
                            with_comma(r#"{ $0 }"#.into(), comma_before),
                            space_before,
                        )),
                        insert_text_format: Some(InsertTextFormat::Snippet),
                        label: "table".into(),
                        ..Default::default()
                    }])
                } else {
                    let mut snippet = "{ ".to_string();

                    let mut idx: usize = 1;

                    for key in o.properties.keys().sorted() {
                        let prop_schema = o.properties.get(key).unwrap();

                        if let Some(prop_schema) = ExtendedSchema::resolved(defs, prop_schema) {
                            if o.required.contains(key)
                                || schema
                                    .ext
                                    .init_keys
                                    .as_ref()
                                    .map(|i| i.iter().any(|i| i == key))
                                    .unwrap_or(false)
                            {
                                if idx != 1 {
                                    snippet += ", "
                                }

                                snippet += &format!(
                                    "{} = {}",
                                    key,
                                    default_value_snippet(defs, prop_schema, idx)
                                );

                                idx += 1;
                            }
                        }
                    }

                    snippet += "$0 }";

                    Some(vec![CompletionItem {
                        text_edit: range.map(|range| {
                            CompletionTextEdit::Edit(TextEdit {
                                range,
                                new_text: with_leading_space(
                                    with_comma(snippet.clone(), comma_before),
                                    space_before,
                                ),
                            })
                        }),
                        kind: Some(CompletionItemKind::Value),
                        insert_text: Some(with_leading_space(
                            with_comma(snippet, comma_before),
                            space_before,
                        )),
                        insert_text_format: Some(InsertTextFormat::Snippet),
                        label: "table".into(),
                        ..Default::default()
                    }])
                }
            }
            None => Some(vec![CompletionItem {
                text_edit: range.map(|range| {
                    CompletionTextEdit::Edit(TextEdit {
                        range,
                        new_text: with_leading_space(
                            with_comma(r#"{ $0 }"#.into(), comma_before),
                            space_before,
                        ),
                    })
                }),
                kind: Some(CompletionItemKind::Value),
                insert_text: Some(with_leading_space(
                    with_comma(r#"{ $0 }"#.into(), comma_before),
                    space_before,
                )),
                insert_text_format: Some(InsertTextFormat::Snippet),
                label: "table".into(),
                ..Default::default()
            }]),
        },
        InstanceType::Null => None,
    }
}

fn format_value(value: &Value, snippet: bool, snippet_index: usize) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(b) => {
            if snippet {
                format!(r#"${{{}:{}}}"#, snippet_index, b)
            } else {
                b.to_string()
            }
        }
        Value::Number(n) => {
            if snippet {
                format!(r#"${{{}:{}}}"#, snippet_index, n)
            } else {
                n.to_string()
            }
        }
        Value::String(s) => {
            if snippet {
                format!(r#""${{{}:{}}}""#, snippet_index, s)
            } else {
                format!(r#""{}""#, s)
            }
        }
        Value::Array(arr) => {
            let mut s = String::new();
            s += "[ ";
            if snippet {
                s += &format!("${{{}:", snippet_index);
            }
            for (i, val) in arr.iter().enumerate() {
                if i != 0 {
                    s += ", ";
                    s += &format_value(val, false, 0);
                }
            }
            if snippet {
                s += "}"
            }
            s += " ]";

            s
        }
        Value::Object(obj) => {
            let mut s = String::new();
            s += "{ ";
            if snippet {
                s += &format!("${{{}:", snippet_index);
            }
            for (i, (key, val)) in obj.iter().enumerate() {
                if i != 0 {
                    s += ", ";
                    s += key;
                    s += " = ";
                    s += &format_value(val, false, 0);
                }
            }
            if snippet {
                s += "}"
            }
            s += " }";

            s
        }
    }
}

fn default_value_snippet(
    _defs: &Map<String, Schema>,
    schema: ExtendedSchema,
    idx: usize,
) -> String {
    if let Some(c) = &schema.schema.const_value {
        return format_value(c, true, idx);
    }

    if let Some(e) = &schema.schema.enum_values {
        if let Some(e) = e.iter().next() {
            return format_value(e, true, idx);
        }
    }

    if let Some(default) = schema
        .schema
        .metadata
        .as_ref()
        .and_then(|m| m.default.as_ref())
    {
        return format_value(default, true, idx);
    }

    format!("${}", idx)
}

// Whether the key should be completed according to the contents of the tree,
// e.g. we shouldn't offer completions for value paths that already exist.
fn valid_key(path: &dom::Path, dom_paths: &HashSet<dom::Path>, _dom: &RootNode) -> bool {
    !dom_paths.contains(path)
}
