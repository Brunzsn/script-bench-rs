use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const N: i64 = 32;

pub const CASE: Case = Case {
    name: "Fib (recursive)",
    scripts: Scripts {
        js: Some("scripts/fib.js"),
        koto: Some("scripts/fib.koto"),
        lua: Some("scripts/fib.lua"),
        martite_script: Some("scripts/fib.mt"),
        martite_interface: Some("scripts/fib.mti"),
        rhai: Some("scripts/fib.rhai"),
        roto: Some("scripts/fib.roto"),
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn fib(n: i64) -> i64 {
    if n < 2 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

fn reference() -> i64 {
    fib(N)
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
