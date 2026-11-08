use anyhow::Result;

use crate::harness::{ensure_close, expect_float, ResultKind, Scripts, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropOp};

const SCRIPTS: Scripts = Scripts {
    js: Some("scripts/interop_call_in.js"),
    koto: Some("scripts/interop_call_in.koto"),
    lua: Some("scripts/interop_call_in.lua"),
    martite_script: Some("scripts/interop_call_in.mt"),
    martite_interface: Some("scripts/interop_call_in.mti"),
    rhai: Some("scripts/interop_call_in.rhai"),
    roto: Some("scripts/interop_call_in.roto"),
    wasm_source: None,
    wasm_module: None,
};

pub const CALL_IN: InteropCase = InteropCase {
    name: "Interop call-in update",
    op: InteropOp::CallIn {
        iterations: host::DRIVEN_ITERATIONS,
    },
    entry: "update",
    scripts: SCRIPTS,
    result: ResultKind::Float,
    native: host::native_call_in,
    validate: validate_call_in,
};

fn validate_call_in(value: &Value) -> Result<()> {
    ensure_close(expect_float(value)?, host::expected_call_in(), 1e-9)
}
