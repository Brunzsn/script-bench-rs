use criterion::{criterion_group, criterion_main, Criterion};

use script_bench::martite::Value;

fn benchmark(c: &mut Criterion) {
    script_bench::martite::sort_userdata(
        |func| {
            c.bench_function("Sort Rust objects", |b| {
                b.iter(|| func());
            });
        },
        |value| {
            // Validate that the results are sorted
            let Value::List(items) = value else {
                panic!("Expected a list");
            };
            assert_eq!(items.len(), 10000);

            let mut previous: Option<String> = None;
            for item in items.iter() {
                let Value::Handle(handle) = item else {
                    panic!("Expected a RustData handle");
                };
                let current = handle.token.clone();
                if let Some(previous) = &previous {
                    assert!(previous.as_str() <= current.as_str());
                }
                previous = Some(current);
            }
        },
    )
    .unwrap();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark,
}

criterion_main!(benches);
