# ARM native and WASM baseline, 2026-10-26

Source: `81f28bf572433b9990b20f282d0ad52c24ec4a7d`.
Ampere Altra Neoverse-N1, Linux aarch64, eight homogeneous cores, verified
cpuset placement. Frequency was not verified. Built with rustc 1.99.1,
zenbench 0.1.11 and Wasmtime 49.0.4, thin LTO, no `target-cpu=native`.

This is one exploratory process with 15–30 interleaved rounds per group.
It is not independent-process confirmation or a result for browsers/x86.
The full run completed all 170 groups without an unreliable/incomplete flag.

Reproduce with the selected Rust 1.99.1 toolchain on PATH:

```sh
just wasm-workloads
just bench-portable --format=json --rounds=30 --workload-id=81f28bf5-portable-byte-pattern-v1
```

Each batch constructs identical bytes (`i * 37 + (i >> 8)`, truncated to u8)
and hashes them `clamp(1048576 / len, 1, 16384)` times. Tables divide the
measured batch mean by that repetition count. Input allocation/initialization
is amortized, included equally across algorithms and runtimes. Results are
hot-input throughput. JIT compilation and module instantiation are outside
the timed batch. Both halves of 128-bit results contribute to the checksum.

Before timing, native streaming and both WASM modules agree for all benchmark
methods, four seeds, eight lengths and three chunk schedules. The scalar
module also loads with SIMD forbidden in the engine. Fixed C fixtures remain
the independent algorithm-output oracle.

The full JSON includes default/seeded/custom-secret methods, both XXH3 widths,
XXH32/64, all FNV variants, and 4 KiB streaming. The compact tables below
highlight selection tradeoffs and comparison ports.

| Runtime | Input bytes | FNV-1a64 ns/hash | Rapidhash v3 ns/hash | XXH3-64 ns/hash |
|---|---:|---:|---:|---:|
| native | 4 | 4.978 | 4.758 | 4.876 |
| WASM scalar | 4 | 6.212 | 20.600 | 9.766 |
| WASM SIMD128 | 4 | 5.958 | 20.691 | 9.709 |
| native | 16 | 20.641 | 4.738 | 5.299 |
| WASM scalar | 16 | 19.802 | 20.484 | 16.670 |
| WASM SIMD128 | 16 | 19.994 | 20.496 | 16.733 |
| native | 64 | 98.491 | 15.125 | 12.404 |
| WASM scalar | 64 | 95.977 | 66.654 | 39.673 |
| WASM SIMD128 | 64 | 97.014 | 66.644 | 39.645 |
| native | 241 | 427.967 | 43.363 | 38.187 |
| WASM scalar | 241 | 402.580 | 147.435 | 105.681 |
| WASM SIMD128 | 241 | 402.113 | 148.265 | 112.777 |
| native | 1024 | 1878.821 | 155.990 | 90.484 |
| WASM scalar | 1024 | 1742.092 | 496.161 | 222.735 |
| WASM SIMD128 | 1024 | 1733.671 | 496.033 | 265.614 |
| native | 65536 | 122146.354 | 10549.906 | 6234.181 |
| WASM scalar | 65536 | 115993.410 | 32123.636 | 13125.080 |
| WASM SIMD128 | 65536 | 112130.487 | 31029.363 | 14902.444 |

| 64 KiB method | Chunk bytes | Native GiB/s | WASM scalar GiB/s | WASM SIMD128 GiB/s |
|---|---:|---:|---:|---:|
| xxh3_64 | 0 | 9.716 | 4.726 | 4.131 |
| xxh3_64 | 4096 | 8.743 | 4.510 | 3.864 |
| xxh3_128 | 0 | 9.750 | 4.717 | 4.120 |
| xxh3_128 | 4096 | 8.673 | 4.501 | 3.858 |
| rapid_v3 | 0 | 5.689 | 1.876 | 1.946 |
| rapid_v3 | 4096 | 5.462 | 1.861 | 1.938 |
| xxhash_rust_64 | 0 | 12.057 | 5.464 | 7.591 |
| xxhash_rust_64 | 4096 | 9.989 | 4.089 | 6.464 |
| xxhash_rust_128 | 0 | 12.097 | 5.460 | 7.411 |
| xxhash_rust_128 | 4096 | 9.966 | 4.055 | 6.454 |
| rapidhash_ref | 0 | 5.741 | 2.044 | 2.117 |
| rapidhash_ref | 4096 | 5.669 | 1.821 | 1.878 |

Native rapidhash is competitive for tiny keys. WASM changes the tradeoff:
with the baseline WASM target features, 64×64→128 multiplication is emulated, and rapidhash is slower than FNV for
4-byte inputs here. XXH3 is the more promising default across these measured
native/WASM cells. Existing persisted FNV identifiers still require FNV.

The initial XXH3 WASM SIMD path is slower than its scalar path at 64 KiB,
and the comparison port is faster on both NEON and WASM SIMD. These gaps
motivate profiling; cross-ISA competitive performance is not yet achieved.

The measurement wrapper reported:

```text
run-heavy: done rc=0 210s | peak-RSS 0.04GiB | min-avail 15000MiB | peak-load 1.18
```

Native benchmark binary SHA-256:
`26729823fc87529375d6faafce1bb64ed7dac96249df1a8976ef88667bcb921c`.
Scalar module: `1c578bc97707a69b0da97c087edcc57c587e87b5b96a931dab1bb028fdaaac7f`.
SIMD module: `370ffce85a1bb0cd81b3133bd27d439fe9f5c75c9bab9ff1c3895f449bd6d4b7`.

Raw artifacts are retained outside git under
`/mnt/v/output/zenhash/arm-wasm-2026-10-26/`:

- `baseline.log` SHA-256 `c6c3deb6e380d5bdff2224b704a14e04cd6c3abe979e6c0d4690ae47a79fc8fc`.
- `baseline.json` SHA-256 `98530080151caee168d365f5a60c1204d3abe03d99676fa57190fdc640cab020`.
- `build-199.log` SHA-256 `8907ec2d5d66844d0ceded770ba4706ee63c92f7147ef1f838ed8a3020fb51a1`.
