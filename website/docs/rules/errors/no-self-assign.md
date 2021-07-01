<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-self-assign

Disallow Self Assignment

Because self assignments have no effects, this is mostly an indicator for errors.

## Invalid Code Examples

```js
foo = foo;
```

```js
[a, b] = [a, b];
```

```js
[a, ...b] = [x, ...b];
```

```js
({a, b} = {a, x});
```

## Valid Code Examples

```js
foo = bar;
```

```js
[a, b] = [b, a];
```

```js
obj.a = obj.b;
```


::: details More incorrect examples

```js
[a, b, c] = [c, b, a]
```

```js
({b, a} = {a, b})
```
:::
::: details More correct examples

```js
let foo = foo
```

```js
[foo = 1] = [foo]
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_self_assign.rs)