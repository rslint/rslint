//! CLI options

use crate::{lint_err, DOCS_LINK_BASE, REPO_LINK};
use ansi_term::Color::{White, RGB, Green};
use regex::{Captures, Regex};
use rslint_lexer::{ansi_term, color};
use ureq::get;

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
                let res = fetch_doc_file(&rule);
                if res.is_none() {
                    lint_err!("Invalid rule: {}", rule);
                }
                res
            })
            .collect();

        Self { rules, rule_names }
    }

    pub fn strip_rule_preludes(&mut self) {
        for rule in self.rules.iter_mut() {
            rule.replace_range(0..70, "");
        }
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
                .replace_all(rule, |cap: &Captures| format!("\n{}\n", color(cap.get(1).unwrap().as_str())))
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
                    ansi_term::Style::new().on(color).fg(White).paint(cap.get(1).unwrap().as_str()).to_string()
                })
                .to_string();
        }
    }

    pub fn append_link_to_docs(&mut self) {
        for (docs, name) in self.rules.iter_mut().zip(self.rule_names.iter()) {
            let group = rslint_core::get_rule_by_name(&name).unwrap().group();
            let link = format!("{}/docs/rules/{}/{}.md", REPO_LINK, group, name);
            docs.push_str(&format!("{}: {}\n", Green.paint("Docs").to_string(), link));
        }
    }

    pub fn render(&mut self) {
        self.strip_rule_preludes();
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

/// Try to resolve a rule name, then fetch its remote docs file.
///
/// # Panics
/// Panics if the remote docs file cant be fetched for some reason.
fn fetch_doc_file(rule: &str) -> Option<String> {
    let resolved_rule = rslint_core::get_rule_by_name(rule)?;
    Some(
        get(&format!(
            "{}/{}/{}.md",
            DOCS_LINK_BASE,
            resolved_rule.group(),
            rule
        ))
        .call()
        .into_string()
        .expect("Failed to fetch remote rule docs file"),
    )
}
