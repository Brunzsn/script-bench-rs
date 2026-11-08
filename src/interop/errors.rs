use anyhow::{ensure, Result};

use crate::harness::{expect_int, ResultKind, Scripts, Value};
use crate::interop::host;
use crate::interop::{ErrorOp, InteropCase, InteropOp};

const SCRIPTS: Scripts = Scripts {
    js: Some("scripts/interop_errors.js"),
    koto: Some("scripts/interop_errors.koto"),
    lua: Some("scripts/interop_errors.lua"),
    martite_script: Some("scripts/interop_errors.mt"),
    martite_interface: Some("scripts/interop_errors.mti"),
    rhai: Some("scripts/interop_errors.rhai"),
    roto: None,
    wasm_source: None,
    wasm_module: None,
};

pub const ERROR_CAUGHT: InteropCase = InteropCase {
    name: "Interop error caught in script",
    op: InteropOp::Error(ErrorOp::CaughtInScript),
    entry: "bench_error_caught",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_error_caught,
    validate: validate_error_caught,
};

pub const ERROR_PROPAGATES: InteropCase = InteropCase {
    name: "Interop error propagates to host",
    op: InteropOp::Error(ErrorOp::PropagatesToHost),
    entry: "bench_error_propagates",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_error_propagates,
    validate: validate_error_propagates,
};

fn validate_error_caught(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_error_caught(),
        "unexpected caught-error count"
    );
    Ok(())
}

fn validate_error_propagates(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == 1,
        "expected exactly one propagated error"
    );
    Ok(())
}
