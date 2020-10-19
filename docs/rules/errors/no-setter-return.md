<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# no-setter-return

Disallow setters to return values.

Setters cannot return values. To be more precise, a setter that returns a value is not treated as an error, but we
cannot use the returned value at all. Thus, if you write a setter that will return something, it is most likely
either completely unnecessary or a possible error.

Note that `return` without a value is allowed because it is considered a control flow statement.

This rule checks setters in:

- Object literals
- Class declarations and class expressions
- Property descriptors in `Object.create`, `Object.defineProperty`, `Object.defineProperties`, and `Reflect.defineProperty`

## Incorrect code examples

```js
let foo = {
    set a(value) {
        this.val = value;
        // The setter always returns a value
        return value;
    }
};

class Foo {
    set a(value) {
        this.val = value;
        // The setter always returns a value
        return this.val;
    }
}

const Bar = class {
    static set a(value) {
        if (value < 0) {
            this.val = 0;
            // The setter returns `0` if the value is negative
            return 0;
        }
        this.val = value;
    }
};

Object.defineProperty(foo, "bar", {
    set(value) {
        if (value < 0) {
            // The setter returns `false` if the value is negative
            return false;
        }
        this.val = value;
    }
});
```

## Correct code examples

```js
let foo = {
    set a(value) {
        this.val = value;
    }
};

class Foo {
    set a(value) {
        this.val = value;
    }
}

const Bar = class {
    static set a(value) {
        if (value < 0) {
            this.val = 0;
            // Returning without a value is allowed
            return;
        }
        this.val = value;
    }
};

Object.defineProperty(foo, "bar", {
    set(value) {
        if (value < 0) {
            // Throwing an error is also allowed
            throw new Error("Negative value is not allowed.");
        }
        this.val = value;
    }
});
```

<details>
 <summary> More incorrect examples </summary>

```js
let foo = {
    set bar(val) {
        return 42;
    }
};
```

```js
let bar = {
    set foo(val) {
        if (bar) {
            return 42;
        }
    }
};
```

```js
let bar = {
    set foo(val) {
        switch (bar) {
            case 5:
            case 6:
            if (bar) {
                return 42;
            }
        }
    }
};
```

```js
let bar = {
    set foo(val) {
        if (bar) {

        } else {
            return 42;
        }
    }
};
```

```js
class Foo {
    set bar(val) {
        return 42;
    }
}
```

```js
let Foo = class {
    set bar(val) {
        return 42;
    }
};
```

```js
Object.create(null, {
    foo: {
        set(val) {
            return 42;
        }
    }
});
```

```js
Object.defineProperty(foo, 'bar', {
    set(val) {
        return 42;
    }
});
```

```js
Object.defineProperties(foo, 'bar', {
    set(val) {
        return 42;
    }
});
```

```js
Reflect.defineProperties(foo, 'bar', {
    set(val) {
        return 42;
    }
});
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
({ set foo(val) { return; } })
```

```js
({ set foo(val) { if (val) { return; } } })
```

```js
class A { set foo(val) { return; } }
```

```js
(class { set foo(val) { if (val) { return; } else { return; } return; } })
```

```js
class A { set foo(val) { try {} catch(e) { return; } } }
```

```js
Object.defineProperty(foo, 'bar', { set(val) { return; } })
```
</details>

[Source](https://github.com/RDambrosio016/RSLint/tree/master/crates/rslint_core/src/groups/errors/no_setter_return.rs)