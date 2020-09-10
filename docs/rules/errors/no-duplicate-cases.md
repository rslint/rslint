<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-duplicate-cases

Disallow duplicate test cases in `switch` statements. 

`switch` statement clauses can freely have duplicate tests, however this is almost always a mistake, because
the second case is unreachable. It is likely that the programmer copied a case clause but did not change the test for it.

## Invalid Code Examples 

```js
switch (a) {
    case 1:
        break;
    case 2:
        break;
    case 1:
        break;
    default:
        break;
}
```

```js
switch (a) {
    case foo.bar:
        break;

    case foo . bar:
        break;
}
```

<details>
 <summary> More incorrect examples </summary>

```js
switch (foo) {
    case foo. bar:
    break;

    case foo.bar:
    break;
}
```

```js
switch foo {
    case 5:
    break;

    case 6:
    break;

    case 5:
    break;
}
```
</details>

[Source](../../../rslint_core/src/groups/errors/no_duplicate_cases.rs)