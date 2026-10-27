# FNV compatibility and replacement

Enable `fnv` for FNV-1a32/64/128. These are byte-oriented, allocation-free,
Sans-I/O algorithms with streaming `update`, non-consuming `digest`, clone,
reset and `core::hash::Hasher`. Enable `io` separately for `std::io::Write`.
`Hasher::finish` returns the low 64 bits for FNV-1a128; use `digest` for all bits.

All three widths XOR each input byte into the state, then multiply with
wrapping arithmetic. The standard offset bases and primes are:

| Width | Offset basis | Prime |
|---|---|---|
| 32 | `811c9dc5` | `01000193` |
| 64 | `cbf29ce484222325` | `100000001b3` |
| 128 | `6c62272e07bb014262b821756295c58d` | `0000000001000000000000000000013b` |

`with_basis` accepts an initial state, rather than a seed mixed by a separate
algorithm. Passing the preceding raw digest continues the same byte stream.
`reset` restores the basis supplied at construction. Never continue from a
folded or finalized digest if the intended result is the original raw FNV.

The formulas preserve these existing uses:

- [zenanalyze byte fingerprints](https://github.com/imazen/zenanalyze/blob/208ee7ecf5ffa97691d8e74fab35d9d4d9d2db81/zenanalyze-api/src/lib.rs): raw FNV-1a64 and continuation.
- [zensim feature-set IDs](https://github.com/imazen/zensim/blob/9877ab3d744c174b1b5fb1ed641ab5d395d98382/zensim/src/feature_set_id.rs): FNV-1a64 followed by `(h ^ (h >> 32)) as u32`, exposed as `fnv1a64_fold32` and `Fnv1a64::digest_fold32`. Slot sorting, deduplication and decimal formatting remain the caller's responsibility.
- [zensr generated fingerprints](https://github.com/imazen/zensr/blob/1cbc449216d9da2a0d7e4ee5ac206277ff114d08/crates/zensr-bench/src/bin/gen_detect.rs): FNV-1a64 followed by Murmur3 fmix64, exposed as `fnv1a64_fmix` and `Fnv1a64::digest_fmix`.
- [zenav1-svt output pins](https://github.com/imazen/zenav1-svt/blob/c4df139196038b74428ac1ae0b28fe0a303b922b/rust/svtav1/tests/output_pins.rs): raw FNV-1a128. Length prefixes and hexadecimal formatting remain with the caller.

FNV is useful for compatibility with existing stored fingerprints and simple
deterministic checks. Its byte-by-byte dependency chain limits instruction-level
parallelism; low output bits have limited mixing. Folding reduces the output
space, and adding fmix64 does not recover information from collisions already
present in FNV64. None of these formulas is cryptographic or suitable for hostile
hash-table keys.

For new non-adversarial byte keys, prefer the opt-in standard `rapidhash` v3
algorithm. Its upstream [SMHasher3 results](https://gitlab.com/fwojcik/smhasher3/-/raw/main/results/raw/rapidhash.txt)
provide statistical evidence for that algorithm, not a security guarantee.
There is no universal fastest replacement across key sizes and architectures;
benchmark the actual workload. XXH3 is another option when 128-bit output or
SIMD throughput matters. Neither replaces a cryptographic digest.

Switching algorithms changes IDs and cache keys. Retain the original formula
for persisted formats, or version the identifier and migrate it explicitly.
