// TODO: don't use expect because we treat panics as linter bugs
#[macro_export]
macro_rules! lint_diagnostic {
    ($severity:ident, $($format_args:tt)*) => {
        let diag = ::rslint_core::errors::Diagnostic::$severity(1, "", format!($($format_args)*));
        let file = ::rslint_core::errors::file::SimpleFile::new("".into(), "".into());

        let mut emitter = ::rslint_core::errors::Emitter::new(&file);
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
