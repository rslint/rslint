<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
Forbid the use of assignment expressions in conditions which may yield unwanted behavior. 

Assignment expressions return the value assigned: 

```js
let foo = 5;

console.log(foo = 8); // 8
console.log(foo += 4) // foo + 4 (12 in this case)
```

Users often make a typo and end up using `=` instead of `==` or `===` in conditions in statements
like `if`, `while`, `do_while`, and `for`. This is erroneous and is most likely unwanted behavior
since the condition used will actually be the value assigned.

# Incorrect Code Examples

```js
let foo = 5;

if (foo = 6) {
//      ^^^ assignments return the value assigned, therefore the condition checks `6`
//          `6` is always truthy, therefore the if statement always runs even if we dont want it to.

} else {}
//^^^^ it makes this else unreachable

foo // 6
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `allow_parens` | bool |  Allow an assignment if they are enclosed in parentheses to allow
things like reassigning a variable. |

<details>
 <summary> More incorrect examples </summary>

```js
if (foo = 54) {}
```

```js
while (foo = 1) {}
```

```js
do { /* */ } while (bar = 1)
```

```js
for(;foo = 4; bar) {}
```

```js
if (bar = 5 ? foo : bar) {}
```
</details>

[`Source`](rslint_core/src/groups/errors/no-cond-assign)