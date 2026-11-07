use std::hint::black_box;

use anyhow::Result;

use crate::harness::{ensure_close, expect_float, Case, ResultKind, Scripts, Value};

const SIZE: i64 = 400;
const MAX_ITERATIONS: i64 = 50;

pub const CASE: Case = Case {
    name: "Mandelbrot",
    scripts: Scripts {
        js: Some("scripts/mandelbrot.js"),
        koto: Some("scripts/mandelbrot.koto"),
        lua: Some("scripts/mandelbrot.lua"),
        martite_script: Some("scripts/mandelbrot.mt"),
        martite_interface: Some("scripts/mandelbrot.mti"),
        rhai: Some("scripts/mandelbrot.rhai"),
        roto: Some("scripts/mandelbrot.roto"),
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Float,
    native,
    validate,
};

fn reference() -> f64 {
    let mut checksum = 0.0f64;
    for y in 0..SIZE {
        let ci = 2.0 * y as f64 / SIZE as f64 - 1.0;
        for x in 0..SIZE {
            let cr = 2.0 * x as f64 / SIZE as f64 - 1.5;
            let mut zr = 0.0f64;
            let mut zi = 0.0f64;
            let mut iteration = 0i64;
            while iteration < MAX_ITERATIONS && zr * zr + zi * zi <= 4.0 {
                let next_zr = zr * zr - zi * zi + cr;
                zi = 2.0 * zr * zi + ci;
                zr = next_zr;
                iteration += 1;
            }
            checksum += iteration as f64;
        }
    }
    checksum
}

fn native() {
    black_box(reference());
}

fn validate(value: &Value) -> Result<()> {
    let actual = expect_float(value)?;
    ensure_close(actual, reference(), 1e-3)
}
