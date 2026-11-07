mod common;

use gungraun::prelude::*;
use script_bench::roto::RotoEngine;

fn setup(_: u32) -> RotoEngine {
    common::setup::<RotoEngine>()
}

#[library_benchmark(setup = setup)]
#[bench::sort(0)]
fn sort_userdata(mut engine: RotoEngine) {
    common::run(&mut engine);
}

library_benchmark_group!(
    name = roto_group,
    benchmarks = sort_userdata
);

main!(library_benchmark_groups = roto_group);
