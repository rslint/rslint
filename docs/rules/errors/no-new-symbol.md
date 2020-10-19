<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-new-symbol

Disallow constructing `Symbol` using `new`.

`Symbol` shouldn't be constructed using `new` keyword since it results in a `TypeError`, instead
it should be called as a function.

## Incorrect code examples

```js
// This call results in TypeError
const fooSymbol = new Symbol("foo"); 
```

## Correct code examples

```js
const fooSymbol = Symbol("foo");
```

<details>
 <summary> More incorrect examples </summary>

```js
new Symbol()
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
Symbol()
```

```js
new SomeClass()
```
</details>

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_new_symbol.rs)