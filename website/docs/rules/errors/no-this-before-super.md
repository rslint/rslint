<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-this-before-super

Prevent the use of `this` / `super` before calling `super()`.

In the constructor of a derived class (`extends` a class), using `this` / `super` before the
`super()` call, will throw an error.

## Incorrect Code Examples

```js
class A extends B {
    constructor() {
        this.a = 0;
        super();
    }
}
```

```js
class A extends B {
    constructor() {
        this.foo();
        super();
    }
}
```

```js

class A extends B {
    constructor() {
        super.foo();
        super();
    }
}
```

```js
class A extends B {
    constructor() {
        super(this.foo());
    }
}
```

## Correct Code Examples


```js
class A {
    constructor() {
        this.a = 0; // OK, this class doesn't have an `extends` clause.
    }
}
```

```js
class A extends B {
    constructor() {
        super();
        this.a = 0; // OK, this is after `super()`.
    }
}
```

```js
class A extends B {
    foo() {
        this.a = 0; // OK. this is not in a constructor.
    }
}
```

::: details More incorrect examples

```js
class A extends B { constructor() { this.a = 0; super(); } }
```

```js
class A extends B { constructor() { this.foo(); super(); } }
```

```js
class A extends B { constructor() { super.foo(); super(); } }
```

```js
class A extends B { constructor() { super(this.foo()); } }
```
:::
::: details More correct examples

```js
class A { constructor() { this.a = 0; } }
```

```js
class A extends B { constructor() { super(); this.a = 0; } }
```

```js
class A extends B { foo() { this.a = 0; } }
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_this_before_super.rs)