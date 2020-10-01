mod cli;
mod config;
mod files;
mod panic_hook;
mod watch;

pub use self::{cli::ExplanationRunner, config::*, files::*, panic_hook::*};
pub use rslint_core::{Diagnostic, DiagnosticBuilder, Outcome};

use codespan_reporting::diagnostic::Severity;
use codespan_reporting::term::Config;
use codespan_reporting::term::{
    emit,
    termcolor::{self, ColorChoice, StandardStream},
};
use colored::*;
use rayon::prelude::*;
use rslint_core::{lint_file, CstRuleStore, LintResult, RuleLevel};
use watch::start_watcher;

pub(crate) const DOCS_LINK_BASE: &str =
    "https://raw.githubusercontent.com/RDambrosio016/RSLint/dev/docs/rules";
pub(crate) const REPO_LINK: &str = "https://github.com/RDambrosio016/RSLint/tree/dev";

pub fn codespan_config() -> Config {
    let mut base = Config::default();
    base.chars.multi_top_left = '┌';
    base.chars.multi_bottom_left = '└';
    base
}

#[allow(unused_must_use)]
pub fn run(glob: String, verbose: bool, watch: bool) {
    let res = glob::glob(&glob);
    if let Err(err) = res {
        lint_err!("Invalid glob pattern: {}", err);
        return;
    }

    let handle = config::Config::new_threaded();
    let walker = FileWalker::from_glob(res.unwrap());
    let joined = handle.join();

    let config = if let Ok(Some(Err(err))) = joined.as_ref() {
        // this is a bit of a hack. we should do this in a better way in the future
        // toml also seems to give incorrect column numbers so we can't use it currently
        let regex = regex::Regex::new(r"\. did you mean '(.*?)'\?").unwrap();
        let location_regex = regex::Regex::new(r"at line \d+").unwrap();
        let mut msg = err
            .to_string()
            .split_at(location_regex.find(&err.to_string()).unwrap().range().start)
            .0
            .to_string();
        let old = msg.clone();

        let diagnostic = if let Some(found) = regex.find(&old) {
            msg.replace_range(found.range(), "");
            DiagnosticBuilder::error(0, "config", &msg).note(format!(
                "help: did you mean '{}'?",
                regex.captures(&old).unwrap().get(1).unwrap().as_str()
            ))
        } else {
            DiagnosticBuilder::error(0, "config", msg)
        };

        return emit_diagnostic(diagnostic, &FileWalker::empty());
    } else {
        joined.unwrap().map(|res| res.unwrap())
    };

    let store = if let Some(cfg) = config.as_ref().and_then(|cfg| cfg.rules.as_ref()) {
        cfg.store()
    } else {
        CstRuleStore::new().builtins()
    };

    let results = walker
        .files
        .par_iter()
        .map(|(id, file)| {
            (
                lint_file(
                    *id,
                    &file.source,
                    file.kind == JsFileKind::Module,
                    &store,
                    verbose,
                ),
                file,
            )
        })
        .filter_map(|(res, file)| {
            if let Err(diagnostic) = res {
                emit_diagnostic(diagnostic, &walker);
                None
            } else {
                res.ok().map(|res| (res, file))
            }
        })
        .collect::<Vec<_>>();

    let unzipped_results = results.clone().into_iter().map(|(r, _)| r).collect();
    print_results(unzipped_results, &walker, config.as_ref());

    if watch {
        use std::io::Write;
        use termcolor::{Color, ColorSpec, WriteColor};

        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)));
        writeln!(&mut stdout, "\nWatching for file changes...\n");
        start_watcher(walker.clone(), results, config.as_ref());
    }
}

pub(crate) fn print_results(
    mut results: Vec<LintResult>,
    walker: &FileWalker,
    config: Option<&config::Config>,
) {
    // Map each diagnostic to the correct level according to configured rule level
    for result in results.iter_mut() {
        for (rule_name, diagnostics) in result.rule_diagnostics.iter_mut() {
            if let Some(conf) = config.and_then(|cfg| cfg.rules.as_ref()) {
                remap_diagnostics_to_level(diagnostics, conf.rule_level_by_name(rule_name));
            }
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

    for result in results.into_iter() {
        for diagnostic in result.diagnostics() {
            emit(
                &mut StandardStream::stderr(ColorChoice::Always),
                &codespan_config(),
                walker,
                diagnostic,
            )
            .expect("Failed to throw diagnostic");
        }
    }

    output_overall(failures, warnings, successes);
    if overall == Outcome::Failure {
        println!("\nhelp: for more information about the errors try the explain command: `rslint explain <rules>`");
    }
}

#[allow(unused_must_use)]
fn output_overall(failures: usize, warnings: usize, successes: usize) {
    println!(
        "{}: {} fail, {} warn, {} success",
        "Outcome".white(),
        failures.to_string().red(),
        warnings.to_string().yellow(),
        successes.to_string().green()
    );
}

/// Remap each error diagnostic to a warning diagnostic based on the rule's level.
/// this leaves warnings untouched because rules should be able to emit errors and warnings for context without
/// the warnings being remapped to errors.
pub fn remap_diagnostics_to_level(diagnostics: &mut Vec<Diagnostic>, level: RuleLevel) {
    for diagnostic in diagnostics.iter_mut() {
        match diagnostic.severity {
            Severity::Error | Severity::Bug if level == RuleLevel::Warning => {
                diagnostic.severity = Severity::Warning
            }
            _ => {}
        }
    }
}

pub fn emit_diagnostic(diagnostic: impl Into<Diagnostic>, walker: &FileWalker) {
    use codespan_reporting::term::termcolor::ColorChoice::Always;

    emit(
        &mut termcolor::StandardStream::stderr(Always),
        &crate::codespan_config(),
        walker,
        &diagnostic.into(),
    )
    .expect("Failed to throw linter diagnostic");
}

// TODO: don't use expect because we treat panics as linter bugs
#[macro_export]
macro_rules! lint_diagnostic {
    ($severity:ident, $($format_args:tt)*) => {
        use $crate::DiagnosticBuilder;
        use codespan_reporting::{
            files::SimpleFiles,
            term::{termcolor::{ColorChoice::Always, self}, emit}
        };

        let diag = DiagnosticBuilder::$severity(0, "", format!($($format_args)*));
        emit(
            &mut termcolor::StandardStream::stderr(Always),
            &$crate::codespan_config(),
            &SimpleFiles::<String, String>::new(),
            &diag.into()
        ).expect("Failed to throw linter diagnostic");
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
        $crate::lint_diagnostic!(note_diagnostic, $($format_args)*);
    }};
}
