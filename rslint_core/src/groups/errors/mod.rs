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
    no_async_promise_executor::NoAsyncPromiseExecutor
}
