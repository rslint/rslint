use criterion::{criterion_group, criterion_main, Criterion};
use rslint_parse::parser::Parser;

pub fn literal_serialization(b: &mut Criterion) {
    let expr = Parser::with_source("foobar", "benches", true)
        .unwrap()
        .parse_expr()
        .unwrap();

    b.bench_function("serialize_literal_expr",
        |b| b.iter(|| {
            expr.to_string("foobar")
        })
    );
}

pub fn sequence_serialization(b: &mut Criterion) {
    let expr = Parser::with_source("1, 2, 3, new foo, bar", "benches", true)
        .unwrap()
        .parse_expr()
        .unwrap();
    
    b.bench_function("serialize sequence expr",
     |b| b.iter(|| {
         expr.to_string("1, 2, 3, new foo, bar")
     }));
}

pub fn complex_exprs(b: &mut Criterion) {
    let expr = Parser::with_source("1 + 3 * 7 / 8 ? true : new foo.bar.foo", "benches", true)
        .unwrap()
        .parse_conditional_expr(None)
        .unwrap();

    b.bench_function("serialize_complex_expr",
        |b| b.iter(|| {
            expr.to_string("1 + 3 * 7 / 8 ? true : new foo.bar.foo")
        })
    );
}

criterion_group!(benches, literal_serialization, sequence_serialization, complex_exprs);
criterion_main!(benches);