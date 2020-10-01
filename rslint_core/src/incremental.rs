//! Incremental relinting facilities using incremental reparsing.

use crate::*;
use rslint_parser::{
    parse_module, parse_text, try_incrementally_reparsing_module,
    try_incrementally_reparsing_script, SyntaxKind, TextRange, TextSize,
};
use text_diff::{diff, Difference};
use SyntaxKind::*;

/// Convert an old and new string to a vector of indels based on diff
pub fn diff_to_indels(old: &str, new: &str) -> Vec<Indel> {
    let (_, diffs) = diff(old, new, " ");
    let mut indels = Vec::with_capacity(diffs.len() / 2);
    let mut len: TextSize = 0.into();

    for diff in diffs {
        match diff {
            Difference::Add(string) => {
                let offset = TextSize::from(string.len() as u32);
                indels.push(Indel::insert(len, string));
                len += offset;
            }
            Difference::Rem(string) => {
                let offset = TextSize::from(string.len() as u32);
                indels.push(Indel::delete(TextRange::new(len, len + offset)));
                len += offset;
            }
            Difference::Same(string) => len += TextSize::from(string.len() as u32),
        }
    }
    indels
}

pub fn incrementally_relint<'a>(
    old: LintResult<'a>,
    new: &str,
) -> Result<LintResult<'a>, Diagnostic> {
    let old_str = &old.parsed.text().to_string();
    if old_str == new {
        return Ok(old);
    }
    let indels = diff_to_indels(&old.parsed.text().to_string(), new);
    let mut reparsed = None;

    for indel in indels {
        let reparse = if old.parsed.kind() == SCRIPT {
            try_incrementally_reparsing_script(
                old.parsed.clone(),
                old.parser_diagnostics.clone(),
                &indel,
                old.file_id,
            )
            .map(|p| (p.syntax(), p.errors().to_owned()))
        } else {
            try_incrementally_reparsing_module(
                old.parsed.clone(),
                old.parser_diagnostics.clone(),
                &indel,
                old.file_id,
            )
            .map(|p| (p.syntax(), p.errors().to_owned()))
        };
        // if reparsing failed then there is no point in trying to reparse again because
        // the alternative to reparsing failure is parsing the whole text
        // so at that point reparsing has no value
        if reparse.is_none() {
            reparsed = if old.parsed.kind() == SCRIPT {
                let res = parse_text(new, old.file_id);
                Some((res.syntax(), res.errors().to_owned()))
            } else {
                let res = parse_module(new, old.file_id);
                Some((res.syntax(), res.errors().to_owned()))
            };
            break;
        }
        reparsed = reparse;
    }
    let (node, errors) = reparsed.unwrap();
    lint_file_inner(node, errors, old.file_id, old.store, old.verbose)
}
