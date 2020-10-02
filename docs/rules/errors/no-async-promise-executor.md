<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-async-promise-executor

Disallow async functions as promise executors. 

Promise executors are special functions inside `new Promise()` constructors which take a `resolve` and 
`reject` parameter to resolve or reject the promise. The function is a normal function therefore it could be 
an async function. However this is usually wrong because: 
    - Any errors thrown by the function are lost.
    - It usually means the new promise is unnecessary. 

## Incorrect code examples 

```js
let foo = new Promise(async (resolve, reject) => {
    doSomething(bar, (err, res)) => {
       /* */ 
    });
});
```

```js
let foo = new Promise(async function(resolve, reject) => {
    /* */
});
```

## Correct code examples 

Use a normal non-async function. 

```js
let foo = new Promise(function(resolve, reject) => {
    /* */
})
```

<details>
 <summary> More incorrect examples </summary>

```js
new Promise(async () => {})
```

```js
new Promise(async function*() {})
```

```js
new Promise(async function() {}, foo)
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
new Promise(() => {})
```

```js
new Promise(function foo() {}, foo)
```
</details>

[Source](../../../rslint_core/src/groups/errors/no_async_promise_executor.rs)