rule_test! {
    typeof_undef,
    filter: DatalogLint::is_typeof_undef,
    // Should pass
    { "var a = 10; typeof a" },
    { "let x = 10; typeof ((1, 4, (x)))" },

    // Should fail
    { "typeof a", errors: [DatalogLint::typeof_undef(0..8, 7..8)] },
    { "typeof (a)", errors: [DatalogLint::typeof_undef(0..10, 8..9)] },
    { "var b = typeof a", errors: [DatalogLint::typeof_undef(8..16, 15..16)] },
    { "typeof a === 'undefined'", errors: [DatalogLint::typeof_undef(0..8, 7..8)] },
    { "if (typeof a === 'undefined') {}", errors: [DatalogLint::typeof_undef(4..12, 11..12)] },
    { "typeof ((((((a))))))", errors: [DatalogLint::typeof_undef(0..20, 13..14)] },
    { "typeof (1, 2, 3, a)", errors: [DatalogLint::typeof_undef(0..19, 17..18)] },
    { "typeof (1, 2, 3, (((1, 2, 3, a))))", errors: [DatalogLint::typeof_undef(0..34, 29..30)] },
}
