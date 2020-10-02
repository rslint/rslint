<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-unsafe-finally

Forbid the use of unsafe control flow statements in try and catch blocks.

JavaScript suspends any running control flow statements inside of `try` and `catch` blocks until
`finally` is done executing. This means that any control statements such as `return`, `throw`, `break`,
and `continue` which are used inside of a `finally` will override any control statements in `try` and `catch`.
This is almost always unexpected behavior.

## Incorrect Code Examples

```js
// We expect 10 to be returned, but 5 is actually returned
function foo() {
    try {
        return 10;
    //  ^^^^^^^^^ this statement is executed, but actually returning is paused...
    } finally {
        return 5;
    //  ^^^^^^^^^ ...finally is executed, and this statement returns from the function, **the previous is ignored**
    }
}
foo() // 5
```

Throwing errors inside try statements

```js
// We expect an error to be thrown, then 5 to be returned, but the error is not thrown
function foo() {
    try {
        throw new Error("bar");
    //  ^^^^^^^^^^^^^^^^^^^^^^^ this statement is executed but throwing the error is paused...
    } finally {
        return 5;
    //  ^^^^^^^^^ ...we expect the error to be thrown and then for 5 to be returned,
    //  but 5 is returned early, **the error is not thrown**.
    }
}
foo() // 5
```

<details>
 <summary> More incorrect examples </summary>

```js
try {
    throw A;
} finally {
    return;
}
```

```js
try {
    throw new Error();
} catch {

} finally {
    continue;
}
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
try {
    throw A;
} finally {
    if (false) {
        return true;
    }
}
```
</details>

[Source](../../../rslint_core/src/groups/errors/no_unsafe_finally.rs)