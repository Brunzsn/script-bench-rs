use std::hint::black_box;

use anyhow::Result;

use crate::harness::{ensure_close, expect_float, Case, ResultKind, Scripts, Value};

const N: usize = 300;
const ITERATIONS: usize = 10;

pub const CASE: Case = Case {
    name: "Spectral norm",
    scripts: Scripts {
        js: Some("scripts/spectral_norm.js"),
        koto: Some("scripts/spectral_norm.koto"),
        lua: Some("scripts/spectral_norm.lua"),
        martite_script: Some("scripts/spectral_norm.mt"),
        martite_interface: Some("scripts/spectral_norm.mti"),
        rhai: Some("scripts/spectral_norm.rhai"),
        roto: None,
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Float,
    native,
    validate,
};

fn eval_a(i: usize, j: usize) -> f64 {
    let i = i as f64;
    let j = j as f64;
    1.0 / ((i + j) * (i + j + 1.0) / 2.0 + i + 1.0)
}

fn multiply_av(vector: &[f64], output: &mut [f64]) {
    for i in 0..N {
        let mut sum = 0.0f64;
        for j in 0..N {
            sum += eval_a(i, j) * vector[j];
        }
        output[i] = sum;
    }
}

fn multiply_atv(vector: &[f64], output: &mut [f64]) {
    for value in output.iter_mut() {
        *value = 0.0;
    }
    for i in 0..N {
        for j in 0..N {
            output[j] += eval_a(i, j) * vector[i];
        }
    }
}

fn reference() -> f64 {
    let mut u = vec![1.0f64; N];
    let mut v = vec![0.0f64; N];

    for _ in 0..ITERATIONS {
        multiply_av(&u, &mut v);
        multiply_atv(&v, &mut u);
    }

    multiply_av(&u, &mut v);

    let mut v_bv = 0.0f64;
    let mut vv = 0.0f64;
    for i in 0..N {
        v_bv += u[i] * v[i];
        vv += v[i] * v[i];
    }

    (v_bv / vv).sqrt()
}

fn native() {
    black_box(reference());
}

fn validate(value: &Value) -> Result<()> {
    let actual = expect_float(value)?;
    ensure_close(actual, reference(), 1e-9)
}
