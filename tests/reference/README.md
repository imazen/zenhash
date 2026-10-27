# Hash reference fixtures

The C sources are unmodified and used only by the development tools, never the
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

## Coverage limits

The fixed matrix exercises all eleven public one-shot hash functions and all
four streaming hashers. It runs with all algorithm features enabled on each CI
platform and under the available SIMD dispatch permutations. The feature
powerset is compile-checked; individual algorithms are runtime-tested on Linux.
This is not the full runtime cross-product of feature sets, platforms and inputs.
Bare-metal no_std targets are compile-checked, not executed.

Input and secret patterns currently use the same pattern family in each row;
the six data families are not independently crossed with six secret families.
The seed set is four fixed values. The corpus does not enumerate correlated
seed/key families, all single-bit edits, word permutations, text keys or varied
cycle lengths. It is a bounded output-stability corpus, not an implementation
of SMHasher3's statistical avalanche, bit-independence or distribution tests.

Streaming conformance uses one deterministic split and an empty update per
matrix case. Separate differential tests add fixed chunk sizes, randomized
splits, unaligned inputs, digest/clone/reset checks and io::Write coverage;
those behaviors are not each multiplied by the entire matrix. The compact C
expectations are per-variant fingerprints over output sequences, supplemented
by per-case Rust-oracle checks and individual rapidhash C vectors.
