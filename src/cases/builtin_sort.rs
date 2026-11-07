use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const ITEM_COUNT: usize = 20_000;
const MODULUS: i64 = 1_000_000_007;
const LCG_MODULUS: i64 = 2_147_483_647;
const LCG_MULTIPLIER: i64 = 48_271;
const LCG_SEED: i64 = 54_321;

pub const CASE: Case = Case {
    name: "Builtin sort",
    scripts: Scripts {
        js: Some("scripts/builtin_sort.js"),
        koto: Some("scripts/builtin_sort.koto"),
        lua: Some("scripts/builtin_sort.lua"),
        martite_script: Some("scripts/builtin_sort.mt"),
        martite_interface: Some("scripts/builtin_sort.mti"),
        rhai: Some("scripts/builtin_sort.rhai"),
        roto: None,
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn reference() -> i64 {
    let mut values = Vec::with_capacity(ITEM_COUNT);
    let mut seed = LCG_SEED;
    for _ in 0..ITEM_COUNT {
        seed = (seed * LCG_MULTIPLIER) % LCG_MODULUS;
        values.push(seed);
    }

    values.sort_by(|left, right| right.cmp(left));

    let mut checksum = 0i64;
    for (index, value) in values.iter().enumerate() {
        checksum = (checksum + value * (index as i64 + 1)) % MODULUS;
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
