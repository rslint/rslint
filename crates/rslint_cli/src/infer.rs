use crate::*;
use colored::Colorize;
use rslint_core::get_inferable_rules;
use rslint_core::parser::SyntaxNode;
use toml::to_string_pretty;

pub async fn infer(globs: Vec<String>) {
    let walker = FileWalker::from_globs(globs).await;
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
