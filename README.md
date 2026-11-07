# Rust scripting languages benchmark

The project goal is to benchmark most popular embedded scripting languages for Rust.

- [boa](https://boajs.dev)
- [koto](https://crates.io/crates/koto)
- [martite](https://github.com/Brunzsn/Martite)
- [mlua](https://crates.io/crates/mlua) (Lua 5.5 and Luau)
- [rhai](https://crates.io/crates/rhai)
- [roto](https://crates.io/crates/roto)
- [rquickjs](https://crates.io/crates/rquickjs)
- [wasmi](https://crates.io/crates/wasmi)
- [wasmtime](https://crates.io/crates/wasmtime)

The benchmark is designed to cover not only the performance of code evaluation but interoperability with Rust too.

## Getting your own results

Simply run `uv run bench.py` to generate images. It requires `cargo criterion` and [uv](https://docs.astral.sh/uv/) installed (dependencies are declared inline in the script).

You also must have `wasm32-unknown-unknown` target installed for webassembly benchmarks.

The Martite benchmark uses a sibling checkout at `../martite` for its path dependency. Clone both repositories side by side, or let the GitHub Actions workflow configure the same layout.

The `benchmarks` workflow runs the full suite on `ubuntu-latest`. Those results are saved as artifacts and are not directly comparable to the macOS numbers below.

The `checks` workflow is the fast path for pull requests and pushes: it compiles every engine and runs each registered case once (criterion `--test` mode) without collecting measurements.

## Adding a benchmark case

A case is one entry in the registry plus one script per language:

1. Add the scripts under `scripts/`.
2. Add a `Case` entry in `src/cases/` (including its native Rust reference implementation) and list it in `src/cases/mod.rs`.
3. For WebAssembly, point `wasm_source` at a `.wasm.rs` file; `bench.py` compiles every wasm source declared by the registry.

Everything else (criterion targets, validation reporting, plots) is driven by the registry.

## Methodology

- **Deterministic inputs.** Every script receives the same input stream from a fixed-seed SplitMix64 generator (`src/rng.rs`). The generator is reset before each validation and timed iteration, so every engine and iteration sorts identical data.
- **Cross-engine validation.** Each engine's result is checked against the native Rust reference by item count, ordering, and an FNV-1a checksum. A mismatch fails `cargo bench -- --test` in the checks workflow.
- **Native baseline.** The `native` criterion target runs the same workload with `Vec::sort`. Plots label every engine with its multiple of that baseline, and `target/benchmark-relative.json` contains the raw ratios.
- **Statistics.** Criterion uses 50 samples, a 3 second warm-up, and a 10 second measurement window.
- **Lua.** No engine receives a `table` preallocation shim; all scripts use ordinary empty tables or lists.
- **WebAssembly.** `wasmi` and `wasmtime` execute a Rust module precompiled to wasm. They are labelled as compiled-wasm entries and are not scripting-language measurements.
- **Instruction counts.** On Linux with Valgrind, `cargo bench --bench instructions_<engine> --features instructions,<engine>` runs gungraun for deterministic instruction counts. The `instruction counts` workflow runs this matrix on a schedule.

## Environment

| Component | Version |
|-----------|---------|
| OS | macOS 27.0, Apple M5 Max |
| rustc | v1.99.0 |
| boa | v0.22.0 |
| koto | v0.16.1 |
| martite | v0.1.0 |
| mlua | v0.12.2 |
| rhai | v1.26.1 |
| roto | v0.12.0 |
| rquickjs | v0.14.0 |
| wasmi | v2.0.0 |
| wasmtime | v49.0.2 |

## Results

![Sort Rust objects](Sort%20Rust%20objects.png)

Rev 1791068096
