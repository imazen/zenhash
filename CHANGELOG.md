# Changelog

## [Unreleased]

### QUEUED BREAKING CHANGES
<!-- Breaks that ship together in the next leading-digit bump (0.x: the minor). None yet. -->

### Added
<!-- Everything below landed in the squash merge of #1 (ce2bbdf). -->
- `xxh3_64`, `xxh3_128` and their `_with_seed` / `_with_secret` variants;
  streaming `Xxh3` with 64- and 128-bit digests
- `xxh64`, `xxh32`, streaming `Xxh64`, `Xxh32`, with `new()` and
  `std::io::Write` like `Xxh3`
- `Debug` for the hashers shows only the byte count (no seeds, secrets or
  buffered input); `#[must_use]` on hash functions and digests
- XXH3 long-input accumulator with runtime dispatch to AVX-512 (`avx512`
  feature, on by default), AVX2, SSE2, NEON, WASM SIMD128 and scalar

### Performance
- XXH3 129..=240 B: unrolled, scalar mid-size paths (2x faster at 240 B)
- XXH3 long inputs: whole-block fast path, `srli` instead of a second
  shuffle, unchecked secret windows, lean dispatch and partial
  first block fast path
- Streaming XXH3: inlined buffer-only `update` fast path
- XXH3 241..1024 B: initial accumulators loaded from the constant (no
  store-forwarding stall); seed-0 mid path stays scalar in
  `target-cpu=native` builds
- Benchmarks committed in `benchmarks/xxhash_2026-10-07.md` and
  `benchmarks/adversarial_2026-10-07.md`
