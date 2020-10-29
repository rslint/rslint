use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rslint_lexer::Lexer;

const ENGINE_262: &str = include_str!("../../../tests/engine262.js");

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine262");
    group.throughput(Throughput::Bytes(ENGINE_262.len() as u64));
    group.bench_function("tokenize", |b| {
        b.iter(|| Lexer::from_str(black_box(&ENGINE_262), 0).for_each(drop))
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
