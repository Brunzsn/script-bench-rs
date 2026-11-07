use criterion::{criterion_group, criterion_main, Criterion};

use script_bench::wasm::WasmEngine;

fn benchmark(c: &mut Criterion) {
    script_bench::harness::run_cases::<WasmEngine>(|name, func| {
        c.bench_function(name, |b| b.iter(|| func()));
    })
    .unwrap();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark,
}

criterion_main!(benches);
