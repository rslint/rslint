use criterion::{criterion_group, criterion_main, Criterion};
use rslint_parse::lexer::lexer::Lexer;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn lex_js(source: String) {
  Lexer::new(&source, "bench").for_each(drop);
}

fn bench(c: &mut Criterion) {
  let mut file = File::open(Path::new("benches/files/es5.js")).unwrap();
  let mut buf: Vec<u8> = vec![];
  file.read_to_end(&mut buf)
    .expect("Failed to read bencher file");
  let source = String::from_utf8_lossy(&buf).to_string();
  c.bench_function("lexer-es5", |b| b.iter(|| lex_js(source.clone())));
}

criterion_group!(benches, bench);
criterion_main!(benches);