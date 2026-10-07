# Changelog

## [Unreleased]

### QUEUED BREAKING CHANGES
<!-- Breaks that ship together in the next leading-digit bump (0.x: the minor). None yet. -->

### Added
- `xxh3_64`, `xxh3_128` and their `_with_seed` / `_with_secret` variants;
  streaming `Xxh3` with 64- and 128-bit digests (1c0b386)
- `xxh64`, `xxh32`, streaming `Xxh64`, `Xxh32` (1c0b386)
- XXH3 long-input accumulator with runtime dispatch to AVX-512 (`avx512`
  feature), AVX2, SSE2, NEON, WASM SIMD128 and scalar (1c0b386)

### Performance
- XXH3 129..=240 B: unrolled, scalar mid-size paths (2x faster at 240 B)
  (875fb62, 18186a3)
- XXH3 long inputs: whole-block fast path, `srli` instead of a second
  shuffle, unchecked secret windows (be0ab62), lean dispatch and partial
  first block fast path (2da9a29)
- Streaming XXH3: inlined buffer-only `update` fast path (639d932)
- Benchmarks committed in `benchmarks/xxhash_2026-10-07.md`
