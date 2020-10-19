<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-prototype-builtins

Disallow direct use of `Object.prototype` builtins directly.

ES 5.1 added `Object.create` which allows creation of object with a custom prototype. This
pattern is frequently used for objects used as Maps. However this pattern can lead to errors
if something else relies on prototype properties/methods.

Moreover, the methods could be shadowed, this can lead to random bugs and denial of service
vulnerabilities. For example, calling `hasOwnProperty` directly on parsed json could lead to vulnerabilities.
Instead, you should use get the method directly from the object using `Object.prototype.prop.call(item, args)`.

## Invalid Code Examples

```js
var bar = foo.hasOwnProperty("bar");

var bar = foo.isPrototypeOf(bar);

var bar = foo.propertyIsEnumerable("bar");
```

## Correct Code Examples

```js
var bar = Object.prototype.hasOwnProperty.call(foo, "bar");

var bar = Object.prototype.isPrototypeOf.call(foo, bar);

var bar = Object.propertyIsEnumerable.call(foo, "bar");
```

<details>
 <summary> More incorrect examples </summary>

```js
foo.hasOwnProperty("bar");
```

```js
foo.isPrototypeOf("bar");
```

```js
foo.propertyIsEnumberable("bar");
```

```js
foo.bar.baz.hasOwnProperty("bar");
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
Object.prototype.hasOwnProperty.call(foo, 'bar');
```

```js
Object.prototype.isPrototypeOf.call(foo, 'bar');
```

```js
Object.prototype.propertyIsEnumberable.call(foo, 'bar');
```

```js
Object.prototype.hasOwnProperty.apply(foo, ['bar']);
```

```js
Object.prototype.isPrototypeOf.apply(foo, ['bar']);
```

```js
Object.prototype.propertyIsEnumberable.apply(foo, ['bar']);
```

```js
hasOwnProperty(foo, 'bar');
```

```js
isPrototypeOf(foo, 'bar');
```

```js
propertyIsEnumberable(foo, 'bar');
```

```js
({}.hasOwnProperty.call(foo, 'bar'));
```

```js
({}.isPrototypeOf.call(foo, 'bar'));
```

```js
({}.propertyIsEnumberable.call(foo, 'bar'));
```

```js
({}.hasOwnProperty.apply(foo, ['bar']));
```

```js
({}.isPrototypeOf.apply(foo, ['bar']));
```

```js
({}.propertyIsEnumberable.apply(foo, ['bar']));
```
</details>

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_prototype_builtins.rs)