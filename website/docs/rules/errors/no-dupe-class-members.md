<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-dupe-class-members

Disallows defining a class method more than once, unless that method is overload in TypeScript.

If there are declarations of the same name in class members, the last declaration overwrites other declarations silently.
It can cause unexpected behaviors.

## Incorrect code examples

```js
class Foo {
    bar() { }
    bar() { }
}

class Foo {
    bar() { }
    get bar() { }
}

class Foo {
    static bar() { }
    static bar() { }
}

```

## Correct code examples

```js
class Foo {
    bar() { }
    qux() { }
}

class Foo {
    get bar() { }
    set bar(value) { }
}

class Foo {
    static bar() { }
    bar() { }
}

```

```ts
// note: this is valid because of method overloading in TypeScript
class Foo {
    foo(a: string): string;
    foo(a: number): number;
    foo(a: any): any {}
}
```

::: details More incorrect examples

```js
class A { get foo() {} get foo() {} }
```

```js
class A { foo() {} foo() {} }
```

```js
!class A { foo() {} foo() {} };
```

```js
class A { 'foo'() {} 'foo'() {} }
```
:::
::: details More correct examples

```js
class A { constructor() {} constructor() {} }
```

```js
class A { foo() {} bar() {} }
```

```js
class A { get foo() {} set foo(value) {} }
```

```js
class A { static foo() {} foo() {} }
```

```js
class A { static foo() {} get foo() {} set foo(value) {} }
```

```js
class A { foo() { } } class B { foo() { } }
```

```js
class A { 1() {} 2() {} }
```

```js
class A { [12]() {} [123]() {} }
```

```js
class A { [0x1]() {} [`0x1`]() {} }
```

```js
class A { [null]() {} ['']() {} }
```

```js
class Foo {
  foo(a: string): string;
  foo(a: number): number;
  foo(a: any): any {}
}
```
:::

[Source](https://github.com/rslint/rslint/tree/master/crates/rslint_core/src/groups/errors/no_dupe_class_members.rs)