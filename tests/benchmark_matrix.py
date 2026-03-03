from __future__ import annotations

import argparse
import ctypes
import gc
import json
import csv
import os
import pickle
import random
import statistics
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path

THIS_FILE = Path(__file__).resolve()
ROOT_DIR = THIS_FILE.parent.parent
SRC_DIR = ROOT_DIR / "src"
RESULTS_DIR = ROOT_DIR / "benchmark_results"
if os.fspath(SRC_DIR) not in sys.path:
    sys.path.insert(0, os.fspath(SRC_DIR))

from perfect_hash import generate_hash


DEFAULT_SIZES = [10_000, 50_000, 100_000, 500_000, 1_000_000]
DEFAULT_LOOKUP_BATCH = 2_048
DEFAULT_LOOKUP_REPEATS = 64
RANDOM_SEED = 17


def current_rss_bytes() -> int:
    with open("/proc/self/status", encoding="utf-8") as handle:
        for line in handle:
            if line.startswith("VmRSS:"):
                parts = line.split()
                return int(parts[1]) * 1024
    raise RuntimeError("VmRSS not available in /proc/self/status")


def make_keys(size: int) -> list[str]:
    return [f"key-{index:08x}-payload" for index in range(size)]


@dataclass
class BenchmarkResult:
    implementation: str
    size: int
    build_seconds: float
    hit_lookup_seconds: float
    miss_lookup_seconds: float
    lookup_repeats: int
    lookup_batch: int
    rss_delta_bytes: int
    serialized_bytes: int


def trim_allocator() -> None:
    try:
        libc = ctypes.CDLL("libc.so.6")
        libc.malloc_trim(0)
    except OSError:
        return


def build_mapping(implementation: str, keys: list[str]):
    if implementation == "dict":
        return {key: index for index, key in enumerate(keys)}
    if implementation == "perfect_hash":
        return generate_hash(keys)
    raise ValueError(f"unsupported implementation: {implementation}")


def time_hit_lookups(mapping, sample: list[str], repeats: int) -> float:
    timings = []
    for _ in range(5):
        start = time.perf_counter()
        for _ in range(repeats):
            for key in sample:
                mapping[key]
        timings.append(time.perf_counter() - start)
    return statistics.median(timings)


def time_miss_lookups(mapping, sample: list[str], repeats: int) -> float:
    timings = []
    misses = [f"{key}-missing" for key in sample]
    for _ in range(5):
        start = time.perf_counter()
        for _ in range(repeats):
            for key in misses:
                try:
                    mapping[key]
                except KeyError:
                    pass
        timings.append(time.perf_counter() - start)
    return statistics.median(timings)


def run_single(implementation: str, size: int, lookup_batch: int, lookup_repeats: int) -> BenchmarkResult:
    random.seed(RANDOM_SEED)
    keys = make_keys(size)
    sample = random.sample(keys, min(len(keys), lookup_batch))

    rss_before = current_rss_bytes()
    build_start = time.perf_counter()
    mapping = build_mapping(implementation, keys)
    build_seconds = time.perf_counter() - build_start
    gc.collect()
    trim_allocator()
    rss_after = current_rss_bytes()
    serialized_bytes = len(pickle.dumps(mapping, protocol=pickle.HIGHEST_PROTOCOL))

    hit_lookup_seconds = time_hit_lookups(mapping, sample, lookup_repeats)
    miss_lookup_seconds = time_miss_lookups(mapping, sample, lookup_repeats)

    return BenchmarkResult(
        implementation=implementation,
        size=size,
        build_seconds=build_seconds,
        hit_lookup_seconds=hit_lookup_seconds,
        miss_lookup_seconds=miss_lookup_seconds,
        lookup_repeats=lookup_repeats,
        lookup_batch=len(sample),
        rss_delta_bytes=max(0, rss_after - rss_before),
        serialized_bytes=serialized_bytes,
    )


def run_child(args: argparse.Namespace) -> int:
    result = run_single(
        implementation=args.impl,
        size=args.size,
        lookup_batch=args.lookup_batch,
        lookup_repeats=args.lookup_repeats,
    )
    print(json.dumps(asdict(result)))
    return 0


def run_matrix(args: argparse.Namespace) -> int:
    sizes = args.sizes or DEFAULT_SIZES
    results: list[BenchmarkResult] = []

    for size in sizes:
        for implementation in ("dict", "perfect_hash"):
            command = [
                sys.executable,
                os.fspath(THIS_FILE),
                "--child",
                "--impl",
                implementation,
                "--size",
                str(size),
                "--lookup-batch",
                str(args.lookup_batch),
                "--lookup-repeats",
                str(args.lookup_repeats),
            ]
            completed = subprocess.run(
                command,
                check=True,
                capture_output=True,
                text=True,
            )
            payload = completed.stdout.strip().splitlines()[-1]
            results.append(BenchmarkResult(**json.loads(payload)))

    print(
        "implementation,size,build_seconds,hit_lookup_seconds,miss_lookup_seconds,"
        "lookup_batch,lookup_repeats,rss_delta_mb,serialized_mb"
    )
    for result in results:
        print(
            f"{result.implementation},{result.size},{result.build_seconds:.6f},"
            f"{result.hit_lookup_seconds:.6f},{result.miss_lookup_seconds:.6f},"
            f"{result.lookup_batch},{result.lookup_repeats},"
            f"{result.rss_delta_bytes / (1024 * 1024):.2f},"
            f"{result.serialized_bytes / (1024 * 1024):.2f}"
        )

    if args.output_stem:
        RESULTS_DIR.mkdir(exist_ok=True)
        csv_path = RESULTS_DIR / f"{args.output_stem}.csv"
        json_path = RESULTS_DIR / f"{args.output_stem}.json"

        with csv_path.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=list(asdict(results[0]).keys()))
            writer.writeheader()
            for result in results:
                writer.writerow(asdict(result))

        with json_path.open("w", encoding="utf-8") as handle:
            json.dump([asdict(result) for result in results], handle, indent=2)

    return 0


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--child", action="store_true")
    parser.add_argument("--impl", choices=("dict", "perfect_hash"))
    parser.add_argument("--size", type=int)
    parser.add_argument("--sizes", type=int, nargs="*")
    parser.add_argument("--lookup-batch", type=int, default=DEFAULT_LOOKUP_BATCH)
    parser.add_argument("--lookup-repeats", type=int, default=DEFAULT_LOOKUP_REPEATS)
    parser.add_argument("--output-stem")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.child:
        if args.impl is None or args.size is None:
            raise SystemExit("--child requires --impl and --size")
        return run_child(args)
    return run_matrix(args)


if __name__ == "__main__":
    raise SystemExit(main())
