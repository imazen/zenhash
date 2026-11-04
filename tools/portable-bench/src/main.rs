//! Compare identical native, WASM scalar and WASM SIMD128 work on one CPU.
#![forbid(unsafe_code)]
use std::{path::PathBuf, time::Duration};
use zenbench::{prelude::*, wasm::WasmBench};
use zenhash_wasm_workload::{NAMES, hash_batch};

fn portable(suite: &mut Suite) {
    if let Ok(filter) = std::env::var("ZENHASH_BENCH") {
        suite.set_group_filter(filter);
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../wasm-workload/target/modules");
    let modules: [(String, WasmBench, String); 4] = ["scalar", "simd128", "wide", "simd128-wide"]
        .map(|name| {
            let path = root.join(format!("{name}.wasm"));
            {
                let mut config = zenbench::wasm::wasmtime::Config::new();
                config
                    .wasm_relaxed_simd(false)
                    .wasm_simd(name.contains("simd128"))
                    .wasm_wide_arithmetic(name.contains("wide"));
                let engine = zenbench::wasm::wasmtime::Engine::new(&config).unwrap();
                zenbench::wasm::wasmtime::Module::from_file(&engine, &path)
                    .expect("module must match its declared SIMD/wide-arithmetic capabilities");
            }
            let mut runtime = zenbench::wasm::wasmtime::Config::new();
            runtime
                .wasm_relaxed_simd(false)
                .wasm_simd(name.contains("simd128"))
                .wasm_wide_arithmetic(name.contains("wide"))
                .cranelift_opt_level(zenbench::wasm::wasmtime::OptLevel::Speed);
            let wasm =
                WasmBench::from_bytes_with_config(&std::fs::read(&path).unwrap(), runtime).unwrap();
            let symbol = std::fs::read_to_string(root.join(format!("{name}.symbol"))).unwrap();
            (name.into(), wasm, symbol.trim().into())
        });
    // Check every method, including high halves of 128-bit output indirectly
    // through the independent stability suite, before starting any timing.
    for len in [0, 4, 16, 64, 240, 241, 1024, 65536] {
        for algorithm in 0..NAMES.len() as u32 {
            for chunk in [0, 17, 4096] {
                for seed in [0, 1, u64::MAX, 0x9e3779b185ebca87] {
                    let expected = hash_batch(algorithm, len, seed, chunk, 3);
                    assert_eq!(
                        expected,
                        hash_batch(algorithm, len, seed, 0, 3),
                        "native streaming batch parity"
                    );
                    for (_, module, symbol) in &modules {
                        let mut instance = module.instance().unwrap();
                        let actual = instance
                            .call::<(u32, u32, u64, u32, u32), u64>(
                                symbol,
                                (algorithm, len, seed, chunk, 3),
                            )
                            .unwrap();
                        assert_eq!(
                            actual, expected,
                            "WASM parity {algorithm} len {len} chunk {chunk} seed {seed:x}"
                        );
                    }
                }
            }
        }
    }
    // Paired comparisons for the FNV replacement decision on short byte keys.
    for len in [4u32, 16, 64, 241, 1024, 65536] {
        let repetitions = (1048576 / len).clamp(1, 16384);
        for runtime in 0..=modules.len() {
            suite.group(format!("replacement/{len}/runtime{runtime}"), |g| {
                g.throughput(Throughput::Bytes(u64::from(len) * u64::from(repetitions)));
                g.config().warmup_time = Duration::from_millis(50);
                g.config().min_rounds = 15;
                g.config().max_rounds = 30;
                for algorithm in [12usize, 15, 6, 0, 16, 18] {
                    let params = (
                        algorithm as u32,
                        len,
                        0x9e3779b185ebca87u64,
                        0u32,
                        repetitions,
                    );
                    let label = NAMES[algorithm];
                    if runtime == 0 {
                        g.bench(label, move |b| {
                            b.iter(|| {
                                hash_batch(
                                    black_box(params.0),
                                    black_box(params.1),
                                    black_box(params.2),
                                    black_box(params.3),
                                    black_box(params.4),
                                )
                            })
                        });
                    } else {
                        let (_, module, symbol) = &modules[runtime - 1];
                        let module = module.clone();
                        let symbol = symbol.clone();
                        g.bench(label, move |b| {
                            let mut instance = module.instance().unwrap();
                            let function = instance
                                .typed_func::<(u32, u32, u64, u32, u32), u64>(&symbol)
                                .unwrap();
                            b.iter(|| {
                                function
                                    .call(&mut instance.store, black_box(params))
                                    .unwrap()
                            });
                        });
                    }
                }
            });
        }
    }

    for len in [4u32, 16, 64, 241, 1024, 65536] {
        for (algorithm, &name) in NAMES.iter().enumerate() {
            for chunk in [0u32, 4096] {
                // Streaming short inputs add no chunk-boundary coverage.
                if chunk != 0 && len < 1024 {
                    continue;
                }
                let algorithm = algorithm as u32;
                let repetitions = (1048576 / len).clamp(1, 16384);
                let params = (algorithm, len, 0x9e3779b185ebca87, chunk, repetitions);
                suite.group(format!("{name}/{len}/chunk{chunk}"), |g| {
                    g.throughput(Throughput::Bytes(u64::from(len) * u64::from(repetitions)));
                    let config = g.config();
                    config.warmup_time = Duration::from_millis(50);
                    config.max_time = Duration::from_millis(500);
                    config.min_rounds = 15;
                    config.max_rounds = 30;
                    g.bench("native", move |b| {
                        b.iter(|| {
                            hash_batch(
                                black_box(params.0),
                                black_box(params.1),
                                black_box(params.2),
                                black_box(params.3),
                                black_box(params.4),
                            )
                        })
                    });
                    for (label, module, symbol) in &modules {
                        let label = label.clone();
                        let module = module.clone();
                        let symbol = symbol.clone();
                        g.bench(label, move |b| {
                            let mut instance = module.instance().unwrap();
                            let function = instance
                                .typed_func::<(u32, u32, u64, u32, u32), u64>(&symbol)
                                .unwrap();
                            b.iter(|| {
                                function
                                    .call(&mut instance.store, black_box(params))
                                    .unwrap()
                            });
                        });
                    }
                });
            }
        }
    }
}
zenbench::main!(portable);
