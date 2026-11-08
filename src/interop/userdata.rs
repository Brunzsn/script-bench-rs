use anyhow::{ensure, Result};

use crate::harness::{expect_int, ResultKind, Scripts, Value};
use crate::interop::host;
use crate::interop::{InteropCase, InteropOp, UserdataOp};

const SCRIPTS: Scripts = Scripts {
    js: Some("scripts/interop_userdata.js"),
    koto: Some("scripts/interop_userdata.koto"),
    lua: Some("scripts/interop_userdata.lua"),
    martite_script: Some("scripts/interop_userdata.mt"),
    martite_interface: Some("scripts/interop_userdata.mti"),
    rhai: Some("scripts/interop_userdata.rhai"),
    roto: Some("scripts/interop_userdata.roto"),
    wasm_source: None,
    wasm_module: None,
};

const NO_FIELDS: Scripts = Scripts {
    koto: None,
    roto: None,
    martite_script: None,
    martite_interface: None,
    ..SCRIPTS
};

const NO_MARTITE_COPY: Scripts = Scripts {
    martite_script: None,
    martite_interface: None,
    ..SCRIPTS
};

pub const METHOD: InteropCase = InteropCase {
    name: "Interop userdata method",
    op: InteropOp::Userdata(UserdataOp::Method),
    entry: "bench_userdata_method",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_userdata_method,
    validate: validate_method,
};

pub const FIELD_GET: InteropCase = InteropCase {
    name: "Interop userdata field get",
    op: InteropOp::Userdata(UserdataOp::FieldGet),
    entry: "bench_userdata_field_get",
    scripts: NO_FIELDS,
    result: ResultKind::Int,
    native: host::native_userdata_field_get,
    validate: validate_field_get,
};

pub const FIELD_SET: InteropCase = InteropCase {
    name: "Interop userdata field set",
    op: InteropOp::Userdata(UserdataOp::FieldSet),
    entry: "bench_userdata_field_set",
    scripts: NO_FIELDS,
    result: ResultKind::Int,
    native: host::native_userdata_field_set,
    validate: validate_field_set,
};

pub const CTOR_COPY: InteropCase = InteropCase {
    name: "Interop userdata constructor copy",
    op: InteropOp::Userdata(UserdataOp::ConstructorCopy),
    entry: "bench_userdata_ctor_copy",
    scripts: NO_MARTITE_COPY,
    result: ResultKind::Int,
    native: host::native_userdata_ctor_copy,
    validate: validate_ctor,
};

pub const CTOR_RC: InteropCase = InteropCase {
    name: "Interop userdata constructor rc",
    op: InteropOp::Userdata(UserdataOp::ConstructorRc),
    entry: "bench_userdata_ctor_rc",
    scripts: SCRIPTS,
    result: ResultKind::Int,
    native: host::native_userdata_ctor_rc,
    validate: validate_ctor,
};

fn validate_method(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::USERDATA_VALUE * host::USERDATA_ITERATIONS as i64,
        "unexpected userdata method checksum"
    );
    Ok(())
}

fn validate_field_get(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::USERDATA_VALUE * host::USERDATA_ITERATIONS as i64,
        "unexpected userdata field-get checksum"
    );
    Ok(())
}

fn validate_field_set(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::USERDATA_ITERATIONS as i64 - 1,
        "unexpected userdata field-set checksum"
    );
    Ok(())
}

fn validate_ctor(value: &Value) -> Result<()> {
    ensure!(
        expect_int(value)? == host::USERDATA_ITERATIONS as i64,
        "unexpected userdata construction count"
    );
    Ok(())
}
