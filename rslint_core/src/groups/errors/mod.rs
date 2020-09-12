use crate::group;

group! {
    /// Rules which relate to productions which are almost always erroneous or cause
    /// unexpected behavior.
    errors,
    no_unsafe_finally::NoUnsafeFinally,
    no_cond_assign::NoCondAssign,
    no_await_in_loop::NoAwaitInLoop,
    getter_return::GetterReturn,
    no_unsafe_negation::NoUnsafeNegation,
    no_compare_neg_zero::NoCompareNegZero,
    no_async_promise_executor::NoAsyncPromiseExecutor,
    no_constant_condition::NoConstantCondition,
    for_direction::ForDirection,
    no_debugger::NoDebugger,
    no_dupe_keys::NoDupeKeys,
    no_duplicate_cases::NoDuplicateCases,
    no_empty::NoEmpty,
    no_extra_semi::NoExtraSemi
}
