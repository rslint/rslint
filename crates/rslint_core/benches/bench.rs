use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use rslint_core::{CstRuleStore, File};
use rslint_lexer::Lexer;
use rslint_parser::parse_text;

const ENGINE_262_URL: &str = "https://engine262.js.org/engine262/engine262.js";

fn parse(source: &str) {
    parse_text(source, 0);
}

fn tokenize(source: &str) {
    Lexer::from_str(source, 0).for_each(drop);
}

fn lint(file: &File) {
    let _ = rslint_core::lint_file(file, &CstRuleStore::new().builtins(), false);
}

fn bench_source(c: &mut Criterion, file: &File) {
    let mut group = c.benchmark_group(&file.name);
    group.sample_size(10);
    group.throughput(Throughput::Bytes(file.source.len() as u64));
    group.bench_function("tokenize", |b| b.iter(|| tokenize(black_box(&file.source))));
    group.bench_function("parse", |b| b.iter(|| parse(black_box(&file.source))));
    group.bench_function("lint", |b| b.iter(|| lint(black_box(file))));
    group.finish();
}

fn engine262(c: &mut Criterion) {
    let source = ureq::get(ENGINE_262_URL)
        .call()
        .into_string()
        .expect("failed to get engine262 source code");
    let file = File::from_string(source, rslint_parser::FileKind::Module, "engine262");
    bench_source(c, &file);
}

criterion_group!(benches, engine262);
criterion_main!(benches);
