pub mod errors;

#[macro_export]
macro_rules! lint_group {
    ($group:ident, $name:expr, $($filename:ident - $rule:ident, $(,)?)*) => {
        crate::rules::CstRuleGroup::new($name, vec![
            $(
                Box::new(crate::rules::groups::$group::$filename::$rule {}),
            )*
        ])
    }
}