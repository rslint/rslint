<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-extra-boolean-cast

Disallow unnecessary boolean casts.

In contexts where expression will be coerced to a `Boolean` (e.g. `if`),
casting to a boolean (using `!!` or `Boolean(expr)`) is unnecessary.

## Invalid Code Examples

```js
if (!!foo) {}
while (!!foo) {}

var foo = !!!bar;
var foo = Boolean(!!bar);
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `enforceForLogicalOperands` | bool |  If this option is `true`, this rule will also check for unnecessary boolean<br>cast inside logical expression, which is disabled by default. |

::: details More incorrect examples

```js
if (!!foo) {}
```

```js
do {} while (!!foo)
```

```js
while (!!foo) {}
```

```js
!!foo ? bar : baz
```

```js
for (; !!foo;) {}
```

```js
!!!foo
```

```js
Boolean(!!foo)
```

```js
new Boolean(!!foo)
```

```js
if (Boolean(foo)) {}
```

```js
do {} while (Boolean(foo))
```

```js
while (Boolean(foo)) {}
```

```js
Boolean(foo) ? bar : baz
```

```js
for (; Boolean(foo);) {}
```

```js
!Boolean(foo)
```

```js
!Boolean(foo && bar)
```

```js
!Boolean(foo + bar)
```

```js
!Boolean(+foo)
```

```js
!Boolean(foo())
```

```js
!Boolean(foo = bar)
```

```js
!Boolean(...foo);
```

```js
!Boolean(foo, bar());
```

```js
!Boolean((foo, bar()));
```

```js
!Boolean();
```

```js
!(Boolean());
```

```js
if (!Boolean()) { foo() }
```

```js
while (!Boolean()) { foo() }
```

```js
if (Boolean()) { foo() }
```

```js
while (Boolean()) { foo() }
```

```js
Boolean(Boolean(foo))
```

```js
Boolean(!!foo, bar)
```
:::
::: details More correct examples

```js
Boolean(bar, !!baz);
```

```js
var foo = !!bar;
```

```js
function foo() { return !!bar; }
```

```js
var foo = bar() ? !!baz : !!bat
```

```js
for(!!foo;;) {}
```

```js
for(;; !!foo) {}
```

```js
var foo = Boolean(bar);
```

```js
function foo() { return Boolean(bar); }
```

```js
var foo = bar() ? Boolean(baz) : Boolean(bat)
```

```js
for(Boolean(foo);;) {}
```

```js
for(;; Boolean(foo)) {}
```

```js
if (new Boolean(foo)) {}
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_extra_boolean_cast.rs)