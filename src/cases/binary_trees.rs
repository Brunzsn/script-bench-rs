use std::hint::black_box;

use anyhow::{bail, Result};

use crate::harness::{expect_int, Case, ResultKind, Scripts, Value};

const DEPTH: i64 = 14;
const ITERATIONS: i64 = 20;

enum Tree {
    Leaf,
    Node(Box<Tree>, Box<Tree>),
}

pub const CASE: Case = Case {
    name: "Binary trees",
    scripts: Scripts {
        js: Some("scripts/binary_trees.js"),
        koto: Some("scripts/binary_trees.koto"),
        lua: Some("scripts/binary_trees.lua"),
        martite_script: Some("scripts/binary_trees.mt"),
        martite_interface: Some("scripts/binary_trees.mti"),
        rhai: Some("scripts/binary_trees.rhai"),
        roto: None,
        wasm_source: None,
        wasm_module: None,
    },
    result: ResultKind::Int,
    native,
    validate,
};

fn build(depth: i64) -> Tree {
    if depth == 0 {
        Tree::Leaf
    } else {
        Tree::Node(Box::new(build(depth - 1)), Box::new(build(depth - 1)))
    }
}

fn count(tree: &Tree) -> i64 {
    match tree {
        Tree::Leaf => 1,
        Tree::Node(left, right) => 1 + count(left) + count(right),
    }
}

fn reference() -> i64 {
    let mut total = 0i64;
    for _ in 0..ITERATIONS {
        total += count(&build(DEPTH));
    }
    total
}

fn native() {
    black_box(reference());
}

fn validate(value: &Value) -> Result<()> {
    let actual = expect_int(value)?;
    let expected = reference();
    if actual != expected {
        bail!("expected {expected}, got {actual}");
    }
    Ok(())
}
