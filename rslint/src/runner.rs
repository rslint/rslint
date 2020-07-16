//! The main runner and dispatcher for RSLint, its jobs include:  
//! - Loading files from the file walker  
//! - Running CST rules  
//! - Collecting diagnostics  
//! - Dispatching results to formatters  

use crate::cache::{Cache, FileInfo};
use crate::linter::Linter;
use crate::rules::context::RuleContext;
use crate::rules::store::CstRuleStore;
use crate::rules::{CstRule, Outcome, RuleResult};
use codespan_reporting::diagnostic::{Diagnostic};
use codespan_reporting::files::SimpleFile;
use rayon::prelude::*;
use rslint_parse::diagnostic::ParserDiagnostic;
use rslint_parse::parser::cst::CST;
use rslint_parse::parser::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct LintResult<'a> {
    pub outcome: Outcome,
    pub file_id: String,
    pub cst: Option<CST>,
    pub rule_results: HashMap<&'static str, RuleResult<'a>>,
    pub parser_errors: Vec<Diagnostic<&'a str>>,
}

/// The main structure responsible for organizing the linting process and calling formatters
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LintRunner;

impl LintRunner {
    pub fn new() -> Self {
        Self
    }

    /// Parse a single file consisting of an id and its source code into parser errors and a CST (if there were no irrecoverable errors)
    /// An optional offset for the parser can be provided too
    pub fn parse_file<'a>(
        &self,
        file: &'a SimpleFile<String, String>,
        offset: Option<usize>,
    ) -> (Vec<ParserDiagnostic<'a>>, Option<CST>) {
        if file.source().is_empty() {
            (vec![], Some(CST::new()))
        } else {
            let mut parser = Parser::with_source_and_offset(
                file.source(),
                file.name(),
                true,
                offset.unwrap_or(0),
            )
            .unwrap();

            let res = parser.parse_script();

            if let Err(err) = res {
                parser.errors.push(err);
                (parser.errors, None)
            } else {
                (parser.errors, res.ok())
            }
        }
    }

    /// Load files from the file walker, load rules from groups, run rules, and call respective formatters
    pub fn exec(&self, linter: &mut Linter) {
        let load_result = linter.walker.load();

        if let Err(err) = load_result {
            let diagnostic = Diagnostic::<&str>::error().with_message(err.msg);

            return linter.formatter.format(&vec![diagnostic], &linter.walker);
        }

        let mut diagnostics = load_result
            .unwrap()
            .iter()
            .map(|warn| Diagnostic::<&str>::warning().with_message(warn))
            .collect::<Vec<Diagnostic<&str>>>();

        let mut files_to_lint = linter.walker.files.to_owned();

        // Attempt to load cache and skip files that are cached
        if let Some(cache_path) = Cache::get_cwd_cache_file() {
            let mut buf = Vec::with_capacity(200);
            let path = cache_path.as_path();
            let res = File::open(path)
                .expect("Failed to open cache file")
                .read_to_end(&mut buf);

            if res.is_err() {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_message("Skipping cache as the cache file is unreadable"),
                );
            }

            let maybe_cache = Cache::from_bytes(&buf);
            if maybe_cache.is_none() {
                diagnostics.push(
                    Diagnostic::warning()
                        .with_message("Skipping cache as the cache file data is malformed"),
                );
            } else {
                let cache = maybe_cache.unwrap();
                // Cache has been poisoned and we cannot use it
                if Cache::has_been_modified(cache.write_date, path) {
                    let diagnostic = Diagnostic::warning().with_message("Cache has been externally modified after it was generated, ignoring cache")
                        .with_notes(vec![format!("Note: The cache file was first generated on {:#?}\n...Then something modified it on {:#?}", cache.write_date, Cache::get_file_timestamp(path).unwrap())]);
                    diagnostics.push(diagnostic);
                } else {
                    let new_files = linter
                    .walker
                    .files
                    .iter()
                    .map(|file| PathBuf::from(file.0))
                    .collect();
                    let (cached, _) = cache.file_intersect(new_files);

                    let mut cached_diagnostics = Vec::with_capacity(cached.len() * 2);

                    // TODO: check for difference in rules
                    for file in cached.into_iter() {
                        cached_diagnostics.extend(file.diagnostics.to_owned());
                        // If the file is cached we dont need to run lints on it
                        files_to_lint.remove(&file.name.to_string());
                    }

                    linter.formatter.format(&cached_diagnostics, &linter.walker);
                }
            }
        }

        let files_cache = Mutex::new(HashMap::with_capacity(linter.walker.files.len()));

        // Load all of the builtin cst rules
        let cst_rule_store = CstRuleStore::new().load_predefined_groups();

        let rule_diagnostics = files_to_lint
            .par_iter()
            .map(|(file_id, file)| {
                if file.source().is_empty() {
                    return vec![];
                }

                let mut parser = Parser::with_source(file.source(), file_id, true).unwrap();

                let res = parser.parse_script();

                let rules: Vec<&Box<dyn CstRule>> = cst_rule_store
                    .groups
                    .iter()
                    .map(|group| &group.rules)
                    .flatten()
                    .collect();

                let diagnostics = if let Err(err) = res {
                    parser.errors.push(err);
                    parser
                        .errors
                        .into_iter()
                        .map(|diag| diag.diagnostic)
                        .collect()
                } else {
                    let mut diagnostics: Vec<Diagnostic<&str>> = parser
                        .errors
                        .into_iter()
                        .map(|diag| diag.diagnostic)
                        .collect();
                    let cst = &res.unwrap();

                    let rule_diagnostics: Vec<Diagnostic<&str>> = rules
                        .par_iter()
                        .map(|rule| {
                            let mut ctx = RuleContext {
                                file_id: &file_id,
                                file_source: file.source(),
                                diagnostics: vec![],
                            };

                            rule.lint(&mut ctx, cst);
                            ctx.diagnostics
                        })
                        .flatten()
                        .collect();

                    diagnostics.extend(rule_diagnostics);
                    diagnostics
                };

                if let Some(timestamp) = Cache::get_file_timestamp(Path::new(file_id)) {
                    let file_info = FileInfo {
                        diagnostics: diagnostics.to_owned(),
                        timestamp,
                        rules: rules.iter().map(|rule| rule.name().to_string()).collect(),
                        name: file_id.to_string(),
                    };

                    files_cache
                        .lock()
                        .unwrap()
                        .insert(file_id.to_string(), file_info);
                }

                diagnostics
            })
            .flatten()
            .collect::<Vec<_>>();

        Cache::generate(files_cache.into_inner().unwrap())
            .persist()
            .expect("Failed to write cache file to current directory");
        diagnostics.extend(rule_diagnostics);
        linter.formatter.format(&diagnostics, &linter.walker);
    }
}
