<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-sparse-arrays

Disallow sparse arrays.

Sparse arrays are arrays with empty slots, they are denoted by extra commas, such as:

```js
let foo = [,,];
let foo = [bar,, baz];
```

Sparse elements will be filled in as undefined elements and count towards array length.
This is often a typo or is hard to comprehend and an explicit method should be used.

## Invalid Code Examples

```js
let foo = [,];
let bar = [foo,, bar];
```

::: details More incorrect examples

```js
[,]
```

```js
[...2,, 3]
```

```js
[4,,]
```
:::
::: details More correct examples

```js
[1, 2]
```

```js
[3,]
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_sparse_arrays.rs)