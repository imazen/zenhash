# Changelog

## [Unreleased]

### Fixed
- Explicitly select cargo-fuzz in the fuzz workflow; the runner otherwise completed installation without installing a tool.


### Dependency compatibility

- Use archmage 0.9.31-beta.2 with `attune` contexts (`bcbd64e5`).

### QUEUED BREAKING CHANGES
- `std::io::Write` now requires the separate, opt-in `io` feature; `std` alone only enables standard-library integration (`bcbd64e5`).
- XXH32/XXH64 require opt-in `xxh32`/`xxh64` features (approved before the first release; follows `3dea22a4`).

### Added
- README API quick reference and shared crates.io/docs.rs landing page; docs.rs builds every public algorithm and the optional I/O adapter.
- Preserve the xxHash BSD-2-Clause notice alongside the crate MIT/Apache-2.0 and rapidhash MIT licenses.
- Opt-in FNV-1a32/64/128 compatibility APIs, continuation offset bases, XOR-folding and fmix64 finalization (`ce5540f5`).
- Expanded independent C stability fixtures and isolated runtime feature powerset tests.
- XXH3 uses archmage 0.9.31-beta.2 `attune` contexts, preferring v4x with v4 fallback.
- Opt-in safe standard rapidhash v3: one-shot, seeded/custom-secret APIs,
  bounded streaming state, `Hasher` and optional `io::Write` (`16ab3213`).
- Fast C-reference conformance matrix across algorithms, patterns, lengths,
  seeds and custom secrets on every CI platform and available SIMD tier.
- Default-enabled `xxh3` feature gates XXH3 functions, streaming state,
  custom-secret errors and optional SIMD/error dependencies; `std` and
  `avx512` do not enable XXH3 (`3dea22a4`).
- Document safe-Rust and cross-ISA/WASM performance goals, published SMHasher3
  findings, and guidance for trusted, statistical and adversarial workloads.
- `xxh3_64`, `xxh3_128` and their `_with_seed` / `_with_secret` variants;
  streaming `Xxh3` with 64- and 128-bit digests (1c0b386)
- `xxh64`, `xxh32`, streaming `Xxh64`, `Xxh32` (1c0b386), with `new()` and
  `std::io::Write` like `Xxh3`
- `Debug` for the hashers shows only the byte count (no seeds, secrets or
  buffered input); `#[must_use]` on hash functions and digests
- XXH3 long-input accumulator with runtime dispatch to AVX-512 (`avx512`
  feature, on by default), AVX2, SSE2, NEON, WASM SIMD128 and scalar (1c0b386)

### Performance
- Pair 32-bit factors in XXH3's WASM and NEON accumulators; add WASM entry-point
  inlining, with C parity and ARM/Wasmtime measurements (`e9e91d60`, `18eedee8`, `574f1fb8`).
- XXH3 129..=240 B: unrolled, scalar mid-size paths (2x faster at 240 B)
  (875fb62, 18186a3)
- XXH3 long inputs: whole-block fast path, `srli` instead of a second
  shuffle, unchecked secret windows (be0ab62), lean dispatch and partial
  first block fast path (2da9a29)
- Streaming XXH3: inlined buffer-only `update` fast path (639d932)
- Benchmarks committed in `benchmarks/xxhash_2026-10-07.md`
