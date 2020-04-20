use criterion::{criterion_group, criterion_main, Criterion};
use std::io::Read;
use rslint::linter::Linter;
use std::fs::File;

fn lex_js() {
  Linter::new(String::from("benches/files/es5.js")).run();
}

fn bench(c: &mut Criterion) {
  c.bench_function("lexer-es5", |b| b.iter(|| lex_js()));
}

criterion_group!(benches, bench);
criterion_main!(benches);