<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->

# Errors

Rules which relate to productions which are almost always erroneous or cause
unexpected behavior.
## Rules
| Name | Description |
| ---- | ----------- |
| [no-unsafe-negation](./no-unsafe-negation.md) | Deny the use of `!` on the left hand side of an `instanceof` or `in` expression where it is ambiguous. |
| [no-cond-assign](./no-cond-assign.md) | Forbid the use of assignment expressions in conditions which may yield unwanted behavior. |
| [no-unsafe-finally](./no-unsafe-finally.md) | Forbid the use of unsafe control flow statements in try and catch blocks. |
| [getter-return](./getter-return.md) | Disallow getter properties which do not always return a value. |
| [no-await-in-loop](./no-await-in-loop.md) |  |

[Source](../../../rslint_core/src/groups/errors