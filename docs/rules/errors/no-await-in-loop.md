<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-await-in-loop

Disallow await inside of loops.

You may want to `await` a promise until it is fulfilled or rejected, inside of loops. In such cases, to take
full advantage of concurrency, you should __not__ `await` the promise in every iteration, otherwise your async
operations will be executed serially.
Generally it is recommended that you create all promises, then use `Promise.all` for them. This way your async
operations will be performed concurrently.

## Incorrect Code Exapmles

```js
async function foo(xs) {
    const results = [];
    for (const x of xs) {
        // iteration does not proceed until `bar(x)` completes
        results.push(await bar(x));
    }
    return baz(results);
}
```

## Correct Code Examples

```js
async function foo(xs) {
    const results = [];
    for (const x of xs) {
        // push a promise to the array; it does not prevent the iteration
        results.push(bar(x));
    }
    // we wait for all the promises concurrently
    return baz(await Promise.all(results));
}
```

<details>
 <summary> More incorrect examples </summary>

```js
async function foo() {
    const res = [];
    for(var i = 1; i < 20; i++) {
        res.push(await i);
    }
}
```

```js
async () => {
    while(true) {
        await i;
    }
}
```
</details>

[Source](../../../crates/rslint_core/src/groups/errors/no_await_in_loop.rs)