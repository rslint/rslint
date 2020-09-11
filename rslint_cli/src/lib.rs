mod files;
mod cli;
mod panic_hook;
mod config;

pub use self::{
    files::*,
    cli::ExplanationRunner,
    panic_hook::*,
    config::*
};
pub use rslint_core::{DiagnosticBuilder, Diagnostic, Outcome};

use codespan_reporting::diagnostic::Severity;
use codespan_reporting::term::Config;
use codespan_reporting::term::{
    emit,
    termcolor::{ColorChoice, StandardStream, self},
};
use rayon::prelude::*;
use rslint_core::{lint_file, CstRuleStore, RuleLevel};

pub(crate) const DOCS_LINK_BASE: &str = "https://raw.githubusercontent.com/RDambrosio016/RSLint/dev/docs/rules";
pub(crate) const REPO_LINK: &str = "https://github.com/RDambrosio016/RSLint/tree/dev";

pub fn codespan_config() -> Config {
    let mut base = Config::default();
    base.chars.multi_top_left = '┌';
    base.chars.multi_bottom_left = '└';
    base
}

pub fn run(glob: String, verbose: bool) {
    let res = glob::glob(&glob);
    if let Err(err) = res {
        lint_err!("Invalid glob pattern: {}", err);
        return;
    }

    let handle = config::Config::new_threaded();
    let walker = FileWalker::from_glob(res.unwrap());
    let joined = handle.join();

    let config = if let Some(Some(Err(err))) = joined.as_ref().ok() {
        // this is a bit of a hack. we should do this in a better way in the future
        // toml also seems to give incorrect column numbers so we can't use it currently
        let regex = regex::Regex::new(r"\. did you mean '(.*?)'\?").unwrap();
        let location_regex = regex::Regex::new(r"at line \d+").unwrap();
        let mut msg = err.to_string().split_at(location_regex.find(&err.to_string()).unwrap().range().start).0.to_string();
        let old = msg.clone();

        let diagnostic = if let Some(found) = regex.find(&old) {
            msg.replace_range(found.range(), "");
            DiagnosticBuilder::error(0, "config", &msg)
                .note(format!("help: did you mean '{}'?", regex.captures(&old).unwrap().get(1).unwrap().as_str()))
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

    let mut results = walker
        .files
        .par_iter()
        .map(|(id, file)| {
            lint_file(
                *id,
                &file.source,
                file.kind == JsFileKind::Module,
                &store,
                verbose,
            )
        })
        .collect::<Vec<_>>();

    // Map each diagnostic to the correct level according to configured rule level
    for result in results.iter_mut() {
        for (rule_name, diagnostics) in result.rule_diagnostics.iter_mut() {
            if let Some(conf) = config.as_ref().and_then(|cfg| cfg.rules.as_ref()) {
                remap_diagnostics_to_level(diagnostics, conf.rule_level_by_name(rule_name));
            }
        }
    }

    let failures = results.iter().filter(|res| res.outcome() == Outcome::Failure).count();
    let warnings = results.iter().filter(|res| res.outcome() == Outcome::Warning).count();
    let successes = results.iter().filter(|res| res.outcome() == Outcome::Success).count();

    let overall = Outcome::merge(results.iter().map(|res| res.outcome()));

    for result in results.into_iter() {
        for diagnostic in result.diagnostics() {
            emit(
                &mut StandardStream::stderr(ColorChoice::Always),
                &codespan_config(),
                &walker,
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

fn output_overall(failures: usize, warnings: usize, successes: usize) {
    use std::io::Write;
    use termcolor::{Color, ColorSpec, WriteColor};

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::White))).unwrap();
    write!(&mut stdout, "\nOutcome: ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red))).unwrap();
    write!(&mut stdout, "{}", failures).unwrap();
    stdout.reset().unwrap();
    write!(&mut stdout, " fail, ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow))).unwrap();
    write!(&mut stdout, "{}", warnings).unwrap();
    stdout.reset().unwrap();
    write!(&mut stdout, " warn, ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green))).unwrap();
    write!(&mut stdout, "{}", successes).unwrap();
    stdout.reset().unwrap();
    write!(&mut stdout, " success\n").unwrap();
}

/// Remap each error diagnostic to a warning diagnostic based on the rule's level. 
/// this leaves warnings untouched because rules should be able to emit errors and warnings for context without
/// the warnings being remapped to errors. 
pub fn remap_diagnostics_to_level(diagnostics: &mut Vec<Diagnostic>, level: RuleLevel) {
    for diagnostic in diagnostics.iter_mut() {
        match diagnostic.severity {
            Severity::Error | Severity::Bug if level == RuleLevel::Warning => diagnostic.severity = Severity::Warning,
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
        &diagnostic.into()
    ).expect("Failed to throw linter diagnostic");
}

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
