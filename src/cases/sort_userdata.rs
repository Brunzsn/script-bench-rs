use std::hint::black_box;
use std::rc::Rc;

use anyhow::{bail, ensure, Result};

use crate::harness::{Case, Scripts, Value};
use crate::rng::DetRng;

const ITEM_COUNT: usize = 10_000;
const CHARSET: &[u8; 16] = b"0123456789abcdef";
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub const CASE: Case = Case {
    name: "Sort Rust objects",
    scripts: Scripts {
        js: "scripts/sort_userdata.js",
        koto: "scripts/sort_userdata.koto",
        lua: "scripts/sort_userdata.lua",
        martite_script: "scripts/sort_userdata.mt",
        martite_interface: "scripts/sort_userdata.mti",
        rhai: "scripts/sort_userdata.rhai",
        roto: "scripts/sort_userdata.roto",
        wasm_source: "scripts/sort_userdata.wasm.rs",
        wasm_module: "scripts/sort_userdata.wasm",
    },
    native,
    validate,
};

/// Builds the canonical unsorted input shared by the scripts and the native
/// baseline. Keep this in sync with `scripts/sort_userdata.*`.
fn reference_unsorted() -> Vec<Rc<str>> {
    let mut rng = DetRng::new();
    let mut values = Vec::with_capacity(ITEM_COUNT);

    for _ in 0..ITEM_COUNT {
        let len = 8 + rng.below(16) as usize;
        let mut value = String::with_capacity(len);
        for _ in 0..len {
            let index = rng.below(CHARSET.len() as u32) as usize;
            value.push(CHARSET[index] as char);
        }
        values.push(Rc::from(value));
    }

    values
}

fn reference_sorted() -> Vec<Rc<str>> {
    let mut values = reference_unsorted();
    values.sort();
    values
}

/// Native Rust reference implementation used for the baseline benchmark.
fn native() {
    let values = reference_sorted();
    black_box(values);
}

fn checksum_add(mut hash: u64, value: &str) -> u64 {
    hash ^= value.len() as u64;
    hash = hash.wrapping_mul(FNV_PRIME);
    for byte in value.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

fn validate(value: &Value) -> Result<()> {
    let Value::List(items) = value else {
        bail!("expected the script to return a list, got {value:?}");
    };
    ensure!(
        items.len() == ITEM_COUNT,
        "expected {ITEM_COUNT} items, got {}",
        items.len()
    );

    let mut previous: Option<&str> = None;
    let mut actual_checksum = FNV_OFFSET;
    for item in items {
        let Value::Object(current) = item else {
            bail!("expected a RustData object, got {item:?}");
        };
        if let Some(previous) = previous {
            ensure!(previous <= current.as_str(), "result is not sorted");
        }
        previous = Some(current.as_str());
        actual_checksum = checksum_add(actual_checksum, current);
    }

    let expected = reference_sorted();
    let expected_checksum = expected
        .iter()
        .fold(FNV_OFFSET, |hash, value| checksum_add(hash, value));
    if actual_checksum != expected_checksum {
        for (index, (item, expected)) in items.iter().zip(&expected).enumerate() {
            let Value::Object(actual) = item else {
                break;
            };
            if actual.as_str() != &**expected {
                bail!(
                    "result differs from the native reference at index {index}: \
                     got {actual:?}, expected {expected:?}"
                );
            }
        }
        bail!(
            "result checksum mismatch: got {actual_checksum:#018x}, \
             expected {expected_checksum:#018x}"
        );
    }

    Ok(())
}
