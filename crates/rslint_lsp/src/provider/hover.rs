//! On-hover provider for the language server.

use crate::core::session::Session;
use anyhow::Result;
use rslint_errors::file::Files;
use tower_lsp::lsp_types::{
    Hover, HoverContents, HoverParams, MarkedString, Position, TextDocumentIdentifier,
    TextDocumentPositionParams,
};

pub async fn on_hover(session: &Session, params: HoverParams) -> Result<Option<Hover>> {
    let TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri },
        position: Position { line, character },
    } = params.text_document_position_params;

    let doc = session.get_document(&uri).await.unwrap();
    let directives = doc.directives.as_slice();

    if let Some(start) = doc
        .files
        .line_range(doc.file.id, line as usize)
        .map(|r| r.start)
    {
        let idx = start + character as usize;
        let component = directives
            .iter()
            .flat_map(|d| d.component_at(From::from(idx as u32)))
            .next();
        if let Some(documentation) = component.and_then(|c| c.kind.documentation()) {
            let range = rslint_errors::lsp::byte_span_to_range(
                &doc.files,
                doc.file.id,
                component.unwrap().range.into(),
            )?;

            return Ok(Some(Hover {
                contents: HoverContents::Scalar(MarkedString::String(documentation.to_string())),
                range: Some(range),
            }));
        }
    }

    Ok(None)
}
