#!/usr/bin/env python3
"""Export a safe Rust function by its compiler-generated linker symbol.

No unsafe/no_mangle attributes are needed. Keep the symbol alongside the module
so the host runner discovers exactly the export produced by this build.
"""
import os
from pathlib import Path
import re
import shutil
import subprocess
import time

root = Path(__file__).resolve().parent / "wasm-workload"
output = root / "target/modules"
output.mkdir(parents=True, exist_ok=True)
manifest = str(root / "Cargo.toml")
target = "wasm32-unknown-unknown"
for name, flag in [("scalar", "-simd128"), ("simd128", "+simd128")]:
    env = dict(os.environ, RUSTFLAGS=f"-Ctarget-feature={flag} -Csymbol-mangling-version=v0")
    base = ["cargo", "rustc", "--manifest-path", manifest, "--release", "--target", target]
    subprocess.run(base + ["--lib", "--", "--emit=llvm-ir,link"], env=env, check=True)
    files = sorted((root / f"target/{target}/release/deps").glob("zenhash_wasm_workload-*.ll"),
                   key=lambda path: path.stat().st_mtime_ns)
    ir = files[-1].read_text()
    matches = re.findall(r"define [^@\n]*@([A-Za-z_0-9]*hash_batch[A-Za-z_0-9]*)\(", ir)
    if len(matches) != 1:
        raise RuntimeError(f"expected one hash_batch definition, found {matches}")
    symbol = matches[0]
    subprocess.run(base + ["--bin", "zenhash-wasm-workload", "--",
                          f"-Clink-arg=--export={symbol}"], env=env, check=True)
    stamp = time.time_ns()
    for suffix in ["wasm", "symbol"]:
        existing = output / f"{name}.{suffix}"
        if existing.exists():
            existing.rename(output / f"{name}.{suffix}.bak.{stamp}")
    shutil.copyfile(root / f"target/{target}/release/zenhash-wasm-workload.wasm",
                    output / f"{name}.wasm")
    (output / f"{name}.symbol").write_text(symbol + "\n")
    print(f"{name}: {symbol}", flush=True)
