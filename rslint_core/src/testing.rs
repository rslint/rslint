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
    (
    $rule:expr,
    // Optional doc used in the user facing docs for the
    // more invalid code examples section.
    $(#[_:meta])*
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
        #[allow(unused_imports)]
        use $crate::run_rule;
        #[allow(unused_imports)]
        use rslint_parser::parse_module;

        #[test]
        fn invalid() {
            $(
                let res = parse_module($code, 0);
                let errs = run_rule(&(Box::new($rule) as Box<dyn CstRule>), 0, res.syntax(), true, &[]);
                if errs.is_empty() {
                    panic!("\nExpected:\n```\n{}\n```\nto fail linting, but instead it passed (with {} parsing errors)", $code, res.errors().len());
                }
            )*
        }

        #[test]
        fn valid() {
            $(
                let res = parse_module($ok_code, 0);
                let errs = run_rule(&(Box::new($rule) as Box<dyn CstRule>), 0, res.syntax(), true, &[]);

                if !errs.is_empty() {
                    panic!("\nExpected:\n```\n{}\n```\nto pass linting, but instead it threw errors (along with {} parsing errors):\n\n", $ok_code, res.errors().len());
                }
            )*
        }
    };
}
