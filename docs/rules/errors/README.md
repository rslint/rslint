<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->

# Errors

Rules which relate to productions which are almost always erroneous or cause
unexpected behavior.
## Rules
| Name | Description |
| ---- | ----------- |
| [for-direction](./for-direction.md) | Disallow for loops which update their counter in the wrong direction. |
| [getter-return](./getter-return.md) | Disallow getter properties which do not always return a value. |
| [no-async-promise-executor](./no-async-promise-executor.md) | Disallow async functions as promise executors. |
| [no-await-in-loop](./no-await-in-loop.md) |  |
| [no-compare-neg-zero](./no-compare-neg-zero.md) | Disallow comparison against `-0` which yields unexpected behavior. |
| [no-cond-assign](./no-cond-assign.md) | Forbid the use of assignment expressions in conditions which may yield unwanted behavior. |
| [no-constant-condition](./no-constant-condition.md) | Disallow constant conditions which always yield one result. |
| [no-debugger](./no-debugger.md) | Disallow the use of debugger statements. |
| [no-unsafe-finally](./no-unsafe-finally.md) | Forbid the use of unsafe control flow statements in try and catch blocks. |
| [no-unsafe-negation](./no-unsafe-negation.md) | Deny the use of `!` on the left hand side of an `instanceof` or `in` expression where it is ambiguous. |

[Source](../../../rslint_core/src/groups/errors)