use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const ITEM_COUNT: usize = 10_000;
const MODULUS: i64 = 1_000_000_007;
const LCG_MODULUS: i64 = 2_147_483_647;
const LCG_MULTIPLIER: i64 = 48_271;
const LCG_SEED: i64 = 12_345;

pub const CASE: Case = Case {
    name: "Quicksort integers",
    scripts: Scripts {
        js: Some("scripts/quicksort_ints.js"),
        koto: Some("scripts/quicksort_ints.koto"),
        lua: Some("scripts/quicksort_ints.lua"),
        martite_script: Some("scripts/quicksort_ints.mt"),
        martite_interface: Some("scripts/quicksort_ints.mti"),
        rhai: Some("scripts/quicksort_ints.rhai"),
        roto: Some("scripts/quicksort_ints.roto"),
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn partition(values: &mut [i64], lo: usize, hi: usize) -> usize {
    let pivot_index = (lo + hi) / 2;
    values.swap(pivot_index, hi);
    let pivot = values[hi];
    let mut split = lo;
    for index in lo..hi {
        if values[index] < pivot {
            values.swap(index, split);
            split += 1;
        }
    }
    values.swap(split, hi);
    split
}

fn quicksort(values: &mut [i64], lo: usize, hi: usize) {
    if lo >= hi {
        return;
    }
    let pivot = partition(values, lo, hi);
    if pivot > 0 {
        quicksort(values, lo, pivot - 1);
    }
    quicksort(values, pivot + 1, hi);
}

fn reference() -> i64 {
    let mut values = Vec::with_capacity(ITEM_COUNT);
    let mut seed = LCG_SEED;
    for _ in 0..ITEM_COUNT {
        seed = (seed * LCG_MULTIPLIER) % LCG_MODULUS;
        values.push(seed);
    }

    let last = values.len() - 1;
    quicksort(&mut values, 0, last);

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
