use criterion::{criterion_group, criterion_main, Criterion};
use rslint_parse::parser::Parser;

pub fn expr_serialization(b: &mut Criterion) {
    let src = "a += 6 ? {\"a\": b, 7: foo} : 700 * 5 / 1 - [a, b, c]";

    b.bench_function("parse expressions",
        |b| b.iter(|| {
            Parser::with_source(src, 0, true).unwrap().parse_expr(None).unwrap();
        })
    );
}

criterion_group!(benches, expr_serialization);
criterion_main!(benches);