use anyhow::Result;

use crate::harness::{ensure_close, expect_float, ResultKind, Scripts, Value};
use crate::interop::host;
use crate::interop::{
    BulkMode, BulkOp, BulkValue, Direction, InteropCase, InteropOp,
};

const LIST_SCRIPTS: Scripts = Scripts {
    js: Some("scripts/interop_bulk.js"),
    koto: Some("scripts/interop_bulk.koto"),
    lua: Some("scripts/interop_bulk.lua"),
    martite_script: Some("scripts/interop_bulk.mt"),
    martite_interface: Some("scripts/interop_bulk.mti"),
    rhai: Some("scripts/interop_bulk.rhai"),
    roto: None,
    wasm_source: None,
    wasm_module: None,
};

macro_rules! list_case {
    ($constant:ident, $name:literal, $entry:literal, $value:expr, $direction:expr, $size:expr, $native:ident, $validate:ident) => {
        pub const $constant: InteropCase = InteropCase {
            name: $name,
            op: InteropOp::Bulk(BulkOp {
                value: $value,
                direction: $direction,
                mode: BulkMode::List,
                size: $size,
            }),
            entry: $entry,
            scripts: LIST_SCRIPTS,
            result: ResultKind::Float,
            native: $native,
            validate: $validate,
        };
    };
}

list_case!(
    F64_IN_1K,
    "Interop bulk f64 in list 1k",
    "bench_bulk_f64_in_list",
    BulkValue::F64,
    Direction::In,
    1_000,
    native_f64_1k,
    validate_1k
);
list_case!(
    F64_IN_100K,
    "Interop bulk f64 in list 100k",
    "bench_bulk_f64_in_list",
    BulkValue::F64,
    Direction::In,
    100_000,
    native_f64_100k,
    validate_100k
);
list_case!(
    F64_IN_1M,
    "Interop bulk f64 in list 1M",
    "bench_bulk_f64_in_list",
    BulkValue::F64,
    Direction::In,
    1_000_000,
    native_f64_1m,
    validate_1m
);
list_case!(
    F64_OUT_1K,
    "Interop bulk f64 out list 1k",
    "bench_bulk_f64_out_list_1k",
    BulkValue::F64,
    Direction::Out,
    1_000,
    native_f64_1k,
    validate_1k
);
list_case!(
    F64_OUT_100K,
    "Interop bulk f64 out list 100k",
    "bench_bulk_f64_out_list_100k",
    BulkValue::F64,
    Direction::Out,
    100_000,
    native_f64_100k,
    validate_100k
);
list_case!(
    F64_OUT_1M,
    "Interop bulk f64 out list 1M",
    "bench_bulk_f64_out_list_1m",
    BulkValue::F64,
    Direction::Out,
    1_000_000,
    native_f64_1m,
    validate_1m
);
list_case!(
    U8_IN_1K,
    "Interop bulk u8 in list 1k",
    "bench_bulk_u8_in_list",
    BulkValue::U8,
    Direction::In,
    1_000,
    native_u8_1k,
    validate_1k
);
list_case!(
    U8_IN_100K,
    "Interop bulk u8 in list 100k",
    "bench_bulk_u8_in_list",
    BulkValue::U8,
    Direction::In,
    100_000,
    native_u8_100k,
    validate_100k
);
list_case!(
    U8_IN_1M,
    "Interop bulk u8 in list 1M",
    "bench_bulk_u8_in_list",
    BulkValue::U8,
    Direction::In,
    1_000_000,
    native_u8_1m,
    validate_1m
);
list_case!(
    U8_OUT_1K,
    "Interop bulk u8 out list 1k",
    "bench_bulk_u8_out_list_1k",
    BulkValue::U8,
    Direction::Out,
    1_000,
    native_u8_1k,
    validate_1k
);
list_case!(
    U8_OUT_100K,
    "Interop bulk u8 out list 100k",
    "bench_bulk_u8_out_list_100k",
    BulkValue::U8,
    Direction::Out,
    100_000,
    native_u8_100k,
    validate_100k
);
list_case!(
    U8_OUT_1M,
    "Interop bulk u8 out list 1M",
    "bench_bulk_u8_out_list_1m",
    BulkValue::U8,
    Direction::Out,
    1_000_000,
    native_u8_1m,
    validate_1m
);

fn native_f64_1k() {
    host::native_bulk_f64(1_000);
}
fn native_f64_100k() {
    host::native_bulk_f64(100_000);
}
fn native_f64_1m() {
    host::native_bulk_f64(1_000_000);
}
fn native_u8_1k() {
    host::native_bulk_u8(1_000);
}
fn native_u8_100k() {
    host::native_bulk_u8(100_000);
}
fn native_u8_1m() {
    host::native_bulk_u8(1_000_000);
}

fn validate_1k(value: &Value) -> Result<()> {
    ensure_close(expect_float(value)?, host::expected_bulk(1_000), 1e-9)
}
fn validate_100k(value: &Value) -> Result<()> {
    ensure_close(expect_float(value)?, host::expected_bulk(100_000), 1e-9)
}
fn validate_1m(value: &Value) -> Result<()> {
    ensure_close(expect_float(value)?, host::expected_bulk(1_000_000), 1e-9)
}
