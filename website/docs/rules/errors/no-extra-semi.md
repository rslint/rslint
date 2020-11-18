<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-extra-semi

Disallow unneeded semicolons.

Unneeded semicolons are often caused by typing mistakes, while this is not an error, it
can cause confusion when reading the code. This rule disallows empty statements (extra semicolons).

## Invalid Code Examples

```js
if (foo) {
    ;
}
```

```js
class Foo {
    constructor() {};
}
```

::: details More incorrect examples

```js
;
```

```js
if (foo) {
  ;
}
```

```js
class Foo {
  ;
}
```

```js
class Foo extends Bar {
  constructor() {};
}
```
:::
::: details More correct examples

```js
class Foo {}
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_extra_semi.rs)