<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# valid-typeof

Enforce the use of valid string literals in a `typeof` comparison.

## Invalid Code Examples
```js
typeof foo === "strnig"
typeof foo == "undefimed"
typeof bar != "nunber"
typeof bar !== "fucntion"
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `requireStringLiterals` | bool |  |

<details>
 <summary> More incorrect examples </summary>

```js
typeof foo === "strnig"
```

```js
typeof foo == "undefimed"
```

```js
typeof bar != "nunber"
```

```js
typeof bar !== "fucntion"
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
typeof foo === "string"
```

```js
typeof bar == "undefined"
```

```js
typeof foo === baz
```

```js
typeof foo === 4
```

```js
typeof bar === typeof qux
```
</details>

[Source](../../../rslint_core/src/groups/errors/valid_typeof.rs)