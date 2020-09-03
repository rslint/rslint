<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->

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

[Source](../../rslint_core/src/groups/errors/no_await_in_loop.rs)