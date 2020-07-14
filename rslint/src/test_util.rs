//! Macros for testing if rules pass or error

/// Assert that a rule's outcome is an error when matched against one or more pieces of source code  
/// You can also add a `=> start..end` after the source code to assert the location of the error
/// # Examples 
/// ```rust
/// use rslint::rules::groups::errors::no_empty::NoEmpty;
/// use rslint::assert_lint_err;
/// 
/// assert_lint_err! {
///     NoEmpty,
///     "{}" => 0..2,
///     "try { }" => 4..7,
///     "try { /* a */ } catch(e) { }",
///     "try { /* a */ } finally { }",
///     "switch(a) {}",
///     ";",
///     "{{}}",
/// }
/// ```
#[macro_export]
macro_rules! assert_lint_err {
    ($rule:ident, $(
        $string:expr $(=> $span:expr)? $(,)?
    )*) => {
        use crate::rules::context::RuleContext;
        use crate::rules::{RuleResult, Outcome, CstRule};
        use rslint_parse::parser::Parser;
        use codespan_reporting::diagnostic::LabelStyle;
        $(
            let mut ctx = RuleContext {
                file_source: $string,
                file_id: "tests",
                diagnostics: vec![],
            };
            let cst = Parser::with_source($string, "tests", true).unwrap().parse_script().unwrap();

            $rule {}.lint(&mut ctx, &cst);
            let result = RuleResult::from(ctx.diagnostics);

            assert_eq!(result.outcome, Outcome::Error);

            $(
                let err = result.diagnostics.first().unwrap().labels.iter().find(|x| x.style == LabelStyle::Primary).unwrap();
                assert_eq!(err.range, $span);
            )?
        )*
    }
}

/// Assert that a rule's outcome is a success when matched against one or more pieces of source code  
/// # Examples 
/// ```rust
/// use rslint::rules::groups::errors::no_empty::NoEmpty;
/// use rslint::assert_lint_ok;
/// 
/// assert_lint_ok! {
///     NoEmpty,
///     "{ /* */ }",
///     "{\n /* */}",
///     "{/* */\n}",
///     "function a() {}",
///     "try { /* \n*/ } catch (e) { /* \n*/ } finally { /* */\n }",
///     "switch (a) { /* */ }",
///     "switch (a) { \n case 5: }"
/// }
/// ```
#[macro_export]
macro_rules! assert_lint_ok {
    ($rule:ident, $(
        $string:expr $(,)?
    )*) => {
        use crate::rules::context::RuleContext;
        use crate::rules::{RuleResult, Outcome, CstRule};
        use rslint_parse::parser::Parser;
        $(
            let mut ctx = RuleContext {
                file_source: $string,
                file_id: "tests",
                diagnostics: vec![],
            };
            let cst = Parser::with_source($string, "tests", true).unwrap().parse_script().unwrap();

            $rule {}.lint(&mut ctx, &cst);

            assert_eq!(RuleResult::from(ctx.diagnostics).outcome, Outcome::Success);
        )*
    }
}