use criterion::{criterion_group, criterion_main, Criterion};

use script_bench::boa::BoaEngine;

fn benchmark(c: &mut Criterion) {
    script_bench::harness::run_cases::<BoaEngine>(|name, func| {
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
