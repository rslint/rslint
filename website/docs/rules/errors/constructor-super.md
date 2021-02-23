<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# constructor-super

Verify calls of `super()` in constructors

Constructors of derived classes must call `super()`. Constructors of non derived classes must not call `super()`.
If this is not observed, the JavaScript engine will raise a runtime error.

This rule checks whether or not there is a valid `super()` call.

## Incorrect Code Examples

```js
class Foo {
    constructor() {
        super(); // SyntaxError because Foo does not extend any class.
    }
}
```

```js
class Foo extends Bar {
    constructor() {
        // we need to call Bar's constructor through `super()` but we haven't done that
    }
}
```

Classes extending a non-constructor are always an issue because we are required to call
the superclass' constructor, but `null` is not a constructor.

```js
class Foo extends null {
    constructor() {
        super(); // throws a TypeError because null is not a constructor
    }
}
```

```js
class Foo extends null {
    constructor() {
        // throws a ReferenceError
    }
}
```

## Correct Code Examples

```js
class Foo {
    constructor() {
        // this is fine because we don't extend anything
    }
}
```

```js
class Foo extends Bar {
    constructor() {
        super(); // this is fine because we extend a class and we call Bar's constructor through `super()`
    }
}
```

::: details More incorrect examples

```js
class A { constructor() { super(); } }
```

```js
class A extends B { constructor() { } }
```
:::
::: details More correct examples

```js
class A { constructor() { } }
```

```js
class A extends B { constructor() { super(); } }
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/constructor_super.rs)