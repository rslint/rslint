use criterion::*;
use rslint_parser::{parse_module, parse_module_lossy};
use std::fs::read_to_string;

fn bench_module_lossy(source: &str) {
    parse_module_lossy(&source, 0);
}

fn bench_module(source: &str) {
    parse_module(&source, 0);
}

fn bench(c: &mut Criterion) {
    let source = read_to_string("./benches/files/es2021.js").unwrap();

    let mut group = c.benchmark_group("parsing");

    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("parse-module-lossy", |b| {
        b.iter(|| bench_module_lossy(black_box(&source)))
    });
    group.bench_function("parse-module", |b| {
        b.iter(|| bench_module(black_box(&source)))
    });
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
