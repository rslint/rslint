<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-invalid-regexp

Disallow invalid regular expressions in literals and `RegExp` constructors.

Invalid regex patterns in `RegExp` constructors are not caught until runtime. This
rule checks for calls to `RegExp` and validates the pattern given. This also checks regex literals
for errors as RSLint's parser currently does not validate regex patterns.

## Incorrect Code Examples

```js
RegExp('[');

RegExp('a', 'h');

new RegExp('[')
```

::: details More incorrect examples

```js
RegExp('[')
```

```js
new RegExp('[')
```

```js
RegExp('a', 'h')
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/regex/no_invalid_regexp.rs)