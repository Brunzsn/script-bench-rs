mod common;

use gungraun::prelude::*;
use script_bench::rhai::RhaiEngine;

fn setup(_: u32) -> RhaiEngine {
    common::setup::<RhaiEngine>()
}

#[library_benchmark(setup = setup)]
#[bench::sort(0)]
fn sort_userdata(mut engine: RhaiEngine) {
    common::run(&mut engine);
}

library_benchmark_group!(
    name = rhai_group,
    benchmarks = sort_userdata
);

main!(library_benchmark_groups = rhai_group);
