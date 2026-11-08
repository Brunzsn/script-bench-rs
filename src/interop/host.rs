//! Shared host-side helpers used by both the script bindings and the native
//! baseline. Keeping the payloads in one place prevents the engines' scripts
//! and the reference implementation from drifting apart.

use std::hint::black_box;

/// Host calls performed by the script-side call-overhead cases.
pub const CALL_ITERATIONS: usize = 100_000;

/// Iterations used by the Rust-driven call-in and callback cases.
pub const DRIVEN_ITERATIONS: usize = 100_000;

/// Iterations used by userdata cases.
pub const USERDATA_ITERATIONS: usize = 100_000;

/// Iterations used by serde/error cases.
pub const SMALL_ITERATIONS: usize = 10_000;

pub const STRING_IN: &str = "interop";
pub const STRING_OUT: &str = "interop";
pub const I64_ONE: i64 = 7;
pub const I64_ARGS_3: [i64; 3] = [1, 2, 3];
pub const I64_ARGS_5: [i64; 5] = [1, 2, 3, 4, 5];
pub const F64_ARG: f64 = 1.5;
pub const CALLBACK_INCREMENT: i64 = 1;
pub const HOST_ERROR: &str = "interop error";

#[inline(never)]
pub fn ping() -> i64 {
    1
}

#[inline(never)]
pub fn id_i64(value: i64) -> i64 {
    value
}

#[inline(never)]
pub fn sum_i64_3(a: i64, b: i64, c: i64) -> i64 {
    a + b + c
}

#[inline(never)]
pub fn sum_i64_5(a: i64, b: i64, c: i64, d: i64, e: i64) -> i64 {
    a + b + c + d + e
}

#[inline(never)]
pub fn id_f64(value: f64) -> f64 {
    value
}

#[inline(never)]
pub fn strlen(value: &str) -> i64 {
    value.chars().count() as i64
}

#[inline(never)]
pub fn string_out() -> String {
    STRING_OUT.to_string()
}

#[inline(never)]
pub fn callback_value(value: i64) -> i64 {
    value + CALLBACK_INCREMENT
}

#[inline(never)]
pub fn failing_call() -> Result<i64, String> {
    Err(HOST_ERROR.to_string())
}

/// Runs a closure-shaped native reference and keeps its result observable.
pub fn keep<T>(value: T) {
    black_box(value);
}

/// Expected result of the zero-argument call loop.
pub fn expected_call_empty() -> i64 {
    CALL_ITERATIONS as i64 * ping()
}

/// Expected result of the one-argument i64 call loop.
pub fn expected_call_one_i64() -> i64 {
    CALL_ITERATIONS as i64 * id_i64(I64_ONE)
}

/// Expected result of the three-argument i64 call loop.
pub fn expected_call_three_i64() -> i64 {
    CALL_ITERATIONS as i64 * sum_i64_3(I64_ARGS_3[0], I64_ARGS_3[1], I64_ARGS_3[2])
}

/// Expected result of the five-argument i64 call loop.
pub fn expected_call_five_i64() -> i64 {
    CALL_ITERATIONS as i64
        * sum_i64_5(
            I64_ARGS_5[0],
            I64_ARGS_5[1],
            I64_ARGS_5[2],
            I64_ARGS_5[3],
            I64_ARGS_5[4],
        )
}

/// Expected result of the f64 call loop.
pub fn expected_call_f64() -> f64 {
    CALL_ITERATIONS as f64 * id_f64(F64_ARG)
}

/// Expected result of the string-in call loop.
pub fn expected_call_string_in() -> i64 {
    CALL_ITERATIONS as i64 * strlen(STRING_IN)
}

/// Expected result of the string-out call loop.
pub fn expected_call_string_out() -> i64 {
    CALL_ITERATIONS as i64 * strlen(&string_out())
}

/// Native baseline for the zero-argument call loop.
pub fn native_call_empty() {
    let mut sum = 0i64;
    for _ in 0..CALL_ITERATIONS {
        sum = sum.wrapping_add(ping());
    }
    keep(sum);
}

/// Native baseline for the one-argument i64 call loop.
pub fn native_call_one_i64() {
    let mut sum = 0i64;
    for _ in 0..CALL_ITERATIONS {
        sum = sum.wrapping_add(id_i64(I64_ONE));
    }
    keep(sum);
}

/// Native baseline for the three-argument i64 call loop.
pub fn native_call_three_i64() {
    let mut sum = 0i64;
    for _ in 0..CALL_ITERATIONS {
        sum = sum.wrapping_add(sum_i64_3(
            I64_ARGS_3[0],
            I64_ARGS_3[1],
            I64_ARGS_3[2],
        ));
    }
    keep(sum);
}

/// Native baseline for the five-argument i64 call loop.
pub fn native_call_five_i64() {
    let mut sum = 0i64;
    for _ in 0..CALL_ITERATIONS {
        sum = sum.wrapping_add(sum_i64_5(
            I64_ARGS_5[0],
            I64_ARGS_5[1],
            I64_ARGS_5[2],
            I64_ARGS_5[3],
            I64_ARGS_5[4],
        ));
    }
    keep(sum);
}

/// Native baseline for the f64 call loop.
pub fn native_call_f64() {
    let mut sum = 0.0f64;
    for _ in 0..CALL_ITERATIONS {
        sum += id_f64(F64_ARG);
    }
    keep(sum);
}

/// Native baseline for the string-in call loop.
pub fn native_call_string_in() {
    let mut sum = 0i64;
    for _ in 0..CALL_ITERATIONS {
        sum = sum.wrapping_add(strlen(STRING_IN));
    }
    keep(sum);
}

/// Native baseline for the string-out call loop.
pub fn native_call_string_out() {
    let mut sum = 0i64;
    for _ in 0..CALL_ITERATIONS {
        sum = sum.wrapping_add(strlen(&string_out()));
    }
    keep(sum);
}
