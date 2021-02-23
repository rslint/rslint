mod cli;
mod files;
mod infer;
mod panic_hook;

pub use self::{
    cli::{show_all_rules, ExplanationRunner},
    files::*,
    infer::infer,
    panic_hook::*,
};
pub use rslint_config as config;
pub use rslint_core::Outcome;
pub use rslint_errors::{
    file, file::Files, Diagnostic, Emitter, Formatter, LongFormatter, Severity, ShortFormatter,
};

use colored::*;
use rslint_core::{autofix::recursively_apply_fixes, File};
use rslint_core::{lint_file, util::find_best_match_for_name, LintResult, RuleLevel};
use rslint_lexer::Lexer;
#[allow(unused_imports)]
use std::process;
use std::{fs::write, path::PathBuf};
use yastl::Pool;

#[allow(unused_must_use, unused_variables)]
pub fn run(
    globs: Vec<String>,
    verbose: bool,
    fix: bool,
    dirty: bool,
    formatter: Option<String>,
    no_global_config: bool,
    pool: Pool,
) {
    let exit_code = run_inner(
        globs,
        verbose,
        fix,
        dirty,
        formatter,
        no_global_config,
        pool,
    );
    #[cfg(not(debug_assertions))]
    process::exit(exit_code);
}

/// The inner function for run to call destructors before we call [`process::exit`]
fn run_inner(
    globs: Vec<String>,
    verbose: bool,
    fix: bool,
    dirty: bool,
    formatter: Option<String>,
    no_global_config: bool,
    pool: Pool,
) -> i32 {
    let mut config = None;
    let mut walker = FileWalker::empty();

    pool.scoped(|scope| {
        scope.execute(|| {
            config = Some(config::Config::new(no_global_config, |file, d| {
                emit_diagnostic(&d, &file)
            }));
        });

        scope.execute(|| {
            walker.load_files(collect_globs(globs).into_iter());
        });
    });

    let mut config = config.expect("config failed to initialize");
    emit_diagnostics("short", &config.warnings(), &walker);

    let mut formatter = formatter.unwrap_or_else(|| config.formatter());
    let store = config.rules_store();
    verify_formatter(&mut formatter);

    if walker.files.is_empty() {
        lint_err!("No matching files found");
        return 2;
    }

    let (tx, rx) = std::sync::mpsc::channel();
    pool.scoped(|scope| {
        let store = &store;

        for file in walker.files.values() {
            let tx = tx.clone();
            scope.recurse(move |_scope| {
                tx.send(lint_file(file, store, verbose)).unwrap();
            });
        }
    });
    drop(tx);
    let mut results = rx.into_iter().collect::<Vec<_>>();

    let fix_count = if fix {
        apply_fixes(&mut results, &mut walker, dirty)
    } else {
        0
    };
    print_results(&mut results, &walker, &config, fix_count, &formatter);

    // print_results remaps the result to the appropriate severity
    // so these diagnostic severities should be accurate
    if results
        .iter()
        .flat_map(|res| res.diagnostics())
        .any(|d| matches!(d.severity, Severity::Bug | Severity::Error))
    {
        1
    } else {
        0
    }
}

pub fn apply_fixes(results: &mut Vec<LintResult>, walker: &mut FileWalker, dirty: bool) -> usize {
    let mut fix_count = 0;
    // TODO: should we aquire a file lock if we know we need to run autofix?
    for res in results {
        let file = walker.files.get_mut(&res.file_id).unwrap();
        // skip virtual files
        if file.path.is_none() {
            continue;
        }
        if res
            .parser_diagnostics
            .iter()
            .any(|x| x.severity == Severity::Error)
            && !dirty
        {
            lint_note!(
                "skipping autofix for `{}` because it contains syntax errors",
                file.path.as_ref().unwrap().to_string_lossy()
            );
            continue;
        }
        let original_problem_num = res
            .rule_results
            .iter()
            .filter(|(_, x)| x.outcome() == Outcome::Warning || x.outcome() == Outcome::Failure)
            .map(|(_, res)| res.diagnostics.len())
            .sum::<usize>();
        let fixed = recursively_apply_fixes(res, file);
        let new_problem_num = res
            .rule_results
            .iter()
            .filter(|(_, x)| x.outcome() == Outcome::Warning || x.outcome() == Outcome::Failure)
            .map(|(_, res)| res.diagnostics.len())
            .sum::<usize>();
        let path = file.path.as_ref().unwrap();
        if let Err(err) = write(path, fixed.clone()) {
            lint_err!("failed to write to `{:#?}`: {}", path, err.to_string());
        } else {
            file.update_src(fixed);
            fix_count += original_problem_num.saturating_sub(new_problem_num);
        }
    }
    fix_count
}

pub fn dump_ast(globs: Vec<String>) {
    use rslint_parser::{NodeOrToken, WalkEvent};

    for_each_file(globs, |_, file| {
        let header = if let Some(path) = &file.path {
            format!("File {}", path.display())
        } else {
            format!("File {}", file.name)
        };
        println!("{}", header.red().bold());

        let parse = file.parse();
        let mut level = 0;
        for event in parse.preorder_with_tokens() {
            match event {
                WalkEvent::Enter(element) => {
                    for _ in 0..level {
                        print!("  ");
                    }
                    match element {
                        NodeOrToken::Node(node) => {
                            println!(
                                "{}@{}",
                                format!("{:?}", node.kind()).yellow(),
                                format!("{:#?}", node.text_range()).cyan()
                            );
                        }
                        NodeOrToken::Token(token) => {
                            print!(
                                "{}@{}",
                                format!("{:?}", token.kind()).yellow(),
                                format!("{:#?}", token.text_range()).cyan()
                            );
                            if token.text().len() < 25 {
                                print!(" {}", format!("{:#?}", token.text()).green());
                            } else {
                                let text = token.text().as_str();
                                for idx in 21..25 {
                                    if text.is_char_boundary(idx) {
                                        let text = format!("{} ...", &text[..idx]);
                                        print!(" {}", format!("{:#?}", text).green());
                                    }
                                }
                            }
                            println!();
                        }
                    }
                    level += 1;
                }
                WalkEvent::Leave(_) => level -= 1,
            }
        }
        println!();
    })
}

pub fn tokenize(globs: Vec<String>) {
    for_each_file(
        globs,
        |walker,
         File {
             path,
             name,
             id,
             source,
             ..
         }| {
            let header = if let Some(path) = path {
                format!("File {}", path.display())
            } else {
                format!("File {}", name)
            };
            println!("{}", header.red().bold());

            let tokens = Lexer::from_str(source.as_str(), *id)
                .map(|(tok, d)| {
                    if let Some(d) = d {
                        emit_diagnostic(&d, walker);
                    }
                    tok
                })
                .collect::<Vec<_>>();

            rslint_parser::TokenSource::new(source.as_str(), tokens.as_slice()).for_each(|tok| {
                println!("{:?}@{}..{}", tok.kind, tok.range.start, tok.range.end);
            });
            println!();
        },
    )
}

fn collect_globs(globs: Vec<String>) -> Vec<PathBuf> {
    globs
        .into_iter()
        .map(|pat| glob::glob(&pat))
        .flat_map(|path| {
            if let Err(err) = path {
                lint_err!("Invalid glob pattern: {}", err);
                None
            } else {
                path.ok()
            }
        })
        .flat_map(|path| path.filter_map(Result::ok))
        .collect()
}

fn for_each_file(globs: Vec<String>, action: impl Fn(&FileWalker, &File)) {
    let walker = FileWalker::from_glob(collect_globs(globs));
    walker.files.values().for_each(|file| action(&walker, file))
}

pub(crate) fn print_results(
    results: &mut Vec<LintResult>,
    walker: &FileWalker,
    config: &config::Config,
    fix_count: usize,
    formatter: &str,
) {
    // Map each diagnostic to the correct level according to configured rule level
    for result in results.iter_mut() {
        for (rule_name, diagnostics) in result
            .rule_results
            .iter_mut()
            .map(|x| (x.0, &mut x.1.diagnostics))
        {
            remap_diagnostics_to_level(diagnostics, config.rule_level_by_name(rule_name));
        }
    }

    let failures = results
        .iter()
        .filter(|res| res.outcome() == Outcome::Failure)
        .count();
    let warnings = results
        .iter()
        .filter(|res| res.outcome() == Outcome::Warning)
        .count();
    let successes = results
        .iter()
        .filter(|res| res.outcome() == Outcome::Success)
        .count();

    let overall = Outcome::merge(results.iter().map(|res| res.outcome()));

    for result in results.iter_mut() {
        emit_diagnostics(
            formatter,
            &result.diagnostics().cloned().collect::<Vec<_>>(),
            walker,
        );
    }

    output_overall(failures, warnings, successes, fix_count);
    if overall == Outcome::Failure {
        println!("\nhelp: for more information about the errors try the explain command: `rslint explain <rules>`");
    }
}

pub fn verify_formatter(formatter: &mut String) {
    if !matches!(formatter.as_str(), "short" | "long") {
        if let Some(suggestion) =
            find_best_match_for_name(vec!["short", "long"].into_iter(), formatter, None)
        {
            lint_err!(
                "unknown formatter `{}`, using default formatter, did you mean `{}`?",
                formatter,
                suggestion
            );
        } else {
            lint_err!("unknown formatter `{}`, using default formatter", formatter);
        }
        *formatter = "long".to_string();
    }
}

pub fn emit_diagnostics(formatter: &str, diagnostics: &[Diagnostic], files: &dyn Files) {
    match formatter {
        "short" => {
            if let Err(err) = ShortFormatter.emit_stderr(diagnostics, files) {
                lint_err!("failed to emit diagnostic: {}", err);
            }
        }
        "long" => {
            if let Err(err) = LongFormatter.emit_stderr(diagnostics, files) {
                lint_err!("failed to emit diagnostic: {}", err);
            }
        }
        f => {
            if let Some(suggestion) =
                find_best_match_for_name(vec!["short", "long"].into_iter(), f, None)
            {
                lint_err!("unknown formatter `{}`, did you mean `{}`?", f, suggestion);
            } else {
                lint_err!("unknown formatter `{}`", f);
            }
        }
    }
}

#[allow(unused_must_use)]
fn output_overall(failures: usize, warnings: usize, successes: usize, fix_count: usize) {
    println!(
        "{}: {} fail, {} warn, {} success{}",
        "Outcome".white(),
        failures.to_string().red(),
        warnings.to_string().yellow(),
        successes.to_string().green(),
        if fix_count > 0 {
            format!(
                ", {} issue{} fixed",
                fix_count.to_string().green(),
                if fix_count == 1 { "" } else { "s" }
            )
        } else {
            "".to_string()
        }
    );
}

/// Remap each error diagnostic to a warning diagnostic based on the rule's level.
/// this leaves warnings untouched because rules should be able to emit errors and warnings for context without
/// the warnings being remapped to errors.
pub fn remap_diagnostics_to_level(diagnostics: &mut Vec<Diagnostic>, level: RuleLevel) {
    for diagnostic in diagnostics.iter_mut() {
        match diagnostic.severity {
            Severity::Error if level == RuleLevel::Warning => {
                diagnostic.severity = Severity::Warning
            }
            _ => {}
        }
    }
}

pub fn emit_diagnostic(diagnostic: &Diagnostic, walker: &dyn file::Files) {
    let mut emitter = Emitter::new(walker);
    emitter
        .emit_stderr(&diagnostic, true)
        .expect("failed to throw linter diagnostic")
}

// TODO: don't use expect because we treat panics as linter bugs
#[macro_export]
macro_rules! lint_diagnostic {
    ($severity:ident, $($format_args:tt)*) => {
    use rslint_errors::Emitter;

    let diag = $crate::Diagnostic::$severity(1, "", format!($($format_args)*));
    let file = rslint_errors::file::SimpleFile::new("".into(), "".into());
    let mut emitter = Emitter::new(&file);
    emitter
        .emit_stderr(&diag, true)
        .expect("failed to throw linter diagnostic")
    }
}

/// Construct a simple linter error and immediately throw it to stderr
#[macro_export]
macro_rules! lint_err {
    ($($format_args:tt)*) => {{
        $crate::lint_diagnostic!(error, $($format_args)*);
    }};
}

/// Construct a simple linter warning and immediately throw it to stderr
#[macro_export]
macro_rules! lint_warn {
    ($($format_args:tt)*) => {{
        $crate::lint_diagnostic!(warning, $($format_args)*);
    }};
}

/// Construct a simple linter note and immediately throw it to stderr
#[macro_export]
macro_rules! lint_note {
    ($($format_args:tt)*) => {{
        $crate::lint_diagnostic!(note, $($format_args)*);
    }};
}
