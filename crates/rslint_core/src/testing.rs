//! Macros for easily making rule tests which also generate documentation examples.

/// A macro for generating linter rule tests.
///
/// The tests are also used to generate "more examples" sections
/// in user facing docs. You can use a `/// ignore` doc
/// on a code expr to make docgen ignore it for user facing docs.
///
/// test code is run as modules, not scripts.
#[macro_export]
macro_rules! rule_tests {
    ($rule:expr,
    err: {
        $(
            // An optional tag to give to docgen
            $(#[$err_meta:meta])*
            $code:literal
        ),* $(,)?
    },

    // Optional doc used in the user facing docs for the
    // more valid code examples section.
    $(#[_:meta])*
    ok: {
        $(
            // An optional tag to give to docgen
            $(#[$ok_meta:meta])*
            $ok_code:literal
        ),* $(,)?
    } $(,)?) => {
        rule_tests!(valid, invalid, $rule, err: { $($code),* }, ok: { $($ok_code),* });
    };
    (
    $ok_name:ident,
    $err_name:ident,
    $rule:expr,
    err: {
        $(
            // An optional tag to give to docgen
            $(#[$err_meta:meta])*
            $code:literal
        ),* $(,)?
    },
    ok: {
        $(
            // An optional tag to give to docgen
            $(#[$ok_meta:meta])*
            $ok_code:literal
        ),* $(,)?
    } $(,)?) => {
        #[test]
        fn $err_name() {
            $(
                let res = rslint_parser::parse_module($code, 0);
                let errs = $crate::run_rule(&$rule, 0, res.syntax(), true, &[], std::sync::Arc::from($code.to_string()));
                if errs.diagnostics.is_empty() {
                    panic!("\nExpected:\n```\n{}\n```\nto fail linting, but instead it passed (with {} parsing errors)", $code, res.errors().len());
                }
            )*
        }

        #[test]
        fn $ok_name() {
            $(
                let res = rslint_parser::parse_module($ok_code, 0);
                let errs = $crate::run_rule(&$rule, 0, res.syntax(), true, &[], std::sync::Arc::from($ok_code.to_string()));

                if !errs.diagnostics.is_empty() {
                    panic!("\nExpected:\n```\n{}\n```\nto pass linting, but instead it threw errors (along with {} parsing errors):\n\n", $ok_code, res.errors().len());
                }
            )*
        }
    };
}
