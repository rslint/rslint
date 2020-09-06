mod files;
mod cli;
mod panic_hook;

pub use self::{
    files::*,
    cli::ExplanationRunner,
    panic_hook::*,
};
pub use rslint_core::{DiagnosticBuilder, Diagnostic};

use codespan_reporting::term::Config;

pub(crate) const DOCS_LINK_BASE: &str = "https://raw.githubusercontent.com/RDambrosio016/RSLint/dev/docs/rules";

pub fn codespan_config() -> Config {
    let mut base = Config::default();
    base.chars.multi_top_left = '┌';
    base.chars.multi_bottom_left = '└';
    base
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

/// Construct a simple linter error and immediately throw it to stderr
#[macro_export]
macro_rules! lint_warn {
    ($($format_args:tt)*) => {{
        $crate::lint_diagnostic!(warning, $($format_args)*);
    }};
}

/// Construct a simple linter error and immediately throw it to stderr
#[macro_export]
macro_rules! lint_note {
    ($($format_args:tt)*) => {{
        $crate::lint_diagnostic!(note_diagnostic, $($format_args)*);
    }};
}
