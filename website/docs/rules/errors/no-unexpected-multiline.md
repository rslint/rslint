<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-unexpected-multiline

Disallow confusing newlines in expressions.

JavaScript has automatic semicolon insertion, where newlines end statements, however,
expressions can often span across newlines, therefore it can become a bit confusing at times
and ambiguous. Take the following as an example:

```js
let foo = bar
/bar/g.test("foo");
```

you would expect this to be a variable declaration and then a regex test, however, it is actually
a division expression as such: `(bar / bar) / (g.test("foo")).
This rule is aimed at preventing ambiguous and buggy expressions such like these. It disallows
ambiguous tagged templates, property accesses, function calls, and division expressions.

## Invalid Code Examples

```js
var foo = bar
(1 || 2).baz();

var foo = 'bar'
[1, 2, 3].forEach(addNumber);

let x = function() {}
`foo`

let x = function() {}
x
`bar`

let x = foo
/regex/g.test(bar)
```

## Correct Code Examples

```js
var foo = bar;
(1 || 2).baz();

var foo = 'bar';
[1, 2, 3].forEach(addNumber);

let x = function() {};
`foo`

let x = function() {};
x;
`bar`

let x = foo;
/regex/g.test(bar)
```

::: details More incorrect examples

```js
var a = b
(x || y).doSomething()
```

```js
var a = (a || b)
(x || y).doSomething()
```

```js
var a = (a || b)
(x).doSomething()
```

```js
var a = b
[a, b, c].forEach(doSomething)
```

```js
var a = b
(x || y).doSomething()
```

```js
var a = b
[a, b, c].forEach(doSomething)
```

```js
let x = function() {}
`hello`
```

```js
let x = function() {}
x
`hello`
```

```js
x
.y
z
`Invalid Test Case`
```

```js
foo
/ bar /gym
```

```js
foo
/ bar /g
```

```js
foo
/ bar /g.test(baz)
```
:::
::: details More correct examples

```js
(x || y).aFunction()
```

```js
[a, b, c].forEach(doSomething)
```

```js
var a = b;
(x || y).doSomething()
```

```js
var a = b
;(x || y).doSomething()
```

```js
var a = b
void (x || y).doSomething()
```

```js
var a = b;
[1, 2, 3].forEach(console.log)
```

```js
var a = b
void [1, 2, 3].forEach(console.log)
```

```js
"abc\
(123)"
```

```js
var a = (
(123)
)
```

```js
f(
(x)
)
```

```js
(
function () {}
)[1]
```

```js
let x = function() {};
`hello`
```

```js
let x = function() {}
x `hello`
```

```js
String.raw `Hi
${2+3}!`;
```

```js
x
.y
z `Valid Test Case`
```

```js
f(x
)`Valid Test Case`
```

```js
x.
y `Valid Test Case`
```

```js
(x
)`Valid Test Case`
```

```js
foo
/ bar /2
```

```js
foo
/ bar / mgy
```

```js
foo
/ bar /
gym
```

```js
foo
/ bar
/ ygm
```

```js
foo
/ bar /GYM
```

```js
foo
/ bar / baz
```

```js
foo /bar/g
```

```js
foo
/denominator/
2
```

```js
foo
/ /abc/
```

```js
5 / (5
/ 5)
```

```js
var a = b
?.(x || y).doSomething()
```

```js
var a = b
?.[a, b, c].forEach(doSomething)
```
:::

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_unexpected_multiline.rs)