use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const COUNT: i64 = 10_000;
const PASSES: i64 = 50;
const MODULUS: i64 = 1_000_000_007;

pub const CASE: Case = Case {
    name: "Closures",
    scripts: Scripts {
        js: Some("scripts/closures.js"),
        koto: Some("scripts/closures.koto"),
        lua: Some("scripts/closures.lua"),
        martite_script: Some("scripts/closures.mt"),
        martite_interface: Some("scripts/closures.mti"),
        rhai: Some("scripts/closures.rhai"),
        roto: None,
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn reference() -> i64 {
    let values: Vec<i64> = (0..COUNT).collect();
    let mut checksum = 0i64;
    for _ in 0..PASSES {
        let mapped: Vec<i64> = values.iter().map(|value| value * 2).collect();
        let filtered: Vec<i64> = mapped.into_iter().filter(|value| value % 3 == 0).collect();
        let sum: i64 = filtered.into_iter().sum();
        checksum = (checksum + sum) % MODULUS;
    }
    checksum
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
