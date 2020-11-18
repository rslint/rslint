<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-debugger

Disallow the use of debugger statements.

`debugger` statements are used to tell the environment executing the code to start an appropriate
debugger. These statements are rendered useless by modern IDEs which have built in breakpoint support.
Having them in production code is erroneous as it will tell the browser to stop running and open a debugger.

## Invalid Code Examples

```js
function doSomething() {
    debugger;
    doSomethingElse();
}
```

::: details More incorrect examples

```js
debugger
```

```js
debugger;
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_debugger.rs)