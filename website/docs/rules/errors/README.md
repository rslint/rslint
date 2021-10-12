<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->

# Errors

Rules which relate to productions which are almost always erroneous or cause
unexpected behavior.
## Rules
| Name | Description |
| ---- | ----------- |
| [constructor-super](./constructor-super.md) | Verify calls of `super()` in constructors |
| [for-direction](./for-direction.md) | Disallow for loops which update their counter in the wrong direction. |
| [getter-return](./getter-return.md) | Disallow getter properties which do not always return a value. |
| [no-async-promise-executor](./no-async-promise-executor.md) | Disallow async functions as promise executors. |
| [no-await-in-loop](./no-await-in-loop.md) | Disallow await inside of loops. |
| [no-compare-neg-zero](./no-compare-neg-zero.md) | Disallow comparison against `-0` which yields unexpected behavior. |
| [no-cond-assign](./no-cond-assign.md) | Forbid the use of assignment expressions in conditions which may yield unwanted behavior. |
| [no-confusing-arrow](./no-confusing-arrow.md) | Disallow arrow functions where they could be confused with comparisons. |
| [no-constant-condition](./no-constant-condition.md) | Disallow constant conditions which always yield one result. |
| [no-debugger](./no-debugger.md) | Disallow the use of debugger statements. |
| [no-dupe-keys](./no-dupe-keys.md) | Disallow duplicate keys in object literals. |
| [no-duplicate-cases](./no-duplicate-cases.md) | Disallow duplicate test cases in `switch` statements. |
| [no-duplicate-imports](./no-duplicate-imports.md) | Disallow duplicate imports. |
| [no-empty](./no-empty.md) | Disallow empty block statements. |
| [no-extra-boolean-cast](./no-extra-boolean-cast.md) | Disallow unnecessary boolean casts. |
| [no-extra-semi](./no-extra-semi.md) | Disallow unneeded semicolons. |
| [no-inner-declarations](./no-inner-declarations.md) | Disallow variable and function declarations in nested blocks. |
| [no-irregular-whitespace](./no-irregular-whitespace.md) | Disallow weird/irregular whitespace. |
| [no-new-symbol](./no-new-symbol.md) | Disallow constructing `Symbol` using `new`. |
| [no-prototype-builtins](./no-prototype-builtins.md) | Disallow direct use of `Object.prototype` builtins directly. |
| [no-setter-return](./no-setter-return.md) | Disallow setters to return values. |
| [no-sparse-arrays](./no-sparse-arrays.md) | Disallow sparse arrays. |
| [no-this-before-super](./no-this-before-super.md) | Prevent the use of `this` / `super` before calling `super()`. |
| [no-unexpected-multiline](./no-unexpected-multiline.md) | Disallow confusing newlines in expressions. |
| [no-unsafe-finally](./no-unsafe-finally.md) | Forbid the use of unsafe control flow statements in try and catch blocks. |
| [no-unsafe-negation](./no-unsafe-negation.md) | Deny the use of `!` on the left hand side of an `instanceof` or `in` expression where it is ambiguous. |
| [require-yield](./require-yield.md) | Disallow generator functions that do not have `yield`. |
| [use-isnan](./use-isnan.md) | Disallow incorrect comparisons against `NaN`. |
| [valid-typeof](./valid-typeof.md) | Enforce the use of valid string literals in a `typeof` comparison. |

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors)