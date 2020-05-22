#[macro_export]
macro_rules! span {
    ($src:expr, $target:expr) => {{
        let start = $src.find($target).unwrap();
        let end = start + $target.len();
        Span::new(start, end)
    }};

    ($src:expr, $target:expr, $idx:expr) => {{
        let start = $src.matches($target).collect()[$idx];
        let end = start + $target.len();
        Span::new(start, end)
    }};
}

#[macro_export]
macro_rules! expr {
    ($src:expr) => {
        Parser::with_source($src, "tests", true)
            .unwrap()
            .parse_binary_expr(None)
            .unwrap()
    };
}

#[macro_export]
macro_rules! unwrap_enum {
    ($wrapped:expr, $expected:pat) => {{
        if let Expr::$expected(ref data) = $wrapped {
            data
        } else {
            panic!();
        }
    }};
}
