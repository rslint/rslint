#![deny(rust_2018_idioms)]

mod cli;
mod files;
mod infer;
mod macros;
mod panic_hook;

use std::{collections::HashMap, sync::Arc};

pub use self::{
    cli::{show_all_rules, ExplanationRunner},
    files::*,
    infer::infer,
    panic_hook::panic_hook,
};

use colored::Colorize;
use rslint_config::Config;
use rslint_core::{
    autofix::recursively_apply_fixes,
    errors::LongFormatter,
    errors::{file, Diagnostic, Emitter},
    errors::{Formatter, ShortFormatter},
    lexer::Lexer,
    lint_file,
    util::find_best_match_for_name,
    CstRuleStore, LintResult, Outcome, RuleLevel, Severity,
};
use smol::stream::StreamExt;

pub async fn run(
    globs: Vec<String>,
    verbose: bool,
    fix: bool,
    dirty: bool,
    formatter: Option<String>,
    no_global_config: bool,
) {
    #[cfg_attr(debug_assertions, allow(unused))]
    let exit_code = run_inner(globs, verbose, fix, dirty, formatter, no_global_config).await;
    #[cfg(not(debug_assertions))]
    std::process::exit(exit_code);
}

pub async fn run_inner(
    globs: Vec<String>,
    verbose: bool,
    fix: bool,
    dirty: bool,
    formatter: Option<String>,
    no_global_config: bool,
) -> i32 {
    let config = smol::spawn(Config::new(no_global_config, |file, d| {
        emit_diagnostic(&d, &file)
    }));
    let walker = smol::spawn(FileWalker::from_globs(globs));
    let mut config = config.await;
    emit_diagnostics("short", &config.warnings(), &file::empty_files());

    let (tx, rx) = smol::channel::unbounded();

    let store = Arc::new(config.rules_store());
    let walker = walker.await;
    let mut files = walker.into_files();
    let mut tasks = vec![];
    while let Some(file) = files.next() {
        let tx = tx.clone();
        let store = Arc::clone(&store);
        let task = smol::spawn(async move {
            let res = lint_file(
                file.id,
                &file.source,
                file.kind == JsFileKind::Module,
                &store,
                verbose,
            );
            tx.send((res, file))
                .await
                .expect("receiver should never be dropped first");
        });
        tasks.push(task);
    }

    drop(tx);
    let (mut results, files): (Vec<_>, HashMap<_, _>) =
        rx.map(|(res, file)| (res, (file.id, file))).unzip().await;
    let mut walker = FileWalker::new(files);

    let fix_count = if fix {
        apply_fixes(&mut results, &mut walker, &store, dirty)
    } else {
        0
    };

    let mut formatter = formatter.unwrap_or_else(|| config.formatter());
    verify_formatter(&mut formatter);
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

pub fn apply_fixes(
    results: &mut Vec<LintResult>,
    walker: &mut FileWalker,
    store: &CstRuleStore,
    dirty: bool,
) -> usize {
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
        let fixed = recursively_apply_fixes(res, store);
        let new_problem_num = res
            .rule_results
            .iter()
            .filter(|(_, x)| x.outcome() == Outcome::Warning || x.outcome() == Outcome::Failure)
            .map(|(_, res)| res.diagnostics.len())
            .sum::<usize>();
        let path = file.path.as_ref().unwrap();
        if let Err(err) = std::fs::write(path, fixed.clone()) {
            lint_err!("failed to write to `{:#?}`: {}", path, err.to_string());
        } else {
            file.update_src(fixed);
            fix_count += original_problem_num.saturating_sub(new_problem_num);
        }
    }
    fix_count
}

pub async fn dump_ast(files: Vec<String>) {
    FileWalker::walk_files(files.as_ref(), |file| async move {
        println!("{}", file.path_or_name().red().bold());
        println!("{:#?}", file.parse());
    })
    .await
}

pub async fn tokenize(files: Vec<String>) {
    FileWalker::walk_files(files.as_ref(), |file| async move {
        println!("{}", file.path_or_name().red().bold());

        let source = file.source.as_str();
        let tokens = Lexer::from_str(source, file.id)
            .map(|(tok, d)| {
                if let Some(d) = d {
                    emit_diagnostic(&d, &file);
                }
                tok
            })
            .collect::<Vec<_>>();

        rslint_core::parser::TokenSource::new(source, tokens.as_slice()).for_each(|tok| {
            println!("{:?}@{}..{}", tok.kind, tok.range.start, tok.range.end);
        });
        println!();
    })
    .await
}

pub fn emit_diagnostics(formatter: &str, diagnostics: &[Diagnostic], files: &dyn file::Files) {
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

pub fn emit_diagnostic(diagnostic: &Diagnostic, walker: &dyn file::Files) {
    let mut emitter = Emitter::new(walker);
    emitter
        .emit_stderr(&diagnostic, true)
        .expect("failed to throw linter diagnostic")
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

pub(crate) fn print_results(
    results: &mut Vec<LintResult>,
    walker: &FileWalker,
    config: &Config,
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
