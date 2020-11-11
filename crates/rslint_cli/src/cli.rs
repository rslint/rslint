//! CLI options

use crate::lint_err;
use ansi_term::Color::{Green, White, RGB};
use colored::Colorize;
use regex::{Captures, Regex};
use rslint_core::{get_rule_docs, CstRuleStore};
use rslint_lexer::{ansi_term, color};
use std::collections::HashSet;

/// A structure for converting user facing markdown docs to ANSI colored terminal explanations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExplanationRunner {
    pub rules: Vec<String>,
    pub rule_names: Vec<String>,
}

impl ExplanationRunner {
    /// Make a new runner and try to fetch the remote docs files for each rule.
    /// This automatically issues any linter errors for invalid rules.
    pub fn new(mut rules: Vec<String>) -> Self {
        let rule_names = rules.clone();
        rules = rules
            .into_iter()
            .filter_map(|rule| {
                let res = get_rule_docs(&rule).map(|x| x.to_string());
                if res.is_none() {
                    lint_err!("Invalid rule: {}", rule);
                }
                res
            })
            .collect();

        Self { rules, rule_names }
    }

    pub fn replace_headers(&mut self) {
        let regex = Regex::new("#+ (.*)").unwrap();
        for rule in self.rules.iter_mut() {
            *rule = regex
                .replace_all(rule, |cap: &Captures| {
                    White.bold().paint(cap.get(1).unwrap().as_str()).to_string()
                })
                .to_string();
        }
    }

    pub fn replace_code_blocks(&mut self) {
        let regex = Regex::new("```js\n([\\s\\S]*?)\n```").unwrap();
        for rule in self.rules.iter_mut() {
            *rule = regex
                .replace_all(rule, |cap: &Captures| {
                    format!("\n{}\n", color(cap.get(1).unwrap().as_str()))
                })
                .to_string();
        }
    }

    pub fn strip_config_or_extra_examples(&mut self) {
        for rule in self.rules.iter_mut() {
            if let Some(idx) = rule.find("# Config") {
                rule.truncate(idx - 1);
            }
            if let Some(idx) = rule.find("<details>") {
                rule.truncate(idx - 1);
            }
        }
    }

    pub fn replace_inline_code_blocks(&mut self) {
        let regex = Regex::new("`(.+?)`").unwrap();
        for rule in self.rules.iter_mut() {
            *rule = regex
                .replace_all(rule, |cap: &Captures| {
                    let color = RGB(42, 42, 42);
                    ansi_term::Style::new()
                        .on(color)
                        .fg(White)
                        .paint(cap.get(1).unwrap().as_str())
                        .to_string()
                })
                .to_string();
        }
    }

    pub fn append_link_to_docs(&mut self) {
        for (docs, name) in self.rules.iter_mut().zip(self.rule_names.iter()) {
            let group = rslint_core::get_rule_by_name(&name).unwrap().group();
            let link = format!("https://rslint.org/rules/{}/{}.html", group, name);
            docs.push_str(&format!("{}: {}\n", Green.paint("Docs").to_string(), link));
        }
    }

    pub fn render(&mut self) {
        self.strip_config_or_extra_examples();
        self.replace_headers();
        self.replace_code_blocks();
        self.replace_inline_code_blocks();
        self.append_link_to_docs();
    }

    pub fn print(mut self) {
        self.render();
        for rule in self.rules.into_iter() {
            println!("{}", "-".repeat(10));
            println!("{}", rule);
        }
    }
}

pub fn show_all_rules() {
    let rules = CstRuleStore::new().builtins().rules;
    let mut groups = HashSet::new();
    rules.iter().for_each(|r| {
        groups.insert(r.group());
    });

    for group in groups {
        let group_rules = rules.iter().filter(|rule| rule.group() == group);
        println!("{}:", group.bright_green());
        let max_rule_len = group_rules
            .clone()
            .map(|r| r.name().len())
            .max()
            .unwrap_or(0);

        for rule in group_rules {
            println!(
                " {}{} - {}",
                rule.name().white(),
                " ".repeat(max_rule_len - rule.name().len()),
                rule.docs().lines().next().unwrap_or_default()
            );
        }
        println!();
    }
    println!("{}: https://rslint.org/rules/", "more info".bright_green());
    println!(
        "{}: use the explain command to show info about individual rules",
        "help".bright_green()
    );
}
