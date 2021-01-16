rule_test! {
    no_unused_labels,
    default_conf: |analyzer, file| {
        analyzer.no_unused_labels(file, Some(Default::default()));
        Ok(())
    },
    filter: DatalogLint::is_no_unused_labels,
    // Should pass
    { "A: break A;" },
    { "A: { foo(); break A; bar(); }" },
    { "A: if (a) { foo(); if (b) break A; bar(); }" },
    { "A: for (var i = 0; i < 10; ++i) { foo(); if (a) break A; bar(); }" },
    { "A: for (var i = 0; i < 10; ++i) { foo(); if (a) continue A; bar(); }" },
    {
        "A: {",
        "    B: break B;",
        "    C: for (var i = 0; i < 10; ++i) {",
        "        foo();",
        "        if (a)",
        "            break A;",
        "        if (c)",
        "            continue C;",
        "        bar();",
        "    }",
        "}",
    },
    { "A: { var A = 0; console.log(A); break A; console.log(A); }" },

    // Should fail
    {
        "A: var foo = 0;",
        errors: [DatalogLint::unused_label("A", 0..1)],
    },
    {
        "A: { foo(); bar(); }",
        errors: [DatalogLint::unused_label("A", 0..1)],
    },
    {
        "A: if (a) { foo(); bar(); }",
        errors: [DatalogLint::unused_label("A", 0..1)],
    },
    {
        "A: for (var i = 0; i < 10; ++i) { foo(); if (a) break; bar(); }",
        errors: [DatalogLint::unused_label("A", 0..1)],
    },
    {
        "A: for (var i = 0; i < 10; ++i) { foo(); if (a) continue; bar(); }",
        errors: [DatalogLint::unused_label("A", 0..1)],
    },
    {
        "A: for (var i = 0; i < 10; ++i) { B: break A; }",
        errors: [DatalogLint::unused_label("B", 34..35)],
    },
    {
        "A: { var A = 0; console.log(A); }",
        errors: [DatalogLint::unused_label("A", 0..1)],
    },
    {
        "A /* comment */: foo",
        errors: [DatalogLint::unused_label("A", 0..1)],
    },
}
