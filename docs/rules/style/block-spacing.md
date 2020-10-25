<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# block-spacing

Enforce or disallow spaces inside of blocks after the opening and closing brackets.

This rule enforces consistent spacing inside blocks by enforcing the opening token and the next token
being on the same line. It also enforces consistent spacing with a closing token and the previous token being
on the same line.

## Always

### Incorrect code examples

```js
function foo() {return true;}
if (foo) { bar = 0;}
function baz() {let i = 0;
    return i;
}
```

### Correct code examples

```js
function foo() { return true; }
if (foo) { bar = 0; }
```

## Never

### Incorrect code examples

```js
function foo() { return true; }
if (foo) { bar = 0;}
```

### Correct code examples

```js
function foo() {return true;}
if (foo) {bar = 0;}
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `style` | String |  The style of spacing, either "always" (default) to require one or more spaces, or<br>"never" to disallow spaces |

<details>
 <summary> More incorrect examples </summary>

```js
{foo();}
```

```js
{foo();}
```

```js
{ foo();}
```

```js
{foo(); }
```

```js
{foo();
}
```

```js
if (a) {foo();}
```

```js
if (a) {} else {foo();}
```

```js
switch (a) {case 0: foo();}
```

```js
while (a) {foo();}
```

```js
do {foo();} while (a);
```

```js
for (;;) {foo();}
```

```js
for (var a in b) {foo();}
```

```js
for (var a of b) {foo();}
```

```js
try {foo();} catch (e) {foo();} finally {foo();}
```

```js
function foo() {bar();}
```

```js
(function() {bar();});
```

```js
(() => {bar();});
```

```js
if (a) {//comment
foo(); }
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
{ foo(); }
```

```js
{ foo();
}
```

```js
{
foo(); }
```

```js
{
foo();
}
```

```js
if (a) { foo(); }
```

```js
if (a) {} else { foo(); }
```

```js
switch (a) {}
```

```js
switch (a) { case 0: foo(); }
```

```js
while (a) { foo(); }
```

```js
do { foo(); } while (a);
```

```js
for (;;) { foo(); }
```

```js
for (var a in b) { foo(); }
```

```js
for (var a of b) { foo(); }
```

```js
try { foo(); } catch (e) { foo(); }
```

```js
function foo() { bar(); }
```

```js
(function() { bar(); });
```

```js
(() => { bar(); });
```

```js
if (a) { /* comment */ foo(); /* comment */ }
```

```js
if (a) { //comment
foo(); }
```
</details>

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/style/block_spacing.rs)