use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use rslint_parser::parse_text;

const ENGINE_262: &str = include_str!("../../../tests/engine262.js");

pub fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("engine262");
    group.sample_size(10);
    group.throughput(Throughput::Bytes(ENGINE_262.len() as u64));
    group.bench_function("parse", |b| b.iter(|| parse_text(ENGINE_262, 0)));
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
