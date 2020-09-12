<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-unsafe-negation

Deny the use of `!` on the left hand side of an `instanceof` or `in` expression where it is ambiguous.

JavaScript precedence is higher for logical not than it is for in or instanceof. Oftentimes you see
expressions such as `!foo instanceof bar`, which most of the times produces unexpected behavior. 
precedence will group the expressions like `(!foo) instanceof bar`. Most of the times the developer expects
the expression to check if `foo` is not an instance of `bar` however.

## Incorrect Code Examples

```js
if (!foo instanceof String) {

}
```

```js
if (!bar in {}) {

}
```

<details>
 <summary> More incorrect examples </summary>

```js
!foo in bar
```

```js
![5] instanceof !4
```
</details><br>
<details>
 <summary> More correct examples </summary>
 If this is intended behavior, you can wrap the expression
```js
(!foo) instanceof bar
```

```js
key in bar
```

```js
bar instanceof bar
```
</details>

[Source](../../../rslint_core/src/groups/errors/no_unsafe_negation.rs)