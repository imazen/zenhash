#!/usr/bin/env python3
"""Print compact tables from an existing zenhash portable benchmark JSON file."""
import json
from pathlib import Path
import sys

result = json.loads(Path(sys.argv[1]).read_text())
assert not result["incomplete"] and not result["unreliable"]
groups = {g["group_name"]: g for g in result["comparisons"]}

def ns_each(group, benchmark, length):
    repetitions = max(1, min(16384, 1048576 // length))
    value = next(b for b in groups[group]["benchmarks"] if b["name"] == benchmark)
    return value["summary"]["mean"] / repetitions

print("| Runtime | Input bytes | FNV-1a64 ns/hash | Rapidhash v3 ns/hash | XXH3-64 ns/hash |")
print("|---|---:|---:|---:|---:|")
for length in [4, 16, 64, 241, 1024, 65536]:
    for runtime, name in enumerate(["native", "WASM scalar", "WASM SIMD128"]):
        group = f"replacement/{length}/runtime{runtime}"
        numbers = [ns_each(group, b, length) for b in ["fnv64", "rapid_v3", "xxh3_64"]]
        print(f"| {name} | {length} | " + " | ".join(f"{x:.3f}" for x in numbers) + " |")
print()
print("| 64 KiB method | Chunk bytes | Native GiB/s | WASM scalar GiB/s | WASM SIMD128 GiB/s |")
print("|---|---:|---:|---:|---:|")
for algorithm in ["xxh3_64", "xxh3_128", "rapid_v3", "xxhash_rust_64", "xxhash_rust_128", "rapidhash_ref"]:
    for chunk in [0, 4096]:
        group = f"{algorithm}/65536/chunk{chunk}"
        numbers = [65536 / ns_each(group, b, 65536) * 1e9 / 2**30
                   for b in ["native", "scalar", "simd128"]]
        print(f"| {algorithm} | {chunk} | " + " | ".join(f"{x:.3f}" for x in numbers) + " |")
