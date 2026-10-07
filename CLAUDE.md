# zenhash — agent notes

xxHash (XXH3 64/128, XXH64, XXH32) in safe Rust (`#![forbid(unsafe_code)]`,
`no_std` + `alloc`), MIT OR Apache-2.0. Output must equal the reference C
implementation bit for bit on every target and SIMD tier.

## Layout

- `src/xxh3/mod.rs`: constants, default secret, seed-derived secrets, the
  short paths (`len <= 240`, scalar, read only the first 136 secret bytes),
  long-input finalization, one-shot API.
- `src/xxh3/accumulate.rs`: the stripe accumulator, the only SIMD code. One
  shared loop (`consume_body!`) stamped per tier with four helpers (load, store,
  accumulate one 64-byte stripe, scramble). Tiers: v4 (behind `avx512`), v3,
  v1 (SSE2), neon, wasm128, scalar. Dispatch is one `incant!` per `consume`
  call. Big-endian targets always take scalar.
- `src/xxh3/stream.rs`: streaming state, mirrors the C buffering (256-byte
  buffer, keeps the last consumed stripe at the buffer's end for the
  digest's "previous 64 bytes" case).
- `src/xxh64.rs`, `src/xxh32.rs`: classic algorithms, scalar only (the C
  reference has no SIMD path for them either).

## Recipes

`just test`, `just clippy`, `just clippy-arch` (NEON/WASM type-check from x86),
`just test-aarch64-qemu`, `just test-wasm`, `just bench`, `just bench-avx512`,
`just ci`. Without wasmtime, Node's `node:wasi` works as a WASI runner.

## Design decisions

- **Oracles:** tests compare against xxhash-rust and twox-hash (dev-deps only)
  across all path boundaries, seeds, secret sizes 136..1000 and streaming
  splits. The fuzz target (`fuzz/fuzz_targets/hash_parity_core.rs`) does the
  same and is replayed by `tests/it/fuzz_regression.rs`.
- **Why intrinsics, not magetypes:** magetypes 0.9.29 `u64x*` vectors have no
  32x32->64 widening multiply (`pmuludq`/`vmull_u32`/`extmul`) and no 64-bit
  lane swap. XXH3 needs both on every stripe. If magetypes gains them, the
  x86/NEON/WASM tiers could collapse into one `#[magetypes]` kernel over
  `u64x8`, which happens to be exactly the 8-accumulator state.
- **No dispatch for short inputs:** inputs of at most 240 bytes never touch
  `incant!`; the per-call summon cost would dominate an 8-byte hash.
- **Custom secrets** are validated (>= 136 bytes) and returned as
  `At<Error>`; streaming copies the secret into a `Box<[u8]>`.

## Optimization log (dead ends and why)

Bench host for these entries: Xeon family 6 model 207 (Emerald Rapids),
4 vCPU VM, rustc 1.97.0, `cargo bench --bench hash -- --no-busy-gate`.

- 129..=240 paths: an `#[inline(never)]` function with an indexed
  `for i in 8..rounds` loop let LLVM's SLP vectorizer pack the 64-bit
  xor/add pairs into SSE registers (unpack + stack spills around every
  `mulq`): 21.1 ns at 240 B. Inlining (default secret folds into immediates)
  plus fixed-size chunk iterators with a constant `take(N)` bound: 10.9 ns.
  twox-hash solves the same problem with an inline-asm register barrier,
  which `forbid(unsafe_code)` rules out here.
- zenbench's resource gate stalled the bench on this VM (13 s CPU in 20 min).
  Run with `--no-busy-gate` there.

## Known Bugs

None open.
