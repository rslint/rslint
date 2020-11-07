//! Tests for detecting undefined variables

// let ast = parse_text($valid_code).expect("failed to parse test code");
// analyzer.clear_globals().expect("failed to clear globals");

macro_rules! rule_test {
    (
        $rule_name:ident,
        $({
            $code:literal
            $(, globals: [$($global:literal),* $(,)?])?
            $(, browser: $browser:literal)?
            $(, node: $node:literal)?
            $(, ecma: $ecma:literal)?
            $(, module: $module:literal)?
            $(, es2021: $es2021:literal)?
            $(, invalid_name_uses: [$($name:literal),* $(,)?])?
            $(,)?
        }),* $(,)?
    ) => {
        #[test]
        fn $rule_name() {
            use crate::tests::DatalogTestHarness;
            use std::borrow::Cow;

            let mut analyzer = DatalogTestHarness::new();

            $(
                analyzer
                    .test($code, stringify!($rule_name))
                    $(.with_globals(vec![$(Cow::Borrowed($global)),*]))?
                    $(.with_browser($browser))?
                    $(.with_node($node))?
                    $(.with_ecma($ecma))?
                    $(.is_module($module))?
                    $(.with_es2021($es2021))?
                    $(.with_invalid_name_uses(vec![$(Cow::Borrowed($name)),*]))?
                    .run();
            )?

            analyzer.report_outcome();
        }
    };
}

rule_test! {
    no_undef,
    { "var a = 1, b = 2; a;" },
    { "function f() { b; }", globals: ["b"] },
    { "a; function f() { b; a; }", globals: ["b", "a"] },
    { "function a(){}  a();" },
    { "function f(b) { b; }" },
    { "var a; a = 1; a++;" },
    { "var a; function f() { a = 1; }" },
    { "b++;", globals: ["b"] },
    { "window;", browser: true },
    { "require(\"a\");", node: true },
    { "Object; isNaN();", ecma: true },
    { "toString()" },
    { "hasOwnProperty()" },
    { "function evilEval(stuffToEval) { var ultimateAnswer; ultimateAnswer = 42; eval(stuffToEval); }" },
    { "typeof a" },
    { "typeof (a)" },
    { "var b = typeof a" },
    { "typeof a === 'undefined'" },
    { "if (typeof a === 'undefined') {}" },
    { "typeof ((((((a))))))" },
    { "typeof (1, 2, 3, a)" },
    { "typeof (1, 2, 3, (((1, 2, 3, a))))" },
    { "function foo() { var [a, b=4] = [1, 2]; return {a, b}; }" },
    { "var toString = 1;" },
    // FIXME: Requires JSX
    // { "var React, App, a=1; React.render(<App attr={a} />);" },
    { "function myFunc(...foo) {  return foo; }" },
    { "var console; [1,2,3].forEach(obj => { console.log(obj); });", node: true },
    { "var Foo; class Bar extends Foo { constructor() { super(); }}" },
    { "import Warning from '../lib/warning'; var warn = new Warning('text');", module: true },
    { "import * as Warning from '../lib/warning'; var warn = new Warning('text');", module: true },
    { "var a; [a] = [0];" },
    { "var a; ({a} = {});" },
    { "var obj; [obj.a, obj.b] = [0, 1];" },
    { "var a; ({b: a} = {});" },
    { "URLSearchParams;", browser: true },
    { "Intl;", browser: true },
    { "IntersectionObserver;", browser: true },
    { "Credential;", browser: true },
    { "requestIdleCallback;", browser: true },
    { "customElements;", browser: true },
    { "PromiseRejectionEvent;", browser: true },
    { "(foo, bar) => { foo ||= WeakRef; bar ??= FinalizationRegistry; }", es2021: true },
    { "function f() { b = 1; }", globals: ["b"] },
    { "function f() { b++; }", globals: ["b"] },
    { "b = 1;", globals: ["b"] },
    { "var b = 1;", globals: ["b"] },
    { "Array = 1;" },
    { "class A { constructor() { new.target; } }" },
    { "var {bacon, ...others} = stuff; foo(others)", globals: ["stuff", "foo"] },
    { "export * as ns from \"source\"", module: true },
    { "import.meta", module: true },
}

/*
ruleTester.run("no-undef", rule, {
    invalid: [
        { code: "a = 1;", errors: [{ messageId: "undef", data: { name: "a" }, type: "Identifier" }] },
        { code: "if (typeof anUndefinedVar === 'string') {}", options: [{ typeof: true }], errors: [{ messageId: "undef", data: { name: "anUndefinedVar" }, type: "Identifier" }] },
        { code: "var a = b;", errors: [{ messageId: "undef", data: { name: "b" }, type: "Identifier" }] },
        { code: "function f() { b; }", errors: [{ messageId: "undef", data: { name: "b" }, type: "Identifier" }] },
        { code: "window;", errors: [{ messageId: "undef", data: { name: "window" }, type: "Identifier" }] },
        { code: "require(\"a\");", errors: [{ messageId: "undef", data: { name: "require" }, type: "Identifier" }] },
        { code: "var React; React.render(<img attr={a} />);", parserOptions: { ecmaVersion: 6, ecmaFeatures: { jsx: true } }, errors: [{ messageId: "undef", data: { name: "a" } }] },
        { code: "var React, App; React.render(<App attr={a} />);", parserOptions: { ecmaVersion: 6, ecmaFeatures: { jsx: true } }, errors: [{ messageId: "undef", data: { name: "a" } }] },
        { code: "[a] = [0];", parserOptions: { ecmaVersion: 6 }, errors: [{ messageId: "undef", data: { name: "a" } }] },
        { code: "({a} = {});", parserOptions: { ecmaVersion: 6 }, errors: [{ messageId: "undef", data: { name: "a" } }] },
        { code: "({b: a} = {});", parserOptions: { ecmaVersion: 6 }, errors: [{ messageId: "undef", data: { name: "a" } }] },
        { code: "[obj.a, obj.b] = [0, 1];", parserOptions: { ecmaVersion: 6 }, errors: [{ messageId: "undef", data: { name: "obj" } }, { messageId: "undef", data: { name: "obj" } }] },

        // Experimental
        {
            code: "const c = 0; const a = {...b, c};",
            parserOptions: {
                ecmaVersion: 2018
            },
            errors: [{ messageId: "undef", data: { name: "b" } }]
        }
    ]
});
*/
