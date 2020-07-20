//! The main runner and dispatcher for RSLint, its jobs include:  
//! - Loading files from the file walker  
//! - Running CST rules  
//! - Collecting diagnostics  
//! - Dispatching results to formatters  
//! - Collecting info on how long each operation took

use crate::cache::Cache;
use crate::linter::file_walker::FileWalker;
use crate::linter::Linter;
use crate::rules::context::RuleContext;
use crate::rules::store::CstRuleStore;
use crate::rules::{CstRule, Outcome, RuleResult};
use crate::tablegen::*;
use codespan_reporting::diagnostic::Diagnostic;
use rayon::prelude::*;
use rslint_parse::diagnostic::ParserDiagnostic;
use rslint_parse::parser::cst::CST;
use rslint_parse::parser::Parser;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use termcolor::Color;
use std::env;

#[derive(Debug, Clone)]
pub struct LintResult {
    pub outcome: Outcome,
    pub file_id: usize,
    pub cst: Option<CST>,
    pub rule_results: HashMap<&'static str, RuleResult>,
    pub parser_errors: Vec<Diagnostic<usize>>,
    pub duration: FileLintDuration,
}

/// A structure detailing how long it took for each linting operation to complete on a file  
/// If parsing failed, the rules_total duration will be 0 and the rules hashmap will be empty
#[derive(Debug, Clone)]
pub struct FileLintDuration {
    /// How long parsing the file into a CST took
    parsing: Duration,
    /// How long it took to run all of the rules
    rules_total: Duration,
    /// How it took for each individual rule
    rules: HashMap<&'static str, Duration>,
    /// The overall duration of linting the file
    overall: Duration,
}

/// A structure detailing how long it took in total for linting operations to complete
#[derive(Debug, Clone)]
pub struct LintDuration {
    /// How long it took to load and serialize cache from disk
    cache_loading: Duration,
    /// How long it took to load files from disk, this does not include cached files
    file_loading: Duration,
    /// How long it took to lint all of the files
    file_linting: Duration,
    /// The overall time it took to load all files, load cache, run rules, and format diagnostics
    overall: Duration,
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
        file_id: usize,
        file_source: &'a str,
        offset: Option<usize>,
    ) -> (Vec<ParserDiagnostic>, Option<CST>) {
        if file_source.is_empty() {
            (vec![], Some(CST::new()))
        } else {
            let mut parser =
                Parser::with_source_and_offset(file_source, file_id, true, offset.unwrap_or(0))
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

    /// Run a rule on a single file's CST and return a duration representing how long it took
    pub fn run_rule(&self, cst: &CST, file_id: usize, file_source: &str, rule: &Box<dyn CstRule>) -> (RuleResult, Duration) {
        let start = Instant::now();

        let mut ctx = RuleContext {
            file_source,
            file_id,
            diagnostics: vec![],
        };

        rule.lint(&mut ctx, cst);
        (ctx.diagnostics.into(), Instant::now().duration_since(start))
    }

    /// Run all of the rules of a store on a single file and return the duration it took for each rule and the total time for running all the rules
    pub fn lint_file(&self, file_id: usize, file_source: &str, store: &CstRuleStore) -> LintResult {
        let start = Instant::now();

        let (parser_diagnostics, cst) = self.parse_file(file_id, file_source, None);
        let parse_time = Instant::now().duration_since(start);

        let mut file_lint_duration = FileLintDuration {
            parsing: parse_time,
            rules: HashMap::new(),
            rules_total: Duration::new(0, 0),
            overall: Instant::now().duration_since(start),
        };

        let unwrapped_diagnostics: Vec<Diagnostic<usize>> = parser_diagnostics.into_iter()
            .map(|diag| diag.diagnostic)
            .collect();

        let outcome = Outcome::from(&unwrapped_diagnostics);

        if cst.is_none() {
            return LintResult {
                outcome: Outcome::Error,
                file_id,
                cst,
                rule_results: HashMap::new(),
                parser_errors: unwrapped_diagnostics,
                duration: file_lint_duration
            }
        }

        let rules: &Vec<&Box<dyn CstRule>> = &store.groups.iter().map(|group| &group.rules).flatten().collect();
        let cst_ref = &cst.unwrap();
        let before_rules = Instant::now();
        let rule_results = Mutex::new(HashMap::with_capacity(rules.len()));
        let durations = Mutex::new(HashMap::with_capacity(rules.len()));

        rules.par_iter().for_each(|rule| {
            let res = self.run_rule(cst_ref, file_id, file_source, rule);
            rule_results.lock().unwrap().insert(rule.name(), res.0);
            durations.lock().unwrap().insert(rule.name(), res.1);
        });

        let mut outcomes = rule_results.lock().unwrap().values().map(|res| res.outcome.clone()).collect::<Vec<_>>();
        outcomes.push(outcome);

        file_lint_duration.rules = durations.into_inner().unwrap();
        file_lint_duration.rules_total = Instant::now().duration_since(before_rules);

        LintResult {
            outcome: (&outcomes).into(),
            file_id,
            // TODO: Perhaps we can elide away this copy
            cst: Some(cst_ref.to_owned()),
            rule_results: rule_results.into_inner().unwrap(),
            parser_errors: unwrapped_diagnostics,
            duration: file_lint_duration,
        }
    }

    /// Load cache, Load files from the file walker, load rules from groups, run rules, and call respective formatters
    pub fn exec(&self, linter: &mut Linter) {
        let linting_start = Instant::now();

        let paths = linter.walker.paths.to_owned();
        let file_results = Mutex::new(HashMap::with_capacity(paths.len()));
        let mut diagnostics = vec![];
        let files_to_lint;

        let cache_load_start = Instant::now();
        let cache_res = Cache::load();
        let cache_loading = Instant::now().duration_since(cache_load_start);

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

        let store = CstRuleStore::new().load_predefined_groups();

        if files_to_lint.len() == 0 {
            Cache::generate(file_results.into_inner().unwrap())
                .persist()
                .expect("Failed to write cache file to current directory");

            return linter.formatter.format(&diagnostics, &linter.walker);
        }

        let file_load_start = Instant::now();

        let mut new_walker = FileWalker::with_paths(files_to_lint);
        diagnostics.extend(
            new_walker
                .load()
                .unwrap()
                .into_iter()
                .map(|string| Diagnostic::warning().with_message(string))
                .collect::<Vec<Diagnostic<usize>>>(),
        );
        let file_loading = Instant::now().duration_since(file_load_start);

        let files_start = Instant::now();

        let lint_results = new_walker.files.par_iter().map(|file| {
            self.lint_file(*file.0, file.1.source(), &store)
        }).collect::<Vec<_>>();

        let file_linting = Instant::now().duration_since(files_start);

        let overall = Instant::now().duration_since(linting_start);
        let lint_time = LintDuration {
            cache_loading,
            file_linting,
            file_loading,
            overall
        };

        // If TIMING is 1 we should render linter time tables
        if let Ok(val) = env::var("TIMING") {
            if val == "1".to_string() {
                Self::render_rule_timing_table(lint_results.iter().map(|result| &result.duration.rules).collect::<Vec<_>>(), overall);
                Self::render_linter_timing_table(lint_time);
            }
        }
        
        for lint_result in lint_results.into_iter() {
            diagnostics.extend(lint_result.parser_errors);
            diagnostics.extend(lint_result.rule_results.into_iter().map(|(_, res)| res.diagnostics).flatten().collect::<Vec<_>>());
        }

        Cache::generate(file_results.into_inner().unwrap())
            .persist()
            .expect("Failed to write cache file to current directory");
        linter.formatter.format(&diagnostics, &new_walker);
    }

    /// Render a table describing how long the major operations of the linter took
    pub fn render_linter_timing_table(linter_timing: LintDuration) {
        let columns = vec![
            Cell::with_color("Operation".to_string(), Color::Cyan),
            Cell::with_color("Duration (μs)".to_string(), Color::Cyan),
            Cell::with_color("Percent total".to_string(), Color::Cyan),
        ];

        let mut rows = vec![];
        let overall = linter_timing.overall.as_micros();

        let cache_loading = linter_timing.cache_loading.as_micros();
        rows.push(vec![
            Cell::new("Loading cache".to_string()),
            Cell::with_color(cache_loading.to_string(), Color::Red),
            Cell::new(((cache_loading as f32 / overall as f32) * 100.0).round().to_string()),
        ]);

        let file_loading = linter_timing.file_loading.as_micros();
        rows.push(vec![
            Cell::new("Loading files".to_string()),
            Cell::with_color(file_loading.to_string(), Color::Red),
            Cell::new(((file_loading as f32 / overall as f32) * 100.0).round().to_string()),
        ]);

        let file_linting = linter_timing.file_linting.as_micros();
        rows.push(vec![
            Cell::new("Linting files".to_string()),
            Cell::with_color(file_linting.to_string(), Color::Red),
            Cell::new(((file_linting as f32 / overall as f32) * 100.0).round().to_string()),
        ]);

        rows.push(vec![
            Cell::new("Overall".to_string()),
            Cell::with_color(overall.to_string(), Color::Red),
        ]);

        Table::new(columns, rows, vec![]).render();
    }

    /// Generate a table outlining the top 10 rules that took the longest on average and render it to the terminal  
    /// If any file does not run a rule that another file runs, the rule's duration is assumed to be `0` for the file missing the rule
    pub fn render_rule_timing_table(durations: Vec<&HashMap<&'static str, Duration>>, total_time: Duration) {
        let mut averages = Vec::with_capacity(durations.first().map(|map| map.len()).unwrap_or(0));

        if durations.is_empty() {
            return;
        }

        let total_duration = total_time.as_micros();

        for rule in durations.first().unwrap().keys() {
            let file_times = durations.iter().map(|elem| elem.get(rule).map(|duration| duration.as_micros()).unwrap_or(0)).collect::<Vec<_>>();
            let len = file_times.len();
            averages.push((*rule, file_times.into_iter().sum::<u128>() as f32 / len as f32));
        }

        let columns = vec![
            Cell::with_color("Rule".to_string(), Color::Cyan),
            Cell::with_color("Avg duration (μs)".to_string(), Color::Cyan),
            Cell::with_color("Percent total".to_string(), Color::Cyan),
        ];

        let mut rows = Vec::with_capacity(10);

        // Floats are not Ord
        averages.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        for (rule_name, avg) in averages.into_iter().rev().take(10) {
            let avg_str = avg.to_string();
            let percent_total = ((avg / total_duration as f32) * 100.0).round().to_string();
            rows.push(vec![
                Cell::new(rule_name.to_string()),
                Cell::with_color(avg_str, Color::Red),
                Cell::new(percent_total),
            ]);
        }

        // let notes = vec![
        //     Cell::new("Note: Rules are run in parallel over files in parallel, therefore the total time is closer to the average instead of `rule_time * files`".to_string())
        // ];

        Table::new(columns, rows, vec![]).render();
    }
}
