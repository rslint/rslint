<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-inner-declarations

Disallow variable and function declarations in nested blocks.

Prior to ECMAScript 6, function declarations were only allowed in the first level of a program
or the body of another function, although parsers sometimes incorrectly accept it. This rule only applies to
function declarations, not function expressions.

## Invalid Code Examples

```js
function foo() {
    if (bar) {
        // Move this to foo's body, outside the if statement
        function bar() {}
    }
}
```

```js
if (bar) {
    var foo = 5;
}
```

## Correct Code Examples

```js
function foo() {}

var bar = 5;
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `disallowed` | Vec < String > |  What declarations to disallow in nested blocks, it can include two possible options:<br>"functions" and "variables", you can include either or, or both. Disallows only functions<br>by default. |

::: details More incorrect examples

```js
if (test) { function doSomething() { } }
```

```js
if (foo)  function f(){}
```

```js
function bar() { if (foo) function f(){}; }
```

```js
function doSomething() { do { function somethingElse() { } } while (test); }
```

```js
(function() { if (test) { function doSomething() { } } }());
```

```js
if (foo){ function f(){ if(bar){ var a; } } }
```

```js
if (foo) function f(){ if(bar) var a; }
```
:::
::: details More correct examples

```js
function doSomething() { }
```

```js
if (test) { let x = 1; }
```

```js
if (test) { const x = 1; }
```

```js
export const foo = [];
export function bar() {}
```
:::

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_inner_declarations.rs)