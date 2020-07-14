pub mod no_empty;

#[macro_export]
macro_rules! register_errors_group {
    ($groups:expr) => {
        use crate::lint_group;
        $groups.push(lint_group! {
            errors,
            "errors",
            no_empty - NoEmpty,
        })
    }
}
