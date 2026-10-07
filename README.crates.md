<!-- GENERATED FROM README.md by zenutils gen-readme-crates.sh — DO NOT EDIT. -->

# zenhash

zenhash is an implementation of the xxHash family (XXH3 64/128, XXH64, XXH32) in safe Rust, with runtime SIMD dispatch for XXH3.

`no_std` + `alloc` · `#![forbid(unsafe_code)]` · MSRV 1.89 · x86-64 (AVX-512, AVX2, SSE2) / aarch64 (NEON) / wasm32 (SIMD128) · MIT OR Apache-2.0

## Quick start

```toml
[dependencies]
zenhash = "0.1.0"
```

```rust
let h64: u64 = zenhash::xxh3_64(b"hello world");
let h128: u128 = zenhash::xxh3_128(b"hello world");
assert_eq!(h64, 0xD447_B1EA_40E6_988B);
assert_eq!(h128, 0xDF8D_09E9_3F87_4900_A99B_8775_CC15_B6C7);

// Streaming gives the same values for any split of the input.
let mut h = zenhash::Xxh3::new();
h.update(b"hello ");
h.update(b"world");
assert_eq!(h.digest(), h64);
assert_eq!(h.digest128(), h128);
```

## Status

Preview (pre-release). Output matches the xxHash specification; the API may
still change before 0.1.0 ships.

Not supported:

- XXH3 with both a seed and a custom secret (`XXH3_*_withSecretandSeed` in C).
- Generating a secret from a seed or entropy (`XXH3_generateSecret` in C).
- Canonical (big-endian byte) representations of hashes.
- `BuildHasher` types for `HashMap`: use
  `BuildHasherDefault<zenhash::Xxh3>` (seed 0).
- `const fn` hashing.
- SIMD for XXH64 and XXH32 (the C reference has none either), and SVE.

## API

| Algorithm | One-shot | Streaming |
|---|---|---|
| XXH3, 64-bit | `xxh3_64`, `xxh3_64_with_seed`, `xxh3_64_with_secret` | `Xxh3::digest` |
| XXH3, 128-bit | `xxh3_128`, `xxh3_128_with_seed`, `xxh3_128_with_secret` | `Xxh3::digest128` |
| XXH64 | `xxh64(data, seed)` | `Xxh64` |
| XXH32 | `xxh32(data, seed)` | `Xxh32` |

`Xxh3::new`, `Xxh3::with_seed` and `Xxh3::with_secret` pick the variant; one
streaming state produces both widths. All streaming types implement
`core::hash::Hasher`, and `Xxh3` implements `std::io::Write` with the `std`
feature. A custom secret must be at least `XXH3_SECRET_SIZE_MIN` (136) bytes;
shorter ones return `Err(At<Error::SecretTooShort>)` (errors carry a
[whereat](https://crates.io/crates/whereat) location trace).

### SIMD

Only XXH3 inputs over 240 bytes use SIMD. The accumulator dispatches once per
call (per `update` call when streaming) to the best tier the CPU has:

| Tier | Target | Enabled by |
|---|---|---|
| AVX-512 | x86-64 | the `avx512` cargo feature |
| AVX2 | x86-64 | default |
| SSE2 | x86-64 | default (baseline) |
| NEON | aarch64 | default |
| SIMD128 | wasm32 | building with `-Ctarget-feature=+simd128` |
| scalar | everything else, big-endian targets | always |

Runtime CPU detection needs the `std` feature (on by default). Without it,
dispatch sees only the features enabled at compile time. Dispatch uses
[archmage](https://crates.io/crates/archmage), so the crate contains no
`unsafe`.

### Features

| Feature | Default | Effect |
|---|---|---|
| `std` | yes | Runtime CPU detection; `std::io::Write` for `Xxh3`. |
| `avx512` | no | Compiles the AVX-512 tier. On the benchmark CPU, XXH3-64 at 64 KiB ran at 42.3 GiB/s with it and 32.3 GiB/s without (separate runs). Measure on your hardware: AVX-512 behaves differently across CPU generations. |

## Testing

Every algorithm is compared against two independent implementations,
[xxhash-rust](https://crates.io/crates/xxhash-rust) and
[twox-hash](https://crates.io/crates/twox-hash): every length from 0 to 300,
block and buffer edges up to 100,003 bytes, four seeds, custom secrets from 136
to 1000 bytes, and ten streaming split patterns. A tier-permutation test runs
the XXH3 long path with each SIMD tier the machine has disabled in turn. CI runs
the tests on x86-64 and aarch64 Linux, Windows and macOS, i686, and
wasm32-wasip1 with and without SIMD128. A differential fuzz target lives in
`fuzz/`.


## License

MIT OR Apache-2.0, at your option.

## AI-Generated Code Notice

Developed with Claude (Anthropic). Not all code manually reviewed. Review
critical paths before production use.

## Image tech I maintain

| | |
|:--|:--|
| **Codecs** ¹ | [zenjpeg] · [zenpng] · [zenwebp] · [zengif] · [zenavif] · [zenjxl] · [zenjxl-decoder] · [jxl-encoder] · [zenbitmaps] · [heic] · [zentiff] · [zenpdf] · [zensvg] · [zenjp2] · [zenraw] · [ultrahdr] |
| Codec internals | [zenrav1e] · [rav1d-safe] · [zenravif] · [zenavif-parse] · [zenavif-serialize] |
| Compression | [zenflate] · [zenzop] · [zenzstd] |
| Processing | [zenresize] · [zenquant] · [zenblend] · [zenfilters] · [zensally] · [zentone] |
| Pixels & color | [zenpixels] · [zenpixels-convert] · [linear-srgb] · [garb] · [zenyuv] |
| Pipeline & framework | [zenpipe] · [zencodec] · [zencodecs] · [zenlayout] · [zennode] · [zenwasm] · [zentract] |
| Metrics | [zensim] · [fast-ssim2] · [butteraugli] · [zenmetrics] · [resamplescope-rs] |
| Pickers & ML | [zenanalyze] · [zenpredict] · [zenpicker] · [zenanalyze-api] |
| Test corpora | [codec-corpus] · [imazen-26] |
| Products | [Imageflow] image engine ([.NET][imageflow-dotnet] · [Node][imageflow-node] · [Go][imageflow-go]) · [Imageflow Server] · [ImageResizer] (C#) |

<sub>¹ pure-Rust, `#![forbid(unsafe_code)]` codecs, as of 2026</sub>

### General Rust awesomeness

[zenbench] · [archmage] · [magetypes] · [enough] · [whereat] · [cargo-copter] · [zenutils]

[Open source](https://www.imazen.io/open-source) · [@imazen](https://github.com/imazen) · [@lilith](https://github.com/lilith) · [lib.rs/~lilith](https://lib.rs/~lilith)

[zenjpeg]: https://github.com/imazen/zenjpeg
[zenpng]: https://github.com/imazen/zenpng
[zenwebp]: https://github.com/imazen/zenwebp
[zengif]: https://github.com/imazen/zengif
[zenavif]: https://github.com/imazen/zenavif
[zenjxl]: https://github.com/imazen/zenjxl
[zenjxl-decoder]: https://github.com/imazen/zenjxl-decoder
[jxl-encoder]: https://github.com/imazen/jxl-encoder
[zenbitmaps]: https://github.com/imazen/zenbitmaps
[heic]: https://github.com/imazen/heic
[zentiff]: https://github.com/imazen/zenextras
[zenpdf]: https://github.com/imazen/zenextras
[zensvg]: https://github.com/imazen/zenextras
[zenjp2]: https://github.com/imazen/zenextras
[zenraw]: https://github.com/imazen/zenraw
[ultrahdr]: https://github.com/imazen/ultrahdr
[zenrav1e]: https://github.com/imazen/zenrav1e
[rav1d-safe]: https://github.com/imazen/rav1d-safe
[zenravif]: https://github.com/imazen/cavif-rs
[zenavif-parse]: https://github.com/imazen/zenavif
[zenavif-serialize]: https://github.com/imazen/zenavif
[zenflate]: https://github.com/imazen/zenflate
[zenzop]: https://github.com/imazen/zenzop
[zenzstd]: https://github.com/imazen/zenzstd
[zenresize]: https://github.com/imazen/zenresize
[zenquant]: https://github.com/imazen/zenquant
[zenblend]: https://github.com/imazen/zenblend
[zenfilters]: https://github.com/imazen/zenpipe
[zensally]: https://github.com/imazen/zensally
[zentone]: https://github.com/imazen/zentone
[zenpixels]: https://github.com/imazen/zenpixels
[zenpixels-convert]: https://github.com/imazen/zenpixels
[linear-srgb]: https://github.com/imazen/linear-srgb
[garb]: https://github.com/imazen/garb
[zenyuv]: https://github.com/imazen/zenjpeg
[zenpipe]: https://github.com/imazen/zenpipe
[zencodec]: https://github.com/imazen/zencodec
[zencodecs]: https://github.com/imazen/zenpipe
[zenlayout]: https://github.com/imazen/zenpipe
[zennode]: https://github.com/imazen/zennode
[zenwasm]: https://github.com/imazen/zenwasm
[zentract]: https://github.com/imazen/zentract
[zensim]: https://github.com/imazen/zensim
[fast-ssim2]: https://github.com/imazen/fast-ssim2
[butteraugli]: https://github.com/imazen/butteraugli
[zenmetrics]: https://github.com/imazen/zenmetrics
[resamplescope-rs]: https://github.com/imazen/resamplescope-rs
[zenanalyze]: https://github.com/imazen/zenanalyze
[zenpredict]: https://github.com/imazen/zenanalyze
[zenpicker]: https://github.com/imazen/zenanalyze
[zenanalyze-api]: https://github.com/imazen/zenanalyze
[codec-corpus]: https://github.com/imazen/codec-corpus
[imazen-26]: https://github.com/imazen/imazen-26
[zenbench]: https://github.com/imazen/zenbench
[archmage]: https://github.com/imazen/archmage
[magetypes]: https://github.com/imazen/archmage
[enough]: https://github.com/imazen/enough
[whereat]: https://github.com/lilith/whereat
[cargo-copter]: https://github.com/imazen/cargo-copter
[zenutils]: https://github.com/imazen/zenutils
[Imageflow]: https://github.com/imazen/imageflow
[Imageflow Server]: https://github.com/imazen/imageflow-dotnet-server
[ImageResizer]: https://github.com/imazen/resizer
[imageflow-dotnet]: https://github.com/imazen/imageflow-dotnet
[imageflow-node]: https://github.com/imazen/imageflow-node
[imageflow-go]: https://github.com/imazen/imageflow-go
