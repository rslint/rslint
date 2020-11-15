rule_test! {
    use_before_def,
    rule_conf: |conf| conf.no_use_before_def(true),
    filter: DatalogLint::is_use_before_def,
    // Should pass
    { "var a = 10; alert(a);" },
    { "function b(a) { alert(a); }" },
    { "Object.hasOwnProperty.call(a);" },
    { "function a() { alert(arguments);}" },
    { "(() => { var a = 42; alert(a); })();" },
    { "a(); try { throw new Error() } catch (a) {}" },
    { "class A {} new A();" },
    { "var a = 0, b = a;" },
    { "var {a = 0, b = a} = {};" },
    { "var [a = 0, b = a] = {};" },
    { "function foo() { foo(); }" },
    { "var foo = function() { foo(); };" },
    { "var a; for (a in a) {}" },
    { "var a; for (a of a) {}" },

    // Should fail
    {
        "a++; var a = 19;",
        module: true,
        errors: [DatalogLint::use_before_def("a", 0..1, 5..16)],
    },
    { "a++; var a = 19;", errors: [DatalogLint::use_before_def("a", 0..1, 5..16)] },
    {
        "a(); var a = function() {};",
        errors: [DatalogLint::use_before_def("a", 0..1, 5..27)],
    },
    {
        "alert(a[1]); var a = [1, 3];",
        errors: [DatalogLint::use_before_def("a", 6..7, 13..28)],
    },
    {
        "a(); function a() { alert(b); var b = 10; a(); }",
        errors: [
            DatalogLint::use_before_def("a", 0..3, 14..15),
            DatalogLint::use_before_def("b", 26..27, 30..41),
        ],
    },
    {
        "(() => { alert(a); var a = 42; })();",
        errors: [DatalogLint::use_before_def("a", 15..16, 19..30)],
    },
    {
        "(() => a())(); function a() {}",
        errors: [DatalogLint::use_before_def("a", 7..10, 24..25)],
    },
    {
        "a(); try { throw new Error() } catch (foo) { var a; }",
        errors: [DatalogLint::use_before_def("a", 0..1, 45..51)],
    },
    {
        "var f = () => a; var a;",
        errors: [DatalogLint::use_before_def("a", 14..15, 17..23)],
    },
    {
        "new A(); class A {};",
        errors: [DatalogLint::use_before_def("A", 0..7, 15..16)],
    },
    {
        "function foo() { new A(); } class A {};",
        errors: [DatalogLint::use_before_def("A", 17..24, 34..35)],
    },
    {
        "new A(); var A = class {};",
        errors: [DatalogLint::use_before_def("A", 0..7, 17..25)],
    },
    {
        "function foo() { new A(); } var A = class {};",
        errors: [DatalogLint::use_before_def("A", 17..24, 36..44)],
    },
}
