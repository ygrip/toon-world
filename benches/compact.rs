use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::Value;
use toon_world::{output, sparse};

mod support;

fn standard(value: &Value) -> String {
    toon_format::encode_default(value).expect("standard TOON must encode")
}

fn compact(value: &Value) -> String {
    let standard = standard(value);
    output::select_compact_toon(
        standard,
        sparse::encode(value).expect("sparse encoding must not fail"),
    )
    .into_rendered()
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("compact");
    for fixture in support::fixtures() {
        group.bench_with_input(
            BenchmarkId::new("standard", &fixture.name),
            &fixture.value,
            |b, value| b.iter(|| black_box(standard(black_box(value)))),
        );
        group.bench_with_input(
            BenchmarkId::new("sparse", &fixture.name),
            &fixture.value,
            |b, value| b.iter(|| black_box(sparse::encode(black_box(value)))),
        );
        group.bench_with_input(
            BenchmarkId::new("compact", &fixture.name),
            &fixture.value,
            |b, value| b.iter(|| black_box(compact(black_box(value)))),
        );
    }
    group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
