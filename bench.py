# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "matplotlib",
# ]
# ///

import glob
import json
import subprocess
import sys
from os.path import basename, splitext
import matplotlib.pyplot as plt

NATIVE = "native"
WASM_ENGINES = {"wasmi", "wasmtime"}
ENGINE_LABELS = {
    "boa": "boa (JS)",
    "koto": "koto",
    "martite": "martite",
    "mlua_lua55": "Lua 5.5",
    "mlua_luau": "Luau",
    "native": "native Rust",
    "rhai": "rhai",
    "roto": "roto",
    "rquickjs": "QuickJS",
    "wasmi": "wasmi (Rust to wasm)",
    "wasmtime": "wasmtime (Rust to wasm)",
}


def registry(*args):
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--example", "list_cases", "--", *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=True,
    )
    return [line for line in proc.stdout.splitlines() if line.strip()]


def display_name(name):
    return ENGINE_LABELS.get(name, name)


def run_benchmark(name):
    print(f"Running benchmark for {name}", flush=True)
    command = ["cargo", "criterion", "--bench", name, "--message-format", "json"]
    if name != NATIVE:
        command.extend(["--features", name])
    proc = subprocess.run(command, stdout=subprocess.PIPE, text=True, check=True)
    res = []
    for line in proc.stdout.splitlines():
        x = json.loads(line)
        if "id" in x:
            res.append(x)
    return res


# Compile the wasm modules declared by the case registry.
for entry in registry("wasm"):
    source, module = entry.split("\t")
    crate_name = splitext(basename(source))[0].split(".")[0]
    subprocess.run(
        "rustc --target wasm32-unknown-unknown -Cpanic=abort -O "
        f"--crate-name {crate_name} {source} -o {module}",
        shell=True,
        check=True,
    )

cases = registry()

benches = dict()
for f in glob.glob("benches/*.rs"):
    name = splitext(basename(f))[0]
    benches[name] = dict()
    for res in run_benchmark(name):
        benches[name][res["id"]] = res

with open("target/benchmark-results.json", "w") as f:
    json.dump(benches, f, indent=2)

known = set(cases)
unexpected = {
    (engine, bench_id)
    for engine, results in benches.items()
    for bench_id in results
    if bench_id not in known
}
if unexpected:
    raise RuntimeError(
        "benchmark ids missing from the case registry: "
        f"{sorted(unexpected)}"
    )

relative = dict()
for case in cases:
    rows = [
        (engine, results[case])
        for engine, results in benches.items()
        if case in results
    ]
    if not rows:
        print(f"warning: no benchmark results for {case!r}", file=sys.stderr)
        continue

    native = benches.get(NATIVE, {}).get(case)
    native_ms = native["typical"]["estimate"] / 1_000_000 if native else None
    case_relative = dict()

    fig, ax = plt.subplots()

    ymax = 0
    for (name, bench) in sorted(rows, key=lambda x: x[1]["typical"]["estimate"]):
        val = bench["typical"]["estimate"] / 1_000_000
        ratio = val / native_ms if native_ms else None
        if ratio is None:
            label = f"{val:.2f} ms"
        else:
            label = f"{val:.2f} ms\n{ratio:.1f}x"
        rect = ax.bar(
            display_name(name),
            val,
            width=0.6,
            color="0.65" if name == NATIVE else None,
            hatch="//" if name in WASM_ENGINES else None,
        )
        ax.bar_label(rect, labels=[label], padding=3)
        ymax = max(ymax, val)
        case_relative[name] = {
            "time_ms": round(val, 3),
            "relative_to_native": round(ratio, 3) if ratio is not None else None,
        }

    ax.set_title("lower is better; labels show time and multiple of native" if native_ms else "lower is better")
    ax.set_ylabel("time (ms)", fontweight="bold")
    ax.set_ylim(0, ymax * 1.2)
    fig.autofmt_xdate()
    fig.suptitle(case, fontsize=18)
    fig.tight_layout()

    plt.savefig(f"{case}.png", dpi=300)
    plt.close(fig)

    relative[case] = case_relative

    if native_ms:
        print(f"\n{case}: relative to native Rust")
        for name, values in sorted(
            case_relative.items(), key=lambda item: item[1]["time_ms"]
        ):
            print(
                f"  {display_name(name):28} "
                f"{values['time_ms']:9.3f} ms  "
                f"{values['relative_to_native']:6.2f}x"
            )

with open("target/benchmark-relative.json", "w") as f:
    json.dump(relative, f, indent=2)
