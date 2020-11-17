use crate::*;
use colored::Colorize;
use glob::glob;
use rslint_core::get_inferable_rules;
use rslint_parser::SyntaxNode;
use toml::to_string_pretty;

pub fn infer(files: Vec<String>) {
    let globs = files
        .into_iter()
        .filter_map(|x| match glob(&x) {
            Ok(res) => Some(res),
            Err(err) => {
                lint_err!("invalid glob pattern: {}", err.to_string());
                None
            }
        })
        .flatten()
        .flat_map(Result::ok)
        .collect();

    let walker = FileWalker::from_glob(globs);
    let parsed = walker.files.values().map(|f| f.parse());
    let nodes: Vec<SyntaxNode> = parsed.flat_map(|n| n.descendants()).collect();
    let rules = get_inferable_rules();
    let mut inferred = Vec::with_capacity(rules.len());

    for mut rule in rules {
        rule.infer(&nodes);
        inferred.push(rule);
    }

    println!("{}\n", "Inferred rules:".bright_green());
    println!("{}", to_string_pretty(&inferred).unwrap());
}
