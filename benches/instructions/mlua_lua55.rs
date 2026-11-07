mod common;

use gungraun::prelude::*;
use script_bench::mlua::MluaEngine;

fn setup(_: u32) -> MluaEngine {
    common::setup::<MluaEngine>()
}

#[library_benchmark(setup = setup)]
#[bench::sort(0)]
fn sort_userdata(mut engine: MluaEngine) {
    common::run(&mut engine);
}

library_benchmark_group!(
    name = mlua_lua55_group,
    benchmarks = sort_userdata
);

main!(library_benchmark_groups = mlua_lua55_group);
