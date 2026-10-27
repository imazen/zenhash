# Changelog

## [Unreleased]

### Fixed
- Explicitly select cargo-fuzz in the fuzz workflow; the runner otherwise completed installation without installing a tool.


### Dependency compatibility

- Admit archmage 0.9.31-beta.1 in the current-main tool graph while retaining compatible stable requirements (`e2261fa1`). Strict library lint, all 20 integration tests, the separate tier-parity test and 13 doctests pass.

### QUEUED BREAKING CHANGES
- XXH32/XXH64 require opt-in `xxh32`/`xxh64` features (approved before the first release; follows `3dea22a4`).

### Added
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
- XXH3 129..=240 B: unrolled, scalar mid-size paths (2x faster at 240 B)
  (875fb62, 18186a3)
- XXH3 long inputs: whole-block fast path, `srli` instead of a second
  shuffle, unchecked secret windows (be0ab62), lean dispatch and partial
  first block fast path (2da9a29)
- Streaming XXH3: inlined buffer-only `update` fast path (639d932)
- Benchmarks committed in `benchmarks/xxhash_2026-10-07.md`
