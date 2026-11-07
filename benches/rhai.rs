mod common;

use criterion::{criterion_group, criterion_main, Criterion};

use script_bench::rhai::RhaiEngine;

fn benchmark(c: &mut Criterion) {
    script_bench::harness::run_cases::<RhaiEngine>(|name, func| {
        c.bench_function(name, |b| b.iter(|| func()));
    })
    .unwrap();
}

criterion_group! {
    name = benches;
    config = common::config();
    targets = benchmark,
}

criterion_main!(benches);
