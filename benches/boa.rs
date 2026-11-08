mod common;

use criterion::{criterion_group, criterion_main, Criterion};

use script_bench::boa::{BoaEngine, BoaInterop};

fn benchmark(c: &mut Criterion) {
    script_bench::harness::run_cases::<BoaEngine>(|name, func| {
        c.bench_function(name, |b| b.iter(|| func()));
    })
    .unwrap();
    script_bench::interop::run_interop_cases::<BoaInterop>(|name, func| {
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
