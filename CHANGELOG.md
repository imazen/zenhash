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
