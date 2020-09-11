<!--
 generated docs file, do not edit by hand, see xtask/docgen 
-->
# getter-return

 
Disallow getter properties which do not always return a value. 

Getters are special properties introduced in ES5 which call a function when a property is accessed.
The value returned will be the value returned for the property access:

```js
let obj = {
    // Using object literal syntax
    get foo() {
        return 5;
    }
}

// Using the defineProperty function
Object.defineProperty(obj, "foo", {
    get: function() {
        return 5;
    }
})
```

Getters are expected to return a value, it is a bad practice to use getters to run some function
without a return. This rule makes sure that does not happen and enforces a getter always returns a value.

## Incorrect code examples 

```js
// The getter does not always return a value, it would not return anything
// if bar is falsey
let obj = {
    get foo() {
        if (bar) {
            return foo;
        }
    }
}
```

## Correct code examples 

```js
// The getter always returns a value
let obj = {
    get foo() {
        if (bar) {
            return foo;
        } else {
            return bar;
        }
    }
}
```

## Config
| Name | Type | Description |
| ---- | ---- | ----------- |
| `allowImplicit` | bool |  Whether to allow implicitly returning undefined with `return;`. <br>`true` by default.  |

<details>
 <summary> More incorrect examples </summary>

```js
let foo = {
    get bar() {
        
    }
}
```

```js
let bar = {
    get foo() {
        if (bar) {
            return bar;
        }
    }
}
```

```js
let bar = {
    get foo() {
        switch (bar) {
            case 5:
            case 6:
            if (bar) {
                return 5;
            }
        }
    }
}
```

```js
let bar = {
    get foo() {
        if (bar) {

        } else {
            return foo;
        }
    }
}
```
</details><br>
<details>
 <summary> More correct examples </summary>

```js
let bar = {
    get foo() {
        return bar;
    }
}
```

```js
let bar = {
    get foo() {
        if(bar) {
            if (bar) {
                return foo;
            } else {
                return 6;
            }
        } else {
            return 7;
        }
    }
}
```
</details>

[Source](../../../rslint_core/src/groups/errors/getter_return.rs)