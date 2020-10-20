<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-confusing-arrow

Disallow arrow functions where they could be confused with comparisons.

Arrow functions (`=>`) are similar in syntax to some comparison operators (`>`, `<`, `<=`, and `>=`).
This rule warns against using the arrow function syntax in places where it could be confused with
a comparison operator

Here's an example where the usage of `=>` could be confusing:

```js
// The intent is not clear
var x = a => 1 ? 2 : 3;
// Did the author mean this
var x = function (a) { return 1 ? 2 : 3 };
// Or this
var x = a >= 1 ? 2 : 3;
```

## Incorrect Code Examples

```js
var x = a => 1 ? 2 : 3;
var x = (a) => 1 ? 2 : 3;
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `allowParens` | bool |  Relaxes the rule and accepts parenthesis as a valid "confusion-preventing" syntax. |

<details>
 <summary> More incorrect examples </summary>

```js
a => 1 ? 2 : 3
```

```js
var x = a => 1 ? 2 : 3
```

```js
var x = (a) => 1 ? 2 : 3
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
a => { return 1 ? 2 : 3; }
```

```js
var x = a => { return 1 ? 2 : 3; }
```

```js
var x = (a) => { return 1 ? 2 : 3; }
```

```js
var x = a => (1 ? 2 : 3)
```
</details>

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_confusing_arrow.rs)