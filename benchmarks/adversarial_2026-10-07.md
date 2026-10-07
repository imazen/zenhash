# Adversarial benchmark: zenhash vs xxHash at its best, 2026-10-07

Every competitor gets its most favorable setup. The question is not "is
zenhash fast as shipped" (see [xxhash_2026-10-07.md](xxhash_2026-10-07.md))
but "where does zenhash lose when the others are tuned for this exact CPU".

Decision this supports, from within-table comparisons:

- For XXH3 below 241 B zenhash is the fastest or tied in both
  configurations, with one exception: batches of mixed 1-64 byte keys,
  where xxhash-rust is faster.
- From 241 B up, implementations compiled for this exact CPU beat zenhash's
  runtime-dispatched AVX-512 kernel: the C reference by 8-44% for XXH3-64
  (both configs; 0-11% for XXH3-128) and xxhash-rust by 5-15% (config A,
  where it gets compile-time AVX-512).
- Without native compilation (config B, the usual Rust build), zenhash is
  the fastest Rust crate from 1 KiB up (29-82% over xxhash-rust, 10-29% over
  twox-hash); only the native C library is faster.

If long-input speed on one known CPU matters most, the C library built with
`-march=native` is faster. If the build cannot target one CPU, zenhash is the
fastest of the Rust crates measured here; a C build without `-march=native`
was not part of these runs.

## Competitors and their best configuration

| Contender | Version | Configuration |
|---|---|---|
| C reference | xxHash 0.8.3 (xxhash-c 0.8.2 over xxhash-c-sys 0.8.7), gcc 13.3.0 | `CFLAGS=-O3 -march=native`: compile-time AVX-512 path (1,179 zmm instructions in libxxhash.a). Safe Rust wrapper; FFI calls are not inlined across languages. |
| xxhash-rust | 0.8.19 | SIMD chosen at compile time, so `-C target-cpu=native` gives it its AVX-512 path. Streaming uses `Xxh3Default`, its default-secret type. |
| twox-hash | 2.1.5 | Runtime AVX2 dispatch (no AVX-512 path). Benefits from native codegen elsewhere. |
| zenhash | this repo | Runtime dispatch (AVX-512 tier on this CPU). |

All Rust crates: fat LTO, `codegen-units = 1` (via
`CARGO_PROFILE_BENCH_LTO` / `CARGO_PROFILE_BENCH_CODEGEN_UNITS`).
Each crate uses its one-shot API; the C wrapper has no seeded one-shot, so
it is absent from the seeded rows. The `keys_mixed_*` groups hash 4096 (or
1024) keys of uniformly random length 1..=N in shuffled order per call,
which defeats branch prediction on the length dispatch; throughput counts
key bytes.

- **Config A**: every Rust crate built with `-C target-cpu=native` too
  (zenhash included: compile-time features let `incant!` skip detection).
- **Config B**: zenhash as users get it. No Rust native flags for anyone,
  so xxhash-rust is on SSE2 and twox-hash on runtime AVX2; C keeps
  `-march=native` (CFLAGS are independent of RUSTFLAGS).

## Environment

Intel Xeon family 6 model 207 (Emerald Rapids), 4 vCPUs in a Firecracker
VM, AVX-512 available, no PMU access. Linux 6.18, rustc 1.97.0, zenbench
0.1.10 `--no-busy-gate`, single thread, inputs generated in memory before
timing, all contenders hash the same buffers. Run-to-run drift on this VM
is several percent; compare within a table.

## Reproduce

```sh
git clone https://github.com/imazen/zenhash && cd zenhash
git checkout 383409ca3fc5dc966e316db4a1c6f7d6c6e0dd6d   # full tables below
just bench-adversarial            # config A
just bench-adversarial-shipped    # config B
# one group: just bench-adversarial -- --no-busy-gate  with ZENHASH_BENCH=xxh3_64/241
```

## Results

Percentages: zenbench's paired 95% CI of the competitor's mean time
relative to zenhash (positive = competitor slower).

### A. Every crate built for this CPU (`-C target-cpu=native`, `CFLAGS=-O3 -march=native`, fat LTO, 1 codegen unit)

| Group | zenhash mean | zenhash | C xxHash 0.8.3 vs zenhash | xxhash-rust vs zenhash | twox-hash vs zenhash |
|---|---|---|---|---|---|
| `xxh3_64/8` | 1.65 ±0.06ns | 4.52GiB/s | [+30.3%–+35.3%] | [+13.1%–+18.0%] | [+22.1%–+27.2%] |
| `xxh3_64/16` | 1.71 ±0.08ns | 8.73GiB/s | [+9.5%–+12.2%] | [+17.5%–+21.0%] | [+26.9%–+32.0%] |
| `xxh3_64/32` | 2.04 ±0.06ns | 14.6GiB/s | [+32.2%–+36.2%] | [+5.9%–+8.8%] | [+36.9%–+39.0%] |
| `xxh3_64/64` | 3.28 ±0.11ns | 18.2GiB/s | [+14.4%–+18.2%] | [+0.4%–+3.8%] | [+32.8%–+36.1%] |
| `xxh3_64/128` | 5.44 ±0.27ns | 21.9GiB/s | [+7.8%–+9.6%] | [+3.8%–+5.5%] | [+37.1%–+39.3%] |
| `xxh3_64/129` ¹ | 13.3 ±0.3ns | 9.07GiB/s | [-31.6%–-30.6%] | [-0.6%–+0.5%] | [-36.4%–-35.4%] |
| `xxh3_64/240` ¹ | 15.6 ±0.4ns | 14.3GiB/s | [-5.5%–-4.2%] | [+44.5%–+45.6%] | [-9.1%–-8.1%] |
| `xxh3_64/241` | 16.1 ±0.3ns | 14.0GiB/s | [-27.7%–-26.8%] | [-10.6%–-9.6%] | [+31.5%–+32.4%] |
| `xxh3_64/512` | 24.9 ±1.3ns | 19.1GiB/s | [-21.2%–-19.1%] | [-15.3%–-13.0%] | [+15.4%–+17.3%] |
| `xxh3_64/1024` | 32.0 ±2.2ns | 29.8GiB/s | [-11.3%–-9.8%] | [-8.4%–-6.5%] | [+23.0%–+25.0%] |
| `xxh3_64/4096` | 113 ±6ns | 33.8GiB/s | [-10.3%–-8.9%] | [-6.5%–-4.5%] | [+8.0%–+10.3%] |
| `xxh3_64/65536` | 1.7 ±0.0µs | 36.2GiB/s | [-10.5%–-9.3%] | [-13.4%–-12.1%] | [+18.5%–+20.0%] |
| `xxh3_64/1048576` | 26.8 ±1.5µs | 36.4GiB/s | [-13.0%–-11.3%] | [-13.7%–-11.9%] | [+20.3%–+22.3%] |
| `xxh3_128/16` | 2.77 ±0.06ns | 5.39GiB/s | [+17.8%–+20.3%] | [+10.3%–+13.1%] | [+21.5%–+24.1%] |
| `xxh3_128/240` | 15.9 ±0.6ns | 14.0GiB/s | [+19.7%–+21.4%] | [+37.4%–+38.8%] | [+35.3%–+36.5%] |
| `xxh3_128/1024` | 36.8 ±2.3ns | 25.9GiB/s | [-1.7%–+0.4%] | [-8.5%–-6.8%] | [+11.7%–+13.4%] |
| `xxh3_128/65536` | 1.8 ±0.1µs | 34.6GiB/s | [-11.1%–-9.8%] | [-13.9%–-12.2%] | [+17.1%–+18.6%] |
| `xxh3_64_seeded/16` | 1.85 ±0.05ns | 8.05GiB/s | n/a | [+15.9%–+19.0%] | [+126.6%–+131.7%] |
| `xxh3_64_seeded/240` | 14.1 ±0.7ns | 15.9GiB/s | n/a | [+72.8%–+74.6%] | [+12.9%–+14.5%] |
| `xxh3_64_seeded/65536` | 1.9 ±0.1µs | 32.9GiB/s | n/a | [-12.5%–-10.5%] | [+5.4%–+7.1%] |
| `keys_mixed_1to16` | 14.1 ±1.3µs | 2.29GiB/s | [+66.4%–+70.2%] | [+15.4%–+19.5%] | [+9.7%–+13.5%] |
| `keys_mixed_1to64` | 18.7 ±1.0µs | 6.67GiB/s | [+3.8%–+6.4%] | [-13.4%–-10.6%] | [-14.5%–-12.3%] |
| `keys_mixed_1to512` | 20.8 ±2.6µs | 11.7GiB/s | [-18.4%–-16.5%] | [-6.7%–-4.6%] | [+11.0%–+12.6%] |
| `xxh3_64_stream/1MiB_in_64B_chunks` | 109.1 ±5.3µs | 8.95GiB/s | [+7.5%–+9.2%] | [-17.1%–-15.5%] | [-10.6%–-9.3%] |
| `xxh3_64_stream/1MiB_in_4096B_chunks` | 34.2 ±1.9µs | 28.6GiB/s | [-2.5%–-0.8%] | [-6.9%–-5.1%] | [+14.8%–+16.8%] |
| `xxh64/16` | 4.11 ±0.14ns | 3.63GiB/s | [-14.5%–-12.0%] | [-11.8%–-9.4%] | [-8.0%–-5.7%] |
| `xxh32/16` | 4.06 ±0.34ns | 3.67GiB/s | [+23.0%–+26.3%] | [-8.0%–-5.5%] | [-11.9%–-9.3%] |
| `xxh64/65536` | 11.2 ±0.4µs | 5.44GiB/s | [-52.0%–-51.2%] | [-0.3%–+0.7%] | [-0.6%–+0.3%] |
| `xxh32/65536` | 14.7 ±0.1µs | 4.16GiB/s | [-33.2%–-32.2%] | [+0.5%–+1.3%] | [-0.2%–+0.9%] |

### B. zenhash as shipped (runtime dispatch, no native flags), same competitor flags except Rust native codegen; C still `-O3 -march=native`; fat LTO

| Group | zenhash mean | zenhash | C xxHash 0.8.3 vs zenhash | xxhash-rust vs zenhash | twox-hash vs zenhash |
|---|---|---|---|---|---|
| `xxh3_64/8` | 1.61 ±0.04ns | 4.63GiB/s | [+34.6%–+39.0%] | [+9.5%–+12.7%] | [+23.4%–+27.1%] |
| `xxh3_64/16` | 1.44 ±0.03ns | 10.4GiB/s | [+40.8%–+45.7%] | [+19.7%–+23.2%] | [+30.3%–+32.7%] |
| `xxh3_64/32` | 1.94 ±0.17ns | 15.3GiB/s | [+48.2%–+54.5%] | [+6.7%–+10.0%] | [+39.5%–+44.1%] |
| `xxh3_64/64` | 3.24 ±0.08ns | 18.4GiB/s | [+17.5%–+21.4%] | [+2.5%–+4.7%] | [+35.3%–+37.6%] |
| `xxh3_64/128` | 5.51 ±0.15ns | 21.6GiB/s | [+4.5%–+6.1%] | [+0.0%–+1.6%] | [+33.4%–+35.2%] |
| `xxh3_64/129` | 7.04 ±0.28ns | 17.1GiB/s | [+34.2%–+36.7%] | [+75.5%–+79.3%] | [+22.0%–+24.0%] |
| `xxh3_64/240` | 12.5 ±0.7ns | 17.9GiB/s | [+34.4%–+37.0%] | [+79.7%–+86.7%] | [+26.2%–+27.9%] |
| `xxh3_64/241` | 20.4 ±1.6ns | 11.0GiB/s | [-44.5%–-42.7%] | [-34.1%–-32.3%] | [-15.7%–-13.9%] |
| `xxh3_64/512` | 24.4 ±1.1ns | 19.5GiB/s | [-28.1%–-26.8%] | [-4.6%–-3.1%] | [-2.8%–-1.4%] |
| `xxh3_64/1024` | 30.5 ±1.0ns | 31.3GiB/s | [-13.8%–-12.8%] | [+42.2%–+44.0%] | [+9.8%–+11.2%] |
| `xxh3_64/4096` | 104 ±4ns | 36.7GiB/s | [-9.4%–-8.2%] | [+77.8%–+79.8%] | [+12.5%–+14.4%] |
| `xxh3_64/65536` | 1.6 ±0.1µs | 37.8GiB/s | [-9.9%–-8.5%] | [+79.1%–+82.3%] | [+27.4%–+29.0%] |
| `xxh3_64/1048576` | 28.2 ±1.7µs | 34.6GiB/s | [-11.4%–-9.5%] | [+75.5%–+78.8%] | [+24.2%–+26.0%] |
| `xxh3_128/16` | 2.90 ±0.10ns | 5.15GiB/s | [+16.8%–+20.2%] | [+8.9%–+12.2%] | [+42.2%–+46.0%] |
| `xxh3_128/240` | 16.7 ±0.8ns | 13.4GiB/s | [+19.4%–+21.4%] | [-0.6%–+0.7%] | [+2.8%–+4.6%] |
| `xxh3_128/1024` | 39.3 ±1.6ns | 24.2GiB/s | [-7.0%–-6.0%] | [+29.0%–+30.3%] | [+10.0%–+11.2%] |
| `xxh3_128/65536` | 1.6 ±0.1µs | 37.2GiB/s | [-11.0%–-10.0%] | [+75.6%–+77.9%] | [+27.2%–+28.3%] |
| `xxh3_64_seeded/16` | 1.75 ±0.04ns | 8.51GiB/s | n/a | [+18.0%–+21.8%] | [+114.3%–+118.8%] |
| `xxh3_64_seeded/240` | 12.7 ±0.2ns | 17.6GiB/s | n/a | [+56.6%–+58.8%] | [+11.7%–+13.6%] |
| `xxh3_64_seeded/65536` | 1.7 ±0.1µs | 35.5GiB/s | n/a | [+69.1%–+70.5%] | [+6.9%–+8.2%] |
| `keys_mixed_1to16` | 20.1 ±1.6µs | 1.61GiB/s | [+13.1%–+16.9%] | [-22.5%–-18.6%] | [+5.8%–+9.0%] |
| `keys_mixed_1to64` | 19.7 ±1.9µs | 6.31GiB/s | [+8.7%–+11.9%] | [-11.6%–-7.7%] | [-5.0%–-1.5%] |
| `keys_mixed_1to512` | 16.3 ±0.5µs | 14.8GiB/s | [-11.7%–-10.1%] | [+5.0%–+5.9%] | [+6.9%–+7.7%] |
| `xxh3_64_stream/1MiB_in_64B_chunks` | 116.8 ±6.7µs | 8.36GiB/s | [-5.5%–-4.1%] | [-19.0%–-17.9%] | [+7.3%–+8.4%] |
| `xxh3_64_stream/1MiB_in_4096B_chunks` | 38.6 ±3.1µs | 25.3GiB/s | [-5.7%–-3.6%] | [+38.4%–+41.2%] | [+8.0%–+10.3%] |
| `xxh64/16` | 3.82 ±0.15ns | 3.90GiB/s | [-1.9%–+1.1%] | [-8.5%–-5.4%] | [-6.7%–-3.8%] |
| `xxh32/16` | 3.65 ±0.23ns | 4.09GiB/s | [-2.1%–+0.5%] | [-16.3%–-13.9%] | [-6.5%–-3.9%] |
| `xxh64/65536` | 5.0 ±0.3µs | 12.3GiB/s | [-0.1%–+0.6%] | [-0.6%–+0.1%] | [-0.4%–+0.3%] |
| `xxh32/65536` | 10.3 ±0.3µs | 5.94GiB/s | [-0.5%–+0.3%] | [-0.4%–+0.4%] | [-0.6%–+0.1%] |

¹ Fixed after this run (commit 099246d): with `target-cpu=native` LLVM
vectorized the seed-0 mid-size path. Re-measured in config A at 099246d:

| Group | zenhash mean | C xxHash 0.8.3 vs zenhash | xxhash-rust vs zenhash | twox-hash vs zenhash |
|---|---|---|---|---|
| `xxh3_64/129` | 7.00 ±0.28ns | [+28.9%–+30.3%] | [+84.5%–+86.1%] | [+19.8%–+21.2%] |
| `xxh3_64/240` | 12.4 ±0.8ns | [+28.8%–+31.1%] | [+92.8%–+95.4%] | [+23.5%–+25.4%] |
| `xxh3_128/240` | 15.9 ±0.6ns | [+19.9%–+21.9%] | [+42.0%–+43.8%] | [+39.2%–+40.7%] |
| `xxh3_64_seeded/240` | 13.1 ±0.7ns | n/a | [+70.0%–+71.6%] | [+12.0%–+13.3%] |

## Where zenhash loses

- **C xxHash with `-march=native`, 241 B and up**: 8-44% faster for XXH3-64
  in both configs (XXH3-128 at 1 KiB: tied in config A, 6-7% in B) (largest at 241 B, about 9-13% from 4 KiB up). Valgrind counts the
  same ~270 instructions per 241-byte hash for both (AVX2 builds), so the
  gap is latency, not work; see the optimization log in CLAUDE.md.
- **xxhash-rust in config A, 241 B and up**: 5-15% faster (its AVX-512 path,
  compiled in). twox-hash is 14-16% faster at 241 B and 1-3% at 512 B in
  config B.
- **Mixed-length key batches**: xxhash-rust 19-22% faster at 1..=16 (config
  B), 8-13% at 1..=64 (both), 5-7% at 1..=512 (config A; zenhash 5-6%
  faster in B); twox-hash 2-15% faster at 1..=64.
- **64-byte streaming updates**: xxhash-rust's `Xxh3Default` 16-19% faster in
  both configs; C 4-6% faster in config B, also at 4 KiB updates; at 4 KiB
  updates in config A, xxhash-rust 5-7% and C 1-3% faster.
- **XXH64/XXH32 at 16 B**: 4-16% slower than one or more competitors.
- **XXH64/XXH32 at 64 KiB in config A**: the C library is 32-52% faster. All
  three Rust crates slow down about 2x under `-C target-cpu=native` (zenhash
  XXH64 11.2 us vs 5.0 us in config B): LLVM vectorizes the four
  independent 64-bit lanes into AVX-512 `vpmullq`, which is slower than
  scalar `imul` here. The C library blocks that vectorization with an
  inline-asm barrier, which `#![forbid(unsafe_code)]` rules out. Open.

## Found and fixed by this benchmark

- 241-1024 B fixed cost: initial accumulators were reloaded as a wide vector
  from narrower stack stores (store-forwarding failure). 241 B in config B:
  27.0 -> 20.4 ns; 512 B 35.4 -> 24.4 ns (commit 383409c).
- 129-240 B in native builds (footnote 1, commit 099246d).
