use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const ITERATIONS: i64 = 10_000_000;
const MODULUS: i64 = 1_000_000_007;

pub const CASE: Case = Case {
    name: "Integer loop",
    scripts: Scripts {
        js: Some("scripts/int_loop.js"),
        koto: Some("scripts/int_loop.koto"),
        lua: Some("scripts/int_loop.lua"),
        martite_script: Some("scripts/int_loop.mt"),
        martite_interface: Some("scripts/int_loop.mti"),
        rhai: Some("scripts/int_loop.rhai"),
        roto: Some("scripts/int_loop.roto"),
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn reference() -> i64 {
    let mut value = 12_345i64;
    for index in 0..ITERATIONS {
        value = (value * 31 + index) % MODULUS;
    }
    value
}

fn native() {
    black_box(reference());
}

fn validate(value: &Value) -> Result<()> {
    let actual = expect_int(value)?;
    let expected = reference();
    if actual != expected {
        bail!("expected {expected}, got {actual}");
    }
    Ok(())
}
