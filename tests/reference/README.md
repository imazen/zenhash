# Hash reference fixtures

The vendored reference headers are unmodified and used only by development tools, never the
shipped Rust library. Both hash outputs and all fingerprint inputs are independent
of native byte order, pointer width and CPU instruction set.

- [xxHash v0.8.3](https://github.com/Cyan4973/xxHash/blob/e626a72bc2321cd320e953a0ccf1584cad60f363/xxhash.h),
  commit `e626a72bc2321cd320e953a0ccf1584cad60f363`, BSD-2-Clause (notice in header).
- [rapidhash v3](https://github.com/Nicoshev/rapidhash/blob/1ae7842f59f25929a338e5a76e3fa350673e48e1/rapidhash.h),
  commit `1ae7842f59f25929a338e5a76e3fa350673e48e1`, MIT (notice in header and
  `LICENSE-RAPIDHASH`, also shipped with the Rust port).

`just rapidhash-vectors` regenerates `rapidhash-v3.txt`: individual standard-v3
C outputs for fixed splitmix64 inputs and seeds. `tests/it/rapid.rs` checks both
one-shot and streaming against these values and the independent Rust port.

`just conformance-vectors` regenerates `conformance.txt`. Each row identifies an
input pattern, seed and secret size; each algorithm/variant has its own FNV-1a
fingerprint of canonical output bytes over the fixed length list. Numeric results
are encoded as little-endian u64s (XXH32 zero-extended; XXH128 low half then high).
This compact fixture checks portability without a large output table. It is a
regression oracle, not a cryptographic collision-resistance test.

The matrix includes six data/secret patterns, four seeds, six secret sizes, all
lengths 0–256 and 31 larger boundaries. Rapidhash secret words are decoded from
the first 64 secret bytes as little-endian numbers, including the eighth word.
Zero-filled/repeated secrets deliberately test output compatibility; they are not
recommended secret choices. XXH3 secret calls have no seed parameter: the row's
seed changes the generated inputs and secrets, not that API's semantics.

Every CI platform runs this matrix and checks streaming against one-shot. Its
separate SIMD-tier process repeats it with each available dispatch permutation.
A Linux CI job recompiles the C generator and compares its output to the fixture.
Existing randomized and per-case differential tests remain enabled for their
algorithm features.

## Expanded fixtures and coverage limits

`just stability-vectors` regenerates `conformance-extended.txt` using independent
C input scheduling and the same pinned reference algorithms. It crosses twelve
input families with six secret patterns and six secret lengths. Additional
rows cover bit edits, correlated seeds and word permutations. Whole and streaming
prefix outputs are fingerprinted separately in sequence. The original fixture
and its expectations remain unchanged.

`just fnv-vectors` regenerates `fnv.txt` and `fnv-bases.txt` using unsigned arithmetic in C,
including 128-bit multiplication. It checks FNV-1a32/64/128 and both existing
zen finalizers. The formulas and caller provenance are in [docs/fnv.md](../../docs/fnv.md).
The C generator requires GCC or Clang for unsigned `__int128`; the resulting
text fixture executes on every target without a C dependency.

The isolated runner executes the feature powerset on Linux. Native, i686,
big-endian and WASM targets run the full corpus with all algorithms enabled;
native and WASM jobs also run a genuine no_std dependency graph. Bare-metal
no_std targets are compile-checked. This is a factored matrix, not an exhaustive
platform × feature-set product. Exact schedules and method coverage are in
[docs/stability.md](../../docs/stability.md).

These are bounded output-stability tests, not SMHasher3's statistical avalanche,
bit-independence or distribution tests. C expectations are compact per-variant
fingerprints, supplemented by per-case Rust oracles and individual C vectors.
