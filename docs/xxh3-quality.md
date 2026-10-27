# XXH3 quality and usage

XXH3 is a non-cryptographic hash. zenhash preserves its reference output,
including its statistical limitations. Safe Rust prevents unsafe code in this
crate; it does not change the algorithm's collision properties.

## Published SMHasher3 results

Checked 2026-10-26 against SMHasher3's [results summary](https://gitlab.com/fwojcik/smhasher3/-/blob/main/results/README.md)
and its published AVX2 logs:

| Algorithm | Passed | Failed categories |
|---|---|---|
| [XXH3-64](https://gitlab.com/fwojcik/smhasher3/-/raw/main/results/raw/XXH3-64.txt) | 223/250 | BIC, Sparse, PerlinNoise, Bitflip, SeedZeroes, SeedSparse, SeedBlockLen, SeedBlockOffset, SeedBIC |
| [XXH3-128](https://gitlab.com/fwojcik/smhasher3/-/raw/main/results/raw/XXH3-128.txt) | 214/250 | BIC, Sparse, PerlinNoise, Bitflip, SeedZeroes, SeedSparse, SeedBlockLen, SeedBlockOffset, SeedBIC |

These counts describe those published runs and their tested implementations,
not a fresh SMHasher3 run against zenhash. Our differential tests check output
compatibility; they do not establish statistical quality or cryptographic security.

The logs pass the ordinary avalanche tests but fail some bit-independence
(BIC) tests: output-bit changes can be correlated even when each bit's change
rate looks good in isolation. Sparse, PerlinNoise and Bitflip failures expose
weaknesses on particular structured inputs. Not every failed test is a
full-width collision; tests also examine subsets of output bits and distributions.

Seed tests vary seeds as well as inputs. For example, the XXH3-128 log's
8-byte SeedBlockLen case reports 534,343 full-width collisions among 2,196,480
hashes when seeds and a 4-byte input block each have at most two set bits.
This is a constructed joint seed/input workload, not a collision rate for
ordinary files hashed under one fixed seed. Do not treat seeds as independent
hash functions or a security boundary for separating domains.

A longer output alone does not repair these weaknesses. Nor does a custom
secret establish cryptographic security. The reference's
[custom-secret API](https://xxhash.com/doc/v0.8.3/group___x_x_h3__family.html)
requires high-entropy secret contents for hash quality; zenhash checks the
minimum length, not entropy. Repeated or zero-filled secrets are poor choices.

## Choosing a use case

The following guidance is an application-level interpretation of these results
and xxHash's [non-cryptographic design](https://github.com/Cyan4973/xxHash).

| Use | Assessment |
|---|---|
| Trusted-data hash tables, caches and lookup indexes | Reasonable when full keys are compared on collisions. Measure the real key distribution and workload. |
| Deduplication candidate selection | Reasonable as a fast filter followed by byte comparison or an appropriate cryptographic hash before declaring identity. |
| Accidental-change checksums | Reasonable when non-cryptographic error detection meets the application's requirements and the output width suits the data volume. |
| Stable xxHash interoperability | A strength: byte-oriented one-shot and streaming APIs preserve reference outputs across platforms. Specify algorithm, seed/secret and byte encoding. |
| HyperLogLog, MinHash and other statistical estimators | Validate the exact bit subsets, key distribution and seed usage; the documented bias and seed correlations matter. Prefer a hash with suitable quality evidence for the estimator's assumptions. |
| Attacker-controlled hash-table keys | Use a hasher designed for HashDoS resistance. Fixed-seed `BuildHasherDefault<Xxh3>` is intended for trusted inputs. |
| Authentication, signatures, passwords, or adversarial content identity | Use an appropriate cryptographic hash, MAC or password-hashing scheme. XXH3 is not suitable. |

XXH3's strengths are throughput, compact digests, established interoperability,
streaming and SIMD acceleration. Its costs include collision risk, statistical
weaknesses on some inputs, and different performance tradeoffs for short inputs
and different ISAs. See the [measured benchmarks](https://github.com/imazen/zenhash/blob/main/benchmarks/xxhash_2026-10-07.md)
for one x86 CPU; they do not establish ARM or WASM performance parity.

## Other algorithm choices

The same published SMHasher3 suite reports [XXH64 at 241/250](https://gitlab.com/fwojcik/smhasher3/-/raw/main/results/raw/XXH-64.txt),
with SeedBlockLen, SeedBlockOffset and SeedBIC failures, and
[XXH32 at 167/250](https://gitlab.com/fwojcik/smhasher3/-/raw/main/results/raw/XXH-32.txt),
with broader structured-input and bit-independence failures as well as seed
failures. These findings differ substantially; the counts alone are not a
universal ranking. Their compatibility value is reproducing existing xxHash
checksums and formats. XXH32's small output space is also a limitation for
new applications hashing many distinct items.

[Reference rapidhash v3 passes 250/250](https://gitlab.com/fwojcik/smhasher3/-/raw/main/results/raw/rapidhash.txt)
in its published run and is a candidate for another algorithm feature; it is
not implemented here yet. A safe Rust port can express its byte reads with
slices and its widening multiplication with `u128`. Native and WASM performance
would need measurement; passing statistical tests does not make it cryptographic.

The existing [hoxxep/rapidhash crate root](https://github.com/hoxxep/rapidhash/blob/master/rapidhash/src/lib.rs)
does not forbid unsafe code. Its [read helpers](https://github.com/hoxxep/rapidhash/blob/master/rapidhash/src/util/read.rs)
use unsafe unaligned pointer reads even with its `unsafe` feature disabled;
that feature changes bounds assertions to debug assertions. This observation
was checked on 2026-10-26 and applies to that source revision, not every
historical release.
