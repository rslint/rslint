use crate::{DatalogResult, ScopeAnalyzer};
use ast::FileId;
use config::{NoShadowConfig, NoShadowHoisting};

fn hoisting_never(analyzer: &ScopeAnalyzer, file: FileId) -> DatalogResult<()> {
    analyzer.no_shadow(
        file,
        Some(NoShadowConfig {
            hoisting: NoShadowHoisting::Never,
        }),
    );

    Ok(())
}

fn hoisting_always(analyzer: &ScopeAnalyzer, file: FileId) -> DatalogResult<()> {
    analyzer.no_shadow(
        file,
        Some(NoShadowConfig {
            hoisting: NoShadowHoisting::Always,
        }),
    );

    Ok(())
}

rule_test! {
    no_shadow,
    default_conf: hoisting_never,
    filter: DatalogLint::is_no_shadow,
    // Should pass
    { "var a = 3; function b(x) { a++; return x + a; }; setTimeout(function() { b(a); }, 0);" },
    { "(function() { var doSomething = function doSomething() {}; doSomething() }())" },
    { "var arguments;\nfunction bar() { }" },
    {
        "var a = 3;",
        "var b = (x) => {",
        "   a++;",
        "   return x + a;",
        "};",
        "setTimeout(",
        "   () => { b(a); },",
        "   0,",
        ");",
    },
    { "class A {}" },
    { "class A { constructor() { var a; } }" },
    { "(function() { var A = class A {}; })()" },
    { "function foo(a) { } let a;" },
    { "{ const a = 0; } const a = 1;" },
    { "function foo(a) { } let a;" },
    { "function foo() { var Object = 0; }" },
    { "function foo() { var top = 0; }", browser: true },
    { "var Object = 0;" },
    { "var top = 0;", browser: true },
    { "function foo() { let a; } let a;" },
    { "function foo() { var a; } let a;" },
    { "{ const a = 0; } const a = 1;" },
    { "{ const a = 0; } var a;" },
    { "{ let a; } var a;" },
    { "{ let a; } function a() {}" },
    { "{ const a = 0; } var a;" },
    { "{ const a = 0; } function a() {}" },
    { "function foo() { let a; } var a;" },
    { "function foo() { var a; } var a;" },
    { "function foo() { let a; } function a() {}" },
    { "function foo() { var a; } function a() {}" },

    // Should fail
    {
        "{ var a; } var a;",
        errors: [DatalogLint::no_shadow("a", 15..16, 6..7, false)],
        config: hoisting_always,
    },
    {
        "function a(x) { var b = function c() { var x = 'foo'; }; }",
        errors: [DatalogLint::no_shadow("x", 11..12, 43..44, false)],
    },
    {
        "var a = (x) => { var b = () => { var x = 'foo'; }; }",
        errors: [DatalogLint::no_shadow("x", 9..10, 37..38, false)],
    },
    {
        "var x = 1; { let x = 2; }",
        errors: [DatalogLint::no_shadow("x", 4..5, 17..18, false)],
    },
    {
        "let x = 1; { const x = 2; }",
        errors: [DatalogLint::no_shadow("x", 4..5, 19..20, false)],
    },
    {
        "function foo(a) { } var a;",
        errors: [DatalogLint::no_shadow("a", 24..25, 13..14, false)],
        config: hoisting_always,
    },
    {
        "function foo(a) { } function a() {}",
        errors: [DatalogLint::no_shadow("a", 29..30, 13..14, false)],
        config: hoisting_always,
    },
    {
        "{ let a; } function a() {}",
        errors: [DatalogLint::no_shadow("a", 20..21, 6..7, false)],
        config: hoisting_always,
    },
    {
        "{ const a = 0; } function a() {}",
        errors: [DatalogLint::no_shadow("a", 26..27, 8..9, false)],
        config: hoisting_always,
    },
    {
        "function foo() { let a; } function a() {}",
        errors: [DatalogLint::no_shadow("a", 35..36, 21..22, false)],
        config: hoisting_always,
    },
    {
        "function foo() { var a; } function a() {}",
        errors: [DatalogLint::no_shadow("a", 35..36, 21..22, false)],
        config: hoisting_always,
    },
    {
        "function foo() { let a; } var a;",
        errors: [DatalogLint::no_shadow("a", 30..31, 21..22, false)],
        config: hoisting_always,
    },
    {
        "function foo() { var a; } var a;",
        errors: [DatalogLint::no_shadow("a", 30..31, 21..22, false)],
        config: hoisting_always,
    },
}
