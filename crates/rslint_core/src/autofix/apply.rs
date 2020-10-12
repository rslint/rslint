use crate::{lint_file_inner, LintResult, RuleResult};
use rslint_parser::*;
use std::collections::HashMap;

pub const MAX_FIX_ITERATIONS: usize = 10;

#[derive(Debug, Clone)]
struct TaggedIndel {
    tag: &'static str,
    indel: Indel,
}

fn get_runnable_indels(mut tagged: Vec<TaggedIndel>) -> Vec<TaggedIndel> {
    tagged.sort_by_key(|TaggedIndel { indel, .. }| (indel.delete.start(), indel.delete.end()));

    // We need to throw out any overlapping indels, but we can't just throw out individual indels, we
    // must throw out the entire fixer's indels or else we risk getting partial fixes which dont work well.
    // any fixer indels thrown out will hopefully be applied in the next recursive lint run, if not then ¯\_(ツ)_/¯
    let mut excluded_ids = vec![];
    tagged.iter().zip(tagged.iter().skip(1)).for_each(|(l, r)| {
        if l.indel.delete.end() > r.indel.delete.start() {
            excluded_ids.push(r.tag);
        }
    });

    tagged
        .into_iter()
        .filter(|TaggedIndel { tag, .. }| !excluded_ids.contains(&tag))
        .collect()
}

fn apply_indels(indels: &[TaggedIndel], text: &mut String) {
    match indels.len() {
        0 => return,
        1 => {
            indels[0].indel.apply(text);
            return;
        }
        _ => (),
    }

    let mut total_len = TextSize::of(&*text);
    for indel in indels.iter() {
        total_len += TextSize::of(&indel.indel.insert);
        total_len -= indel.indel.delete.end() - indel.indel.delete.start();
    }
    let mut buf = String::with_capacity(total_len.into());
    let mut prev = 0;
    for indel in indels.iter() {
        let start: usize = indel.indel.delete.start().into();
        let end: usize = indel.indel.delete.end().into();
        if start > prev {
            buf.push_str(&text[prev..start]);
        }
        buf.push_str(&indel.indel.insert);
        prev = end;
    }
    buf.push_str(&text[prev..text.len()]);
    assert_eq!(TextSize::of(&buf), total_len);
    *text = buf;
}

pub fn recursively_apply_fixes(result: &mut LintResult) -> String {
    let script = result.parsed.kind() == SyntaxKind::SCRIPT;
    let mut parsed = result.parsed.clone();
    let file_id = result.file_id;
    let mut cur_results = result.rule_results.clone();

    for _ in 0..=MAX_FIX_ITERATIONS {
        let indels = get_runnable_indels(rule_results_to_tagged_indels(&cur_results));

        let mut reparsed = None;
        if indels.is_empty() {
            break;
        }
        for tagged in &indels {
            let mut reparse = if script {
                try_incrementally_reparsing_script(
                    parsed.clone(),
                    // we dont care about errors, we just want that nice
                    // reparsed syntax tree
                    vec![],
                    &tagged.indel,
                    file_id,
                )
                .map(|p| p.syntax())
            } else {
                try_incrementally_reparsing_module(parsed.clone(), vec![], &tagged.indel, file_id)
                    .map(|p| p.syntax())
            };
            // reparsing indels individually failed, we need to go back and apply all the indels
            // and to a string then run that through the parser, this is a lot slower.
            if reparse.is_none() {
                let mut string = parsed.text().to_string();
                apply_indels(&indels, &mut string);
                reparse = if script {
                    let res = parse_text(&string, file_id);
                    Some(res.syntax())
                } else {
                    let res = parse_module(&string, file_id);
                    Some(res.syntax())
                };
            }
            reparsed = reparse;
        }
        let node = reparsed.unwrap();
        parsed = node.clone();
        // TODO: should we panic on Err? autofix causing the linter to fail should always be incorrect
        let res = lint_file_inner(node, vec![], file_id, result.store, result.verbose);
        if let Ok(res) = res {
            cur_results = res.rule_results;
        } else {
            continue;
        }
    }
    result.rule_results = cur_results;
    parsed.text().to_string()
}

fn rule_results_to_tagged_indels(results: &HashMap<&'static str, RuleResult>) -> Vec<TaggedIndel> {
    results
        .iter()
        .filter_map(|(tag, res)| Some((tag, res.fixer.clone()?)))
        .flat_map(|(tag, fixer)| {
            fixer
                .indels
                .into_iter()
                .map(|indel| TaggedIndel { tag, indel })
                .collect::<Vec<_>>()
                .into_iter()
        })
        .collect()
}
