use std::hint::black_box;

use anyhow::Result;

use crate::harness::{ensure_close, expect_float, Case, ResultKind, Scripts, Value};

const ITERATIONS: i64 = 5_000_000;

pub const CASE: Case = Case {
    name: "Float loop",
    scripts: Scripts {
        js: Some("scripts/float_loop.js"),
        koto: Some("scripts/float_loop.koto"),
        lua: Some("scripts/float_loop.lua"),
        martite_script: Some("scripts/float_loop.mt"),
        martite_interface: Some("scripts/float_loop.mti"),
        rhai: Some("scripts/float_loop.rhai"),
        roto: Some("scripts/float_loop.roto"),
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Float,
    native,
    validate,
};

fn reference() -> f64 {
    let mut value = 1.0f64;
    for _ in 0..ITERATIONS {
        value = value * 0.999_999 + 0.000_001;
    }
    value
}

fn native() {
    black_box(reference());
}

fn validate(value: &Value) -> Result<()> {
    let actual = expect_float(value)?;
    ensure_close(actual, reference(), 1e-9)
}
