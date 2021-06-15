//! Rules which relate to productions which are almost always erroneous or cause
//! unexpected behavior.

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
    no_duplicate_imports::NoDuplicateImports,
    no_empty::NoEmpty,
    no_extra_semi::NoExtraSemi,
    no_inner_declarations::NoInnerDeclarations,
    no_irregular_whitespace::NoIrregularWhitespace,
    no_new_symbol::NoNewSymbol,
    no_prototype_builtins::NoPrototypeBuiltins,
    no_sparse_arrays::NoSparseArrays,
    no_unexpected_multiline::NoUnexpectedMultiline,
    use_isnan::UseIsnan,
    no_setter_return::NoSetterReturn,
    valid_typeof::ValidTypeof,
    no_extra_boolean_cast::NoExtraBooleanCast,
    no_confusing_arrow::NoConfusingArrow,
    constructor_super::ConstructorSuper,
    no_this_before_super::NoThisBeforeSuper,
}
