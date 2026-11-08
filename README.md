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

## Benchmark cases

The suite separates pure-compute cost from Rust interop:

- `Sort Rust objects` — 10k generated strings wrapped in host userdata objects, sorted through a host `<` call.
- `Fib (recursive)` — `fib(32)`, call overhead.
- `Integer loop` and `Float loop` — tight arithmetic loops.
- `N-body`, `Spectral norm`, `Mandelbrot` — floating-point numeric kernels.
- `Binary trees` — allocation and GC churn at depth 14.
- `Quicksort integers` — the userdata sort algorithm on plain integers, with no host calls.
- `Strings` — concatenation, split/join, search/replace, formatting, and non-ASCII character counting.
- `Array operations`, `Hash map`, `Builtin sort` — core collection operations.
- `Closures` — map/filter/reduce over closures.

Pure-compute scripts use deterministic in-script inputs and do not call host functions. Roto has no maps or first-class functions and no in-place list mutation, so it does not run the cases that need them. `wasmi` and `wasmtime` run only the precompiled-wasm `Sort Rust objects` case; compiled wasm is not an interpreter baseline.

### Interop benchmarks

Interop cases live in a parallel registry (`src/interop/`) because their timed path is usually driven from Rust rather than from a script entry point. They cover:

- `Interop call ...` — script-to-host call overhead for zero, one, three and five integer arguments, plus f64 and string arguments/results.
- `Interop call-in update` — Rust calls a script `update(dt)` function repeatedly.
- `Interop userdata ...` — method calls, field get/set, and copy-like versus handle-like constructors.
- `Interop callback reentry` — Rust invokes a script closure/hook that calls back into Rust.
- `Interop bulk ...` — f64 and u8 data passed in and out at 1k, 100k and 1M elements, through engine-native lists and, where available, typed buffers.
- `Interop serde nested ...` — a nested object is converted between Rust and script values in both directions.
- `Interop error ...` — a host error is caught in the script, or propagated to the host.

Call, call-in, callback and userdata cases use 100k iterations per timed invocation; serde and error cases use 10k. Bulk list cases use engine-native arrays/tables; buffer cases use typed arrays for Boa/QuickJS and Luau buffers for Luau, and are skipped by the other engines. Martite has no host methods, so its userdata case calls free host functions on opaque handles; its callbacks are hooks and its error paths use `Result` values. Martite runs only the from-script serde direction. Roto skips callbacks, bulk data, serde and error paths. `wasmi` and `wasmtime` do not run interop cases.

## Adding a benchmark case

A case is one entry in the registry plus one script per language that can express it:

1. Add the scripts under `scripts/`.
2. Add a `Case` entry in `src/cases/` (including its native Rust reference implementation and `ResultKind`) and list it in `src/cases/mod.rs`.
3. Leave an engine's script path as `None` when it cannot express the case without host functions; that engine skips the case.
4. For WebAssembly, point `wasm_source` at a `.wasm.rs` file; `bench.py` compiles every wasm source declared by the registry.

Interop cases follow the same pattern in `src/interop/`: add an `InteropCase` entry, its native baseline and validator, its scripts, and an engine runner implementation in `src/<engine>/interop.rs`. Leave the engine's script path as `None` to skip an unsupported operation.

Everything else (criterion targets, validation reporting, plots) is driven by the registry.

## Methodology

- **Deterministic inputs.** The interop case receives the same input stream from a fixed-seed SplitMix64 generator (`src/rng.rs`). Pure-compute cases use fixed constants and in-script deterministic generators, so no host RNG is involved. The generator is reset before each validation and timed iteration.
- **Cross-engine validation.** Each engine's result is checked against the native Rust reference. The sort case is checked by item count, ordering, and an FNV-1a checksum; scalar cases check exact integers or floating-point results within a tolerance. A mismatch fails `cargo bench -- --test` in the checks workflow.
- **Native baseline.** The `native` criterion target runs the same workload as every case, including interop cases. Plots label every engine with its multiple of that baseline, and `target/benchmark-relative.json` contains the raw ratios.
- **Interop workloads.** Interop cases use fixed iteration counts and deterministic one-valued payloads. Script-to-host calls and userdata operations use 100k iterations; Rust-driven call-in and callbacks use 100k; serde and error paths use 10k. Bulk cases use 1k, 100k and 1M elements. The native baseline performs the same number of operations and is kept observable with `black_box`.
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
