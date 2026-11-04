# Paired WASM widening products, 2026-10-26

Guest source: `e9e91d609a344ee67635f1f4dc0a66a1057ef61a`.
Frozen native host source: `81f28bf572433b9990b20f282d0ad52c24ec4a7d`.
Same Neoverse-N1, compiler, Wasmtime and batches as the
[baseline](arm_wasm_baseline_2026-10-26.md). These are separate exploratory runs.

Perf placed 95.70% of native and 95.89% of WASM SIMD sampled cycles in
`xxh3::accumulate::consume`, with no lost samples. JIT disassembly showed that
separately packing each vector's low/high halves became general 64-bit vector
multiplication and table shuffles. Packing two input vectors together preserves
32-bit widening products. This changes no algorithm constants, input reads or
output expectations.

| 64 KiB operation | Before GiB/s | After GiB/s |
|---|---:|---:|
| WASM SIMD128 one-shot | 4.131 | 7.212 |
| WASM SIMD128, 4 KiB updates | 3.864 | 6.571 |
| Frozen native one-shot control | 9.716 | 9.554 |
| WASM scalar one-shot control | 4.726 | 4.636 |

The scalar module SHA-256 is unchanged:
`1c578bc97707a69b0da97c087edcc57c587e87b5b96a931dab1bb028fdaaac7f`.
The new SIMD module SHA-256 is
`d7552844a4bfe9cda01544143f8bb25775c2f2e68f32d21b05416e1736b31fea`.

Validation: full root WASI tests with SIMD both off and on; the full independent
C stability corpus under Wasmtime 49.0.4 on ARM with SIMD; all benchmark method,
seed and streaming parity checks before timing. Every expected C value is retained.

The combined validation/build/measurement wrapper reported:

```text
run-heavy: done rc=0 30s | peak-RSS 0.18GiB | min-avail 14852MiB | peak-load 0.39
```

Raw logs and JSON are outside git under
`/mnt/v/output/zenhash/arm-wasm-2026-10-26/paired-wasm.{log,json}`.
