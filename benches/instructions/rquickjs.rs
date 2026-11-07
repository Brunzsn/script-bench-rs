mod common;

use gungraun::prelude::*;
use script_bench::rquickjs::QuickJsEngine;

fn setup(_: u32) -> QuickJsEngine {
    common::setup::<QuickJsEngine>()
}

#[library_benchmark(setup = setup)]
#[bench::sort(0)]
fn sort_userdata(mut engine: QuickJsEngine) {
    common::run(&mut engine);
}

library_benchmark_group!(
    name = rquickjs_group,
    benchmarks = sort_userdata
);

main!(library_benchmark_groups = rquickjs_group);
