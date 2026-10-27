//! Fixed workloads for perf/JIT profiling, with no in-code timing.
#![forbid(unsafe_code)]
use std::{hint::black_box, path::PathBuf};
use zenbench::wasm::{WasmBench, wasmtime};
use zenhash_wasm_workload::hash_batch;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(
        args.len(),
        7,
        "mode(native/scalar/simd128/wide/simd128-wide) algorithm len chunk outer repetitions"
    );
    let algorithm: u32 = args[2].parse().unwrap();
    let len: u32 = args[3].parse().unwrap();
    let chunk: u32 = args[4].parse().unwrap();
    let outer: usize = args[5].parse().unwrap();
    let repetitions: u32 = args[6].parse().unwrap();
    let params = (algorithm, len, 0x9e3779b185ebca87u64, chunk, repetitions);
    assert!(outer > 0 && repetitions > 0);
    if args[1] == "native" {
        for _ in 0..outer {
            black_box(hash_batch(
                black_box(algorithm),
                black_box(len),
                black_box(params.2),
                black_box(chunk),
                black_box(repetitions),
            ));
        }
    } else {
        assert!(["scalar", "simd128", "wide", "simd128-wide"].contains(&args[1].as_str()));
        let root =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../wasm-workload/target/modules");
        let mut config = wasmtime::Config::new();
        config
            .wasm_relaxed_simd(false)
            .wasm_simd(args[1].contains("simd128"))
            .wasm_wide_arithmetic(args[1].contains("wide"))
            .cranelift_opt_level(wasmtime::OptLevel::Speed)
            .profiler(wasmtime::ProfilingStrategy::JitDump);
        let module = WasmBench::from_bytes_with_config(
            &std::fs::read(root.join(format!("{}.wasm", args[1]))).unwrap(),
            config,
        )
        .unwrap();
        let symbol = std::fs::read_to_string(root.join(format!("{}.symbol", args[1]))).unwrap();
        let mut instance = module.instance().unwrap();
        let function = instance
            .typed_func::<(u32, u32, u64, u32, u32), u64>(symbol.trim())
            .unwrap();
        assert_eq!(
            function.call(&mut instance.store, params).unwrap(),
            hash_batch(algorithm, len, params.2, chunk, repetitions)
        );
        for _ in 0..outer {
            black_box(
                function
                    .call(&mut instance.store, black_box(params))
                    .unwrap(),
            );
        }
    }
}
