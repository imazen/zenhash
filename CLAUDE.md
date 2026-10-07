# zenhash — agent notes

xxHash (XXH3 64/128, XXH64, XXH32) in safe Rust (`#![forbid(unsafe_code)]`,
`no_std` + `alloc`), MIT OR Apache-2.0. Output must equal the reference C
implementation bit for bit on every target and SIMD tier.

## Layout

- `src/xxh3/mod.rs`: constants, default secret, seed-derived secrets, the
  short paths (`len <= 240`, scalar, read only the first 136 secret bytes),
  long-input finalization, one-shot API.
- `src/xxh3/accumulate.rs`: the stripe accumulator, the only SIMD code. One
  shared loop (`consume_body!`) stamped per tier with four helpers (load, store,
  accumulate one 64-byte stripe, scramble). Tiers: v4 (`avx512` feature, on by
  default), v3,
  v1 (SSE2), neon, wasm128, scalar. Dispatch is one `incant!` per `consume`
  call. Big-endian targets always take scalar.
- `src/xxh3/stream.rs`: streaming state, mirrors the C buffering (256-byte
  buffer, keeps the last consumed stripe at the buffer's end for the
  digest's "previous 64 bytes" case).
- `src/xxh64.rs`, `src/xxh32.rs`: classic algorithms, scalar only (the C
  reference has no SIMD path for them either).

## Recipes

`just test`, `just clippy`, `just clippy-arch` (NEON/WASM type-check from x86),
`just test-aarch64-qemu`, `just test-wasm`, `just bench`, `just bench-no-avx512`,
`just ci`. Without wasmtime, Node's `node:wasi` works as a WASI runner.

## CI workflows

- `ci.yml`: test matrix (6 OSes incl. ARM Windows/Linux), i686 via cross,
  wasm32 (wasmtime, +-simd128), no_std, MSRV, feature powerset, lint (incl.
  aarch64/wasm clippy), package size, bench and fuzz compile, API snapshot,
  coverage. Also `workflow_call`ed by `publish.yml`.
- `fuzz.yml`: hash_parity on x86_64 and aarch64; 60 s per push to main,
  600 s nightly, crash artifacts uploaded.
- `examples/hash_file.rs`: streams a file through all four hashes.
- `bench.yml`: zenbench on all 6 platforms on main / by hand
  (`ZENHASH_BENCH` filter input); artifacts only, shared runners are noisy.
- `publish.yml`: on a published GitHub release, checks tag == `v<version>`,
  runs full CI at that commit, then `cargo publish` from the `crates-io`
  environment (needs `CARGO_REGISTRY_TOKEN`).
- `api-guard.yml`: zenutils' advisory PR comment with the API diff.
- After the first publish, add a `cargo semver-checks` job.

## Design decisions

- **Tier test isolation:** `tests/tiers.rs` is its own test binary (process)
  because tier disabling is process-wide; keep other tests out of it.
- **Oracles:** tests compare against xxhash-rust and twox-hash (dev-deps only)
  across all path boundaries, seeds, secret sizes 136..1000 and streaming
  splits. The fuzz target (`fuzz/fuzz_targets/hash_parity_core.rs`) does the
  same and is replayed by `tests/it/fuzz_regression.rs`.
- **Why intrinsics, not magetypes:** magetypes 0.9.29 `u64x*` vectors have no
  32x32->64 widening multiply (`pmuludq`/`vmull_u32`/`extmul`) and no 64-bit
  lane swap. XXH3 needs both on every stripe. If magetypes gains them, the
  x86/NEON/WASM tiers could collapse into one `#[magetypes]` kernel over
  `u64x8`, which happens to be exactly the 8-accumulator state.
- **No dispatch for short inputs:** inputs of at most 240 bytes never touch
  `incant!`; the per-call summon cost would dominate an 8-byte hash.
- **Custom secrets** are validated (>= 136 bytes) and returned as
  `At<Error>`; streaming copies the secret into a `Box<[u8]>`.

## Public API decisions (audit 2026-10-07)

- All three hashers: `new()`, `with_seed()`, `Default`, `Clone`, `Hasher`,
  `io::Write` (std). Same shape on purpose.
- `Debug` is hand-written: byte count and secret kind only. The derived one
  printed custom secrets, seeds and buffered input.
- `#[must_use]` on one-shot functions and `digest*`.
- Not added (no current caller): seed + secret combined, secret generation,
  `BuildHasher` types, canonical byte output, `const fn` hashing.
- `Error::SecretTooShort { len }` is the only error; `Error` is
  `#[non_exhaustive]`.

## Optimization log (dead ends and why)

Bench host for these entries: Xeon family 6 model 207 (Emerald Rapids),
4 vCPU VM, rustc 1.97.0, `cargo bench --bench hash -- --no-busy-gate`.

- 129..=240 paths: an `#[inline(never)]` function with an indexed
  `for i in 8..rounds` loop let LLVM's SLP vectorizer pack the 64-bit
  xor/add pairs into SSE registers (unpack + stack spills around every
  `mulq`): 21.1 ns at 240 B. Inlining (default secret folds into immediates)
  plus fixed-size chunk iterators with a constant `take(N)` bound: 10.9 ns.
  twox-hash solves the same problem with an inline-asm register barrier,
  which `forbid(unsafe_code)` rules out here.
- Long-input loop, AVX2 asm per stripe: 18 vector ops + 7 scalar (bounds
  check, and a `cmp`/`cmovb` from `key_at`'s unreachable fallback). Secret
  `windows(64).step_by(8)` removed the check (still 7 scalar). A whole-block
  fast path for 192-byte secrets (default and seed-derived) with constant key
  offsets, unrolled 2 stripes per iteration, leaves 4 scalar per 2 stripes.
  Together with `srli` for the hi->lo move (C uses a second `pshufd`, which
  competes with the data-swap shuffle for one port on Intel): xxh3_64/64 KiB
  went from 11-13% behind twox-hash to 4% ahead. Separate runs on this VM
  drift +-4% relative to competitors, so judge changes by repeated runs.
- Streaming: `update` was one out-of-line function; 64-byte updates ran
  4-8% behind both competitors (~0.7 ns/call). An `#[inline]` buffer-only
  fast path plus `#[inline(never)] update_slow` made them 10-16% faster than
  both. Partial blocks (every streaming update leaves the block position
  non-zero) use a masked key index `(k & 15) * 8` under 192-byte secrets, so
  no bounds check. 4 KiB updates remain 1.4-3.7% behind twox-hash.
- Entry-point prologue: inlining the 129..=240 and long paths into
  `xxh3_64` gave it `push`/`push`/`sub rsp` on every call; 8 B went from
  10-16% faster than xxhash-rust to 18% slower and 16 B to 29% slower. Fix:
  entry points inline only the <= 128 B paths and call `#[inline(never)]`
  helpers that hard-code the default secret (still folded to immediates).
- Mid-size paths and the SLP vectorizer: the u64-xor form vectorizes when the
  seed is a runtime value; a u128-xor form vectorizes (pxor + extracts) when
  the key is constant. `mix16_chunk::<U128_KEY>` picks per caller, with
  separate seed-0 and seeded helpers. Check `xmm` counts in the asm of
  `mid_64_*` / `mid_128_*` after touching them. Result at 240 B:
  xxh3_64 26-31%, seeded 10-14% faster than twox-hash, 128-bit tied.
- 1 KiB one-shot (15 stripes, no whole block): the generic partial-block
  loop plus a dispatcher that had the scalar and SSE2 tiers inlined (six
  pushes/pops before the tail jump to AVX2) put us 3.5-5% behind twox-hash,
  21-22% behind with `avx512`. Fallback tiers are now `#[inline(never)]` (the
  dispatch inlines to one cached-byte check) and the fast path also takes a
  partial block starting at stripe 0 (every one-shot's last block): 2-4%
  ahead by default, 1-3% behind with `avx512`.
- 241 B..1 KiB fixed cost (found by benches/adversarial.rs, sizes the main
  bench lacked): one-shot hashes copied INIT_ACC to the stack as four
  16-byte stores and the kernel reloaded it as one 64-byte (AVX-512) or two
  32-byte vectors, a store-forwarding failure. Loading from the constant
  instead (`fresh`): AVX-512 241 B 25.9 -> 18.7 ns, 320 B 26.9 -> 18.5,
  512 B 32.5 -> 22-24, 1 KiB 35.9 -> 30.5 (ad-hoc Instant loop, best of 7).
  Dead ends, both measured: a 64-byte-aligned accumulator (C's
  `XXH_ALIGN(64)`; no change, 18.8 -> 18.6 ns) and merging inside the tier
  function to avoid the vector-store/scalar-reload (no gain; 512 B worse).
  Valgrind (AVX2 build) counts ~270 instructions per 241-byte hash for both
  zenhash and C, so the remaining gap to C at -march=native (10.9 ns at
  241 B) is stalls or code placement, not work. No PMU in this VM.
- `-C target-cpu=native` builds: with AVX2/AVX-512 enabled for all code, the
  SLP vectorizer packs the seed-0 mid path (constant-key u64 form) into
  ymm/zmm: 129 B 12.7 ns vs 6.8 in baseline builds. `mid_64_default` uses
  `cfg!(target_feature = "avx2")` to switch to the u128 form with the
  secret behind `core::hint::black_box` (scalar in both builds; 4-8% slower
  than the constant form in baseline builds, so only there). CI runs the
  suite with `target-cpu=native` on ubuntu x64/arm for this branch.
- XXH32/XXH64 at 16 B: 2-9% behind both competitors. The asm matches
  xxhash-rust's nearly instruction for instruction (we add a `push rbx` and a
  mask); not chased further.
- zenbench's resource gate stalled the bench on this VM (13 s CPU in 20 min).
  Run with `--no-busy-gate` there.

## Test coverage log

- 2026-10-07, `cargo llvm-cov --all-features`: 99.57% lines, 100% functions.
  Uncovered: the big-endian early return (covered on s390x in CI, not in
  the x86 coverage run), the unreachable `key_at` fallback, the fuzz body's
  short-input return (seed added).
- 2026-10-07, `cargo mutants --all-features` (598 mutants, 15 min): 545
  caught, 23 unviable, 2 timeouts (infinite recursion, i.e. caught), 28
  missed. Missed and equivalent: 8 `|`->`^` on disjoint bits, 4 `seed == 0`
  fast-path guards, `default_block_keys -> None` (fast path off, same
  result), 3 `>`->`>=` buffer-flush conditions in streaming (same digest).
  Missed because not compiled on x86: 11 NEON/WASM tier functions (CI runs
  those tiers on ARM runners and wasmtime). One real gap: a broken scalar
  kernel (`acc[i ^ 1]` -> `acc[i | 1]`) survived because the tier test ran
  in the shared test binary, where `for_each_token_permutation`'s
  process-wide tier disabling raced with parallel tests. It now lives in
  its own binary, `tests/tiers.rs`, and catches that mutant every run.
  Re-run: `just mutants`.

## Fuzzing log

- 2026-10-07: `hash_parity`, 600 s, 11,283,109 runs, `-max_len=8192`, no
  findings (AVX2 tier; host had AVX-512 but the `avx512` feature was off).

## Open performance issues

- `-C target-cpu=native` makes XXH64/XXH32 about 2x slower at 64 KiB (all
  three Rust crates; C is unaffected): LLVM vectorizes the four independent
  64-bit lanes into AVX-512 `vpmullq`. C blocks it with an inline-asm
  barrier, which forbid(unsafe_code) rules out. Needs a safe formulation
  the SLP vectorizer won't pack (see benchmarks/adversarial_2026-10-07.md).
- 241 B+: C at -march=native is 8-44% faster; same instruction count, so
  latency. No PMU in the dev VM to dig further.

## History

#1 was squash-merged (ce2bbdf). Its 20 original commits, which the
optimization log, CHANGELOG and benchmark reports cite by hash, are kept
on the `archive/squashed-pr1` branch.

## Known Bugs

None open.
