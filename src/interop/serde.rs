use anyhow::{ensure, Result};

use crate::harness::{expect_int, ResultKind, Scripts, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropOp, SerdeOp};

const SCRIPTS: Scripts = Scripts {
    js: Some("scripts/interop_serde.js"),
    koto: Some("scripts/interop_serde.koto"),
    lua: Some("scripts/interop_serde.lua"),
    martite_script: None,
    martite_interface: None,
    rhai: Some("scripts/interop_serde.rhai"),
    roto: None,
    wasm_source: None,
    wasm_module: None,
};

const MARTITE_FROM_SCRIPTS: Scripts = Scripts {
    martite_script: Some("scripts/interop_serde.mt"),
    martite_interface: Some("scripts/interop_serde.mti"),
    ..SCRIPTS
};

pub const TO_SCRIPT: InteropCase = InteropCase {
    name: "Interop serde nested to script",
    op: InteropOp::Serde(SerdeOp::ToScript),
    entry: "bench_serde_to_script",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_serde,
    validate: validate_serde,
};

pub const FROM_SCRIPT: InteropCase = InteropCase {
    name: "Interop serde nested from script",
    op: InteropOp::Serde(SerdeOp::FromScript),
    entry: "bench_serde_from_script",
    scripts: MARTITE_FROM_SCRIPTS,
    result: ResultKind::Int,
    native: host::native_serde,
    validate: validate_serde,
};

fn validate_serde(value: &Value) -> Result<()> {
    let actual = expect_int(value)?;
    let expected = host::expected_serde();
    ensure!(
        actual == expected,
        "unexpected serde checksum: got {actual}, expected {expected}"
    );
    Ok(())
}
