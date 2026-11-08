use anyhow::{ensure, Result};

use crate::harness::{ensure_close, expect_float, expect_int, ResultKind, Scripts, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropOp};

const SCRIPTS: Scripts = Scripts {
    js: Some("scripts/interop_calls.js"),
    koto: Some("scripts/interop_calls.koto"),
    lua: Some("scripts/interop_calls.lua"),
    martite_script: Some("scripts/interop_calls.mt"),
    martite_interface: Some("scripts/interop_calls.mti"),
    rhai: Some("scripts/interop_calls.rhai"),
    roto: Some("scripts/interop_calls.roto"),
    wasm_source: None,
    wasm_module: None,
};

pub const CALL_EMPTY: InteropCase = InteropCase {
    name: "Interop call 0 args",
    op: InteropOp::Call,
    entry: "bench_call_empty",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_call_empty,
    validate: validate_call_empty,
};

pub const CALL_ONE_I64: InteropCase = InteropCase {
    name: "Interop call 1 i64",
    op: InteropOp::Call,
    entry: "bench_call_one_i64",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_call_one_i64,
    validate: validate_call_one_i64,
};

pub const CALL_THREE_I64: InteropCase = InteropCase {
    name: "Interop call 3 i64",
    op: InteropOp::Call,
    entry: "bench_call_three_i64",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_call_three_i64,
    validate: validate_call_three_i64,
};

pub const CALL_FIVE_I64: InteropCase = InteropCase {
    name: "Interop call 5 i64",
    op: InteropOp::Call,
    entry: "bench_call_five_i64",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_call_five_i64,
    validate: validate_call_five_i64,
};

pub const CALL_F64: InteropCase = InteropCase {
    name: "Interop call f64",
    op: InteropOp::Call,
    entry: "bench_call_f64",
    scripts: SCRIPTS,
    result: ResultKind::Float,
    native: host::native_call_f64,
    validate: validate_call_f64,
};

pub const CALL_STRING_IN: InteropCase = InteropCase {
    name: "Interop call string in",
    op: InteropOp::Call,
    entry: "bench_call_string_in",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_call_string_in,
    validate: validate_call_string_in,
};

pub const CALL_STRING_OUT: InteropCase = InteropCase {
    name: "Interop call string out",
    op: InteropOp::Call,
    entry: "bench_call_string_out",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_call_string_out,
    validate: validate_call_string_out,
};

fn validate_call_empty(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_call_empty(),
        "unexpected zero-argument call checksum"
    );
    Ok(())
}

fn validate_call_one_i64(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_call_one_i64(),
        "unexpected one-argument call checksum"
    );
    Ok(())
}

fn validate_call_three_i64(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_call_three_i64(),
        "unexpected three-argument call checksum"
    );
    Ok(())
}

fn validate_call_five_i64(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_call_five_i64(),
        "unexpected five-argument call checksum"
    );
    Ok(())
}

fn validate_call_f64(value: &Value) -> Result<()> {
    ensure_close(
        expect_float(value)?,
        host::expected_call_f64(),
        1e-9,
    )
}

fn validate_call_string_in(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_call_string_in(),
        "unexpected string-in call checksum"
    );
    Ok(())
}

fn validate_call_string_out(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::expected_call_string_out(),
        "unexpected string-out call checksum"
    );
    Ok(())
}
