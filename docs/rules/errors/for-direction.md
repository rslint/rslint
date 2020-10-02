<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# for-direction

Disallow for loops which update their counter in the wrong direction.

A for loop with a counter may update its value in the wrong direction. that is to say, if i made
a counter with a value of `0`, if the for statement checked if `counter < 10` and the update went `counter--`,
that loop would be infinite. This is because `counter` will never be smaller than `10` because `counter--` always
yields a value smaller than 10. A for loop which does this is almost always a bug because it is either
unreachable or infinite.

## Incorrect Code Examples

```js
for (var i = 0; i < 10; i--) {
    /* infinite loop */
}
```

```js
for (var i = 10; i >= 20; i++) {
    /* unreachable */
}
```

## Correct Code Examples

```js
for (var i = 0; i < 10; i++) {

}
```

<details>
 <summary> More incorrect examples </summary>

```js
for (var i = 0; i < 10; i--) {}
```

```js
for(let i = 0; i < 2; i--) {}
```

```js
for(let i = 0; i <= 2; i += -1) {}
```

```js
for(let i = 2; i >= 0; i -= -1) {}
```

```js
for(let i = 0; i < 2; i -= 1) {}
```

```js
for(let i = 2; i > 2; i++) {}
```

```js
for(let i = 2; i > 2; i += 1) {}
```

```js
for(let i = 5n; i < 2; i--) {}
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
for (var i = 0; i < 10; i++) {}
```

```js
for(let i = 2; i > 2; i -= 1) {}
```

```js
for(let i = 2; i >= 0; i -= 1) {}
```

```js
for(let i = 2; i > 2; i += -1) {}
```

```js
for(let i = 2; i >= 0; i += -1) {}
```

```js
for(let i = 0; i < 3;) {}
```

```js
for(let i = 5; i < 2; i |= 2) {}
```

```js
for(let i = 5n; i < 2n; i &= 2) {}
```
</details>

[Source](../../../rslint_core/src/groups/errors/for_direction.rs)