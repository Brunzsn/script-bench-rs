mod common;

use gungraun::prelude::*;
use script_bench::martite::MartiteEngine;

fn setup(_: u32) -> MartiteEngine {
    common::setup::<MartiteEngine>()
}

#[library_benchmark(setup = setup)]
#[bench::sort(0)]
fn sort_userdata(mut engine: MartiteEngine) {
    common::run(&mut engine);
}

library_benchmark_group!(
    name = martite_group,
    benchmarks = sort_userdata
);

main!(library_benchmark_groups = martite_group);
