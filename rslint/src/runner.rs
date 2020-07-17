//! The main runner and dispatcher for RSLint, its jobs include:  
//! - Loading files from the file walker  
//! - Running CST rules  
//! - Collecting diagnostics  
//! - Dispatching results to formatters  

use crate::cache::{Cache, FileInfo};
use crate::linter::file_walker::FileWalker;
use crate::linter::Linter;
use crate::rules::context::RuleContext;
use crate::rules::store::CstRuleStore;
use crate::rules::{CstRule, Outcome, RuleResult};
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::SimpleFile;
use rayon::prelude::*;
use rslint_parse::diagnostic::ParserDiagnostic;
use rslint_parse::parser::cst::CST;
use rslint_parse::parser::Parser;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct LintResult {
    pub outcome: Outcome,
    pub file_id: String,
    pub cst: Option<CST>,
    pub rule_results: HashMap<&'static str, RuleResult>,
    pub parser_errors: Vec<Diagnostic<usize>>,
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
        file_id: usize,
        offset: Option<usize>,
    ) -> (Vec<ParserDiagnostic>, Option<CST>) {
        if file.source().is_empty() {
            (vec![], Some(CST::new()))
        } else {
            let mut parser =
                Parser::with_source_and_offset(file.source(), file_id, true, offset.unwrap_or(0))
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

    /// Load cache, Load files from the file walker, load rules from groups, run rules, and call respective formatters
    pub fn exec(&self, linter: &mut Linter) {
        let paths = linter.walker.paths.to_owned();
        let file_results = Mutex::new(HashMap::with_capacity(paths.len()));
        let mut diagnostics = vec![];
        let files_to_lint;

        let cache_res = Cache::load();

        if let Err(warning) = cache_res {
            diagnostics.push(warning);
            files_to_lint = paths;
        } else {
            if let Some(cache) = cache_res.unwrap() {
                let (cached, uncached) = cache.file_intersect(paths);
                diagnostics.extend(
                    linter
                        .walker
                        .load()
                        .unwrap()
                        .into_iter()
                        .map(|string| Diagnostic::warning().with_message(string))
                        .collect::<Vec<Diagnostic<usize>>>(),
                );

                for cached_file in cached {
                    linter
                        .formatter
                        .format(&cached_file.diagnostics, &linter.walker);
                    file_results
                        .lock()
                        .unwrap()
                        .insert(cached_file.name.to_owned(), cached_file);
                }
                diagnostics.clear();

                files_to_lint = uncached;
            } else {
                files_to_lint = paths;
            }
        }

        if files_to_lint.len() == 0 {
            Cache::generate(file_results.into_inner().unwrap())
                .persist()
                .expect("Failed to write cache file to current directory");

            return linter.formatter.format(&diagnostics, &linter.walker);
        }

        let mut new_walker = FileWalker::with_paths(files_to_lint);
        diagnostics.extend(
            new_walker
                .load()
                .unwrap()
                .into_iter()
                .map(|string| Diagnostic::warning().with_message(string))
                .collect::<Vec<Diagnostic<usize>>>(),
        );

        // let files_cache = Mutex::new(HashMap::with_capacity(linter.walker.files.len()));

        // Load all of the builtin cst rules
        let cst_rule_store = CstRuleStore::new().load_predefined_groups();

        let rule_diagnostics = new_walker
            .files
            .par_iter()
            .map(|(file_id, file)| {
                if file.source().is_empty() {
                    return vec![];
                }

                let mut parser = Parser::with_source(file.source(), *file_id, true).unwrap();

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
                    let mut diagnostics: Vec<Diagnostic<usize>> = parser
                        .errors
                        .into_iter()
                        .map(|diag| diag.diagnostic)
                        .collect();
                    let cst = &res.unwrap();

                    let rule_diagnostics: Vec<Diagnostic<usize>> = rules
                        .par_iter()
                        .map(|rule| {
                            let mut ctx = RuleContext {
                                file_id: *file_id,
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

                if let Some(timestamp) = Cache::get_file_timestamp(Path::new(file.name())) {
                    let file_info = FileInfo {
                        diagnostics: diagnostics.to_owned(),
                        timestamp,
                        rules: rules.iter().map(|rule| rule.name().to_string()).collect(),
                        name: file.name().to_string(),
                    };

                    file_results
                        .lock()
                        .unwrap()
                        .insert(file.name().to_string(), file_info);
                }

                diagnostics
            })
            .flatten()
            .collect::<Vec<_>>();

        Cache::generate(file_results.into_inner().unwrap())
            .persist()
            .expect("Failed to write cache file to current directory");
        diagnostics.extend(rule_diagnostics);
        linter.formatter.format(&diagnostics, &new_walker);
    }
}
