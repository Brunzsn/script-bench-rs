use anyhow::{ensure, Result};

use crate::harness::{expect_int, ResultKind, Scripts, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropOp};

const SCRIPTS: Scripts = Scripts {
    js: Some("scripts/interop_callback.js"),
    koto: Some("scripts/interop_callback.koto"),
    lua: Some("scripts/interop_callback.lua"),
    martite_script: Some("scripts/interop_callback.mt"),
    martite_interface: Some("scripts/interop_callback.mti"),
    rhai: Some("scripts/interop_callback.rhai"),
    roto: None,
    wasm_source: None,
    wasm_module: None,
};

pub const CALLBACK: InteropCase = InteropCase {
    name: "Interop callback reentry",
    op: InteropOp::Callback {
        iterations: host::DRIVEN_ITERATIONS,
    },
    entry: "make_callback",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_callback,
    validate: validate_callback,
};

fn validate_callback(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_callback(),
        "unexpected callback checksum"
    );
    Ok(())
}
