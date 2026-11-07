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


def registry(*args):
    proc = subprocess.run(
        ["cargo", "run", "--quiet", "--example", "list_cases", "--", *args],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=True,
    )
    return [line for line in proc.stdout.splitlines() if line.strip()]


def run_benchmark(name):
    print(f"Running benchmark for {name}", flush=True)
    proc = subprocess.run(
        f"cargo criterion --bench {name} --message-format json --features {name}",
        shell=True,
        stdout=subprocess.PIPE,
        text=True,
        check=True,
    )
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

for case in cases:
    rows = [
        (engine, results[case])
        for engine, results in benches.items()
        if case in results
    ]
    if not rows:
        print(f"warning: no benchmark results for {case!r}", file=sys.stderr)
        continue

    fig, ax = plt.subplots()

    ymax = 0
    for (name, bench) in sorted(rows, key=lambda x: x[1]["typical"]["estimate"]):
        val = round(bench["typical"]["estimate"] / 1000000, 2)
        rect = ax.bar(name, val, width=0.3)
        ax.bar_label(rect, padding=3)
        ymax = max(ymax, val)

    ax.set_title("lower is better")
    ax.set_ylabel("time (ms)", fontweight="bold")
    ax.set_ylim(0, ymax * 1.2)
    fig.autofmt_xdate()
    fig.suptitle(case, fontsize=18)
    fig.tight_layout()

    plt.savefig(f"{case}.png", dpi=300)
    plt.close(fig)
