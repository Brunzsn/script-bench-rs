use anyhow::{bail, ensure, Result};

use crate::harness::{Case, Scripts, Value};

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
    validate,
};

fn validate(value: &Value) -> Result<()> {
    let Value::List(items) = value else {
        bail!("expected the script to return a list, got {value:?}");
    };
    ensure!(
        items.len() == 10_000,
        "expected 10000 items, got {}",
        items.len()
    );

    let mut previous: Option<&str> = None;
    for item in items {
        let Value::Object(current) = item else {
            bail!("expected a RustData object, got {item:?}");
        };
        if let Some(previous) = previous {
            ensure!(previous <= current.as_str(), "result is not sorted");
        }
        previous = Some(current);
    }

    Ok(())
}
