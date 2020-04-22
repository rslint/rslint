use criterion::{criterion_group, criterion_main, Criterion};
use rslint::linter::Linter;

fn lex_js() {
  Linter::new(String::from("benches/files/es5.js")).run()
    .expect("Failed to run linter");
}

fn bench(c: &mut Criterion) {
  c.bench_function("lexer-es5", |b| b.iter(|| lex_js()));
}

criterion_group!(benches, bench);
criterion_main!(benches);