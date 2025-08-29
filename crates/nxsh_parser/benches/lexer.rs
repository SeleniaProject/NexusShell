use criterion::{criterion_group, criterion_main, Criterion};

fn bench_tokenize(c: &mut Criterion) {
    let script = include_str!("../../../docs/SPEC.md"); // large-ish real text as sample
    c.bench_function("tokenize_spec", |b| {
        b.iter(|| nxsh_parser::lexer::tokenize(script))
    });
}

criterion_group!(benches, bench_tokenize);
criterion_main!(benches);
