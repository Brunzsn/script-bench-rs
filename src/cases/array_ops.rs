use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const COUNT: i64 = 100_000;
const MODULUS: i64 = 1_000_000_007;

pub const CASE: Case = Case {
    name: "Array operations",
    scripts: Scripts {
        js: Some("scripts/array_ops.js"),
        koto: Some("scripts/array_ops.koto"),
        lua: Some("scripts/array_ops.lua"),
        martite_script: Some("scripts/array_ops.mt"),
        martite_interface: Some("scripts/array_ops.mti"),
        rhai: Some("scripts/array_ops.rhai"),
        roto: Some("scripts/array_ops.roto"),
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn reference() -> i64 {
    let mut values = Vec::with_capacity(COUNT as usize);
    for index in 0..COUNT {
        values.push(index);
    }

    let mut indexed_sum = 0i64;
    for index in 0..COUNT {
        indexed_sum += values[index as usize];
    }

    let mut iterated_sum = 0i64;
    for value in &values {
        iterated_sum += *value;
    }

    (indexed_sum + iterated_sum) % MODULUS
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
