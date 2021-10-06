//! Rules which relate to productions which are almost always erroneous or cause
//! unexpected behavior.

use crate::group;

group! {
    /// Rules which relate to productions which are almost always erroneous or cause
    /// unexpected behavior.
    errors,
    constructor_super::ConstructorSuper,
    for_direction::ForDirection,
    getter_return::GetterReturn,
    no_async_promise_executor::NoAsyncPromiseExecutor,
    no_await_in_loop::NoAwaitInLoop,
    no_compare_neg_zero::NoCompareNegZero,
    no_cond_assign::NoCondAssign,
    no_confusing_arrow::NoConfusingArrow,
    no_constant_condition::NoConstantCondition,
    no_debugger::NoDebugger,
    no_dupe_keys::NoDupeKeys,
    no_duplicate_cases::NoDuplicateCases,
    no_duplicate_imports::NoDuplicateImports,
    no_empty::NoEmpty,
    no_extra_boolean_cast::NoExtraBooleanCast,
    no_extra_semi::NoExtraSemi,
    no_inner_declarations::NoInnerDeclarations,
    no_irregular_whitespace::NoIrregularWhitespace,
    no_new_symbol::NoNewSymbol,
    no_prototype_builtins::NoPrototypeBuiltins,
    no_self_assign::NoSelfAssign,
    no_setter_return::NoSetterReturn,
    no_sparse_arrays::NoSparseArrays,
    no_this_before_super::NoThisBeforeSuper,
    no_unexpected_multiline::NoUnexpectedMultiline,
    no_unsafe_finally::NoUnsafeFinally,
    no_unsafe_negation::NoUnsafeNegation,
    use_isnan::UseIsnan,
    valid_typeof::ValidTypeof,
}
