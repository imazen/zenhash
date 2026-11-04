# Pinned rustc-hash evaluation

This is an evaluation snapshot of [rustc-hash v2.1.3](https://github.com/rust-lang/rustc-hash/tree/v2.1.3),
revision [`c13e7ccca705e6255387a2ebc6dca142d6881621`](https://github.com/rust-lang/rustc-hash/tree/c13e7ccca705e6255387a2ebc6dca142d6881621).
`upstream/` keeps the source, README, manifest and MIT/Apache-2.0 licenses unchanged.
[sources.json](sources.json) records each source URL and SHA-256 for later comparison.
The original manifest is named `Cargo.toml.upstream` so it cannot accidentally
be selected as another Cargo package.

The harness is a separate, unpublished workspace. It compiles the unchanged
upstream library with `unsafe_code = "forbid"`, no default features and no
dependencies. It also offers upstream's `std` feature. Its optional `rand` and
nightly modes are archived but not enabled by this harness. It is excluded from
the published zenhash package. The separate opt-in zenhash
`nonportable::FxHasher` wrapper now depends on the exactly pinned registry
version; it does not compile or expose this snapshot.

## Reproduce

From the zenhash root:

```sh
just fxhash-snapshot-test
just fxhash-snapshot-test --features std
just fxhash-snapshot-probe
CARGO_TARGET_WASM32_WASIP1_RUNNER=wasmtime just fxhash-snapshot-test --target wasm32-wasip1
CARGO_TARGET_WASM32_WASIP1_RUNNER=wasmtime just fxhash-snapshot-probe --target wasm32-wasip1
# Experimental compiler/runtime proposal; inspect the probe cfg result.
RUSTFLAGS="-Ctarget-feature=+wide-arithmetic" \
  CARGO_TARGET_WASM32_WASIP1_RUNNER="wasmtime -Wwide-arithmetic=y" \
  just fxhash-snapshot-probe --target wasm32-wasip1
```

Keep upstream source byte-identical. Format only our `examples/contract_probe.rs`
when changing the probe; do not run rustfmt over the upstream snapshot.

## Observed contract differences, 2026-11-04

Rust 1.99.1, native x86-64 and wasm32-wasip1 under Wasmtime, with wide arithmetic
disabled. Upstream tests pass on both targets. These are exact hash values,
not benchmark timings.

| Byte writes | x86-64 output | wasm32 output |
|---|---|---|
| `hello world` | `ac6f814ed569f75c` | `0000000098ef41a4` |
| `hello`, then ` world` | `aa1cc7e688fc1de9` | `000000000e8d67dc` |
| empty, then `hello world` | `5cfac8441a52a4c8` | `0000000063108f12` |
| `hello world`, then empty | `b4c786059a9668b9` | `0000000092fcfdc5` |

The optional wide-arithmetic probe also ran under Rust 1.99.1 and Wasmtime
40.0.1. It retained the same wasm32 outputs: despite the experimental compiler
flag, both `rustc --print cfg` and the probe reported no `wide-arithmetic`
target-feature cfg. The upstream conditional wide-multiply branch was therefore
not selected. This run does not validate that additional source branch.

`FxHasher` stores `usize`, uses pointer-width-specific constants and final
rotation, and chooses different multiplication mixes on different targets.
Each `write` compresses its slice independently. In particular, concatenating
two writes is not equivalent to one write with concatenated bytes, and an empty
write is not an identity operation. `core::hash::Hasher` permits this behavior.
It differs from zenhash's byte-streaming and portable-output guarantees.
Those guarantees are not prerequisites for ordinary in-memory hash tables. The
64-bit typed-integer fast path is a good candidate for trusted integer/struct
keys; tiny byte strings use a separate compression path and need their own
comparison. Compilation time and table throughput versus rapidhash are not
measured in this evaluation.

The source describes a small polynomial state and a wyhash-inspired byte
compression function. This makes it useful to study for trusted in-memory hash
tables. Its source comments' SMHasher3 discussion concerns the byte compressor
with an additional avalanche permutation; it is not a claim that the exposed
FxHasher has that identical tested output. No SMHasher3 run or compile-time
benchmark of this snapshot has been performed here.

Freezing the source prevents upstream algorithm drift. It does not remove these
platform/write-boundary differences. A fixed-width portable version with split-
invariant streaming would be a distinct, separately named design requiring its
own reference values and validation before becoming zenhash API.
