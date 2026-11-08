mod common;

use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    for case in script_bench::cases::CASES {
        c.bench_function(case.name, |b| b.iter(|| (case.native)()));
    }
    for case in script_bench::interop::INTEROP_CASES {
        c.bench_function(case.name, |b| b.iter(|| (case.native)()));
    }
}

criterion_group! {
    name = benches;
    config = common::config();
    targets = benchmark,
}

criterion_main!(benches);
