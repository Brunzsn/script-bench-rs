mod common;

use gungraun::prelude::*;
use script_bench::wasm::WasmEngine;

fn setup(_: u32) -> WasmEngine {
    common::setup::<WasmEngine>()
}

#[library_benchmark(setup = setup)]
#[bench::sort(0)]
fn sort_userdata(mut engine: WasmEngine) {
    common::run(&mut engine);
}

library_benchmark_group!(
    name = wasmtime_group,
    benchmarks = sort_userdata
);

main!(library_benchmark_groups = wasmtime_group);
