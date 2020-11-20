rule_test! {
    no_unused_vars,
    rule_conf: |conf| conf.no_unused_vars(true),
    filter: DatalogLint::is_no_unused_vars,
    // Should pass
    { "var foo = 5;\nlabel: while (true) {\n  console.log(foo);\n  break label;\n}", node: true },
    { "var foo = 5;\nwhile (true) {\n  console.log(foo);\n  break;\n}", node: true },
    { "let box;\nfor (let prop in box) {\n  box[prop] = parseInt(box[prop]);\n}" },
    { "var box = { a: 2 };\nfor (var prop in box) {\n  box[prop] = parseInt(box[prop]);\n}" },
    { "a;\nvar a;" },
    { "var a = 10;\nalert(a);" },
    { "var a = 10;\n(function() { alert(a); })();" },
    { "var a = 10;\n(function() { setTimeout(function() { alert(a); }, 0); })();", browser: true },
    { "var a = 10;\nd[a] = 0;" },
    { "(function() { var a = 10; return a; })();" },
    { "function f(a) { alert(a); };\nf();", browser: true },
    { "var c = 0;\nfunction f(a) { var b = a; return b; };\nf(c);" },
    {
        "var arr1 = [1, 2];",
        "var arr2 = [3, 4];",
        "for (var i in arr1) {",
        "    arr1[i] = 5;",
        "}",
        "for (var i in arr2) {",
        "    arr2[i] = 10;",
        "}",
    },
    { "var min = \"min\";\nMath[min];" },
    { "Foo.bar = function(baz) { return baz; };" },
    { "myFunc(function foo() {}.bind(this))" },
    { "myFunc(function foo() {}.toString())" },
    { "(function() { var doSomething = function doSomething() {}; doSomething() }())" },
    { "a;", globals: ["a"] },
    { "var a=10; (function() { alert(a); })();", browser: true },
    { "function g(bar, baz) { return bar + baz; }; g();" },
    { "(function z() { z(); })();" },
    { " ", globals: ["a"] },
    { "var who = \"Paul\";\nmodule.exports = `Hello ${who}!`;", node: true },
    { "export var foo = 123;", module: true },
    { "export function foo () {}", module: true },
    { "let toUpper = (partial) => partial.toUpperCase;\nexport { toUpper }", module: true },
    { "export class foo {}", module: true },
    { "class Foo {};\nvar x = new Foo();\nx.foo();" },
    {
        "const foo = \"hello!\";",
        "function bar(foobar = foo) {",
        "    foobar.replace(/!$/, \" world!\");",
        "}",
        "bar();",
    },
    { "function Foo() {};\nvar x = new Foo();\nx.foo();" },
    { "function foo() {\n  var foo = 1;\n  return foo\n};\nfoo();" },
    { "function foo(foo) { return foo };\nfoo(1);" },
    { "function foo() { function foo() { return 1; }; return foo() }; foo();" },
    { "function foo() { var foo = 1; return foo }; foo();" },
    { "function foo(foo) { return foo }; foo(1);" },
    { "function foo() { function foo() { return 1; }; return foo() }; foo();" },
    { "const x = 1; const [y = x] = []; foo(y);" },
    { "const x = 1; const {y = x} = {}; foo(y);" },
    { "const x = 1; const {z: [y = x]} = {}; foo(y);" },
    { "const x = []; const {z: [y] = x} = {}; foo(y);" },
    { "const x = 1; let y; [y = x] = []; foo(y);" },
    { "const x = 1; let y; ({z: [y = x]} = {}); foo(y);" },
    { "const x = []; let y; ({z: [y] = x} = {}); foo(y);" },
    { "const x = 1; function foo(y = x) { bar(y); } foo();" },
    { "const x = 1; function foo({y = x} = {}) { bar(y); } foo();" },
    { "const x = 1; function foo(y = function(z = x) { bar(z); }) { y(); } foo();" },
    { "const x = 1; function foo(y = function() { bar(x); }) { y(); } foo();" },
    { "var x = 1; var [y = x] = []; foo(y);" },
    { "var x = 1; var {y = x} = {}; foo(y);" },
    { "var x = 1; var {z: [y = x]} = {}; foo(y);" },
    { "var x = []; var {z: [y] = x} = {}; foo(y);" },
    { "var x = 1, y; [y = x] = []; foo(y);" },
    { "var x = 1, y; ({z: [y = x]} = {}); foo(y);" },
    { "var x = [], y; ({z: [y] = x} = {}); foo(y);" },
    { "var x = 1; function foo(y = x) { bar(y); } foo();" },
    { "var x = 1; function foo({y = x} = {}) { bar(y); } foo();" },
    { "var x = 1; function foo(y = function(z = x) { bar(z); }) { y(); } foo();" },
    { "var x = 1; function foo(y = function() { bar(x); }) { y(); } foo();" },
    { "export { x };\nvar { x } = y;", module: true },
    { "var { x } = y;\nexport { x };", module: true },
    { "export { x, y };\nvar { x, y } = z;", module: true },
    { "var { x, y } = z;\nexport { x, y };", module: true },
    { "try {} catch(err) { console.error(err); }" },
    { "var a = 0, b;\nb = a = a + 1;\nfoo(b);" },
    { "var a = 0, b;\nb = a += a + 1;\nfoo(b);" },
    { "var a = 0, b;\nb = a++;\nfoo(b);" },
    { "function foo(a) {\n  var b = a = a + 1;\n  bar(b)\n}\nfoo();" },
    { "function foo(a) {\n  var b = a += a + 1;\n  bar(b)\n}\nfoo();" },
    { "function foo(a) {\n  var b = a++;\n  bar(b)\n}\nfoo();" },
    { "(function(obj) { var name; for ( name in obj ) return; })({});" },
    { "(function(obj) { var name; for ( name in obj ) { return; } })({});" },
    { "(function(obj) { let name; for ( name in obj ) return; })({});" },
    { "(function(obj) { let name; for ( name in obj ) { return; } })({});" },
    {
        "var unregisterFooWatcher;",
        "unregisterFooWatcher = $scope.$watch( \"foo\", function() {",
        "    unregisterFooWatcher();",
        "});",
    },
    {
        "var ref;",
        "ref = setInterval(",
        "    function() {",
        "        clearInterval(ref);",
        "    },",
        "    10,",
        ");",
    },
    {
        "var _timer;",
        "function f() {",
        "    _timer = setTimeout(function () {}, _timer ? 100 : 0);",
        "}",
        "f();",
    },
    {
        "function foo(cb) {",
        "    cb = function() {",
        "        function something(a) {",
        "            cb(1 + a);",
        "        }",
        "        register(something);",
        "    }();",
        "}",
        "foo();",
    },
    {
        "function* foo(cb) {",
        "    cb = yield function(a) { cb(1 + a); };",
        "}",
        "foo();",
    },
    {
        "function foo(cb) {",
        "    cb = tag`hello${function(a) { cb(1 + a); }}`;",
        "}",
        "foo();",
    },
    {
        "function foo(cb) {",
        "    var b;",
        "    cb = b = function(a) {cb(1 + a); };",
        "    b();",
        "}",
        "foo();",
    },
    {
        "function someFunction() {",
        "    var a = 0, i;",
        "    for (i = 0; i < 2; i++) {",
        "        a = myFunction(a);",
        "    }",
        "}",
        "someFunction();",
    },
    { "var a = function () { a(); }; a();" },
    { "var a = function(){ return function () { a(); } }; a();" },
    { "const a = () => { a(); }; a();" },
    { "const a = () => () => { a(); }; a();" },
    {
        "const obj = {",
        "   set latest(foo, bar) {",
        "       this.foo = foo;",
        "       this.bar = bar;",
        "   }",
        "};",
        "foo(obj);",
    },
    {
        "const keys = Q(object.OwnPropertyKeys());",
        "// ii. for each key of keys in List order, do",
        "for (const key of keys) {",
        "    // 1. If Type(key) is String, then",
        "    if (Type(key) === 'String') {",
        "        // a. Append key to remaining.",
        "        remaining.push(key);",
        "    }",
        "}",
    },

    // Should fail
    { "f({ set foo(a) { return; } });", errors: [DatalogLint::no_unused_vars("a", 12..13)] },
    { "function a(x, y){ return y; }; a();", errors: [DatalogLint::no_unused_vars("x", 11..12)] },
    { "var a = 10;", errors: [DatalogLint::no_unused_vars("a", 4..5)] },
    { "function g(bar, baz) { return baz; }; g();", errors: [DatalogLint::no_unused_vars("bar", 11..14)] },
    {
        "function g(bar, baz) { return 2; }; g();",
        errors: [
            DatalogLint::no_unused_vars("bar", 11..14),
            DatalogLint::no_unused_vars("baz", 16..19),
        ],
    },
    { "try {} catch(e) {}", errors: [DatalogLint::no_unused_vars("e", 13..14)] },
    {
        "function f(a) {",
        "    f({",
        "        set foo(a) {",
        "            return;",
        "        }",
        "    });",
        "}",
        errors: [
            DatalogLint::no_unused_vars("a", 11..12),
            DatalogLint::no_unused_vars("a", 40..41),
        ],
    },
    {
        "function doStuff(f) {",
        "    f()",
        "}",
        "function foo(first, second) {",
        "    doStuff(function() {",
        "        console.log(second);",
        "    });",
        "};",
        "foo()",
        node: true,
        errors: [DatalogLint::no_unused_vars("first", 45..50)],
    },
    {
        "(function(obj) { for ( let name in obj ) { return true } })({})",
        errors: [DatalogLint::no_unused_vars("name", 27..31)],
    },
    {
        "(function(obj) { for ( let name in obj ) return true })({})",
        errors: [DatalogLint::no_unused_vars("name", 27..31)],
    },
    {
        "(function(obj) { for ( const name in obj ) { return true } })({})",
        errors: [DatalogLint::no_unused_vars("name", 29..33)],
    },
    {
        "(function(obj) { for ( const name in obj ) return true })({})",
        errors: [DatalogLint::no_unused_vars("name", 29..33)],
    },
    {
        "(function(obj) { for ( var name in obj ) { return true } })({})",
        errors: [DatalogLint::no_unused_vars("name", 27..31)],
    },
    {
        "(function(obj) { for ( var name in obj ) return true })({})",
        errors: [DatalogLint::no_unused_vars("name", 27..31)],
    },
    { "var a = 10", errors: [DatalogLint::no_unused_vars("a", 4..5)] },
    {
        "function foo(first, second) {",
        "    doStuff(function() {",
        "        console.log(second);",
        "    });",
        "}",
        errors: [
            DatalogLint::no_unused_vars("foo", 9..12),
            DatalogLint::no_unused_vars("first", 13..18),
        ],
    },
    {
        "var a = 10, b = 0, c = null;",
        "alert(a + b)",
        errors: [DatalogLint::no_unused_vars("c", 19..20)],
    },
    {
        "function f() {",
        "    var a = [];",
        "    return a.map(function() {});",
        "}",
        errors: [DatalogLint::no_unused_vars("f", 9..10)],
    },
    {
        "function f() {",
        "    var a = [];",
        "    return a.map(function g() {});",
        "}",
        errors: [DatalogLint::no_unused_vars("f", 9..10)],
    },
    {
        "const obj = {",
        "   set latest(foo, bar) {}",
        "};",
        "foo(obj);",
        errors: [
            DatalogLint::no_unused_vars("foo", 28..31),
            DatalogLint::no_unused_vars("bar", 33..36),
        ],
    },
    {
        "const obj = {",
        "   set latest(foo, bar) {",
        "       this.foo = foo;",
        "   }",
        "};",
        "foo(obj);",
        errors: [DatalogLint::no_unused_vars("bar", 33..36)],
    },
    {
        "function f() {",
        "    var x;",
        "    function a() {",
        "        x = 42;",
        "    }",
        "    function b() {",
        "        alert(x);",
        "    }",
        "}",
        errors: [
            DatalogLint::no_unused_vars("f", 9..10),
            DatalogLint::no_unused_vars("a", 39..40),
            DatalogLint::no_unused_vars("b", 80..81),
        ],
    },
    { "function f(a) {}; f();", errors: [DatalogLint::no_unused_vars("a", 11..12)] },
    {
        "function a(x, y, z){ return y; }; a();",
        errors: [
            DatalogLint::no_unused_vars("x", 11..12),
            DatalogLint::no_unused_vars("z", 17..18),
        ],
    },
    { "var min = Math.min", errors: [DatalogLint::no_unused_vars("min", 4..7)] },
    { "var min = {min: 1}", errors: [DatalogLint::no_unused_vars("min", 4..7)] },
    {
        "Foo.bar = function(baz) { return 1; };",
        errors: [DatalogLint::no_unused_vars("baz", 19..22)],
    },
    { "var min = {min: 1}", errors: [DatalogLint::no_unused_vars("min", 4..7)] },
    {
        "function gg(baz, bar) { return baz; }; gg();",
        errors: [DatalogLint::no_unused_vars("bar", 17..20)],
    },
    {
        "(function(foo, baz, bar) { return baz; })();",
        errors: [
            DatalogLint::no_unused_vars("foo", 10..13),
            DatalogLint::no_unused_vars("bar", 20..23),
        ],
    },
    {
        "(function z(foo) { var bar = 33; })();",
        errors: [
            DatalogLint::no_unused_vars("foo", 12..15),
            DatalogLint::no_unused_vars("bar", 23..26),
        ],
    },
    {
        "(function z(foo) { z(); })();",
        errors: [DatalogLint::no_unused_vars("foo", 12..15)],
    },
    { "function f() { var a = 1; return function(){ f(a = 2); }; }", errors: [] },
    {
        "import x from \"y\";",
        module: true,
        errors: [DatalogLint::no_unused_vars("x", 7..8)],
    },
    {
        "export function fn2({ x, y }) {",
        "    console.log(x);",
        "};",
        module: true,
        errors: [DatalogLint::no_unused_vars("y", 25..26)],
    },
    {
        "export function fn2(x, y) {",
        "    console.log(x);",
        "};",
        module: true,
        errors: [DatalogLint::no_unused_vars("y", 23..24)],
    },

}

/*
ruleTester.run("no-unused-vars", rule, {
    valid: [
        // Using object rest for variable omission
        {
            "const data = { type: 'coords', x: 1, y: 2 };\nconst { type, ...coords } = data;\n console.log(coords);",
            options: [{ ignoreRestSiblings: true }],
            parserOptions: { ecmaVersion: 2018 }
        },

        // https://github.com/eslint/eslint/issues/7124
        {
            "(function(a, b, {c, d}) { d })",
            options: [{ argsIgnorePattern: "c" }],
            parserOptions: { ecmaVersion: 6 }
        },
        {
            "(function(a, b, {c, d}) { c })",
            options: [{ argsIgnorePattern: "d" }],
            parserOptions: { ecmaVersion: 6 }
        },

        // https://github.com/eslint/eslint/issues/7250
        {
            "(function(a, b, c) { c })",
            options: [{ argsIgnorePattern: "c" }]
        },
        {
            "(function(a, b, {c, d}) { c })",
            options: [{ argsIgnorePattern: "[cd]" }],
            parserOptions: { ecmaVersion: 6 }
        },

        // https://github.com/eslint/eslint/issues/7351
        {
            "(class { set foo(UNUSED) {} })",
            parserOptions: { ecmaVersion: 6 }
        },
        {
            "class Foo { set bar(UNUSED) {} } console.log(Foo)",
            parserOptions: { ecmaVersion: 6 }
        },

        // https://github.com/eslint/eslint/issues/8119
        {
            "(({a, ...rest}) => rest)",
            options: [{ args: "all", ignoreRestSiblings: true }],
            parserOptions: { ecmaVersion: 2018 }
        },

        // https://github.com/eslint/eslint/issues/10952
        "/*eslint use-every-a:1*/ !function(b, a) { return 1 }",

        // export * as ns from "source"
        {
            'export * as ns from "source"',
            parserOptions: { ecmaVersion: 2020, sourceType: "module" }
        },

        // import.meta
        {
            "import.meta",
            parserOptions: { ecmaVersion: 2020, sourceType: "module" }
        }
    ],
    invalid: [
        // exported
        { "/*exported max*/ var max = 1, min = {min: 1}", errors: [assignedError("min")] },
        { "/*exported x*/ var { x, y } = z", errors: [assignedError("y")] },

        // ignore pattern
        {
            "var _a; var b;",
            options: [{ vars: "all", varsIgnorePattern: "^_" }],
            errors: [{
                line: 1,
                column: 13,
                messageId: "unusedVar",
                data: {
                    varName: "b",
                    action: "defined",
                    additional: ". Allowed unused vars must match /^_/u"
                }
            }]
        },
        {
            "var a; function foo() { var _b; var c_; } foo();",
            options: [{ vars: "local", varsIgnorePattern: "^_" }],
            errors: [{
                line: 1,
                column: 37,
                messageId: "unusedVar",
                data: {
                    varName: "c_",
                    action: "defined",
                    additional: ". Allowed unused vars must match /^_/u"
                }
            }]
        },
        {
            "function foo(a, _b) { } foo();",
            options: [{ args: "all", argsIgnorePattern: "^_" }],
            errors: [{
                line: 1,
                column: 14,
                messageId: "unusedVar",
                data: {
                    varName: "a",
                    action: "defined",
                    additional: ". Allowed unused args must match /^_/u"
                }
            }]
        },
        {
            "function foo(a, _b, c) { return a; } foo();",
            options: [{ args: "after-used", argsIgnorePattern: "^_" }],
            errors: [{
                line: 1,
                column: 21,
                messageId: "unusedVar",
                data: {
                    varName: "c",
                    action: "defined",
                    additional: ". Allowed unused args must match /^_/u"
                }
            }]
        },
        {
            "function foo(_a) { } foo();",
            options: [{ args: "all", argsIgnorePattern: "[iI]gnored" }],
            errors: [{
                line: 1,
                column: 14,
                messageId: "unusedVar",
                data: {
                    varName: "_a",
                    action: "defined",
                    additional: ". Allowed unused args must match /[iI]gnored/u"
                }
            }]
        },
        {
            "var [ firstItemIgnored, secondItem ] = items;",
            options: [{ vars: "all", varsIgnorePattern: "[iI]gnored" }],
            parserOptions: { ecmaVersion: 6 },
            errors: [{
                line: 1,
                column: 25,
                messageId: "unusedVar",
                data: {
                    varName: "secondItem",
                    action: "assigned a value",
                    additional: ". Allowed unused vars must match /[iI]gnored/u"
                }
            }]
        },

        // for-in loops (see #2342)
        {
            "(function(obj) { var name; for ( name in obj ) { i(); return; } })({});",
            errors: [{
                line: 1,
                column: 34,
                messageId: "unusedVar",
                data: {
                    varName: "name",
                    action: "assigned a value",
                    additional: ""
                }
            }]
        },
        {
            "(function(obj) { var name; for ( name in obj ) { } })({});",
            errors: [{
                line: 1,
                column: 34,
                messageId: "unusedVar",
                data: {
                    varName: "name",
                    action: "assigned a value",
                    additional: ""
                }
            }]
        },
        {
            "(function(obj) { for ( var name in obj ) { } })({});",
            errors: [{
                line: 1,
                column: 28,
                messageId: "unusedVar",
                data: {
                    varName: "name",
                    action: "assigned a value",
                    additional: ""
                }
            }]
        },

        // https://github.com/eslint/eslint/issues/3617
        {
            "\n/* global foobar, foo, bar */\nfoobar;",
            errors: [
                {
                    line: 2,
                    endLine: 2,
                    column: 19,
                    endColumn: 22,
                    messageId: "unusedVar",
                    data: {
                        varName: "foo",
                        action: "defined",
                        additional: ""
                    }
                },
                {
                    line: 2,
                    endLine: 2,
                    column: 24,
                    endColumn: 27,
                    messageId: "unusedVar",
                    data: {
                        varName: "bar",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },
        {
            "\n/* global foobar,\n   foo,\n   bar\n */\nfoobar;",
            errors: [
                {
                    line: 3,
                    column: 4,
                    endLine: 3,
                    endColumn: 7,
                    messageId: "unusedVar",
                    data: {
                        varName: "foo",
                        action: "defined",
                        additional: ""
                    }
                },
                {
                    line: 4,
                    column: 4,
                    endLine: 4,
                    endColumn: 7,
                    messageId: "unusedVar",
                    data: {
                        varName: "bar",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },

        // Rest property sibling without ignoreRestSiblings
        {
            "const data = { type: 'coords', x: 1, y: 2 };\nconst { type, ...coords } = data;\n console.log(coords);",
            parserOptions: { ecmaVersion: 2018 },
            errors: [
                {
                    line: 2,
                    column: 9,
                    messageId: "unusedVar",
                    data: {
                        varName: "type",
                        action: "assigned a value",
                        additional: ""
                    }
                }
            ]
        },

        // Unused rest property with ignoreRestSiblings
        {
            "const data = { type: 'coords', x: 2, y: 2 };\nconst { type, ...coords } = data;\n console.log(type)",
            options: [{ ignoreRestSiblings: true }],
            parserOptions: { ecmaVersion: 2018 },
            errors: [
                {
                    line: 2,
                    column: 18,
                    messageId: "unusedVar",
                    data: {
                        varName: "coords",
                        action: "assigned a value",
                        additional: ""
                    }
                }
            ]
        },

        // Unused rest property without ignoreRestSiblings
        {
            "const data = { type: 'coords', x: 3, y: 2 };\nconst { type, ...coords } = data;\n console.log(type)",
            parserOptions: { ecmaVersion: 2018 },
            errors: [
                {
                    line: 2,
                    column: 18,
                    messageId: "unusedVar",
                    data: {
                        varName: "coords",
                        action: "assigned a value",
                        additional: ""
                    }
                }
            ]
        },

        // Nested array destructuring with rest property
        {
            "const data = { vars: ['x','y'], x: 1, y: 2 };\nconst { vars: [x], ...coords } = data;\n console.log(coords)",
            parserOptions: { ecmaVersion: 2018 },
            errors: [
                {
                    line: 2,
                    column: 16,
                    messageId: "unusedVar",
                    data: {
                        varName: "x",
                        action: "assigned a value",
                        additional: ""
                    }
                }
            ]
        },

        // Nested object destructuring with rest property
        {
            "const data = { defaults: { x: 0 }, x: 1, y: 2 };\nconst { defaults: { x }, ...coords } = data;\n console.log(coords)",
            parserOptions: { ecmaVersion: 2018 },
            errors: [
                {
                    line: 2,
                    column: 21,
                    messageId: "unusedVar",
                    data: {
                        varName: "x",
                        action: "assigned a value",
                        additional: ""
                    }
                }
            ]
        },

        // https://github.com/eslint/eslint/issues/8119
        {
            "(({a, ...rest}) => {})",
            options: [{ args: "all", ignoreRestSiblings: true }],
            parserOptions: { ecmaVersion: 2018 },
            errors: [definedError("rest")]
        },

        // https://github.com/eslint/eslint/issues/3714
        {
            "/* global a$fooz,$foo */\na$fooz;",
            errors: [
                {
                    line: 1,
                    column: 18,
                    endLine: 1,
                    endColumn: 22,
                    messageId: "unusedVar",
                    data: {
                        varName: "$foo",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },
        {
            "/* globals a$fooz, $ */\na$fooz;",
            errors: [
                {
                    line: 1,
                    column: 20,
                    endLine: 1,
                    endColumn: 21,
                    messageId: "unusedVar",
                    data: {
                        varName: "$",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },
        {
            "/*globals $foo*/",
            errors: [
                {
                    line: 1,
                    column: 11,
                    endLine: 1,
                    endColumn: 15,
                    messageId: "unusedVar",
                    data: {
                        varName: "$foo",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },
        {
            "/* global global*/",
            errors: [
                {
                    line: 1,
                    column: 11,
                    endLine: 1,
                    endColumn: 17,
                    messageId: "unusedVar",
                    data: {
                        varName: "global",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },
        {
            "/*global foo:true*/",
            errors: [
                {
                    line: 1,
                    column: 10,
                    endLine: 1,
                    endColumn: 13,
                    messageId: "unusedVar",
                    data: {
                        varName: "foo",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },

        // non ascii.
        {
            "/*global 変数, 数*/\n変数;",
            errors: [
                {
                    line: 1,
                    column: 14,
                    endLine: 1,
                    endColumn: 15,
                    messageId: "unusedVar",
                    data: {
                        varName: "数",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },

        // surrogate pair.
        {
            "/*global 𠮷𩸽, 𠮷*/\n\\u{20BB7}\\u{29E3D};",
            env: { es6: true },
            errors: [
                {
                    line: 1,
                    column: 16,
                    endLine: 1,
                    endColumn: 18,
                    messageId: "unusedVar",
                    data: {
                        varName: "𠮷",
                        action: "defined",
                        additional: ""
                    }
                }
            ]
        },

        // https://github.com/eslint/eslint/issues/4047
        {
            "export default function(a) {}",
            module: true,
            errors: [definedError("a")]
        },
        {
            "export default function(a, b) { console.log(a); }",
            module: true,
            errors: [definedError("b")]
        },
        {
            "export default (function(a) {});",
            module: true,
            errors: [definedError("a")]
        },
        {
            "export default (function(a, b) { console.log(a); });",
            module: true,
            errors: [definedError("b")]
        },
        {
            "export default (a) => {};",
            module: true,
            errors: [definedError("a")]
        },
        {
            "export default (a, b) => { console.log(a); };",
            module: true,
            errors: [definedError("b")]
        },

        // caughtErrors
        {
            "try{}catch(err){};",
            options: [{ caughtErrors: "all" }],
            errors: [definedError("err")]
        },
        {
            "try{}catch(err){};",
            options: [{ caughtErrors: "all", caughtErrorsIgnorePattern: "^ignore" }],
            errors: [definedError("err", ". Allowed unused args must match /^ignore/u")]
        },

        // multiple try catch with one success
        {
            "try{}catch(ignoreErr){}try{}catch(err){};",
            options: [{ caughtErrors: "all", caughtErrorsIgnorePattern: "^ignore" }],
            errors: [definedError("err", ". Allowed unused args must match /^ignore/u")]
        },

        // multiple try catch both fail
        {
            "try{}catch(error){}try{}catch(err){};",
            options: [{ caughtErrors: "all", caughtErrorsIgnorePattern: "^ignore" }],
            errors: [
                definedError("error", ". Allowed unused args must match /^ignore/u"),
                definedError("err", ". Allowed unused args must match /^ignore/u")
            ]
        },

        // caughtErrors with other configs
        {
            "try{}catch(err){};",
            options: [{ vars: "all", args: "all", caughtErrors: "all" }],
            errors: [definedError("err")]
        },

        // no conflict in ignore patterns
        {
            "try{}catch(err){};",
            options: [
                {
                    vars: "all",
                    args: "all",
                    caughtErrors: "all",
                    argsIgnorePattern: "^er"
                }
            ],
            errors: [definedError("err")]
        },

        // Ignore reads for modifications to itself: https://github.com/eslint/eslint/issues/6348
        { "var a = 0; a = a + 1;", errors: [assignedError("a")] },
        { "var a = 0; a = a + a;", errors: [assignedError("a")] },
        { "var a = 0; a += a + 1;", errors: [assignedError("a")] },
        { "var a = 0; a++;", errors: [assignedError("a")] },
        { "function foo(a) { a = a + 1 } foo();", errors: [assignedError("a")] },
        { "function foo(a) { a += a + 1 } foo();", errors: [assignedError("a")] },
        { "function foo(a) { a++ } foo();", errors: [assignedError("a")] },
        { "var a = 3; a = a * 5 + 6;", errors: [assignedError("a")] },
        { "var a = 2, b = 4; a = a * 2 + b;", errors: [assignedError("a")] },

        // https://github.com/eslint/eslint/issues/6576 (For coverage)
        {
            "function foo(cb) { cb = function(a) { cb(1 + a); }; bar(not_cb); } foo();",
            errors: [assignedError("cb")]
        },
        {
            "function foo(cb) { cb = function(a) { return cb(1 + a); }(); } foo();",
            errors: [assignedError("cb")]
        },
        {
            "function foo(cb) { cb = (function(a) { cb(1 + a); }, cb); } foo();",
            errors: [assignedError("cb")]
        },
        {
            "function foo(cb) { cb = (0, function(a) { cb(1 + a); }); } foo();",
            errors: [assignedError("cb")]
        },

        // https://github.com/eslint/eslint/issues/6646
        {
            [
                "while (a) {",
                "    function foo(b) {",
                "        b = b + 1;",
                "    }",
                "    foo()",
                "}"
            ].join("\n"),
            errors: [assignedError("b")]
        },

        // https://github.com/eslint/eslint/issues/7124
        {
            "(function(a, b, c) {})",
            options: [{ argsIgnorePattern: "c" }],
            errors: [
                definedError("a", ". Allowed unused args must match /c/u"),
                definedError("b", ". Allowed unused args must match /c/u")
            ]
        },
        {
            "(function(a, b, {c, d}) {})",
            options: [{ argsIgnorePattern: "[cd]" }],
            parserOptions: { ecmaVersion: 6 },
            errors: [
                definedError("a", ". Allowed unused args must match /[cd]/u"),
                definedError("b", ". Allowed unused args must match /[cd]/u")
            ]
        },
        {
            "(function(a, b, {c, d}) {})",
            options: [{ argsIgnorePattern: "c" }],
            parserOptions: { ecmaVersion: 6 },
            errors: [
                definedError("a", ". Allowed unused args must match /c/u"),
                definedError("b", ". Allowed unused args must match /c/u"),
                definedError("d", ". Allowed unused args must match /c/u")
            ]
        },
        {
            "(function(a, b, {c, d}) {})",
            options: [{ argsIgnorePattern: "d" }],
            parserOptions: { ecmaVersion: 6 },
            errors: [
                definedError("a", ". Allowed unused args must match /d/u"),
                definedError("b", ". Allowed unused args must match /d/u"),
                definedError("c", ". Allowed unused args must match /d/u")
            ]
        },
        {
            "/*global\rfoo*/",
            errors: [{
                line: 2,
                column: 1,
                endLine: 2,
                endColumn: 4,
                messageId: "unusedVar",
                data: {
                    varName: "foo",
                    action: "defined",
                    additional: ""
                }
            }]
        },

        // https://github.com/eslint/eslint/issues/8442
        {
            "(function ({ a }, b ) { return b; })();",
            parserOptions: { ecmaVersion: 2015 },
            errors: [
                definedError("a")
            ]
        },
        {
            "(function ({ a }, { b, c } ) { return b; })();",
            parserOptions: { ecmaVersion: 2015 },
            errors: [
                definedError("a"),
                definedError("c")
            ]
        },
        {
            "(function ({ a, b }, { c } ) { return b; })();",
            parserOptions: { ecmaVersion: 2015 },
            errors: [
                definedError("a"),
                definedError("c")
            ]
        },
        {
            "(function ([ a ], b ) { return b; })();",
            parserOptions: { ecmaVersion: 2015 },
            errors: [
                definedError("a")
            ]
        },
        {
            "(function ([ a ], [ b, c ] ) { return b; })();",
            parserOptions: { ecmaVersion: 2015 },
            errors: [
                definedError("a"),
                definedError("c")
            ]
        },
        {
            "(function ([ a, b ], [ c ] ) { return b; })();",
            parserOptions: { ecmaVersion: 2015 },
            errors: [
                definedError("a"),
                definedError("c")
            ]
        },

        // https://github.com/eslint/eslint/issues/9774
        {
            "(function(_a) {})();",
            options: [{ args: "all", varsIgnorePattern: "^_" }],
            errors: [definedError("_a")]
        },
        {
            "(function(_a) {})();",
            options: [{ args: "all", caughtErrorsIgnorePattern: "^_" }],
            errors: [definedError("_a")]
        },

        // https://github.com/eslint/eslint/issues/10982
        {
            "var a = function() { a(); };",
            errors: [assignedError("a")]
        },
        {
            "var a = function(){ return function() { a(); } };",
            errors: [assignedError("a")]
        },
        {
            "const a = () => { a(); };",
            parserOptions: { ecmaVersion: 2015 },
            errors: [assignedError("a")]
        },
        {
            "const a = () => () => { a(); };",
            parserOptions: { ecmaVersion: 2015 },
            errors: [assignedError("a")]
        },
        {
            `let myArray = [1,2,3,4].filter((x) => x == 0);
    myArray = myArray.filter((x) => x == 1);`,
            parserOptions: { ecmaVersion: 2015 },
            errors: [{ ...assignedError("myArray"), line: 2, column: 15 }]
        },
        {
            "const a = 1; a += 1;",
            parserOptions: { ecmaVersion: 2015 },
            errors: [{ ...assignedError("a"), line: 1, column: 14 }]
        },
        {
            "var a = function() { a(); };",
            errors: [{ ...assignedError("a"), line: 1, column: 22 }]
        },
        {
            "var a = function(){ return function() { a(); } };",
            errors: [{ ...assignedError("a"), line: 1, column: 41 }]
        },
        {
            "const a = () => { a(); };",
            parserOptions: { ecmaVersion: 2015 },
            errors: [{ ...assignedError("a"), line: 1, column: 19 }]
        },
        {
            "const a = () => () => { a(); };",
            parserOptions: { ecmaVersion: 2015 },
            errors: [{ ...assignedError("a"), line: 1, column: 25 }]
        },
        {

            `let a = 'a';
            a = 10;
            function foo(){
                a = 11;
                a = () => {
                    a = 13
                }
            }`,
            parserOptions: { ecmaVersion: 2020 },
            errors: [{ ..., line: 3, column: 22 }, { ...assignedError("a"), line: 6, column: 21 }]
        },
        {
            `let c = 'c'
c = 10
function foo1() {
  c = 11
  c = () => {
    c = 13
  }
}
c = foo1`,
            parserOptions: { ecmaVersion: 2020 },
            errors: [{ ...assignedError("c"), line: 10, column: 1 }]
        }
    ]
});
*/
