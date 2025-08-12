#!/usr/bin/env python3
import argparse
import json
import os
import sys
from pathlib import Path


def find_latest_estimates(root: Path, bench_group: str, bench_fn: str) -> Path | None:
    candidates = list(root.rglob(f"{bench_group}/{bench_fn}/**/estimates.json"))
    if not candidates:
        return None
    candidates.sort(key=lambda p: p.stat().st_mtime, reverse=True)
    return candidates[0]


def read_median_ns(estimates_path: Path) -> float:
    with estimates_path.open("r", encoding="utf-8") as f:
        data = json.load(f)
    # Criterion estimates.json uses fields like {"median": {"point_estimate": <ns>}}
    # Units are nanoseconds by default.
    return float(data["median"]["point_estimate"])


def main() -> int:
    parser = argparse.ArgumentParser(description="Check JIT/MIR speedup vs interpreter using Criterion outputs")
    parser.add_argument("--target-dir", default="target/criterion", help="Criterion output directory root")
    parser.add_argument("--bench-group", default="jit_vs_interp", help="Benchmark group name")
    parser.add_argument("--interp-name", default="interp_execute", help="Interpreter bench function name")
    parser.add_argument("--jit-name", default="mir_execute", help="JIT/MIR bench function name")
    parser.add_argument("--required-speedup", type=float, default=2.0, help="Required speedup (interp/jit) >= this value")
    args = parser.parse_args()

    root = Path(args.target_dir)
    if not root.exists():
        print(f"ERROR: Criterion target dir not found: {root}")
        return 2

    interp_est = find_latest_estimates(root, args.bench_group, args.interp_name)
    jit_est = find_latest_estimates(root, args.bench_group, args.jit_name)
    if not interp_est or not jit_est:
        print("ERROR: Could not locate estimates.json for required benches.")
        print(f" looked for: {args.bench_group}/{args.interp_name} and {args.bench_group}/{args.jit_name}")
        return 3

    interp_ns = read_median_ns(interp_est)
    jit_ns = read_median_ns(jit_est)
    if jit_ns <= 0.0:
        print("ERROR: Invalid JIT median (<=0)")
        return 4

    speedup = interp_ns / jit_ns
    print(f"Interpreter median: {interp_ns:.0f} ns; JIT/MIR median: {jit_ns:.0f} ns; speedup: {speedup:.2f}x")

    if speedup < args.required_speedup:
        print(f"FAIL: Required speedup >= {args.required_speedup:.2f}x not met.")
        return 1
    print("PASS: Speedup requirement met.")
    return 0


if __name__ == "__main__":
    sys.exit(main())


