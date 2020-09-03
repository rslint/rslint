use crate::group;

group! {
    /// Rules which relate to productions which are almost always erroneous or cause
    errors,
    no_unsafe_finally::NoUnsafeFinally,
    no_cond_assign::NoCondAssign,
    no_await_in_loop::NoAwaitInLoop,
    getter_return::GetterReturn,
    no_unsafe_negation::NoUnsafeNegation
}
