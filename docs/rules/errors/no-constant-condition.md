<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-constant-condition

Disallow constant conditions which always yield one result.

Constant conditions such as `if (true) {}` are almost always a mistake. Constant
conditions always yield a single result which almost always ends up in unwanted behavior.
This rule is aimed at catching those conditions in `if`, `do while`, `while`, and `for` statements, as well as
conditional expressions.

## Incorrect Code Examples

```js
if (true) {
    //    ^ this block is always used
} else {
//^^^^ this else block is unreachable
}
```

```js
// This loop endlessly runs
for(foo = 5; 5; foo++) {

}
```

## Correct Code Examples

```js
if (foo) {
    /* */
}
```

<details>
 <summary> More incorrect examples </summary>

```js
if(6) {}
```

```js
if(6 - 7 || 3 ? 7 && 2 : NaN + NaN || 2) {}
```

```js
if (true) {}
```

```js
if (NaN) {} else {}
```

```js
6 + 2 ? false : NaN
```

```js
false ? false : false ? false : false
```

```js
while (true) {}
```

```js
do { /* */ } while (NaN ? NaN : true)
```

```js
do { } while (NaN ? Infinity : true)
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
if (foo) {}
```

```js
if (false > foo) {} else {}
```

```js
if (foo ? NaN : Infinity) {}
```

```js
do {} while (foo + 6)
```

```js
for(var i = 5; foo; i++) {}
```
</details>

[Source](../../../rslint_core/src/groups/errors/no_constant_condition.rs)