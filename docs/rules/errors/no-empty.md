<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-empty

Disallow empty block statements.

Block statements with nothing in them are very common when refactoring, however
they can get confusing really quickly. This rule reports empty block statements and empty switch
case blocks if they do not have a comment.

## Invalid Code Examples

```js
{}
```

```js
if (foo) {

}
```

## Correct Code Examples

```js
if (foo) {
    /* todo */
}
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `disallowEmptyFunctions` | bool |  Whether to disallow empty block statements in function declarations, arrow functions,<br>getters, setters, and methods. |
| `allowEmptyCatch` | bool |  Whether to allow empty `catch` clauses without a comment. |

<details>
 <summary> More incorrect examples </summary>

```js
{}
```

```js
if (foo) {}
```

```js
do { } while (scoot)
```

```js
for(let i = 5; i < 10; i++) {}
```

```js
switch (foo) {}
```

```js
switch (foo /* bar */) {}
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
{ /* sike you thought it was empty */ }
```

```js
{
// foo   
}
```

```js
if (foo) { /* */ }
```

```js
switch (bar) { /* */ }
```
</details>

[Source](../../../rslint_core/src/groups/errors/no_empty.rs)