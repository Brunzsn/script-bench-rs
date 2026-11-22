# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "matplotlib",
# ]
# ///

"""Benchmark runner and result consolidator.

Modes:

  no arguments      run every bench target locally, then render the
                    consolidated figures and summary JSON files
  --engine NAME     CI matrix mode: run `native` plus one engine bench on
                    this machine and write raw results without rendering
  --summary DIR     CI summary mode: merge the per-engine result artifacts
                    downloaded into DIR and render the consolidated
                    figures and summary JSON files

Bar heights in every figure are multiples of the native Rust run measured
in the same job, so results from different CI runners stay comparable.
Raw milliseconds remain in `benchmark-results.json`; they are measured on
per-engine runners and are not comparable across engines.
"""

import argparse
import glob
import json
import math
import platform
import subprocess
import sys
from os.path import basename, join, splitext

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

# Consolidated figure per group of cases, matched by case-name prefix.
# Cases matching no prefix are compute benchmarks.
CATEGORY_RULES = [
    ("Interop call ", "interop-calls"),
    ("Interop call-in ", "interop-calls"),
    ("Interop callback ", "interop-calls"),
    ("Interop userdata ", "interop-userdata"),
    ("Interop bulk ", "interop-bulk"),
    ("Interop serde ", "interop-serde"),
    ("Interop error ", "interop-errors"),
]
CATEGORY_TITLES = {
    "compute": "Compute benchmarks",
    "interop-calls": "Interop: calls and callbacks",
    "interop-userdata": "Interop: userdata",
    "interop-bulk": "Interop: bulk data",
    "interop-serde": "Interop: serde",
    "interop-errors": "Interop: errors",
}
CATEGORY_FILES = {
    "compute": "compute.png",
    "interop-calls": "interop-calls.png",
    "interop-userdata": "interop-userdata.png",
    "interop-bulk": "interop-bulk.png",
    "interop-serde": "interop-serde.png",
    "interop-errors": "interop-errors.png",
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


def bench_targets():
    return sorted(splitext(basename(f))[0] for f in glob.glob("benches/*.rs"))


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


def collect_results(names):
    benches = dict()
    for name in names:
        benches[name] = dict()
        for res in run_benchmark(name):
            benches[name][res["id"]] = res
    return benches


def build_wasm_modules():
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


def write_json(path, data):
    with open(path, "w") as f:
        json.dump(data, f, indent=2)


def runner_info(engine):
    cpu = None
    try:
        with open("/proc/cpuinfo", encoding="utf-8") as f:
            for line in f:
                if line.startswith("model name"):
                    cpu = line.split(":", 1)[1].strip()
                    break
    except OSError:
        pass
    return {"engine": engine, "platform": platform.platform(), "cpu": cpu}


def validate_ids(benches, cases):
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


def summarize(benches, cases):
    """Build the relative table and per-case plot data.

    Returns (relative, plots): relative maps case -> engine ->
    {time_ms, relative_to_native} for the JSON output; plots maps case ->
    (rows, native_ms) where rows is a list of (engine, ms, ratio) sorted
    by time and native_ms is the native run measured alongside.
    """
    relative = dict()
    plots = dict()
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

        entries = dict()
        prepared = []
        for name, bench in rows:
            val = bench["typical"]["estimate"] / 1_000_000
            ratio = val / native_ms if native_ms else None
            entries[name] = {
                "time_ms": round(val, 3),
                "relative_to_native": round(ratio, 3) if ratio is not None else None,
            }
            prepared.append((name, val, ratio))
        prepared.sort(key=lambda row: row[1])
        relative[case] = entries
        plots[case] = (prepared, native_ms)
    return relative, plots


def draw_case(ax, case, rows, native_ms):
    """One subplot: bar height is the multiple of native when it is known."""
    normalized = native_ms is not None
    ymax = 0
    for name, val, ratio in rows:
        height = ratio if normalized else val
        if normalized:
            label = f"{ratio:.1f}x\n{val:.2f} ms"
        else:
            label = f"{val:.2f} ms"
        rect = ax.bar(
            display_name(name),
            height,
            width=0.6,
            color="0.65" if name == NATIVE else None,
            hatch="//" if name in WASM_ENGINES else None,
        )
        ax.bar_label(rect, labels=[label], padding=3, fontsize=8)
        ymax = max(ymax, height)
    ax.set_ylim(0, ymax * 1.3)
    ax.set_title(case, fontsize=11)


def render_category(category, category_cases):
    count = len(category_cases)
    if count == 0:
        return
    cols = min(3, count)
    rows = -(-count // cols)
    normalized = all(native_ms is not None for _, _, native_ms in category_cases)
    fig, axes = plt.subplots(
        rows,
        cols,
        figsize=(6.5 * cols, 4.5 * rows),
        squeeze=False,
        layout="constrained",
    )
    for index, (case, case_rows, native_ms) in enumerate(category_cases):
        draw_case(axes[index // cols][index % cols], case, case_rows, native_ms)
    for index in range(count, rows * cols):
        axes[index // cols][index % cols].set_visible(False)
    fig.suptitle(CATEGORY_TITLES[category], fontsize=18)
    fig.supxlabel(
        "multiple of native Rust — lower is better"
        if normalized
        else "time (ms) — lower is better",
        fontweight="bold",
    )
    fig.savefig(CATEGORY_FILES[category], dpi=200)
    plt.close(fig)


def render_overview(relative):
    ratios_by_engine = dict()
    for entries in relative.values():
        for engine, values in entries.items():
            ratio = values["relative_to_native"]
            if ratio is None or engine == NATIVE:
                continue
            ratios_by_engine.setdefault(engine, []).append(ratio)
    if not ratios_by_engine:
        return
    ordered = sorted(
        (
            (
                engine,
                math.exp(sum(math.log(r) for r in ratios) / len(ratios)),
                len(ratios),
            )
            for engine, ratios in ratios_by_engine.items()
        ),
        key=lambda row: row[1],
    )
    names = [display_name(engine) for engine, _, _ in ordered]
    geomeans = [geo for _, geo, _ in ordered]
    fig, ax = plt.subplots(
        figsize=(9, 1.5 + 0.7 * len(ordered)), layout="constrained"
    )
    rects = ax.barh(names, geomeans, height=0.6)
    ax.bar_label(
        rects,
        labels=[
            f"{geo:.2f}x ({count} case{'s' if count != 1 else ''})"
            for _, geo, count in ordered
        ],
        padding=4,
    )
    ax.set_xlim(0, max(geomeans) * 1.35)
    ax.set_xlabel(
        "geometric mean of multiples of native Rust — lower is better",
        fontweight="bold",
    )
    fig.suptitle("All benchmark cases per engine", fontsize=18)
    fig.savefig("overview.png", dpi=200)
    plt.close(fig)


def print_report(relative):
    for case, entries in relative.items():
        print(f"\n{case}: relative to native Rust")
        for name, values in sorted(
            entries.items(), key=lambda item: item[1]["time_ms"]
        ):
            ratio = values["relative_to_native"]
            ratio_text = f"{ratio:6.2f}x" if ratio is not None else "      -"
            print(
                f"  {display_name(name):28} "
                f"{values['time_ms']:9.3f} ms  "
                f"{ratio_text}"
            )


def render(relative, plots):
    grouped = dict()
    for case, (case_rows, native_ms) in plots.items():
        category = next(
            (cat for prefix, cat in CATEGORY_RULES if case.startswith(prefix)),
            "compute",
        )
        grouped.setdefault(category, []).append((case, case_rows, native_ms))
    for category, category_cases in grouped.items():
        render_category(category, category_cases)
    render_overview(relative)
    print_report(relative)


def run_local():
    cases = registry()
    build_wasm_modules()
    benches = collect_results(bench_targets())
    write_json("target/benchmark-results.json", benches)
    validate_ids(benches, cases)
    relative, plots = summarize(benches, cases)
    render(relative, plots)
    write_json("target/benchmark-relative.json", relative)


def run_engine(engine):
    cases = registry()
    if engine in WASM_ENGINES:
        build_wasm_modules()
    # Build the engine bench before measuring so the native and engine
    # measurement windows run back to back on the same runner.
    subprocess.run(
        ["cargo", "bench", "--bench", engine, "--features", engine, "--no-run"],
        check=True,
    )
    benches = collect_results([NATIVE, engine])
    validate_ids(benches, cases)
    write_json("target/benchmark-results.json", benches)
    write_json("target/runner.json", runner_info(engine))


def run_summary(results_dir):
    cases = registry()
    benches = dict()
    for path in sorted(glob.glob(join(results_dir, "*", "benchmark-results.json"))):
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        for engine, results in data.items():
            if engine == NATIVE and NATIVE in benches:
                # Every job measures native on its own runner. Keep one copy
                # in the merged dump; per-job natives stay in the artifacts.
                continue
            benches[engine] = results
    if not benches:
        raise RuntimeError(f"no benchmark results found under {results_dir!r}")
    missing = set(bench_targets()) - set(benches)
    if missing:
        print(f"warning: missing engine results: {sorted(missing)}", file=sys.stderr)

    runners = []
    for path in sorted(glob.glob(join(results_dir, "*", "runner.json"))):
        with open(path, encoding="utf-8") as f:
            info = json.load(f)
        runners.append(info)
        print(f"{info.get('engine')}: {info.get('cpu') or 'unknown CPU'}")
    if runners:
        write_json("target/runners.json", runners)

    write_json("target/benchmark-results.json", benches)
    validate_ids(benches, cases)
    relative, plots = summarize(benches, cases)
    render(relative, plots)
    write_json("target/benchmark-relative.json", relative)


def main():
    engines = [name for name in bench_targets() if name != NATIVE]
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument(
        "--engine",
        choices=engines,
        help="run native plus this single engine bench and write raw results only",
    )
    parser.add_argument(
        "--summary",
        metavar="RESULTS_DIR",
        help="merge per-engine result artifacts from RESULTS_DIR and render figures",
    )
    args = parser.parse_args()
    if args.engine and args.summary:
        parser.error("--engine and --summary are mutually exclusive")
    if args.engine:
        run_engine(args.engine)
    elif args.summary:
        run_summary(args.summary)
    else:
        run_local()


if __name__ == "__main__":
    main()
