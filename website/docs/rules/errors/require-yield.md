<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# require_yield

Disallow generator functions that do not have `yield`.

This rule generates warnings for generator functions that do not have the yield keyword.

## Invalid Code Examples

```js
function* foo(){
    return 10;
}
```


## Valid Code Examples

```js
function* foo(){
    yield 5;
    return 10;
}

// This rule does not warn on empty generator functions.
function* foo() { }
```

::: details More correct examples

```js
function foo() {
  return 10;
}
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/require_yield.rs)