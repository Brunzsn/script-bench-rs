mod common;

use gungraun::prelude::*;
use script_bench::koto::KotoEngine;

fn setup(_: u32) -> KotoEngine {
    common::setup::<KotoEngine>()
}

#[library_benchmark(setup = setup)]
#[bench::sort(0)]
fn sort_userdata(mut engine: KotoEngine) {
    common::run(&mut engine);
}

library_benchmark_group!(
    name = koto_group,
    benchmarks = sort_userdata
);

main!(library_benchmark_groups = koto_group);
