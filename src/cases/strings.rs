use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const COUNT: i64 = 5_000;
const UNICODE: &str = "héllo—世界🚀";

pub const CASE: Case = Case {
    name: "Strings",
    scripts: Scripts {
        js: Some("scripts/strings.js"),
        koto: Some("scripts/strings.koto"),
        lua: Some("scripts/strings.lua"),
        martite_script: Some("scripts/strings.mt"),
        martite_interface: Some("scripts/strings.mti"),
        rhai: Some("scripts/strings.rhai"),
        roto: Some("scripts/strings.roto"),
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn reference() -> i64 {
    let mut parts = Vec::with_capacity(COUNT as usize);
    for index in 0..COUNT {
        parts.push(format!("item-{index}"));
    }

    let joined = parts.join(",");
    let split: Vec<&str> = joined.split(',').collect();
    let pipe_joined = split.join("|");
    let replaced = pipe_joined.replace("item-", "row-");
    let marked = replaced.replace("row-4999", "last");
    let unicode_len = UNICODE.chars().count() as i64;

    let mut checksum =
        (joined.len() + pipe_joined.len() + marked.len()) as i64 + split.len() as i64;
    checksum += unicode_len;
    if marked.contains("last") {
        checksum += 1;
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
