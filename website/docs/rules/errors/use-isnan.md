<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# use-isnan

    Disallow incorrect comparisons against `NaN`.

    `NaN` is a special `Number` value used to represent "not a number" results in calculations.
    This value is specified in the IEEE Standard for Binary Floating-Point-Arithmetic.

    In JavaScript, `NaN` is unique, it is not equal to anything, including itself! therefore
    any comparisons to it will either always yield `true` or `false`. Therefore you should
    use `isNaN(/* num */)` instead to test if a value is `NaN`. This rule is aimed at removing this footgun.

    ## Invalid Code Examples

    ```js
    if (foo == NaN) {
        // unreachable
    }

    if (NaN != NaN) {
        // always runs
    }
    ```

    ## Correct Code Examples

    ```js
    if (isNaN(foo)) {
        /* */
    }

    if (!isNaN(foo)) {
        /* */
    }
    ```
    
## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `enforceForSwitchCase` | bool |  Switch statements use `===` internally to match an expression, therefore `switch (NaN)` and `case NaN` will never match.<br>This rule disables uses like that which are always incorrect (true by default) |
| `enforceForIndexOf` | bool |  Index functions like `indexOf` and `lastIndexOf` use `===` internally, therefore matching them against `NaN` will always<br>yield `-1`. This option disallows using `indexOf(NaN)` and `lastIndexOf(NaN)` (false by default) |

::: details More incorrect examples

```js
123 == NaN;
```

```js
123 === NaN;
```

```js
NaN === "abc";
```

```js
NaN == "abc";
```

```js
123 != NaN;
```

```js
123 !== NaN;
```

```js
NaN !== "abc";
```

```js
NaN != "abc";
```

```js
NaN < "abc";
```

```js
"abc" < NaN;
```

```js
NaN > "abc";
```

```js
"abc" > NaN;
```

```js
NaN <= "abc";
```

```js
"abc" <= NaN;
```

```js
NaN >= "abc";
```

```js
"abc" >= NaN;
```
:::
::: details More correct examples

```js
var x = NaN;
```

```js
isNaN(NaN) === true;
```

```js
isNaN(123) !== true;
```

```js
Number.isNaN(NaN) === true;
```

```js
Number.isNaN(123) !== true;
```

```js
foo(NaN + 1);
```

```js
foo(1 + NaN);
```

```js
foo(NaN - 1)
```

```js
foo(1 - NaN)
```

```js
foo(NaN * 2)
```

```js
foo(2 * NaN)
```

```js
foo(NaN / 2)
```

```js
foo(2 / NaN)
```

```js
var x; if (x = NaN) { }
```

```js
foo.indexOf(NaN)
```

```js
foo.lastIndexOf(NaN)
```
:::

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/use_isnan.rs)