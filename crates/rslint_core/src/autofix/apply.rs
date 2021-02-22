use crate::{lint_file_inner, File, LintResult, RuleResult};
use rslint_parser::*;
use rslint_text_edit::{apply_indels, Indel};
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

pub fn recursively_apply_fixes(result: &mut LintResult, file: &File) -> String {
    let script = result.parsed.kind() == SyntaxKind::SCRIPT;
    let mut parsed = result.parsed.clone();
    let file_id = result.file_id;
    let mut cur_results = result.rule_results.clone();

    for _ in 0..=MAX_FIX_ITERATIONS {
        let indels = get_runnable_indels(rule_results_to_tagged_indels(&cur_results));

        if indels.is_empty() {
            break;
        }
        let mut string = parsed.text().to_string();
        apply_indels(
            &indels.iter().map(|x| x.indel.clone()).collect::<Vec<_>>(),
            &mut string,
        );
        parsed = if script {
            let res = parse_text(&string, file_id);
            // this needs to be updated for when fixes are applied "dirty" (when there are parser errors)
            result.parser_diagnostics = res.errors().to_owned();
            res.syntax()
        } else {
            let res = parse_module(&string, file_id);
            result.parser_diagnostics = res.errors().to_owned();
            res.syntax()
        };

        // TODO: should we panic on Err? autofix causing the linter to fail should always be incorrect
        let res = lint_file_inner(parsed.clone(), vec![], file, result.store, result.verbose);
        cur_results = res.rule_results;
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
