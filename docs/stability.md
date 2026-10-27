# Output stability coverage

`just stability` uses a standalone package whose only dependency is zenhash
with default features disabled. This matters: the root package's dev-dependencies
enable archmage/std, so its tests alone cannot demonstrate a no_std library build.

`just stability-configs` executes the feature powerset. CI partitions this work
on Linux. Native portability jobs run default and all-feature integration tests;
i686 and s390x run through cross; WASI runs explicitly with SIMD128 disabled and
enabled. The separate tier process disables available SIMD tokens and checks
XXH3 through every reachable dispatch tier. Multiple token permutations that
select the same kernel share one expanded-corpus run; the original cases still
execute on every permutation. Feature combinations are exhausted
on Linux, not on every OS × target product.

The original pinned C fixture remains unchanged: six patterns, four seeds,
six secret sizes and lengths 0–256 plus 31 larger boundaries. The expanded
fixture crosses twelve data families independently with six secret patterns,
six secret sizes (136, 137, 171, 192, 200, 1000), four base seeds, all short
lengths and stripe, buffer and secret-dependent block boundaries.

Additional families cover single-bit edits, all one-hot and complemented seeds,
adjacent two-bit seeds, seed bytes at multiple input offsets, and all 24
permutations of four distinct words. Repeated cycles, common-prefix text and
seed/complement words exercise structured rather than only random inputs.
The corpus is deterministic and bounded; it approximates selected SMHasher3
key families without running its exhaustive statistical battery.

Per enabled algorithm/configuration, the runner checks:

- Canonical little-endian whole and prefix output against independent C fixtures.
- One-shot versus streaming, a deterministic split, empty updates, repeated digest,
  clone continuation, reset retaining seed/secret, and `Hasher::write`/`finish`.
- Both XXH3 digest widths, seeded and custom-secret APIs; rapidhash seeded and
  custom-secret APIs. Default constructors/functions are checked against seed zero.
- Stable integer `Hasher` writes against their documented native-endian bytes.
  These typed writes intentionally have target-dependent byte representations;
  cross-platform persisted hashes should explicitly encode integers to bytes.
- The `io` adapter, where enabled, including `write_all` and `flush`.

FNV has a separate C arithmetic fixture covering the same twelve data families,
all three widths, standard and
custom bases, continuation, both finalizers, streaming prefixes, clone and reset.
A second fixture covers zero, maximum, high-bit and repeated-byte bases at each
full width, including 128-bit bases with nonzero high halves.

Existing differential tests additionally cover unaligned inputs, exhaustive
short-input split positions and randomized chunk schedules. A stability corpus
is not proof over every input, secret or streaming partition; fuzzing remains
useful for combinations outside this bounded matrix.

Regenerate with `just conformance-vectors`, `just stability-vectors` and
`just fnv-vectors`. CI recompiles the reference generators and compares their
output with the committed text; expected values are never produced by zenhash.

CI also tests `target-cpu=native` on Linux x86/ARM. Those builds use a fresh
artifact directory because CPU-specific dependencies cannot safely share a
cache across different runner CPUs. Portable builds continue to use the cache.
