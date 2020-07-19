pub mod no_empty;
pub mod no_compare_neg_zero;
pub mod no_unsafe_finally;
pub mod no_duplicate_case;
pub mod no_cond_assign;
pub mod no_constant_condition;

#[macro_export]
macro_rules! register_errors_group {
    ($groups:expr) => {
        use crate::lint_group;
        $groups.push(lint_group! {
            errors,
            "errors",
            no_empty - NoEmpty,
            no_compare_neg_zero - NoCompareNegZero,
            no_unsafe_finally - NoUnsafeFinally,
            no_duplicate_case - NoDuplicateCase,
            no_cond_assign - NoCondAssign,
            no_constant_condition - NoConstantCondition,
        })
    }
}
