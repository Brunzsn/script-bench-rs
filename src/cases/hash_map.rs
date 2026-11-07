use std::collections::HashMap;
use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const COUNT: i64 = 20_000;
const MODULUS: i64 = 1_000_000_007;

pub const CASE: Case = Case {
    name: "Hash map",
    scripts: Scripts {
        js: Some("scripts/hash_map.js"),
        koto: Some("scripts/hash_map.koto"),
        lua: Some("scripts/hash_map.lua"),
        martite_script: Some("scripts/hash_map.mt"),
        martite_interface: Some("scripts/hash_map.mti"),
        rhai: Some("scripts/hash_map.rhai"),
        roto: None,
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn reference() -> i64 {
    let mut map = HashMap::new();
    for index in 0..COUNT {
        map.insert(format!("key{index}"), index * 3);
    }

    let mut looked_up = 0i64;
    for index in 0..COUNT {
        looked_up += map[&format!("key{index}")];
    }

    let mut iterated = 0i64;
    for value in map.values() {
        iterated += *value;
    }

    (looked_up + iterated + map.len() as i64) % MODULUS
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
