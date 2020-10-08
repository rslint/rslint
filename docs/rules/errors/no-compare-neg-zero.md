<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-compare-neg-zero

Disallow comparison against `-0` which yields unexpected behavior.

Comparison against `-0` causes unwanted behavior because it passes for both `-0` and `+0`.
That is, `x == -0` and `x == +0` both pass under the same circumstances. If a user wishes
to compare against `-0` they should use `Object.is(x, -0)`.

## Incorrect Code Examples

```js
if (x === -0) {
       // ^^ this comparison works for both -0 and +0
}
```

## Correct code examples

```js
if (x === 0) {
    /* */
}
```

```js
if (Object.is(x, -0)) {
    /* */
}
```

<details>
 <summary> More incorrect examples </summary>

```js
x == -0
```

```js
x != -0
```

```js
x === -0
```

```js
-0 === -0
```

```js
-0 == x
```

```js
-0 >= 1
```

```js
x < -0
```

```js
x !== -0
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
x === 0
```

```js
0 === 0
```

```js
Object.is(x, -0)
```
</details>

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_compare_neg_zero.rs)