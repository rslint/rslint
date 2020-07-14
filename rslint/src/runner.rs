//! The main runner and dispatcher for RSLint, its jobs include:  
//! - Loading files from the file walker  
//! - Running CST rules  
//! - Collecting diagnostics  
//! - Dispatching results to formatters  

use crate::linter::Linter;
use crate::rules::store::CstRuleStore;
use crate::rules::context::RuleContext;
use crate::rules::{RuleResult, CstRule, Outcome};
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::SimpleFile;
use rayon::prelude::*;
use rslint_parse::diagnostic::ParserDiagnostic;
use rslint_parse::parser::Parser;
use rslint_parse::parser::cst::CST;
use std::collections::HashMap;

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
    pub fn parse_file<'a>(&self, file: &'a SimpleFile<String, String>, offset: Option<usize>) -> (Vec<ParserDiagnostic<'a>>, Option<CST>) {
        if file.source().is_empty() {
            (vec![], Some(CST::new()))
        } else {
            let mut parser = Parser::with_source_and_offset(file.source(), file.name(), true, offset.unwrap_or(0)).unwrap();

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

        let mut diagnostics = load_result.unwrap().iter().map(|warn| {
            Diagnostic::<&str>::warning().with_message(warn)
        }).collect::<Vec<Diagnostic<&str>>>();

        // Load all of the builtin cst rules
        let cst_rule_store = CstRuleStore::new().load_predefined_groups();

        let rule_diagnostics = linter.walker.files.par_iter().map(|(file_id, file)| {
            if file.source().is_empty() {
                return vec![];
            }

            let mut parser = Parser::with_source(file.source(), file_id, true).unwrap();

            let res = parser.parse_script();
            
            if let Err(err) = res {
                parser.errors.push(err);
                parser.errors.into_iter().map(|diag| diag.diagnostic).collect()
            } else {
                let mut diagnostics: Vec<Diagnostic<&str>> = parser.errors.into_iter().map(|diag| diag.diagnostic).collect();
                let cst = &res.unwrap();

                let rules: Vec<&Box<dyn CstRule>> = cst_rule_store.groups.iter().map(|group| &group.rules).flatten().collect();

                let rule_diagnostics: Vec<Diagnostic<&str>> = rules.par_iter().map(|rule| {
                    let mut ctx = RuleContext {
                        file_id: &file_id,
                        file_source: file.source(),
                        diagnostics: vec![],
                    };

                    rule.lint(&mut ctx, cst);
                    ctx.diagnostics
                }).flatten().collect();

                diagnostics.extend(rule_diagnostics);
                diagnostics
            }
        }).flatten().collect::<Vec<_>>();

        diagnostics.extend(rule_diagnostics);
        linter.formatter.format(&diagnostics, &linter.walker);
    }
}