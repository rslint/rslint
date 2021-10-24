<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-duplicate-imports

Disallow duplicate imports.

Multiple import statements with the same source can be combined to one statement. This improves readability.

## Incorrect Code Examples

```js
import { foo } from "bla";
import { bar } from "bla";

// including exports
export { foo } from "bla";
```

## Correct Code Examples

```js
import { foo, bar } from "bla";
export { foo };
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `includeExports` | bool |  Whether to check if re-exported |

::: details More incorrect examples

```js
import foo from "bla";
import * as bar from "bla";
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_duplicate_imports.rs)